//! Standalone consumer: no direct Candid, Serde or admission dependencies.
use ic_platform::candid::{
    DecodeError, DecodeLimits,
    arguments::{ArgumentLimits, EmptyArgs},
    contract, decode_args, decode_exactly_one, decode_one,
    nesting::NestingError,
    protocols::{ReplyArity, management::raw_rand},
};

const LIMITS: DecodeLimits = DecodeLimits {
    reply_bytes: 65536,
    decoding_quota: 1_000_000,
    skipping_quota: 1_000_000,
    type_entries: 4096,
    header_bytes: 32768,
};
#[derive(Debug, PartialEq, contract::CandidType, contract::serde::Deserialize)]
#[candid_path("ic_platform::candid::contract")]
#[serde(crate = "ic_platform::candid::contract::serde")]
struct ExternalRecord {
    id: contract::Principal,
    values: Vec<Option<u64>>,
}

/// Automatic depth admission leaves DTO identity, bytes and decoder arity intact.
pub fn qualify() {
    let value = ExternalRecord {
        id: contract::Principal::anonymous(),
        values: vec![None, Some(42)],
    };
    let bytes = contract::encode_one(&value).unwrap();
    let decoded: ExternalRecord = decode_exactly_one(&bytes, LIMITS).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(contract::encode_one(decoded).unwrap(), bytes);
    let extra = contract::encode_args((7u64, true)).unwrap();
    assert_eq!(decode_one::<u64>(&extra, LIMITS).unwrap(), 7);
    assert_eq!(decode_args::<(u64,)>(&extra, LIMITS).unwrap(), (7,));
    assert!(decode_exactly_one::<u64>(&extra, LIMITS).is_err());
    let blob = contract::encode_one(vec![1u8; 8192]).unwrap();
    assert_eq!(
        raw_rand::decode(&blob, LIMITS, ReplyArity::Exact)
            .unwrap()
            .len(),
        8192
    );

    let mut nested = b"DIDL\x01\x6e\0\x01\0".to_vec();
    nested.resize(nested.len() + 8192, 1);
    nested.push(0);
    let error = decode_exactly_one::<contract::Reserved>(&nested, LIMITS).unwrap_err();
    assert_eq!(
        error.nesting_error(),
        Some(&NestingError::ValueDepth { maximum: 16 })
    );
    let protocol_error = raw_rand::decode(&nested, LIMITS, ReplyArity::Exact).unwrap_err();
    assert_eq!(
        protocol_error.nesting_error(),
        Some(&NestingError::ValueDepth { maximum: 16 })
    );
    // Existing exhaustive matches remain source-compatible.
    match error {
        DecodeError::ReplyTooLarge { .. } => panic!("wrong refusal"),
        DecodeError::Candid(_) => (),
    }
    let args = ArgumentLimits {
        max_bytes: LIMITS.reply_bytes,
        decoding_quota: LIMITS.decoding_quota,
        skipping_quota: LIMITS.skipping_quota,
        max_type_len: LIMITS.type_entries,
        max_header_len: LIMITS.header_bytes,
    };
    let error = args
        .decode::<(contract::Reserved,)>(&nested, EmptyArgs::Reject)
        .unwrap_err();
    assert_eq!(
        error.nesting_error(),
        Some(&NestingError::ValueDepth { maximum: 16 })
    );
    args.decode::<()>(&[], EmptyArgs::Unit).unwrap();
    assert!(args.decode::<()>(&[], EmptyArgs::Reject).is_err());
}

#[test]
fn facade_only_nesting_admission() {
    qualify();
}
