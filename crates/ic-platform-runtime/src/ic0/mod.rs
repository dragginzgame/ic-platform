//! Explicit IC 1.2.0 raw system calls used by Timers, Testkit and raw actors.
//!
//! Maintained bindings preserve the upstream safe/unsafe signatures, callback ABI
//! and trap behavior; function identities now belong to Platform.
//! Callers retain callback environments, byte admission, reply/call ownership,
//! cycle policy and executor contexts. No entrypoint, global timer, scheduler or
//! executor is installed here. `stable64_{size,grow,read,read_uninit,write}` are infrastructure
//! primitives with the upstream trap and growth-refusal semantics. Production
//! applications retain Storage governance; these bindings own no allocation or
//! access policy. Native compilation does not qualify IC execution.

// Reviewed imported ABI wrappers are the only unsafe implementation in this module.
// Unexposed upstream entrypoints remain private and can be removed by the linker.
#[expect(
    unsafe_code,
    reason = "Imported raw System API bindings preserve the reviewed IC ABI."
)]
#[expect(
    missing_docs,
    reason = "The selected upstream ABI wrappers retain undocumented exported functions."
)]
#[expect(
    dead_code,
    reason = "Only the explicitly exported subset of imported IC0 is public."
)]
pub(crate) mod upstream;

pub use upstream::{
    call_data_append, call_new, call_on_cleanup, call_perform, call_with_best_effort_response,
    canister_liquid_cycle_balance128, canister_self_copy, canister_self_size, canister_version,
    cost_call, debug_print, global_timer_set, in_replicated_execution, is_controller,
    msg_arg_data_copy, msg_arg_data_size, msg_caller_copy, msg_caller_size, msg_reject,
    msg_reject_code, msg_reply, msg_reply_data_append, performance_counter, stable64_grow,
    stable64_read, stable64_read_uninit, stable64_size, stable64_write, time, trap,
};
