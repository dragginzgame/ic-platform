//! Actual macro consumers with overlapping lifecycle and inspect-message exports.

use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn native_entrypoints_link_across_consuming_crates() {
    let directory = std::env::temp_dir().join(format!(
        "platform-native-link-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("Cargo.toml"), "[workspace]\nmembers=[\"role-a\",\"role-b\",\"consumer\"]\nresolver=\"3\"\n[profile.dev]\ncodegen-units=1\n").unwrap();
    let macros = env!("CARGO_MANIFEST_DIR");
    for (name, value) in [("role-a", 1), ("role-b", 2)] {
        let entry = if name == "role-a" { "" } else { "runtime_" };
        let role = directory.join(name);
        fs::create_dir_all(role.join("src")).unwrap();
        fs::write(role.join("Cargo.toml"), format!("[package]\nname={name:?}\nversion=\"0.0.0\"\nedition=\"2024\"\n[dependencies]\nic-platform-macros={{path={macros:?}}}\n")).unwrap();
        fs::write(
            role.join("src/lib.rs"),
            format!(
                r#"
            pub mod sdk {{ pub mod futures {{ pub mod internals {{
                pub fn in_executor_context(task: impl FnOnce()) {{ task(); }}
                pub fn in_query_executor_context(task: impl FnOnce()) {{ task(); }}
            }} }} }}
            #[ic_platform_macros::{entry}inspect_message(crate = "crate::sdk")]
            fn inspect() {{}}
            #[ic_platform_macros::{entry}init(crate = "crate::sdk")]
            fn init() {{}}
            #[ic_platform_macros::{entry}post_upgrade(crate = "crate::sdk")]
            fn upgrade() {{}}
            pub fn value() -> u8 {{ {value} }}
        "#
            ),
        )
        .unwrap();
    }
    let consumer = directory.join("consumer");
    fs::create_dir_all(consumer.join("src")).unwrap();
    fs::write(consumer.join("Cargo.toml"), "[package]\nname=\"native-link-consumer\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[dependencies]\nrole-a={path=\"../role-a\"}\nrole-b={path=\"../role-b\"}\n").unwrap();
    fs::write(
        consumer.join("src/main.rs"),
        "fn main() { assert_eq!(role_a::value(), 1); assert_eq!(role_b::value(), 2); }",
    )
    .unwrap();
    let output = Command::new(env!("CARGO"))
        .current_dir(&directory)
        .args(["run", "--offline", "--package", "native-link-consumer"])
        .env("CARGO_TARGET_DIR", directory.join("target"))
        .env_remove("CARGO_BUILD_TARGET")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}; fixture {}",
        String::from_utf8_lossy(&output.stderr),
        directory.display()
    );
    fs::remove_dir_all(directory).unwrap();
}
