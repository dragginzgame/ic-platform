//! ICP CLI anonymous preparation and installed original-request recovery.
#![forbid(unsafe_code)]

#[cfg(not(target_family = "wasm"))]
mod native_anonymous_installed;

#[cfg(not(target_family = "wasm"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    native_anonymous_installed::check()
}

#[cfg(target_family = "wasm")]
fn main() {}
