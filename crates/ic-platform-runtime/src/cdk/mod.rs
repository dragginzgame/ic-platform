//! Owned non-cryptographic runtime adapters and explicit-codec entrypoints.
//!
//! `runtime` exposes this module without entrypoint macros; `cdk` adds them.
//! Both are Candid-free. Select the facade's `candid-codec` explicitly for required external
//! management/system calls and consumer-owned legacy data. Controlled RPCs use
//! consumer-owned Wire hooks. This runtime owns the executor and explicit registry IC0 boundary.
//! Raw calls keep owned callback state separate from borrowed request data.
//!
//! The 0.5 hard cut retires the unused checkpoint API:
//! ```compile_fail
//! use ic_platform_runtime::cdk::storage::{stable_save, stable_restore};
//! ```

pub mod reject_code;

#[cfg(target_feature = "atomics")]
compile_error!("This version of the runtime does not support multithreading.");

/// Print with the original native/canister debug behavior.
pub use crate::__platform_println as println;
#[cfg(feature = "cdk")]
pub use ic_platform_macros::{
    heartbeat, init, inspect_message, on_low_wasm_memory, post_upgrade, pre_upgrade, query, update,
};

/// Explicit reviewed system adapters used by reviewed canisters and the generator.
///
/// Principal copies use bounded stack buffers; controller 0/1/panic behavior is retained.
/// Argument copying is not a substitute for consumer admission before allocation.
pub mod api;

/// Reviewed call builders, raw replies and preserved call failure semantics.
///
/// Encode and decode required external Candid messages through the separate
/// Candid boundary. Transport failures never erase typed codec errors.
pub mod call;

/// Stable-memory helpers with supplier-owned bounds and traps.
/// Consumers retain offsets, page admission, layout and rollback policy.
pub mod stable;

/// Task completion policies over the sole executor, including protected spawning.
pub mod futures;

/// Print using Platform's reviewed CDK debug adapter.
#[doc(hidden)]
#[macro_export]
macro_rules! __platform_println {
    ($fmt:expr) => {{
        #[cfg(target_family = "wasm")]
        $crate::cdk::api::debug_print(::std::format!($fmt));
        #[cfg(not(target_family = "wasm"))]
        ::std::println!($fmt);
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        #[cfg(target_family = "wasm")]
        $crate::cdk::api::debug_print(::std::format!($fmt, $($arg)*));
        #[cfg(not(target_family = "wasm"))]
        ::std::println!($fmt, $($arg)*);
    }};
}
