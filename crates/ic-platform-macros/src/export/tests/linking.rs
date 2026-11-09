//! Actual macro consumers with overlapping lifecycle and inspect-message exports.

use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn native_entrypoints_link_across_consuming_crates() {
    let scratch_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    fs::create_dir_all(&scratch_root).unwrap();
    let directory = scratch_root.join(format!(
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
    let fixture_sources = Path::new(macros).join("src/export/tests/linking");
    for (name, source) in [("role-a", "role_a.rs"), ("role-b", "role_b.rs")] {
        let role = directory.join(name);
        fs::create_dir_all(role.join("src")).unwrap();
        fs::write(role.join("Cargo.toml"), format!("[package]\nname={name:?}\nversion=\"0.0.0\"\nedition=\"2024\"\n[package.metadata.toko-tooling.loc]\ncategory=\"tests\"\n[dependencies]\nic-platform-macros={{path={macros:?}}}\n")).unwrap();
        fs::copy(fixture_sources.join(source), role.join("src/lib.rs")).unwrap();
    }
    let consumer = directory.join("consumer");
    fs::create_dir_all(consumer.join("src")).unwrap();
    fs::write(consumer.join("Cargo.toml"), "[package]\nname=\"native-link-consumer\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[package.metadata.toko-tooling.loc]\ncategory=\"tests\"\n[dependencies]\nrole-a={path=\"../role-a\"}\nrole-b={path=\"../role-b\"}\n").unwrap();
    fs::copy(
        fixture_sources.join("consumer.rs"),
        consumer.join("src/main.rs"),
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
