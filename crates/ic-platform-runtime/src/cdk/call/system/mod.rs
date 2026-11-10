//! Private supplier selection; native unit tests replace only this boundary.

#[cfg(not(test))]
pub(super) use crate::cdk::api::{
    canister_liquid_cycle_balance, cost_call, msg_arg_data, msg_reject_code, msg_reject_msg, trap,
};
#[cfg(not(test))]
pub(super) use crate::ic0::upstream::{
    call_cycles_add128, call_data_append, call_new, call_new_oneway, call_on_cleanup, call_perform,
    call_with_best_effort_response,
};

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(super) use tests::*;
