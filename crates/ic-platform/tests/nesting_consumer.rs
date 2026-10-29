//! Standalone consumer: no direct Candid, Serde or admission dependencies.
use ic_platform::candid::{
    DecodeError, DecodeLimits,
    arguments::{ArgumentLimits, EmptyArgs},
    contract,
    nesting::{NestingError, NestingLimits},
    protocols::{ReplyArity, management::raw_rand},
};

const LIMITS: DecodeLimits = DecodeLimits {
    reply_bytes: 65536,
    decoding_quota: 1_000_000,
    skipping_quota: 1_000_000,
    type_entries: 4096,
    header_bytes: 32768,
};
const NESTING: NestingLimits = NestingLimits {
    depth: 64,
    work: 1_000_000,
};

#[derive(Debug, PartialEq, contract::CandidType, contract::serde::Deserialize)]
#[candid_path("ic_platform::candid::contract")]
#[serde(crate = "ic_platform::candid::contract::serde")]
struct ExternalRecord {
    id: contract::Principal,
    values: Vec<Option<u64>>,
}

/// Caller-selected admission leaves DTO identity, bytes and decoder arity intact.
pub fn qualify() {
    let policy = LIMITS.with_nesting(NESTING);
    let value = ExternalRecord {
        id: contract::Principal::anonymous(),
        values: vec![None, Some(42)],
    };
    let bytes = contract::encode_one(&value).unwrap();
    let decoded: ExternalRecord = policy.decode_exactly_one(&bytes).unwrap();
    assert_eq!(value, decoded);
    assert_eq!(contract::encode_one(decoded).unwrap(), bytes);
    let extra = contract::encode_args((7u64, true)).unwrap();
    assert_eq!(policy.decode_one::<u64>(&extra).unwrap(), 7);
    assert_eq!(policy.decode_args::<(u64,)>(&extra).unwrap(), (7,));
    assert!(policy.decode_exactly_one::<u64>(&extra).is_err());
    let blob = contract::encode_one(vec![1u8; 8192]).unwrap();
    policy.admit(&blob).unwrap();
    assert_eq!(
        raw_rand::decode(&blob, LIMITS, ReplyArity::Exact)
            .unwrap()
            .len(),
        8192
    );

    let mut nested = b"DIDL\x01\x6e\0\x01\0".to_vec();
    nested.resize(nested.len() + 8192, 1);
    nested.push(0);
    let error = policy
        .decode_exactly_one::<contract::Reserved>(&nested)
        .unwrap_err();
    assert_eq!(
        error.nesting_error(),
        Some(&NestingError::ValueDepth { maximum: 64 })
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
        .decode_with_nesting::<(contract::Reserved,)>(&nested, EmptyArgs::Reject, NESTING)
        .unwrap_err();
    assert_eq!(
        error.nesting_error(),
        Some(&NestingError::ValueDepth { maximum: 64 })
    );
    args.decode_with_nesting::<()>(&[], EmptyArgs::Unit, NESTING)
        .unwrap();
    assert!(
        args.decode_with_nesting::<()>(&[], EmptyArgs::Reject, NESTING)
            .is_err()
    );
}

#[test]
fn facade_only_nesting_admission() {
    qualify();
}
