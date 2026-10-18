//! Consumer imports only the boundary, including both derive macros.
use ic_platform::candid::{DecodeLimits, contract, decode_args, decode_exactly_one};

#[derive(Debug, PartialEq, contract::CandidType, contract::serde::Deserialize)]
#[candid_path("ic_platform::candid::contract")]
#[serde(crate = "ic_platform::candid::contract::serde")]
struct Reply {
    owner: contract::Principal,
    result: Result<u64, String>,
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
}

#[test]
fn boundary_derives_preserve_external_wire_bytes() {
    roundtrip();
}
