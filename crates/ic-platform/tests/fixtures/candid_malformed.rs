//! Owned decoder malformed-input corpus shared by tests and the fuzz harness.
use ic_platform::candid_codec::{
    decoder::decode_args,
    decoder::{Arity, Limits as DecodeLimits, decode_one as decode_exactly_one},
};
fn decode_one<T: for<'a> ic_platform::candid_codec::decoder::Decode<'a>>(
    bytes: &[u8],
    limits: DecodeLimits,
) -> Result<T, ic_platform::candid_codec::decoder::Error> {
    decode_args::<(T,)>(bytes, limits, Arity::Compatible).map(|v| v.0)
}

/// Exercise typed decoding and validated skipping under fixture-owned budgets.
/// # Panics
/// Panics if a consumer contract assertion fails.
pub fn exercise(bytes: &[u8]) {
    // Test-owned budgets, not a production default. Bound zero-width vectors too.
    let limits = DecodeLimits {
        bytes: 512,
        work: 2_000,
        allocation: 200,
        types: 16,
        header: 128,
    };
    let one = decode_one::<u64>(bytes, limits);
    let args = decode_args::<(u64,)>(bytes, limits, Arity::Compatible);
    assert_eq!(one.as_ref().ok(), args.as_ref().ok().map(|v| &v.0));
    if let Ok(exact) = decode_exactly_one::<u64>(bytes, limits) {
        assert_eq!(one.unwrap(), exact);
    }
    let _ = decode_one::<Vec<()>>(bytes, limits);
    let _ = decode_args::<(Vec<u8>,)>(bytes, limits, Arity::Compatible);
    let _ = decode_exactly_one::<Vec<u8>>(bytes, limits);
}

#[test]
fn malformed_corpus_and_every_truncated_prefix() {
    // Recursive opt, recursive record, overlong LEB, oversized table, extra args.
    let corpus: &[&[u8]] = &[
        b"DIDL\x01\x6e\x00\x01\x00\x01",
        b"DIDL\x01\x6c\x01\x00\x00\x01\x00",
        b"DIDL\x80\x80\x80\x80\x80\x80\x80\x80\x80\x80\x00",
        b"DIDL\xff\xff\xff\xff\x0f",
        b"DIDL\x00\x02\x78\x78\x01\0\0\0\0\0\0\0\x02\0\0\0\0\0\0\0",
        b"DIDL\x01\x6d\x7f\x01\x00\xff\xff\xff\xff\x0f",
    ];
    for bytes in corpus {
        for length in 0..=bytes.len() {
            exercise(&bytes[..length]);
        }
    }
    let valid = ic_platform::candid_codec::arguments::encode_args(&(42_u64,), 64).unwrap();
    exercise(&valid);
    assert_eq!(
        decode_exactly_one::<u64>(
            &valid,
            DecodeLimits {
                bytes: 512,
                work: 2_000,
                allocation: 200,
                types: 16,
                header: 128,
            }
        )
        .unwrap(),
        42
    );
    for position in 0..valid.len() {
        for byte in 0..=255 {
            let mut mutated = valid.clone();
            mutated[position] = byte;
            exercise(&mutated);
        }
    }
}
