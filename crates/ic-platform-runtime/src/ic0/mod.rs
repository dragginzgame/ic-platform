//! Explicit IC 1.2.0 raw system calls used by Timers, Testkit and raw actors.
//!
//! Maintained bindings preserve the upstream safe/unsafe signatures, callback ABI
//! and trap behavior; function identities now belong to Platform.
//! Callers retain callback environments, byte admission, reply/call ownership,
//! cycle policy and executor contexts. No entrypoint, global timer, scheduler or
//! executor is installed here. `stable64_size` observes pages; this boundary does
//! not allocate or own storage. Native compilation does not qualify IC execution.

// Reviewed imported ABI wrappers are the only unsafe implementation in this module.
// Unexposed upstream entrypoints remain private and can be removed by the linker.
#[allow(unsafe_code, missing_docs, dead_code)]
pub(crate) mod upstream;

pub use upstream::{
    call_data_append, call_new, call_on_cleanup, call_perform, call_with_best_effort_response,
    canister_liquid_cycle_balance128, canister_self_copy, canister_self_size, canister_version,
    cost_call, debug_print, global_timer_set, in_replicated_execution, is_controller,
    msg_arg_data_copy, msg_arg_data_size, msg_caller_copy, msg_caller_size, msg_reject,
    msg_reject_code, msg_reply, msg_reply_data_append, performance_counter, stable64_size, time,
    trap,
};
