//! Standalone Cargo consumers qualify native symbol identity across crates.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn role_manifest(role: &str, macros: &Path) -> String {
    let macros = format!("{:?}", macros.to_str().expect("UTF-8 checkout path"));
    format!(
        "[package]\nname = \"entrypoint-link-{role}\"\nversion = \"0.0.0\"\nedition = \"2024\"\nrust-version = \"1.99.1\"\npublish = false\n\n[package.metadata.toko-tooling.loc]\ncategory = \"tests\"\n\n[dependencies]\nic-platform-macros = {{ path = {macros} }}\n"
    )
}

fn native_name(crate_name: &str, export: &str) -> String {
    let hex: String = export
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("{crate_name}____ic_export_{hex}")
}

fn copy(source: &Path, destination: &Path) {
    fs::copy(source, destination).unwrap_or_else(|error| {
        panic!(
            "copy {} to {}: {error}",
            source.display(),
            destination.display()
        )
    });
}

#[test]
fn unrelated_crates_link_exact_lifecycle_and_punctuation_symbols() {
    let macros = fs::canonicalize(env!("CARGO_MANIFEST_DIR")).unwrap();
    let fixtures = macros.join("tests/fixtures/native_link");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = macros.join("../../target").join(format!(
        "entrypoint-native-link-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"left\", \"right\", \"consumer\"]\nresolver = \"3\"\n",
    )
    .unwrap();
    for role in ["left", "right"] {
        let directory = root.join(role);
        fs::create_dir_all(directory.join("src")).unwrap();
        fs::write(directory.join("Cargo.toml"), role_manifest(role, &macros)).unwrap();
        copy(
            &fixtures.join(format!("{role}.rs")),
            &directory.join("src/lib.rs"),
        );
        copy(
            &fixtures.join("runtime.rs"),
            &directory.join("src/runtime.rs"),
        );
    }
    let consumer = root.join("consumer");
    fs::create_dir_all(consumer.join("src")).unwrap();
    fs::write(consumer.join("Cargo.toml"), "[package]\nname = \"entrypoint-link-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\nrust-version = \"1.99.1\"\npublish = false\n\n[package.metadata.toko-tooling.loc]\ncategory = \"tests\"\n\n[dependencies]\nentrypoint-link-left = { path = \"../left\" }\nentrypoint-link-right = { path = \"../right\" }\n").unwrap();
    copy(&fixtures.join("main.rs"), &consumer.join("src/main.rs"));
    let output = Command::new(env!("CARGO"))
        .args([
            "run",
            "--offline",
            "--quiet",
            "--package",
            "entrypoint-link-consumer",
            "--manifest-path",
        ])
        .arg(root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env_remove("CARGO_BUILD_TARGET")
        .env("IC_CDK_DEBUG", "")
        .output()
        .unwrap_or_else(|error| panic!("Cargo fixture {}: {error}", root.display()));
    fs::write(root.join("cargo.stdout"), &output.stdout).unwrap();
    fs::write(root.join("cargo.stderr"), &output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 Cargo stdout");
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 Cargo stderr");
    assert!(
        output.status.success(),
        "native fixture retained at {}\nstdout:\n{}\nstderr:\n{}",
        root.display(),
        stdout,
        stderr
    );
    assert!(
        stderr.contains("canister_query a/b"),
        "empty IC_CDK_DEBUG must enable expansion diagnostics; fixture retained at {}",
        root.display()
    );

    let binary: PathBuf = root.join("target/debug/entrypoint-link-consumer");
    let symbols = Command::new("nm")
        .arg("-g")
        .arg(&binary)
        .output()
        .unwrap_or_else(|error| panic!("nm fixture {}: {error}", root.display()));
    fs::write(root.join("nm.stdout"), &symbols.stdout).unwrap();
    fs::write(root.join("nm.stderr"), &symbols.stderr).unwrap();
    assert!(
        symbols.status.success(),
        "nm fixture retained at {}: {}",
        root.display(),
        String::from_utf8(symbols.stderr).expect("UTF-8 nm stderr")
    );
    let symbols = String::from_utf8(symbols.stdout).expect("UTF-8 native symbols");
    let names: BTreeSet<_> = symbols
        .lines()
        .filter_map(|line| line.split_whitespace().last())
        .collect();
    for (owner, exports) in [
        (
            "entrypoint_link_left",
            &[
                "canister_init",
                "canister_heartbeat",
                "canister_pre_upgrade",
                "canister_post_upgrade",
                "canister_inspect_message",
                "canister_on_low_wasm_memory",
                "canister_query a/b",
                "canister_query a?b",
                "canister_query ",
                "canister_query é",
                "canister_composite_query type",
                "canister_update action",
            ][..],
        ),
        (
            "entrypoint_link_right",
            &[
                "canister_init",
                "canister_heartbeat",
                "canister_query a/b",
                "canister_query a?b",
            ][..],
        ),
    ] {
        for export in exports {
            let expected = native_name(owner, export);
            assert!(
                names.contains(expected.as_str())
                    || names.contains(format!("_{expected}").as_str()),
                "missing {expected}; fixture retained at {}",
                root.display()
            );
        }
    }
    fs::remove_dir_all(&root).unwrap();
}
