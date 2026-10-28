//! Installed external-codec qualification; caller policies live in this fixture.
#[cfg(not(feature = "ic-call"))]
use crate::transport::Execute as _;
use ic_platform::{
    Principal,
    candid::{
        self, contract,
        endpoint::{self, ArgumentLimits},
        protocols::{self, ReplyArity},
    },
};
use std::cell::Cell;
thread_local! { static VALUE: Cell<u64> = const { Cell::new(0) }; }
const LIMITS: ArgumentLimits = ArgumentLimits {
    max_bytes: 1024,
    decoding_quota: 100000,
    skipping_quota: 1000,
    max_type_len: 32,
    max_header_len: 512,
};
#[endpoint::query(name = "external_observed", limits = "LIMITS", empty_args = "reject")]
fn observed() -> u64 {
    VALUE.get()
}
#[endpoint::update(name = "external_effect", limits = "LIMITS", empty_args = "reject")]
fn effect(value: u64) -> u64 {
    VALUE.set(value);
    value
}
fn refuse() -> Result<(), String> {
    Err("fixture access refused".into())
}
#[endpoint::update(
    name = "external_denied",
    limits = "LIMITS",
    empty_args = "reject",
    guard = "refuse"
)]
fn denied(value: u64) -> u64 {
    VALUE.set(value);
    value
}
struct Broken;
impl contract::CandidType for Broken {
    fn _ty() -> contract::types::Type {
        contract::types::TypeInner::Nat64.into()
    }
    fn idl_serialize<S: contract::types::Serializer>(&self, _: S) -> Result<(), S::Error> {
        Err(contract::serde::ser::Error::custom("fixture reply refused"))
    }
}
#[endpoint::update(
    name = "external_encoding_trap",
    limits = "LIMITS",
    empty_args = "reject"
)]
fn encoding_trap() -> Broken {
    VALUE.set(999);
    Broken
}
#[endpoint::update(name = "external_random", limits = "LIMITS", empty_args = "reject")]
async fn random() -> Vec<u8> {
    let bytes = protocols::management::raw_rand::encode().unwrap();
    let response = crate::transport::unbounded_wait(
        Principal::management_canister(),
        protocols::management::raw_rand::METHOD,
    )
    .take_raw_args(bytes)
    .execute()
    .await
    .unwrap();
    protocols::management::raw_rand::decode(
        response.as_ref(),
        candid::DecodeLimits {
            reply_bytes: 512,
            decoding_quota: 10000,
            skipping_quota: 0,
            type_entries: 8,
            header_bytes: 128,
        },
        ReplyArity::Exact,
    )
    .unwrap()
}
#[endpoint::update(name = "external_legacy_empty", limits = "LIMITS", empty_args = "unit")]
fn legacy_empty() {}

#[endpoint::update(name = "external_tuple", limits = "LIMITS", empty_args = "reject")]
fn tuple(value: u64, flag: bool) -> (u64, bool) {
    (value, flag)
}
#[endpoint::update(name = "external_zero", limits = "LIMITS", empty_args = "reject")]
fn zero() {}
type Null = ();
#[endpoint::query(name = "external_one_null", limits = "LIMITS", empty_args = "reject")]
fn one_null() -> Null {}
#[endpoint::query(
    name = "external_composite",
    limits = "LIMITS",
    empty_args = "reject",
    composite
)]
async fn composite(value: u64) -> u64 {
    value + 1
}

const NO_WORK: ArgumentLimits = ArgumentLimits {
    decoding_quota: 0,
    ..LIMITS
};
const NO_HEADER: ArgumentLimits = ArgumentLimits {
    max_header_len: 1,
    ..LIMITS
};
const NO_TYPES: ArgumentLimits = ArgumentLimits {
    max_type_len: 0,
    ..LIMITS
};
const NO_SKIP: ArgumentLimits = ArgumentLimits {
    skipping_quota: 0,
    ..LIMITS
};
#[endpoint::update(name = "external_no_work", limits = "NO_WORK", empty_args = "reject")]
fn no_work(value: u64) -> u64 {
    VALUE.set(value);
    value
}
#[endpoint::update(
    name = "external_no_header",
    limits = "NO_HEADER",
    empty_args = "reject"
)]
fn no_header(value: u64) -> u64 {
    VALUE.set(value);
    value
}
#[endpoint::update(name = "external_no_types", limits = "NO_TYPES", empty_args = "reject")]
fn no_types(value: Option<u64>) -> u64 {
    let value = value.unwrap_or(0);
    VALUE.set(value);
    value
}
#[endpoint::update(name = "external_no_skip", limits = "NO_SKIP", empty_args = "reject")]
fn no_skip(value: u64) -> u64 {
    VALUE.set(value);
    value
}
