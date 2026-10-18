//! A standalone canister consuming Platform's generator and upstream runtime.
use ic_platform::{
    Principal,
    candid::contract::{self, CandidType, Nat, serde::Deserialize},
    cdk::{api, call::Call, futures},
};
use std::cell::Cell;

/// External DTO used by typed entrypoints and service extraction.
#[derive(Clone, CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
#[serde(crate = "ic_platform::candid::contract::serde")]
pub struct Message {
    /// Original caller-supplied principal.
    pub owner: Principal,
    /// Arbitrary-precision external amount.
    pub amount: Nat,
}

thread_local! {
    static VALUE: Cell<u64> = const { Cell::new(0) };
    static CLEANED: Cell<u64> = const { Cell::new(0) };
}

#[ic_platform::cdk::init]
fn initialize(value: u64) {
    VALUE.set(value);
}

#[ic_platform::cdk::post_upgrade]
fn after_upgrade(value: u64) {
    VALUE.set(value);
}

#[ic_platform::cdk::query]
fn value() -> u64 {
    VALUE.get()
}

#[ic_platform::cdk::query]
fn scalar(input: u64) -> u64 {
    input + 1
}

#[ic_platform::cdk::query]
fn tuple() -> (u64, bool) {
    (7, true)
}

#[ic_platform::cdk::query]
fn unit() {}

#[ic_platform::cdk::query]
fn dto(message: Message) -> Message {
    message
}

#[ic_platform::cdk::update]
async fn awaited(input: u64) -> u64 {
    Call::bounded_wait(api::canister_self(), "scalar")
        .with_arg(input)
        .await
        .unwrap()
        .candid()
        .unwrap()
}

#[ic_platform::cdk::query(composite = true)]
async fn composite(input: u64) -> u64 {
    Call::unbounded_wait(api::canister_self(), "scalar")
        .with_arg(input)
        .await
        .unwrap()
        .candid()
        .unwrap()
}

fn refuse() -> Result<(), String> {
    Err("fixture guard refused".into())
}

#[ic_platform::cdk::query(guard = "refuse")]
fn guarded() -> u64 {
    999
}

fn decode_raw(bytes: Vec<u8>) -> Vec<u8> {
    bytes
}

fn encode_raw(bytes: Vec<u8>) -> Vec<u8> {
    bytes
}

#[ic_platform::cdk::query(decode_with = "decode_raw", encode_with = "encode_raw", hidden = true)]
fn custom(bytes: Vec<u8>) -> Vec<u8> {
    bytes
}

#[ic_platform::cdk::query(manual_reply = true, hidden = true)]
fn manual() {
    api::msg_reply(b"\0manual\xff");
}

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        if futures::is_recovering_from_trap() {
            CLEANED.set(CLEANED.get() + 1);
        }
    }
}

#[ic_platform::cdk::update]
async fn callback_trap() {
    let _cleanup = Cleanup;
    Call::unbounded_wait(api::canister_self(), "scalar")
        .with_arg(1u64)
        .await
        .unwrap();
    api::trap("fixture callback trap");
}

#[ic_platform::cdk::query]
fn cleaned() -> u64 {
    CLEANED.get()
}

#[ic_platform::cdk::update]
fn protected_pending() {
    futures::spawn(std::future::pending());
}

#[ic_platform::cdk::update]
fn migratory() {
    futures::spawn_migratory(async { VALUE.set(VALUE.get() + 1) });
}

#[ic_platform::cdk::query]
fn controller() -> bool {
    api::is_controller(&api::msg_caller())
}

#[ic_platform::cdk::update]
async fn missing() -> bool {
    matches!(
        Call::bounded_wait(api::canister_self(), "no_such_method")
            .with_raw_args(&contract::encode_args(()).unwrap())
            .await,
        Err(ic_platform::cdk::call::CallFailed::CallRejected(_))
    )
}

ic_platform::cdk::export_candid!();

/// Extract the service without invoking IC system APIs on a native host.
pub fn service() -> String {
    __export_service()
}

#[test]
fn service_preserves_original_cdk_schema() {
    assert_eq!(service(), include_str!("fixtures/cdk-service.did"));
}

thread_local! {
    static RECEIVED: Cell<u128> = const { Cell::new(0) };
}

// Hidden qualification endpoints do not change the fixed public service fixture.
#[ic_platform::cdk::update(manual_reply = true, hidden = true)]
fn cycle_receiver(limit: u128, mode: u8) {
    let available = api::msg_cycles_available();
    let accepted = api::msg_cycles_accept(limit);
    let remaining = api::msg_cycles_available();
    RECEIVED.set(RECEIVED.get() + accepted);
    match mode {
        1 => api::msg_reject("fixture cycle rejection"),
        2 => api::trap("fixture cycle receiver trap"),
        _ => api::msg_reply(contract::encode_args((available, accepted, remaining)).unwrap()),
    }
}

#[ic_platform::cdk::query(hidden = true)]
fn cycle_total() -> u128 {
    RECEIVED.get()
}

#[ic_platform::cdk::update(hidden = true)]
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
    let result = Call::unbounded_wait(receiver, "cycle_receiver")
        .with_args(&(limit, mode))
        .with_cycles(offered)
        .await;
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
                response.candid_tuple().unwrap();
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
