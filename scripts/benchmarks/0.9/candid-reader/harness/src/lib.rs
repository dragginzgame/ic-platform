use ic_platform::candid::{DecodeLimits, contract, decode_exactly_one};
use ic_platform::management_canister_types::CanisterStatusResult;

fn limits() -> DecodeLimits {
    DecodeLimits {
        reply_bytes: 1_000_000,
        decoding_quota: 1_000_000,
        skipping_quota: 1_000_000,
        type_entries: 10_000,
        header_bytes: 64 * 1024,
    }
}
fn input(bytes: Vec<u8>) -> Result<CanisterStatusResult, String> {
    decode_exactly_one(&bytes, limits()).map_err(|e| e.to_string())
}
fn output(result: Result<CanisterStatusResult, String>) -> Vec<u8> {
    match result {
        Ok(v) => contract::encode_one(v).unwrap(),
        Err(e) => e.into_bytes(),
    }
}
#[ic_platform::cdk::query(decode_with = "input", encode_with = "output")]
fn status(value: Result<CanisterStatusResult, String>) -> Result<CanisterStatusResult, String> {
    value
}
