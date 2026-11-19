//! Runtime ownership for the local replacement cohort.
//!
//! The opt-in call slice prepares inert requests and owns response observations.
//! The wasm32 IC0 feature adds one-way submission and experimental awaited
//! transport with callback custody and an experimental method-attached executor.
//! Explicit entrypoints and selected installed rollback cases are qualified;
//! broader executor contexts remain experimental. This is not a complete runtime.

#[cfg(all(
    not(target_arch = "wasm32"),
    any(feature = "call-ic0", feature = "system-raw")
))]
mod native;
#[cfg(all(not(target_arch = "wasm32"), feature = "system-raw"))]
pub use native::raw;
#[cfg(all(not(target_arch = "wasm32"), feature = "call-ic0"))]
pub use native::system;

#[cfg(feature = "call")]
pub mod call;
#[cfg(all(not(feature = "call"), feature = "system-raw"))]
mod call;
#[cfg(feature = "call-ic0")]
pub mod entrypoints;
#[cfg(any(feature = "call-ic0", all(test, feature = "call")))]
mod execution_types;
#[cfg(any(
    all(test, feature = "call"),
    all(feature = "call-ic0", target_arch = "wasm32")
))]
pub mod executor;
#[cfg(feature = "raw-rand")]
pub mod raw_rand;
/// Execution observations used in endpoint signatures on a native build.
#[cfg(all(feature = "call-ic0", not(target_arch = "wasm32"), not(test)))]
pub mod executor {
    pub use crate::execution_types::{EntryError, ReplyError, TaskHandle, TaskState};
    pub use crate::native::run_update;
}
#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
pub mod system;

#[cfg(all(feature = "system-raw", target_arch = "wasm32"))]
pub use call::ic0::synchronous as raw;
#[cfg(feature = "entrypoint-macros")]
pub use ic_platform_macros::{init, post_upgrade, pre_upgrade, query, update};

#[cfg(feature = "call-ic0")]
const _: () = ic_platform_runtime_guard::assert_runtime_version(env!("CARGO_PKG_VERSION"));
