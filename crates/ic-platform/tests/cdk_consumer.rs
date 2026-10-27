//! Standalone Candid-free runtime ABI proof; installed RPC fixtures use Wire.
use ic_platform::cdk::{api, call};

#[ic_platform::cdk::query(decode_with = "decode_unit", manual_reply)]
fn fixed_reply() {
    api::msg_reply(42_u64.to_le_bytes());
}

#[ic_platform::cdk::query(decode_with = "decode_unit", manual_reply)]
fn static_reply() {
    api::msg_reply(b"explicit-static-codec");
}

fn encode_default<T: Default>() -> T {
    T::default()
}

// Keep the existing Vec expectation: widening encode_with would make this
// previously supported generic encoder ambiguous.
#[ic_platform::cdk::query(decode_with = "decode_unit", encode_with = "encode_default")]
fn generic_reply() {}

mod binding_collisions {
    type Vec = ();
    type String = ();
    type Result = ();

    fn decode(_: std::vec::Vec<u8>) -> (u64, u64) {
        (1, 2)
    }
    fn result(value: u64) -> std::vec::Vec<u8> {
        value.to_le_bytes().to_vec()
    }
    fn admit() -> std::result::Result<(), std::string::String> {
        let _: (Vec, String, Result) = ((), (), ());
        Ok(())
    }

    #[ic_platform::cdk::query(decode_with = "decode", encode_with = "result")]
    fn entry(entry: u64, __arg_entry: u64) -> u64 {
        entry + __arg_entry
    }

    #[ic_platform::cdk::query(decode_with = "decode", encode_with = "result")]
    fn arg_bytes((left, right): (u64, u64)) -> u64 {
        left + right
    }

    #[ic_platform::cdk::update(guard = "admit", decode_with = "decode", encode_with = "result")]
    async fn r((left, right): (u64, u64)) -> u64 {
        left + right
    }

    #[ic_platform::cdk::query(decode_with = "decode", encode_with = "result")]
    fn __platform_arg_0((left, right): (u64, u64)) -> u64 {
        left + right
    }
}

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

#[ic_platform::cdk::query(
    name = "native-name",
    decode_with = "decode_opaque",
    encode_with = "encode_opaque"
)]
fn native_hyphen(value: Opaque) -> Opaque {
    value
}

#[ic_platform::cdk::query(
    name = "native_name",
    decode_with = "decode_opaque",
    encode_with = "encode_opaque"
)]
fn native_underscore(value: Opaque) -> Opaque {
    value
}

#[ic_platform::cdk::query(decode_with = "decode_opaque", encode_with = "encode_opaque")]
fn r#match(value: Opaque) -> Opaque {
    value
}

#[ic_platform::cdk::query(decode_with = "missing_decoder", encode_with = "missing_encoder")]
#[cfg(any())]
fn disabled_after_macro(value: MissingType) -> MissingType {
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

#[test]
fn owned_reject_codes_preserve_numbers_and_diagnostics() {
    use ic_platform::cdk::call::RejectCode;
    for (raw, name) in [
        (1, "SYS_FATAL"),
        (2, "SYS_TRANSIENT"),
        (3, "DESTINATION_INVALID"),
        (4, "CANISTER_REJECT"),
        (5, "CANISTER_ERROR"),
        (6, "SYS_UNKNOWN"),
    ] {
        let code = RejectCode::try_from(raw).unwrap();
        assert_eq!(code as u64, raw);
        assert_eq!(code.to_string(), name);
    }
    for raw in [0, 7, u32::MAX as u64, u64::MAX] {
        assert_eq!(RejectCode::try_from(raw).unwrap_err().0, raw);
    }
}

#[test]
fn transport_failures_are_named_without_a_codec_error_union() {
    use ic_platform::cdk::call::{
        CallFailed, CallPerformFailed, CallRejected, OnewayError, RejectCode,
        UnrecognizedRejectCode,
    };
    let failure: CallFailed = CallPerformFailed.into();
    assert!(matches!(failure, CallFailed::CallPerformFailed(_)));
    let failure: OnewayError = CallPerformFailed.into();
    assert!(matches!(failure, OnewayError::CallPerformFailed(_)));
    let _: fn(&CallRejected) -> Result<RejectCode, UnrecognizedRejectCode> =
        CallRejected::reject_code;
}
