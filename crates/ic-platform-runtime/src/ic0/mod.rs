//! Explicit IC 1.2.0 raw system calls used by Timers, Testkit and raw actors.
//!
//! Explicit exports from the registry IC0 supplier preserve its safe/unsafe
//! signatures, callback ABI and trap behavior. Platform maintains no copied bindings.
//! Callers retain callback environments, byte admission, reply/call ownership,
//! cycle policy and executor contexts. No entrypoint, global timer, scheduler or
//! executor is installed here. `stable64_{size,grow,read,read_uninit,write}` are infrastructure
//! primitives with the upstream trap and growth-refusal semantics. Production
//! applications retain Storage governance; these bindings own no allocation or
//! access policy. Native compilation does not qualify IC execution.

// Keep the internal adapter path without exposing the supplier's entire API.
pub(crate) use ::ic0 as upstream;

pub use upstream::{
    call_data_append, call_new, call_on_cleanup, call_perform, call_with_best_effort_response,
    canister_liquid_cycle_balance128, canister_self_copy, canister_self_size, canister_version,
    cost_call, debug_print, global_timer_set, in_replicated_execution, is_controller,
    msg_arg_data_copy, msg_arg_data_size, msg_caller_copy, msg_caller_size, msg_reject,
    msg_reject_code, msg_reply, msg_reply_data_append, performance_counter, stable64_grow,
    stable64_read, stable64_read_uninit, stable64_size, stable64_write, time, trap,
};
