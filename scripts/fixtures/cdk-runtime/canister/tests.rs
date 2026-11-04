use super::*;

#[test]
fn cycle_and_counter_helper_signatures() {
    let _: fn() -> u128 = api::msg_cycles_available;
    let _: fn(u128) -> u128 = api::msg_cycles_accept;
    let _: fn() -> u128 = api::msg_cycles_refunded;
    let _: fn() -> u64 = api::instruction_counter;
    let _: fn() -> u64 = api::call_context_instruction_counter;
}
