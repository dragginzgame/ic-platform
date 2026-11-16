//! Observe replica-issued certificates and verify them through Platform's public API.
#![forbid(unsafe_code)]

#[cfg(not(target_family = "wasm"))]
mod native;

#[cfg(not(target_family = "wasm"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    native::run()
}

#[cfg(target_family = "wasm")]
fn main() {}
