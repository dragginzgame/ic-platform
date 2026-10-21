//! Explicit codecs for a required external Candid interface, not owned Wire RPCs.
use ic_platform::{
    candid::{DecodeError, DecodeLimits, contract, decode_args},
    cdk,
};

const LIMITS: DecodeLimits = DecodeLimits {
    reply_bytes: 1024,
    decoding_quota: 10_000,
    skipping_quota: 0,
    type_entries: 8,
    header_bytes: 256,
};

fn admit_empty(bytes: &[u8]) -> Result<(), DecodeError> {
    decode_args::<()>(bytes, LIMITS)
}

fn decode_empty(bytes: Vec<u8>) {
    admit_empty(&bytes).unwrap_or_else(|error| cdk::api::trap(error.to_string()));
}

fn encode_reply(value: u64) -> Vec<u8> {
    contract::encode_args((value,)).unwrap_or_else(|error| cdk::api::trap(error.to_string()))
}

fn admit_caller() -> Result<(), String> {
    Ok(())
}

#[cdk::query(
    name = "external_status",
    guard = "admit_caller",
    decode_with = "decode_empty",
    encode_with = "encode_reply"
)]
#[contract::candid_method(query, rename = "external_status")]
fn external_status() -> u64 {
    7
}

contract::export_service!(ic_platform::candid::contract);

#[test]
fn external_request_reply_and_service_bytes() {
    let empty = b"DIDL\0\0";
    assert_eq!(contract::encode_args(()).unwrap(), empty);
    admit_empty(empty).unwrap();
    assert_eq!(
        encode_reply(external_status()),
        b"DIDL\0\x01\x78\x07\0\0\0\0\0\0\0"
    );
    assert_eq!(
        __export_service(),
        "service : { external_status : () -> (nat64) query }"
    );
}

#[test]
fn empty_external_requests_still_require_valid_bounded_candid() {
    for bytes in [
        Vec::new(),
        b"DIDL\0\0\x01".to_vec(),
        contract::encode_args((7_u64,)).unwrap(),
    ] {
        assert!(matches!(admit_empty(&bytes), Err(DecodeError::Candid(_))));
    }
    assert!(matches!(
        admit_empty(&vec![0; LIMITS.reply_bytes + 1]),
        Err(DecodeError::ReplyTooLarge { .. })
    ));
}
