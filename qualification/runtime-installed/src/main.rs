//! Native driver for the candidate's installed initial-update rollback contract.
#![forbid(unsafe_code)]

#[cfg(not(target_family = "wasm"))]
mod native;

#[cfg(not(target_family = "wasm"))]
mod identity;

#[cfg(not(target_family = "wasm"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    native::run()
}

#[cfg(target_family = "wasm")]
fn main() {}

#[cfg(not(target_family = "wasm"))]
mod raw;

#[cfg(not(target_family = "wasm"))]
mod timer;

#[cfg(not(target_family = "wasm"))]
mod accounting;
