//! Platform-owned entrypoint generation with the qualified upstream CDK runtime.
//!
//! Generated runtime and Candid paths default to this module and Platform's
//! contract. Runtime calls, method contexts, protected spawning and traps retain
//! CDK 0.20.3's implementation and executor 2.0.0 identity. This feature selects
//! Candid and the exact-pinned `executor`/`ic0` capabilities; executor-only users
//! keep the independent `executor` capability.
//! Callers retain byte admission, bounded/unbounded waits, cycles and retries.

pub use ic_cdk::println;
pub use ic_platform_macros::{
    export_candid, heartbeat, init, inspect_message, on_low_wasm_memory, post_upgrade, pre_upgrade,
    query, update,
};

/// Explicit upstream system helpers used by reviewed canisters and the generator.
///
/// Principal/Vec construction and controller 0/1/panic behavior remain upstream.
/// Argument copying is not a substitute for consumer admission before allocation.
pub mod api {
    pub use ic_cdk::api::{
        PerformanceCounterType, SignCostError, accept_message, call_context_instruction_counter,
        canister_cycle_balance, canister_liquid_cycle_balance, canister_self, canister_version,
        certified_data_set, cost_call, cost_create_canister, cost_http_request,
        cost_sign_with_ecdsa, cycles_burn, data_certificate, debug_print, global_timer_set,
        in_replicated_execution, instruction_counter, is_controller, msg_arg_data, msg_caller,
        msg_cycles_accept, msg_cycles_available, msg_cycles_refunded, msg_method_name, msg_reject,
        msg_reply, performance_counter, root_key, stable_size, subnet_self, time, trap,
    };
}

/// Upstream call builders, raw replies and original call failure identities.
///
/// Candid response convenience methods retain upstream defaults; callers using
/// finite budgets must retain their explicit decoder and original reply bytes.
pub mod call {
    pub use ic_cdk::call::{
        Call, CallFailed, CallPerformFailed, CallRejected, CandidDecodeFailed, Error,
        InsufficientLiquidCycleBalance, RejectCode, Response,
    };
}

/// Existing CDK stable-memory helpers, with upstream bounds, errors and traps.
/// Consumers retain offsets, page admission, layout and rollback policy.
pub mod stable {
    pub use ic_cdk::stable::{StableMemoryError, stable_grow, stable_read, stable_write};
}

/// Existing CDK Candid checkpoint helpers; no new storage format or policy.
/// Restore retains upstream decoding behavior, including its lack of finalization.
pub mod storage {
    pub use ic_cdk::storage::{stable_restore, stable_save};
}

/// Upstream spawning, without substituting raw executor protected tasks.
pub mod futures {
    pub use ic_cdk::futures::{is_recovering_from_trap, spawn, spawn_migratory, spawn_weak};

    /// Exact upstream contexts required by generated synchronous/asynchronous entries.
    ///
    /// These are an explicitly coupled generator/runtime contract. Do not install
    /// another executor context around an entrypoint or claim timer ownership.
    pub mod internals {
        pub use ic_cdk::futures::internals::{in_executor_context, in_query_executor_context};
    }
}
