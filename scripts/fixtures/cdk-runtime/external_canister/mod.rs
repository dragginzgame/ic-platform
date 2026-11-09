//! Installed external-codec qualification; caller policies live in this fixture.
#[cfg(not(feature = "prepared-call"))]
use crate::transport::Execute as _;
use ic_platform::{
    Principal,
    candid_codec::{
        self, Encode, Error, Reserved, Type, Writer,
        decoder::{Limits, decode_one},
        endpoint,
    },
};
use std::cell::Cell;
thread_local! { static VALUE: Cell<u64> = const { Cell::new(0) }; }
const REPLY_BYTES: usize = 65536;
const LIMITS: Limits = Limits {
    bytes: 1024,
    work: 100000,
    allocation: 1000,
    types: 32,
    header: 512,
};
#[endpoint::query(
    name = "external_observed",
    limits = "LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
fn observed() -> u64 {
    VALUE.get()
}
#[endpoint::update(
    name = "external_effect",
    limits = "LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
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
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject",
    guard = "refuse"
)]
fn denied(value: u64) -> u64 {
    VALUE.set(value);
    value
}
struct Broken;
impl Encode for Broken {
    const TYPE: &'static Type = &Type::Primitive(-8);
    fn write(&self, _: &mut Writer) -> Result<(), Error> {
        Err(Error::Schema)
    }
}
#[endpoint::update(
    name = "external_encoding_trap",
    limits = "LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
fn encoding_trap() -> Broken {
    VALUE.set(999);
    Broken
}
#[endpoint::update(
    name = "external_random",
    limits = "LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
async fn random() -> Vec<u8> {
    let response = crate::transport::unbounded_wait(
        Principal::management_canister(),
        ic_platform::raw_rand::METHOD,
    )
    .with_raw_args(ic_platform::raw_rand::encode().as_slice())
    .execute()
    .await
    .unwrap();
    ic_platform::raw_rand::decode(response.as_ref())
        .unwrap()
        .to_vec()
}
#[endpoint::update(
    name = "external_legacy_empty",
    limits = "LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "unit"
)]
fn legacy_empty() {}

#[endpoint::update(
    name = "external_tuple",
    limits = "LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
fn tuple(value: u64, flag: bool) -> (u64, bool) {
    (value, flag)
}
#[endpoint::update(
    name = "external_zero",
    limits = "LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
fn zero() {}
type Null = ();
#[endpoint::query(
    name = "external_one_null",
    limits = "LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
fn one_null() -> Null {}
#[endpoint::query(
    name = "external_composite",
    limits = "LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject",
    composite
)]
async fn composite(value: u64) -> u64 {
    value + 1
}

const NO_WORK: Limits = Limits { work: 0, ..LIMITS };
const NO_HEADER: Limits = Limits {
    header: 1,
    ..LIMITS
};
const NO_TYPES: Limits = Limits { types: 0, ..LIMITS };
#[endpoint::update(
    name = "external_no_work",
    limits = "NO_WORK",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
fn no_work(value: u64) -> u64 {
    VALUE.set(value);
    value
}
#[endpoint::update(
    name = "external_no_header",
    limits = "NO_HEADER",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
fn no_header(value: u64) -> u64 {
    VALUE.set(value);
    value
}
#[endpoint::update(
    name = "external_no_types",
    limits = "NO_TYPES",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
fn no_types(value: Option<u64>) -> u64 {
    let value = value.unwrap_or(0);
    VALUE.set(value);
    value
}
#[endpoint::update(
    name = "external_no_surplus",
    limits = "LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "exact",
    empty_args = "reject"
)]
fn no_surplus(value: u64) -> u64 {
    VALUE.set(value);
    value
}

const NESTED_LIMITS: Limits = Limits {
    bytes: 65536,
    work: 1_000_000,
    allocation: 1_000_000,
    types: 4096,
    header: 65536,
};
#[endpoint::query(
    name = "external_nesting_probe",
    limits = "NESTED_LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
fn nesting_probe(bytes: Vec<u8>) -> u8 {
    let limits = NESTED_LIMITS;
    match decode_one::<Reserved>(&bytes, limits) {
        Ok(_) => 0,
        Err(candid_codec::decoder::Error::Limit) => 1,
        Err(_) => 3,
    }
}
#[endpoint::update(
    name = "external_nesting_effect",
    limits = "NESTED_LIMITS",
    reply_bytes = "REPLY_BYTES",
    arity = "compatible",
    empty_args = "reject"
)]
fn nesting_effect(_: Reserved) -> u64 {
    VALUE.set(555);
    555
}
