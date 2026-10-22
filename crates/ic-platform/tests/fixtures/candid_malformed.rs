use ic_platform::candid::{DecodeLimits, decode_args, decode_exactly_one, decode_one};

pub fn exercise(bytes: &[u8]) {
    // Test-owned budgets, not a production default. Bound zero-width vectors too.
    let limits = DecodeLimits {
        reply_bytes: 512,
        decoding_quota: 2_000,
        skipping_quota: 200,
        type_entries: 16,
        header_bytes: 128,
    };
    let one = decode_one::<u64>(bytes, limits);
    let args = decode_args::<(u64,)>(bytes, limits);
    assert_eq!(one.as_ref().ok(), args.as_ref().ok().map(|v| &v.0));
    if let Ok(exact) = decode_exactly_one::<u64>(bytes, limits) {
        assert_eq!(one.unwrap(), exact);
    }
    let _ = decode_one::<Vec<()>>(bytes, limits);
    let _ = decode_args::<(Vec<u8>,)>(bytes, limits);
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
    let valid = ic_platform::candid::contract::encode_args((42_u64,)).unwrap();
    exercise(&valid);
    assert_eq!(
        decode_exactly_one::<u64>(
            &valid,
            DecodeLimits {
                reply_bytes: 512,
                decoding_quota: 2_000,
                skipping_quota: 200,
                type_entries: 16,
                header_bytes: 128,
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
