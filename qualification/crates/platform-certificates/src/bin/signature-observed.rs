//! Capture independently replica-signed canister-signature envelopes.
#![forbid(unsafe_code)]

#[cfg(not(target_family = "wasm"))]
mod signature_observed;

#[cfg(not(target_family = "wasm"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    signature_observed::run()
}

#[cfg(target_family = "wasm")]
fn main() {}
