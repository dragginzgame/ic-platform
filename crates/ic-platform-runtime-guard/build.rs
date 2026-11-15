//! Cargo's links contract requires a build entrypoint; no native library is built.
fn main() {
    println!("cargo::rerun-if-changed=Cargo.toml");
}
