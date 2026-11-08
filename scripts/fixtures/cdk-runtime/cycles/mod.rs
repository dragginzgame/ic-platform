//! Installed cycles qualification scenarios.
use super::*;

// Hidden qualification endpoints do not change the fixed public service fixture.
#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    manual_reply = true,
    decode_with = "codec::decode::<_, { protocol::Method::CycleReceiver as u64 }>"
)]
fn cycle_receiver(limit: u128, mode: u8) {
    let available = api::msg_cycles_available();
    let accepted = api::msg_cycles_accept(limit);
    let remaining = api::msg_cycles_available();
    RECEIVED.set(RECEIVED.get() + accepted);
    match mode {
        1 => api::msg_reject("fixture cycle rejection"),
        2 => api::trap("fixture cycle receiver trap"),
        _ => api::msg_reply(
            codec::encode::<_, { protocol::Method::CycleReceiver as u64 }>((
                available, accepted, remaining,
            )),
        ),
    }
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::CycleTotal as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::CycleTotal as u64 }>"
)]
fn cycle_total() -> u128 {
    RECEIVED.get()
}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::CycleRoundtrip as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::CycleRoundtrip as u64 }>"
)]
async fn cycle_roundtrip(
    receiver: Principal,
    offered: u128,
    limit: u128,
    mode: u8,
    trap_callback: bool,
) -> (bool, u128, Vec<u128>, Vec<u64>) {
    let _cleanup = Cleanup;
    let before_message = api::instruction_counter();
    let before_context = api::call_context_instruction_counter();
    let bytes = codec::request("cycle_receiver", &(limit, mode));
    let call = transport::unbounded_wait(receiver, "cycle_receiver")
        .with_raw_args(&bytes)
        .with_cycles(offered);
    assert_eq!(
        call.get_cost(),
        offered.saturating_add(api::cost_call(
            "cycle_receiver".len() as u64,
            bytes.len() as u64
        ))
    );
    let result = call.execute().await;
    // Match Blob's policy: only an actual reply/reject callback owns a refund.
    let refunded = match &result {
        Ok(_) | Err(ic_platform_runtime::cdk::call::CallFailed::CallRejected(_)) => {
            api::msg_cycles_refunded()
        }
        Err(error) => panic!("fixture did not reach a callback: {error}"),
    };
    let after_message = api::instruction_counter();
    let after_context = api::call_context_instruction_counter();
    if trap_callback {
        VALUE.set(999);
        api::trap("fixture cycle callback write trap");
    }
    let reply = match &result {
        Ok(response) => {
            let (available, accepted, remaining): (u128, u128, u128) =
                codec::response("cycle_receiver", response.as_ref());
            vec![available, accepted, remaining]
        }
        Err(_) => Vec::new(),
    };
    (
        result.is_ok(),
        refunded,
        reply,
        vec![before_message, before_context, after_message, after_context],
    )
}
