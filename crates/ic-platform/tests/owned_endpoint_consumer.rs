//! Explicit owned-codec endpoints, compiled without registry Candid or Serde derives.
use ic_platform::candid_codec::{decoder::Limits, endpoint};
const LIMITS: Limits = Limits {
    bytes: 1024,
    header: 512,
    types: 32,
    work: 10000,
    allocation: 10000,
};
const REPLY: usize = 1024;
fn permit() -> Result<(), String> {
    Ok(())
}
#[endpoint::query(
    limits = "LIMITS",
    reply_bytes = "REPLY",
    arity = "exact",
    empty_args = "reject",
    guard = "permit"
)]
fn scalar(value: u64) -> u64 {
    value + 1
}
#[endpoint::update(
    limits = "LIMITS",
    reply_bytes = "REPLY",
    arity = "compatible",
    empty_args = "unit"
)]
fn pair(value: u64, flag: bool) -> (u64, bool) {
    (value, flag)
}
#[endpoint::update(
    limits = "LIMITS",
    reply_bytes = "REPLY",
    arity = "exact",
    empty_args = "reject"
)]
fn zero() {}
type Null = ();
#[endpoint::query(
    limits = "LIMITS",
    reply_bytes = "REPLY",
    arity = "exact",
    empty_args = "reject"
)]
fn null() -> Null {}
#[endpoint::query(
    limits = "LIMITS",
    reply_bytes = "REPLY",
    arity = "exact",
    empty_args = "reject",
    composite
)]
async fn composite(value: u64) -> u64 {
    value
}
mod boundary {
    pub use ic_platform as platform;
}
#[endpoint::query(
    limits = "LIMITS",
    reply_bytes = "REPLY",
    arity = "exact",
    empty_args = "reject",
    platform = "crate::boundary::platform"
)]
fn renamed() -> u64 {
    3
}
#[test]
fn generated_adapters_preserve_argument_and_reply_arity() {
    use ic_platform::candid_codec::{
        arguments::encode_args,
        decoder::{Arity, decode_args},
    };
    let args = encode_args(&(42u64,), 1024).unwrap();
    assert_eq!(__platform_candid_decode_scalar(args), 42);
    assert_eq!(
        decode_args::<(u64,)>(&__platform_candid_encode_scalar(43), LIMITS, Arity::Exact).unwrap(),
        (43,)
    );
    let args = encode_args(&(42u64, true), 1024).unwrap();
    assert_eq!(__platform_candid_decode_pair(args.clone()), (42, true));
    assert_eq!(__platform_candid_encode_pair((42, true)), args);
    assert_eq!(
        __platform_candid_encode_zero(),
        encode_args(&(), 1024).unwrap()
    );
    assert_eq!(
        __platform_candid_encode_null(()),
        encode_args(&((),), 1024).unwrap()
    );
    assert_eq!(
        __platform_candid_encode_renamed(3),
        encode_args(&(3u64,), 1024).unwrap()
    );
    assert_eq!(
        __platform_candid_decode_composite(encode_args(&(7u64,), 1024).unwrap()),
        7
    );
}
