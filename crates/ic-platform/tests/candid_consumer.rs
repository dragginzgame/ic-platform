//! Consumer imports only the boundary, including both derive macros.
#[cfg(test)]
use ic_platform::candid::decoder::IDLDeserialize;
use ic_platform::candid::decoder::{
    DecoderConfig, decode_args_with_config, decode_one_with_config,
};
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
    use contract::{Decode, Encode};
    let bytes = Encode!(&42_u64).unwrap();
    assert_eq!(Decode!(&bytes, u64).unwrap(), 42);
    assert_eq!(
        ic_platform::candid::decoder::decode_one::<u64>(&bytes).unwrap(),
        42
    );
    assert_eq!(
        ic_platform::candid::decoder::decode_args::<(u64,)>(&bytes).unwrap(),
        (42,)
    );
    let config = DecoderConfig::new();
    let generated_config: contract::DecoderConfig = config.clone();
    assert_eq!(
        contract::decode_args_with_config::<(u64,)>(&bytes, &generated_config).unwrap(),
        (42,)
    );
    assert_eq!(decode_one_with_config::<u64>(&bytes, &config).unwrap(), 42);
    assert_eq!(
        decode_args_with_config::<(u64,)>(&bytes, &config).unwrap(),
        (42,)
    );
    let mut reflection = contract::types::internal::TypeContainer::new();
    let ty = reflection.add::<Reply>();
    let env: contract::TypeEnv = reflection.env;
    contract::types::subtype::equal(&mut std::collections::HashSet::new(), &env, &ty, &ty).unwrap();
    assert_eq!(contract::idl_hash("a"), 97);
    // Canic's narrowed reply projections require subtyping, not equality.
    use contract::types::{Field, Label, Type, TypeInner};
    let projected: Type = TypeInner::Record(vec![Field {
        id: Label::Named("owner".into()).into(),
        ty: TypeInner::Principal.into(),
    }])
    .into();
    contract::types::subtype::subtype(&mut std::collections::HashSet::new(), &env, &ty, &projected)
        .unwrap();
    assert!(
        contract::types::subtype::equal(
            &mut std::collections::HashSet::new(),
            &env,
            &ty,
            &projected,
        )
        .is_err()
    );
    let invalid: Type = TypeInner::Record(vec![Field {
        id: Label::Named("owner".into()).into(),
        ty: TypeInner::Nat64.into(),
    }])
    .into();
    let _: contract::Error = contract::types::subtype::subtype(
        &mut std::collections::HashSet::new(),
        &env,
        &ty,
        &invalid,
    )
    .unwrap_err();
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

#[cfg(test)]
#[derive(Debug, PartialEq)]
enum Stage {
    Decode,
    Trailing,
    Noncanonical,
    Limit,
}

// The Wire benchmark's caller-owned admission and stage mapping, on a scalar.
#[cfg(test)]
fn staged_value(bytes: &[u8]) -> Result<u64, Stage> {
    if bytes.len() > 5120 {
        return Err(Stage::Limit);
    }
    let mut config = DecoderConfig::new();
    config
        .set_decoding_quota(100_000)
        .set_skipping_quota(1000)
        .set_max_type_len(64)
        .set_max_header_len(1024);
    let value = {
        let mut decoder =
            IDLDeserialize::new_with_config(bytes, &config).map_err(|_| Stage::Decode)?;
        let value = decoder.get_value::<u64>().map_err(|_| Stage::Decode)?;
        decoder.done().map_err(|_| Stage::Trailing)?;
        value
    };
    if contract::encode_one(value).unwrap() != bytes {
        return Err(Stage::Noncanonical);
    }
    Ok(value)
}

#[test]
fn staged_decoding_preserves_caller_failures_and_optional_configuration() {
    let wire = contract::encode_one(42_u64).unwrap();
    assert_eq!(staged_value(&wire), Ok(42));
    assert_eq!(staged_value(b"bad header"), Err(Stage::Decode));
    assert_eq!(staged_value(&wire[..wire.len() - 1]), Err(Stage::Decode));
    let mut trailing = wire.clone();
    trailing.push(0);
    assert_eq!(staged_value(&trailing), Err(Stage::Trailing));
    let extra = contract::encode_args((42_u64, true)).unwrap();
    assert_eq!(staged_value(&extra), Err(Stage::Noncanonical));
    assert_eq!(
        ic_platform::candid::decoder::decode_one::<u64>(&extra).unwrap(),
        42
    );
    assert_eq!(
        ic_platform::candid::decoder::decode_args::<(u64,)>(&extra).unwrap(),
        (42,)
    );
    let missing = contract::encode_args(()).unwrap();
    assert_eq!(
        ic_platform::candid::decoder::decode_one::<Option<u64>>(&missing).unwrap(),
        None
    );
    let _: contract::Error =
        ic_platform::candid::decoder::decode_one::<u64>(b"bad header").unwrap_err();
    assert_eq!(staged_value(&vec![0; 5121]), Err(Stage::Limit));
    let mut config = DecoderConfig::new();
    config.set_full_error_message(false).set_skipping_quota(0);
    // Retain upstream header defaults by omitting that setter entirely.
    assert_eq!(decode_one_with_config::<u64>(&wire, &config).unwrap(), 42);
    assert!(decode_one_with_config::<u64>(&extra, &config).is_err());
    config.set_max_header_len(0);
    let _: contract::Error = decode_one_with_config::<u64>(&wire, &config).unwrap_err();
}

#[test]
fn owned_principal_converts_at_the_external_candid_boundary() {
    let owned = ic_platform::Principal::from_slice(&[2, 1]);
    let external: contract::Principal = owned.into();
    let bytes = contract::encode_one(external).unwrap();
    // DIDL, zero type-table entries, one principal argument, reference marker,
    // two significant bytes. This is external Candid, never a Wire payload.
    assert_eq!(bytes, [68, 73, 68, 76, 0, 1, 104, 1, 2, 2, 1]);
    let decoded: contract::Principal = ic_platform::candid::decoder::decode_one(&bytes).unwrap();
    assert_eq!(ic_platform::Principal::from(decoded), owned);
}
#[test]
fn borrowed_argument_encoding_keeps_original_external_bytes() {
    use ic_platform::candid::contract::{encode_args, encode_args_ref};
    let arguments = (42_u64, String::from("external"));
    assert_eq!(
        encode_args_ref(&arguments).unwrap(),
        encode_args(arguments.clone()).unwrap()
    );
    assert_eq!(arguments.1, "external");
}
/// Caller-selected sequence and reserved admission through Platform alone.
pub fn strict_admission() {
    use ic_platform::candid::{DecodeError, reserved::StrictReserved, sequence};
    fn bounded<'de, D: contract::serde::Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        sequence::deserialize::<D, u8, 2>(d, "bounded bytes", "too many bytes")
    }
    #[derive(contract::CandidType, contract::serde::Deserialize)]
    #[candid_path("ic_platform::candid::contract")]
    #[serde(crate = "ic_platform::candid::contract::serde")]
    struct StrictReply {
        #[serde(deserialize_with = "bounded")]
        bytes: Vec<u8>,
        marker: StrictReserved,
    }
    let limits = DecodeLimits {
        reply_bytes: 1024,
        decoding_quota: 10_000,
        skipping_quota: 0,
        type_entries: 8,
        header_bytes: 256,
    };
    let encode = |bytes| {
        contract::encode_one(StrictReply {
            bytes,
            marker: StrictReserved,
        })
        .unwrap()
    };
    let wire = encode(vec![1, 2]);
    assert_eq!(
        decode_exactly_one::<StrictReply>(&wire, limits)
            .unwrap()
            .bytes,
        [1, 2]
    );
    assert!(matches!(
        decode_exactly_one::<StrictReply>(&encode(vec![1, 2, 3]), limits),
        Err(DecodeError::Candid(_))
    ));
    assert!(matches!(
        decode_exactly_one::<StrictReply>(
            &wire,
            DecodeLimits {
                reply_bytes: 0,
                ..limits
            }
        ),
        Err(DecodeError::ReplyTooLarge { .. })
    ));
    assert!(matches!(
        decode_exactly_one::<StrictReply>(
            &wire,
            DecodeLimits {
                decoding_quota: 0,
                ..limits
            }
        ),
        Err(DecodeError::Candid(_))
    ));
    assert_eq!(
        contract::encode_one(StrictReserved).unwrap(),
        contract::encode_one(contract::Reserved).unwrap()
    );
    assert!(matches!(
        decode_exactly_one::<StrictReserved>(&contract::encode_one(7_u64).unwrap(), limits),
        Err(DecodeError::Candid(_))
    ));
}

#[test]
fn strict_consumer_admission() {
    strict_admission();
}

#[cfg(test)]
mod fixtures;
