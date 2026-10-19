//! Standalone Candid-free runtime ABI proof; installed RPC fixtures use Wire.
use ic_platform::cdk::{api, call};

struct Opaque(Vec<u8>);
fn decode_opaque(bytes: Vec<u8>) -> Opaque {
    Opaque(bytes)
}
fn encode_opaque(value: Opaque) -> Vec<u8> {
    value.0
}
fn decode_unit(bytes: Vec<u8>) {
    assert!(bytes.is_empty());
}

#[ic_platform::cdk::query(decode_with = "decode_opaque", encode_with = "encode_opaque")]
fn echo(value: Opaque) -> Opaque {
    value
}

#[ic_platform::cdk::update(decode_with = "decode_opaque", encode_with = "encode_opaque")]
async fn awaited(value: Opaque) -> Opaque {
    let response = call::Call::bounded_wait(api::canister_self(), "echo")
        .take_raw_args(value.0)
        .await
        .unwrap();
    Opaque(response.into_bytes())
}

#[ic_platform::cdk::init(decode_with = "decode_unit")]
fn init() {}

#[ic_platform::cdk::pre_upgrade]
fn before_upgrade() {}

#[test]
fn error_and_system_contracts() {
    use ic_platform::cdk::{futures, stable};
    let rejected = call::CallRejected::with_rejection(3, "missing destination".into());
    assert_eq!(rejected.raw_reject_code(), 3);
    assert_eq!(rejected.reject_message(), "missing destination");
    assert_eq!(
        rejected.reject_code().unwrap(),
        call::RejectCode::DestinationInvalid
    );
    assert!(matches!(
        call::CallFailed::from(rejected),
        call::CallFailed::CallRejected(_)
    ));
    assert!(
        call::CallRejected::with_rejection(999, "future code".into())
            .reject_code()
            .is_err()
    );
    let _: fn(u64) -> Result<u64, stable::StableMemoryError> = stable::stable_grow;
    let _: fn(u64, &[u8]) = stable::stable_write;
    let _: fn(u64, &mut [u8]) = stable::stable_read;
    assert_eq!(u32::from(api::PerformanceCounterType::from(17)), 17);
    assert_eq!(api::PerformanceCounterType::InstructionCounter, 0);
    let _: fn() -> bool = futures::is_recovering_from_trap;
    ic_platform::cdk::println!("native debug adapter");
}
