//! Runtime ownership for the local replacement cohort.
//!
//! The opt-in call slice prepares inert requests and owns response observations.
//! The wasm32 IC0 feature adds one-way submission and experimental awaited
//! transport with callback custody and an experimental method-attached executor.
//! Explicit entrypoints and selected installed rollback cases are qualified;
//! broader executor contexts remain experimental. This is not a complete runtime.

#[cfg(feature = "call")]
pub mod call;
#[cfg(all(not(feature = "call"), feature = "system-raw", target_arch = "wasm32"))]
mod call;
#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
pub mod entrypoints;
#[cfg(any(
    all(test, feature = "call"),
    all(feature = "call-ic0", target_arch = "wasm32")
))]
pub mod executor;
#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
pub mod system;

#[cfg(all(feature = "system-raw", target_arch = "wasm32"))]
pub use call::ic0::synchronous as raw;
#[cfg(feature = "entrypoint-macros")]
pub use candidate_macros::{init, post_upgrade, pre_upgrade, query, update};

#[cfg(feature = "call-ic0")]
const _: () = candidate_runtime_guard::assert_runtime_version(env!("CARGO_PKG_VERSION"));
