//! Runtime ownership for the local replacement cohort.
//!
//! The opt-in call slice prepares inert requests and owns response observations.
//! The wasm32 IC0 feature adds one-way submission and experimental awaited
//! transport with callback custody and an experimental method-attached executor.
//! Entrypoint and installed rollback qualification remain pending; this is not
//! a complete canister runtime.

#[cfg(feature = "call")]
pub mod call;

#[cfg(any(test, all(feature = "call-ic0", target_arch = "wasm32")))]
pub mod executor;
