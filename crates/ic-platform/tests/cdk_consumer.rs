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
    CHECKPOINT.set(ic_platform::cdk::storage::stable_restore().unwrap());
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

/// Compile every reviewed signature in standalone native and Wasm consumers.
#[allow(clippy::type_complexity)] // Keep the exact upstream checkpoint signatures visible.
pub fn extended_system_and_error_contract() {
    use ic_platform::cdk::{call, stable, storage};
    let _: fn() -> u128 = api::canister_cycle_balance;
    let _: fn() -> u128 = api::canister_liquid_cycle_balance;
    let _: fn() -> u64 = api::canister_version;
    let _: fn() -> Principal = api::subnet_self;
    let _: fn() -> Vec<u8> = api::root_key;
    let _: fn() -> String = api::msg_method_name;
    let _: fn() -> u128 = api::cost_create_canister;
    let _: fn(u64, u64) -> u128 = api::cost_http_request;
    let _: fn(String, u32) -> Result<u128, api::SignCostError> = api::cost_sign_with_ecdsa;
    let _: fn(u32) -> u64 = api::performance_counter;
    let _: fn(api::PerformanceCounterType) -> u64 = api::performance_counter;
    let _: fn(u64) -> u64 = api::global_timer_set;
    let _: fn(u128) -> u128 = api::cycles_burn;
    let _: fn(String) = api::debug_print;
    let _: fn(Vec<u8>) = api::certified_data_set;
    let _: fn() -> Option<Vec<u8>> = api::data_certificate;
    let _: fn(u64) -> Result<u64, stable::StableMemoryError> = stable::stable_grow;
    let _: fn(u64, &[u8]) = stable::stable_write;
    let _: fn(u64, &mut [u8]) = stable::stable_read;
    let _: fn((bool, bool, u64, u64)) -> Result<(), contract::Error> = storage::stable_save;
    let _: fn() -> Result<(bool, bool, u64, u64), String> = storage::stable_restore;
    let error: call::Error = call::CallPerformFailed.into();
    assert!(matches!(error, call::Error::CallPerformFailed(_)));
    let rejected = call::CallRejected::with_rejection(3, "fixture missing destination".into());
    assert_eq!(
        rejected.reject_code().unwrap(),
        call::RejectCode::DestinationInvalid
    );
    assert!(matches!(
        call::CallFailed::from(rejected),
        call::CallFailed::CallRejected(_)
    ));
    let _ = std::mem::size_of::<call::CandidDecodeFailed>();
    let _ = std::mem::size_of::<call::InsufficientLiquidCycleBalance>();
    assert!(matches!(
        stable::StableMemoryError::OutOfMemory,
        stable::StableMemoryError::OutOfMemory
    ));
    assert!(matches!(
        stable::StableMemoryError::OutOfBounds,
        stable::StableMemoryError::OutOfBounds
    ));
    ic_platform::cdk::println!("Platform macro fixture: {}", 22);
}

#[test]
fn extended_contract_is_usable() {
    extended_system_and_error_contract();
}

thread_local! {
    static CHECKPOINT: Cell<(bool, bool, u64, u64)> = const { Cell::new((false, false, 0, 0)) };
}

#[ic_platform::cdk::pre_upgrade]
fn save_checkpoint() {
    ic_platform::cdk::storage::stable_save((true, false, VALUE.get(), CLEANED.get())).unwrap();
}

#[ic_platform::cdk::query(hidden = true)]
fn checkpoint() -> (bool, bool, u64, u64) {
    CHECKPOINT.get()
}

#[ic_platform::cdk::update(hidden = true)]
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

#[ic_platform::cdk::query(hidden = true)]
fn stable_value() -> (u64, u64) {
    let mut bytes = [0; 8];
    ic_platform::cdk::stable::stable_read(0, &mut bytes);
    (api::stable_size(), u64::from_le_bytes(bytes))
}

#[ic_platform::cdk::query(hidden = true)]
fn checkpoint_bytes() -> Vec<u8> {
    let expected = contract::encode_args(CHECKPOINT.get()).unwrap();
    let mut actual = vec![0; expected.len()];
    ic_platform::cdk::stable::stable_read(0, &mut actual);
    assert_eq!(actual, expected);
    actual
}

#[ic_platform::cdk::update(hidden = true)]
fn publish_root(root: Vec<u8>) -> bool {
    api::certified_data_set(root);
    api::data_certificate().is_none()
}

#[ic_platform::cdk::query(hidden = true)]
fn certificate() -> Option<Vec<u8>> {
    api::data_certificate()
}

#[ic_platform::cdk::update(hidden = true)]
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
