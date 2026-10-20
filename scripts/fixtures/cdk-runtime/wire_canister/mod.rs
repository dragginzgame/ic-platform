//! Qualify caller-owned Wire codecs with the original CDK lifecycle.

use crate::wire_protocol::*;
use ic_platform::cdk::{api, call::Call, futures};
use serde::{Serialize, de::DeserializeOwned};
use std::cell::Cell;

thread_local! {
    static EFFECTS: Cell<u64> = const { Cell::new(0) };
    static CLEANED: Cell<u64> = const { Cell::new(0) };
}

fn admit() -> Result<(), String> {
    if ic_platform::ic0::msg_arg_data_size() > REQUEST.bytes {
        return Err("Wire fixture request exceeds byte allowance".into());
    }
    Ok(())
}

fn decode<T: Serialize + DeserializeOwned, const ID: u64>(bytes: Vec<u8>) -> T {
    ic_wire::native::decode_request(ID, &bytes, request_limits(ID))
        .unwrap_or_else(|error| api::trap(format!("Wire fixture request: {error}")))
}

fn encode<T: Serialize, const ID: u64>(value: T) -> Vec<u8> {
    ic_wire::native::encode_response(ID, &value, RESPONSE)
        .unwrap_or_else(|error| api::trap(format!("Wire fixture response: {error}")))
}

#[ic_platform::cdk::query(
    guard = "admit",
    decode_with = "decode::<_, ECHO>",
    encode_with = "encode::<_, ECHO>"
)]
fn wire_echo(input: Vec<u8>) -> Vec<u8> {
    input
}

#[ic_platform::cdk::update(
    guard = "admit",
    decode_with = "decode::<_, SET>",
    encode_with = "encode::<_, SET>"
)]
fn wire_set(input: Vec<u8>) -> Vec<u8> {
    EFFECTS.set(EFFECTS.get() + 1);
    input
}

#[ic_platform::cdk::update(
    guard = "admit",
    decode_with = "decode::<_, TIGHT>",
    encode_with = "encode::<_, TIGHT>"
)]
fn wire_tight(input: Vec<u8>) -> Vec<u8> {
    EFFECTS.set(EFFECTS.get() + 1);
    input
}

#[ic_platform::cdk::update(
    guard = "admit",
    decode_with = "decode::<_, ALLOCATION>",
    encode_with = "encode::<_, ALLOCATION>"
)]
fn wire_allocation(input: Vec<u8>) -> Vec<u8> {
    EFFECTS.set(EFFECTS.get() + 1);
    input
}

#[ic_platform::cdk::update(
    guard = "admit",
    decode_with = "decode::<_, DEPTH>",
    encode_with = "encode::<_, DEPTH>"
)]
fn wire_depth(input: Vec<u8>) -> Vec<u8> {
    EFFECTS.set(EFFECTS.get() + 1);
    input
}

#[ic_platform::cdk::query(
    guard = "admit",
    decode_with = "decode::<_, OBSERVE>",
    encode_with = "encode::<_, OBSERVE>"
)]
fn wire_observe() -> (u64, u64) {
    (EFFECTS.get(), CLEANED.get())
}

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        if futures::is_recovering_from_trap() {
            CLEANED.set(CLEANED.get() + 1);
        }
    }
}

#[ic_platform::cdk::update(
    guard = "admit",
    decode_with = "decode::<_, AWAIT>",
    encode_with = "encode::<_, AWAIT>"
)]
async fn wire_await(input: Vec<u8>) -> Vec<u8> {
    let request = ic_wire::native::encode_request(ECHO, &input, REQUEST).unwrap();
    let response = Call::unbounded_wait(api::canister_self(), "wire_echo")
        .take_raw_args(request)
        .await
        .unwrap()
        .into_bytes();
    ic_wire::native::decode_response(ECHO, &response, RESPONSE).unwrap()
}

#[ic_platform::cdk::update(
    guard = "admit",
    decode_with = "decode::<_, ENCODE_TRAP>",
    encode_with = "encode::<_, ENCODE_TRAP>"
)]
async fn wire_encode_trap() -> Vec<u8> {
    let _cleanup = Cleanup;
    let _reply = Call::unbounded_wait(api::canister_self(), "wire_echo")
        .take_raw_args(ic_wire::native::encode_request(ECHO, &vec![1u8], REQUEST).unwrap())
        .await
        .unwrap();
    EFFECTS.set(EFFECTS.get() + 1);
    vec![0; RESPONSE.bytes + 1]
}
