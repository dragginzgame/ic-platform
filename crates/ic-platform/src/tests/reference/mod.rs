//! Oracle framing and schema checks, with owned DTO decoding.
mod schema;
use crate::candid_codec::{
    Encode,
    decoder::{Decode, Limits},
};
#[cfg(feature = "icrc-ledger-types")]
pub(crate) use schema::assert_named_schema;
#[cfg(any(feature = "icrc-ledger-types", feature = "nns-registry-types"))]
pub(crate) use schema::frozen_schema;
pub(crate) use schema::schema;

#[derive(Clone, Copy)]
pub(crate) struct DecodeLimits {
    pub reply_bytes: usize,
    pub decoding_quota: usize,
    pub skipping_quota: usize,
    pub type_entries: usize,
    pub header_bytes: usize,
}
#[derive(Debug)]
pub(crate) enum DecodeError {
    ReplyTooLarge {},
    Candid(candid::Error),
}
#[cfg(any(feature = "icrc-ledger-types", feature = "nns-registry-types"))]
pub(crate) fn decode_hex(text: &str) -> Vec<u8> {
    let (pairs, remainder) = text.trim().as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    pairs
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReplyTooLarge {} => f.write_str("oracle input exceeds fixture limit"),
            Self::Candid(e) => e.fmt(f),
        }
    }
}

/// Independently parse and re-encode payloads using the complete declared schema.
/// This checks canonical bytes, not an independently constructed application value.
pub(crate) fn oracle_bytes<T: Encode>(value: &T) -> Vec<u8> {
    let bytes = crate::candid_codec::encode_one(value, 1_000_000).unwrap();
    let (env, ty) = schema::<T>();
    let values =
        candid::IDLArgs::from_bytes_with_types(&bytes, &env, std::slice::from_ref(&ty)).unwrap();
    let reference = values.to_bytes_with_types(&env, &[ty]).unwrap();
    assert_eq!(bytes, reference);
    reference
}
pub(crate) fn limits() -> Limits {
    Limits {
        bytes: 1_000_000,
        header: 65536,
        types: 1024,
        work: 10_000_000,
        allocation: 10_000_000,
    }
}
pub(crate) fn roundtrip<T: Encode + for<'a> Decode<'a> + std::fmt::Debug + PartialEq>(
    value: &T,
) -> Vec<u8> {
    let bytes = oracle_bytes(value);
    assert_eq!(
        &crate::candid_codec::decoder::decode_one::<T>(&bytes, limits()).unwrap(),
        value
    );
    for end in 0..bytes.len() {
        assert!(crate::candid_codec::decoder::decode_one::<T>(&bytes[..end], limits()).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(crate::candid_codec::decoder::decode_one::<T>(&trailing, limits()).is_err());
    bytes
}

pub(crate) fn decode_one<'a, T: candid::CandidType + serde::Deserialize<'a>>(
    bytes: &'a [u8],
    limits: DecodeLimits,
) -> Result<T, DecodeError> {
    if bytes.len() > limits.reply_bytes {
        return Err(DecodeError::ReplyTooLarge {});
    }
    let mut config = candid::de::DecoderConfig::new();
    config
        .set_decoding_quota(limits.decoding_quota)
        .set_skipping_quota(limits.skipping_quota)
        .set_max_type_len(limits.type_entries)
        .set_max_header_len(limits.header_bytes);
    candid::utils::decode_one_with_config(bytes, &config).map_err(DecodeError::Candid)
}
