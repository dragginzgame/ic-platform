//! Owned non-cryptographic runtime adapters and explicit-codec entrypoints.
//!
//! `runtime` exposes this module without entrypoint macros; `cdk` adds them.
//! Both are Candid-free. Select `candid` explicitly for required external
//! management/system calls and consumer-owned legacy data. Controlled RPCs use
//! consumer-owned Wire hooks. Registry executor 2.0.0 and ic0 1.2.0 stay shared.
//! See the imported Apache-2.0 source provenance under `upstream/NOTICE.md`.
//!
//! The 0.5 hard cut retires the unused checkpoint API:
//! ```compile_fail
//! use ic_platform::cdk::storage::{stable_save, stable_restore};
//! ```

pub mod reject_code;
mod upstream;

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
pub mod api {
    pub use super::upstream::api::{
        PerformanceCounterType, SignCostError, accept_message, call_context_instruction_counter,
        canister_cycle_balance, canister_liquid_cycle_balance, canister_self, canister_version,
        certified_data_set, cost_call, cost_create_canister, cost_http_request,
        cost_sign_with_ecdsa, cycles_burn, data_certificate, debug_print, global_timer_set,
        in_replicated_execution, instruction_counter, is_controller, msg_arg_data, msg_caller,
        msg_cycles_accept, msg_cycles_available, msg_cycles_refunded, msg_method_name, msg_reject,
        msg_reply, performance_counter, root_key, stable_size, subnet_self, time, trap,
    };
}

/// Reviewed call builders, raw replies and preserved call failure semantics.
///
/// Encode and decode required external Candid messages through the separate
/// Candid boundary. Transport failures never erase typed codec errors.
pub mod call {
    pub use super::upstream::call::{
        Call, CallFailed, CallPerformFailed, CallRejected, InsufficientLiquidCycleBalance,
        OnewayError, RejectCode, Response, UnrecognizedRejectCode, WaitPolicy,
    };
}

/// Existing CDK stable-memory helpers, with upstream bounds, errors and traps.
/// Consumers retain offsets, page admission, layout and rollback policy.
pub mod stable {
    pub use super::upstream::stable::{StableMemoryError, stable_grow, stable_read, stable_write};
}

/// Imported protected spawning, without substituting raw executor protected tasks.
pub mod futures {
    pub use super::upstream::futures::{
        is_recovering_from_trap, spawn, spawn_migratory, spawn_weak,
    };

    /// Exact upstream contexts required by generated synchronous/asynchronous entries.
    ///
    /// These are an explicitly coupled generator/runtime contract. Do not install
    /// another executor context around an entrypoint or claim timer ownership.
    pub mod internals {
        pub use super::super::upstream::futures::internals::{
            in_executor_context, in_query_executor_context,
        };
    }
}

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
