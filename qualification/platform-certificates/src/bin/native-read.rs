//! Explicit live read-only native transport probe; never an implicit test refresh.
#![forbid(unsafe_code)]

#[cfg(not(target_family = "wasm"))]
mod native_read;

#[cfg(not(target_family = "wasm"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(native_read::run())
}

#[cfg(target_family = "wasm")]
fn main() {}
