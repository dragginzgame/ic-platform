use super::*;
use crate::candid_codec::{
    decoder::{Arity, Error as DecodeError, Limits, decode_args, decode_one},
    encode_one,
};
use candid as reference;
fn limits() -> Limits {
    Limits {
        bytes: 1_000_000,
        header: 100_000,
        types: 1024,
        work: 10_000_000,
        allocation: 4_000_000,
    }
}
#[test]
fn zero_one_many_arguments_have_distinct_frames() {
    assert_eq!(encode_args(&(), 100).unwrap(), b"DIDL\0\0");
    assert_eq!(
        encode_args(&((),), 100).unwrap(),
        reference::encode_args(((),)).unwrap()
    );
    let values = (42u64, "héllo".to_owned(), Some(vec![1u8, 2, 3]));
    let bytes = encode_args(&values, 1000).unwrap();
    assert_eq!(bytes, reference::encode_args(values.clone()).unwrap());
    assert_eq!(
        decode_args::<(u64, String, Option<Vec<u8>>)>(&bytes, limits(), Arity::Exact).unwrap(),
        values
    );
    assert_ne!(
        encode_args(&(1u64, 2u64), 100).unwrap(),
        encode_one(&(1u64, 2u64), 100).unwrap()
    );
    assert_eq!(
        decode_one::<(u64, u64)>(&encode_one(&(1u64, 2u64), 100).unwrap(), limits()).unwrap(),
        (1, 2)
    );
}
#[test]
fn exact_and_compatible_arity_preserve_refusals() {
    let empty = reference::encode_args(()).unwrap();
    assert_eq!(
        decode_args::<(Option<u64>, ())>(&empty, limits(), Arity::Compatible).unwrap(),
        (None, ())
    );
    assert_eq!(
        decode_args::<(Option<u64>,)>(&empty, limits(), Arity::Exact),
        Err(DecodeError::Arity)
    );
    assert_eq!(
        decode_args::<(u64,)>(&empty, limits(), Arity::Compatible),
        Err(DecodeError::Type)
    );
    let extra = reference::encode_args((42u64, Some("extra"))).unwrap();
    assert_eq!(
        decode_args::<(u64,)>(&extra, limits(), Arity::Compatible).unwrap(),
        (42,)
    );
    assert_eq!(
        decode_args::<(u64,)>(&extra, limits(), Arity::Exact),
        Err(DecodeError::Arity)
    );
    let mut bad_extra = reference::encode_args((42u64, true)).unwrap();
    *bad_extra.last_mut().unwrap() = 2;
    assert_eq!(
        decode_args::<(u64,)>(&bad_extra, limits(), Arity::Compatible),
        Err(DecodeError::Malformed)
    );
    for n in 0..extra.len() {
        assert!(decode_args::<(u64,)>(&extra[..n], limits(), Arity::Compatible).is_err());
    }
}
#[test]
fn eight_argument_roundtrip_and_trailing_bytes() {
    let values = (
        (),
        true,
        1u8,
        -3i64,
        "hello".to_owned(),
        vec![2u32],
        Some(false),
        4.0f64,
    );
    let bytes = encode_args(&values, 1000).unwrap();
    type Values = ((), bool, u8, i64, String, Vec<u32>, Option<bool>, f64);
    assert_eq!(
        decode_args::<Values>(&bytes, limits(), Arity::Exact).unwrap(),
        values
    );
    assert_eq!(
        reference::decode_args_with_config::<Values>(&bytes, &reference::DecoderConfig::new())
            .unwrap(),
        values
    );
    let mut bad = bytes;
    bad.push(0);
    assert_eq!(
        decode_args::<Values>(&bad, limits(), Arity::Exact),
        Err(DecodeError::Trailing)
    );
}
