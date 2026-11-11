//! Matched BLS byte and instruction probes, not an application authentication API.
#[cfg(all(feature = "baseline", feature = "candidate"))]
compile_error!("Select exactly one BLS implementation");

#[cfg(all(
    target_family = "wasm",
    not(any(feature = "baseline", feature = "candidate"))
))]
compile_error!("Select a BLS implementation for the canister probe");

#[cfg(all(
    target_family = "wasm",
    any(feature = "baseline", feature = "candidate")
))]
mod canister;
