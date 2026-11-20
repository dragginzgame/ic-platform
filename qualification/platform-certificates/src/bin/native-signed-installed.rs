//! Installed direct/delegated signed ingress, status recovery and query evidence.
#![forbid(unsafe_code)]

#[cfg(not(target_family = "wasm"))]
mod native_signed_installed;

#[cfg(not(target_family = "wasm"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(native_signed_installed::run())
}

#[cfg(target_family = "wasm")]
fn main() {}
