//! Platform-owned entrypoint generation with the qualified upstream CDK runtime.
//!
//! Generated runtime and Candid paths default to this module and Platform's
//! contract. Runtime calls, method contexts, protected spawning and traps retain
//! CDK 0.20.3's implementation and executor 2.0.0 identity. This feature selects
//! Candid and the exact-pinned `executor`/`ic0` capabilities; executor-only users
//! keep the independent `executor` capability.
//! Callers retain byte admission, bounded/unbounded waits, cycles and retries.

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
        accept_message, call_context_instruction_counter, canister_self, cost_call,
        in_replicated_execution, instruction_counter, is_controller, msg_arg_data, msg_caller,
        msg_cycles_accept, msg_cycles_available, msg_cycles_refunded, msg_reject, msg_reply, time,
        trap,
    };
}

/// Upstream call builders, raw replies and original call failure identities.
///
/// Candid response convenience methods retain upstream defaults; callers using
/// finite budgets must retain their explicit decoder and original reply bytes.
pub mod call {
    pub use ic_cdk::call::{Call, CallFailed, Response};
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
