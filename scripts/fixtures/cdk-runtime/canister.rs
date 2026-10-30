//! A standalone Wire canister consuming Platform's owned runtime and generator.
#[cfg(feature = "external-candid")]
use ic_platform::candid::contract;
use ic_platform::{
    Principal,
    cdk::{api, futures},
};
use std::cell::Cell;
#[cfg(not(feature = "ic-call"))]
use transport::Execute as _;
#[cfg(feature = "external-candid")]
// Copied from the workspace-formatted management_consumer.rs by the runtime script.
#[rustfmt::skip]
mod management_proof;
mod transport;

#[cfg(feature = "external-candid")]
#[ic_platform::cdk::query(
    decode_with = "codec::decode::<_, { protocol::Method::ManagementProof as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::ManagementProof as u64 }>"
)]
fn management_proof() -> u8 {
    management_proof::roundtrip();
    management_proof::history_roundtrip();
    management_proof::canister_snapshot_status_roundtrip();
    management_proof::lifecycle_settings_and_ecdsa_contracts();
    4
}

thread_local! {
    static VALUE: Cell<u64> = const { Cell::new(0) };
    static CLEANED: Cell<u64> = const { Cell::new(0) };
}

// Compare just the message-buffer read, excluding Wire decoding/encoding.
// Both variants remain in this qualification fixture, not the production API.
#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::BufferCopy as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::BufferCopy as u64 }>"
)]
fn buffer_copy(baseline: bool, _payload: Vec<u8>) -> (u64, Vec<u8>) {
    let start = api::instruction_counter();
    let bytes = if baseline {
        let len = ic_platform::ic0::msg_arg_data_size();
        let mut bytes = vec![0; len];
        ic_platform::ic0::msg_arg_data_copy(&mut bytes, 0);
        bytes
    } else {
        api::msg_arg_data()
    };
    let instructions = api::instruction_counter() - start;
    (instructions, bytes)
}

#[ic_platform::cdk::init(
    decode_with = "codec::decode::<_, { protocol::Method::Initialize as u64 }>"
)]
fn initialize(value: u64) {
    VALUE.set(value);
    ic_timers::initialize_runtime().unwrap();
}

#[ic_platform::cdk::post_upgrade(
    decode_with = "codec::decode::<_, { protocol::Method::AfterUpgrade as u64 }>"
)]
fn after_upgrade(value: u64) {
    #[cfg(feature = "external-candid")]
    {
        // Fixture-owned legacy bytes: deliberately retain the old no-finalization
        // decoding policy. Platform no longer owns a checkpoint API.
        let mut bytes = vec![0; (api::stable_size() * 65_536) as usize];
        ic_platform::cdk::stable::stable_read(0, &mut bytes);
        let mut decoder = ic_platform::candid::decoder::IDLDeserialize::new(&bytes).unwrap();
        CHECKPOINT.set(contract::ArgumentDecoder::decode(&mut decoder).unwrap());
    }
    VALUE.set(value);
    ic_timers::initialize_runtime().unwrap();
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Value as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Value as u64 }>"
)]
fn value() -> u64 {
    VALUE.get()
}

// A query must drain protected wakes produced while its own future is polling.
#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::SelfWakeQuery as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::SelfWakeQuery as u64 }>"
)]
async fn self_wake_query() -> u64 {
    let mut polls = 0;
    let mut saved: Option<std::task::Waker> = None;
    std::future::poll_fn(|cx| {
        polls += 1;
        if let Some(first) = &saved {
            assert!(first.will_wake(cx.waker()));
            std::task::Poll::Ready(polls)
        } else {
            saved = Some(cx.waker().clone());
            for _ in 0..32 {
                cx.waker().wake_by_ref();
            }
            std::task::Poll::Pending
        }
    })
    .await
}

#[ic_platform::cdk::query(
    name = "scalar",
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Scalar as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Scalar as u64 }>"
)]
fn r#match(input: u64) -> u64 {
    input + 1
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Tuple as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Tuple as u64 }>"
)]
fn tuple() -> (u64, bool) {
    (7, true)
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Unit as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::Unit as u64 }>"
)]
fn unit() {}

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Awaited as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Awaited as u64 }>"
)]
async fn awaited(input: u64) -> u64 {
    let bytes = transport::unbounded_wait(api::canister_self(), "scalar")
        .with_wait_policy(ic_platform::cdk::call::WaitPolicy::Bounded {
            timeout_seconds: 300,
        })
        .take_raw_args(codec::request("scalar", &input))
        .execute()
        .await
        .unwrap()
        .into_bytes();
    codec::response("scalar", &bytes)
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    composite = true,
    decode_with = "codec::decode::<_, { protocol::Method::Composite as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Composite as u64 }>"
)]
async fn composite(input: u64) -> u64 {
    let bytes = transport::bounded_wait(api::canister_self(), "scalar")
        .with_wait_policy(ic_platform::cdk::call::WaitPolicy::Unbounded)
        .take_raw_args(codec::request("scalar", &input))
        .execute()
        .await
        .unwrap()
        .into_bytes();
    codec::response("scalar", &bytes)
}

fn refuse() -> Result<(), String> {
    Err("fixture guard refused".into())
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    guard = "refuse",
    decode_with = "codec::decode::<_, { protocol::Method::Guarded as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Guarded as u64 }>"
)]
fn guarded() -> u64 {
    999
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    manual_reply = true,
    decode_with = "codec::decode::<_, { protocol::Method::Manual as u64 }>"
)]
fn manual() {
    api::msg_reply(codec::encode::<_, { protocol::Method::Manual as u64 }>(
        vec![0u8, 255],
    ));
}

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        if futures::is_recovering_from_trap() {
            CLEANED.set(CLEANED.get() + 1);
        }
    }
}

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::CallbackTrap as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::CallbackTrap as u64 }>"
)]
async fn callback_trap() {
    let _cleanup = Cleanup;
    transport::unbounded_wait(api::canister_self(), "scalar")
        .take_raw_args(codec::request("scalar", &1u64))
        .execute()
        .await
        .unwrap();
    api::trap("fixture callback trap");
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Cleaned as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Cleaned as u64 }>"
)]
fn cleaned() -> u64 {
    CLEANED.get()
}

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::ProtectedPending as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::ProtectedPending as u64 }>"
)]
fn protected_pending() {
    futures::spawn(std::future::pending());
}

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Migratory as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::Migratory as u64 }>"
)]
fn migratory() {
    futures::spawn_migratory(async { VALUE.set(VALUE.get() + 1) });
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Controller as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Controller as u64 }>"
)]
fn controller() -> bool {
    api::is_controller(&api::msg_caller())
}

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Missing as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Missing as u64 }>"
)]
async fn missing() -> bool {
    // A successful transport can return bytes invalid for the chosen codec.
    // Do not reclassify that reply as a submission failure or retry permission.
    let reply = transport::bounded_wait(api::canister_self(), "manual")
        .take_raw_args(codec::request("manual", &()))
        .execute()
        .await
        .unwrap();
    assert!(
        ic_wire::native::decode_response::<()>(
            protocol::Method::Manual as u64,
            reply.as_ref(),
            protocol::LIMITS,
        )
        .is_err()
    );
    matches!(
        transport::bounded_wait(api::canister_self(), "no_such_method")
            .take_raw_args(codec::request("no_such_method", &()))
            .execute()
            .await,
        Err(ic_platform::cdk::call::CallFailed::CallRejected(_))
    )
}

thread_local! {
    static RECEIVED: Cell<u128> = const { Cell::new(0) };
}

// Hidden qualification endpoints do not change the fixed public service fixture.
#[ic_platform::cdk::update(
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

#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::CycleTotal as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::CycleTotal as u64 }>"
)]
fn cycle_total() -> u128 {
    RECEIVED.get()
}

#[ic_platform::cdk::update(
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
        Ok(_) | Err(ic_platform::cdk::call::CallFailed::CallRejected(_)) => {
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

#[test]
fn cycle_and_counter_helper_signatures() {
    let _: fn() -> u128 = api::msg_cycles_available;
    let _: fn(u128) -> u128 = api::msg_cycles_accept;
    let _: fn() -> u128 = api::msg_cycles_refunded;
    let _: fn() -> u64 = api::instruction_counter;
    let _: fn() -> u64 = api::call_context_instruction_counter;
}

thread_local! {
    static CHECKPOINT: Cell<(bool, bool, u64, u64)> = const { Cell::new((false, false, 0, 0)) };
}

#[ic_platform::cdk::pre_upgrade]
fn save_checkpoint() {
    #[cfg(feature = "external-candid")]
    {
        // This fixed fixture record fits one page. Preserve its original Candid
        // bytes without introducing a production checkpoint replacement.
        let bytes = contract::encode_args((true, false, VALUE.get(), CLEANED.get())).unwrap();
        assert!(bytes.len() <= 65_536);
        if api::stable_size() == 0 {
            ic_platform::cdk::stable::stable_grow(1).unwrap();
        }
        ic_platform::cdk::stable::stable_write(0, &bytes);
    }
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Checkpoint as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Checkpoint as u64 }>"
)]
fn checkpoint() -> (bool, bool, u64, u64) {
    CHECKPOINT.get()
}

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::StableEdit as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::StableEdit as u64 }>"
)]
fn stable_edit(value: u64, grow: bool, fail: bool) -> (u64, u64) {
    let before = api::stable_size();
    if before == 0 || grow {
        assert_eq!(ic_platform::cdk::stable::stable_grow(1).unwrap(), before);
    }
    ic_platform::cdk::stable::stable_write(0, &value.to_le_bytes());
    if fail {
        api::trap("fixture stable rollback");
    }
    (before, api::stable_size())
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::StableValue as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::StableValue as u64 }>"
)]
fn stable_value() -> (u64, u64) {
    let mut bytes = [0; 8];
    ic_platform::cdk::stable::stable_read(0, &mut bytes);
    (api::stable_size(), u64::from_le_bytes(bytes))
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::CheckpointBytes as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::CheckpointBytes as u64 }>"
)]
fn checkpoint_bytes() -> Vec<u8> {
    #[cfg(feature = "external-candid")]
    {
        let expected = contract::encode_args(CHECKPOINT.get()).unwrap();
        let mut actual = vec![0; expected.len()];
        ic_platform::cdk::stable::stable_read(0, &mut actual);
        assert_eq!(actual, expected);
        actual
    }
    #[cfg(not(feature = "external-candid"))]
    Vec::new()
}

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::PublishRoot as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::PublishRoot as u64 }>"
)]
fn publish_root(root: Vec<u8>) -> bool {
    api::certified_data_set(root);
    api::data_certificate().is_none()
}

#[ic_platform::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Certificate as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Certificate as u64 }>"
)]
fn certificate() -> Option<Vec<u8>> {
    api::data_certificate()
}

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::SystemHelpers as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::SystemHelpers as u64 }>"
)]
fn system_helpers() -> (u64, u128, bool) {
    assert!(!api::root_key().is_empty());
    assert_ne!(api::subnet_self(), Principal::anonymous());
    assert!(api::canister_cycle_balance() >= api::canister_liquid_cycle_balance());
    assert!(api::cost_create_canister() > 0);
    assert!(api::cost_http_request(100, 100) > 0);
    assert!(api::cost_sign_with_ecdsa("missing-fixture-key", 0).is_err());
    assert!(api::performance_counter(api::PerformanceCounterType::InstructionCounter) > 0);
    let previous = api::global_timer_set(0);
    api::global_timer_set(previous);
    api::debug_print("platform system fixture");
    ic_platform::cdk::println!("platform {}", "macro");
    (
        api::canister_version(),
        api::cycles_burn(123),
        api::data_certificate().is_none(),
    )
}

#[ic_platform::cdk::inspect_message]
fn inspect() {
    assert!(!api::msg_method_name().is_empty());
    api::accept_message();
}

#[cfg(feature = "external-candid")]
#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::ExternalManagement as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::ExternalManagement as u64 }>"
)]
async fn external_management() -> bool {
    let args = ic_platform::candid::contract::encode_args(()).unwrap();
    let response = transport::unbounded_wait(Principal::management_canister(), "raw_rand")
        .with_raw_args(&args)
        .execute()
        .await
        .unwrap();
    // Preserve this fixture's original upstream decode/arity policy explicitly.
    let bytes: Vec<u8> = ic_platform::candid::decoder::decode_one(response.as_ref()).unwrap();
    bytes.len() == 32
}

use ic_timers::{
    OnceDecision, OnceRunResult, TimerCompletion, TimerIdentity, TimerSchedule, TimerStart,
};
use std::time::Duration;

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::TimerStart as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::TimerStart as u64 }>"
)]
fn timer_start(trap: bool, cancel: bool) {
    let registration = ic_timers::register_once(
        TimerIdentity::try_new("platform", "qualification", "once").unwrap(),
        TimerStart::RemoveWhenStopped {
            initial: TimerSchedule::After(Duration::from_secs(1)),
        },
        move |_| async move {
            let _cleanup = Cleanup;
            transport::unbounded_wait(api::canister_self(), "scalar")
                .take_raw_args(codec::request("scalar", &1u64))
                .execute()
                .await
                .unwrap();
            if trap {
                api::trap("fixture timer callback trap");
            }
            VALUE.set(VALUE.get() + 1);
            OnceRunResult::new(TimerCompletion::no_work(), OnceDecision::Stop)
        },
    )
    .unwrap();
    if cancel {
        registration.cancel().unwrap();
    }
}

mod codec;
#[cfg(feature = "external-adapters")]
mod external_canister;
mod protocol;
mod wire_canister;
mod wire_protocol;

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::SubmissionPressure as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::SubmissionPressure as u64 }>"
)]
async fn submission_pressure() -> (u64, u64) {
    let mut calls: Vec<_> = (0..600)
        .map(|_| {
            Some(Box::pin(
                transport::unbounded_wait(api::canister_self(), "scalar")
                    .take_raw_args(codec::request("scalar", &1u64))
                    .execute(),
            ))
        })
        .collect();
    let (mut replies, mut refused) = (0u64, 0u64);
    std::future::poll_fn(move |cx| {
        for slot in &mut calls {
            let Some(call) = slot else { continue };
            match call.as_mut().poll(cx) {
                std::task::Poll::Ready(result) => {
                    match result {
                        Ok(_) => replies += 1,
                        Err(ic_platform::cdk::call::CallFailed::CallPerformFailed(_)) => {
                            refused += 1
                        }
                        Err(error) => panic!("unexpected submission error: {error}"),
                    }
                    *slot = None;
                }
                std::task::Poll::Pending => {}
            }
        }
        if calls.iter().all(Option::is_none) {
            std::task::Poll::Ready((replies, refused))
        } else {
            std::task::Poll::Pending
        }
    })
    .await
}

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::InsufficientCycles as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::InsufficientCycles as u64 }>"
)]
async fn insufficient_cycles() -> bool {
    matches!(transport::bounded_wait(api::canister_self(), "scalar")
        .take_raw_args(codec::request("scalar", &1u64))
        .with_cycles(api::canister_liquid_cycle_balance()).execute().await,
        Err(ic_platform::cdk::call::CallFailed::InsufficientLiquidCycleBalance(error))
            if error.available < error.required)
}

#[ic_platform::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::WeakPending as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::WeakPending as u64 }>"
)]
fn weak_pending() {
    futures::spawn_weak(std::future::pending());
}
