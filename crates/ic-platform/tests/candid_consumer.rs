//! Consumer imports only the boundary, including both derive macros.
use ic_platform::candid::{DecodeLimits, contract, decode_args, decode_exactly_one};

#[derive(Debug, PartialEq, contract::CandidType, contract::serde::Deserialize)]
#[candid_path("ic_platform::candid::contract")]
#[serde(crate = "ic_platform::candid::contract::serde")]
struct Reply {
    owner: contract::Principal,
    result: Result<u64, String>,
}

// Ledger values are arbitrary precision; governance projections skip fields
// using Reserved. The renamed variant and recursive record exercise derive paths.
#[derive(Debug, PartialEq, contract::CandidType, contract::serde::Deserialize)]
#[candid_path("ic_platform::candid::contract")]
#[serde(crate = "ic_platform::candid::contract::serde")]
enum LedgerValue {
    Nat(contract::Nat),
    Int(contract::Int),
    #[serde(rename = "record")]
    Record(Vec<(String, LedgerValue)>),
}

fn encode<A: contract::ArgumentEncoder>(args: A) -> Result<Vec<u8>, contract::Error> {
    contract::encode_args(args)
}

fn decode<T: for<'de> contract::ArgumentDecoder<'de>>(
    bytes: &[u8],
    limits: DecodeLimits,
) -> Result<T, ic_platform::candid::DecodeError> {
    decode_args(bytes, limits)
}

/// Exercise the complete DTO and decoder path on native and Wasm builds.
pub fn roundtrip() {
    let reply = Reply {
        owner: contract::Principal::anonymous(),
        result: Ok(42),
    };
    let bytes = contract::encode_one(&reply).unwrap();
    let limits = DecodeLimits {
        reply_bytes: 1024,
        decoding_quota: 100_000,
        skipping_quota: 1000,
        type_entries: 32,
        header_bytes: 512,
    };
    assert_eq!(decode_exactly_one::<Reply>(&bytes, limits).unwrap(), reply);
    assert_eq!(decode_args::<(Reply,)>(&bytes, limits).unwrap(), (reply,));

    // Independent Candid wire vectors: one nat 2^128 and one int -65.
    // This catches accidental narrowing to Rust fixed-width integer wire types.
    let big: contract::Nat = "340282366920938463463374607431768211456".parse().unwrap();
    let mut nat_wire = b"DIDL\x00\x01\x7d".to_vec();
    nat_wire.extend([0x80; 18]);
    nat_wire.push(4);
    assert_eq!(encode((&big,)).unwrap(), nat_wire);
    assert_eq!(
        decode::<(contract::Nat,)>(&nat_wire, limits).unwrap(),
        (big.clone(),)
    );
    let signed = contract::Int::from(-65);
    let int_wire = b"DIDL\x00\x01\x7c\xbf\x7f";
    assert_eq!(contract::encode_one(&signed).unwrap(), int_wire);
    assert_eq!(
        decode_exactly_one::<contract::Int>(int_wire, limits).unwrap(),
        signed
    );

    let value = LedgerValue::Record(vec![
        ("amount".into(), LedgerValue::Nat(big)),
        ("delta".into(), LedgerValue::Int(signed)),
    ]);
    let bytes = encode((&value,)).unwrap();
    assert_eq!(decode::<(LedgerValue,)>(&bytes, limits).unwrap(), (value,));
    decode_exactly_one::<contract::Reserved>(&bytes, limits).unwrap();
    let error = decode::<(LedgerValue,)>(b"not candid", limits).unwrap_err();
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<contract::Error>()
            .is_some()
    );
}

#[test]
fn boundary_derives_preserve_external_wire_bytes() {
    roundtrip();
}
