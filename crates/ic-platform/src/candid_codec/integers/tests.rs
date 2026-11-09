use super::*;
use crate::candid_codec::{
    decoder::{Limits, decode_one},
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
fn natural(value: BigUint) {
    let wire = reference::encode_one(value.to_string().parse::<reference::Nat>().unwrap()).unwrap();
    assert_eq!(encode_one(&value, wire.len()).unwrap(), wire);
    assert_eq!(decode_one::<BigUint>(&wire, limits()).unwrap(), value);
    assert_eq!(
        decode_one::<BigInt>(&wire, limits()).unwrap(),
        BigInt::from(value)
    );
}
fn integer(value: BigInt) {
    let wire = reference::encode_one(value.to_string().parse::<reference::Int>().unwrap()).unwrap();
    assert_eq!(encode_one(&value, wire.len()).unwrap(), wire);
    assert_eq!(decode_one::<BigInt>(&wire, limits()).unwrap(), value);
    assert!(decode_one::<BigUint>(&wire, limits()).is_err());
}
#[test]
fn arbitrary_precision_boundaries_match_upstream() {
    natural(BigUint::from(0u8));
    integer(BigInt::from(0));
    for bit in [
        0usize, 6, 7, 8, 13, 14, 31, 32, 63, 64, 127, 128, 255, 256, 1024, 8192,
    ] {
        let power = BigUint::from(1u8) << bit;
        for value in [&power - 1u8, power.clone(), &power + 1u8] {
            natural(value.clone());
            integer(BigInt::from(value.clone()));
            integer(-BigInt::from(value));
        }
    }
}
#[test]
fn extended_integer_encodings_and_truncation() {
    for (opcode, payload) in [
        (0x7du8, vec![0x80, 0x80, 0]),
        (0x7c, vec![0xff, 0xff, 0x7f]),
        (0x7c, vec![0xc0, 0x80, 0]),
    ] {
        let mut bytes = vec![b'D', b'I', b'D', b'L', 0, 1, opcode];
        bytes.extend(payload);
        let expected = reference::decode_args_with_config::<(reference::Int,)>(
            &bytes,
            &reference::DecoderConfig::new(),
        )
        .unwrap()
        .0
        .0;
        assert_eq!(
            decode_one::<BigInt>(&bytes, limits()).unwrap().to_string(),
            expected.to_string()
        );
        for n in 0..bytes.len() {
            assert!(decode_one::<BigInt>(&bytes[..n], limits()).is_err());
        }
    }
}
#[test]
fn output_and_integer_allocation_budgets() {
    let value = BigUint::from(1u8) << 8192usize;
    assert_eq!(encode_one(&value, 10), Err(Error::Limit));
    assert_eq!(
        encode_one(&(-BigInt::from(value.clone())), 10),
        Err(Error::Limit)
    );
    let wire = encode_one(&value, 100_000).unwrap();
    let mut budget = limits();
    budget.allocation = 8;
    assert_eq!(
        decode_one::<BigUint>(&wire, budget),
        Err(crate::candid_codec::decoder::Error::Limit)
    );
}

#[test]
fn composite_size_workload_cross_decodes() {
    type Owned = (
        Option<std::result::Result<Vec<u64>, String>>,
        (BigUint, BigInt),
    );
    type Reference = (
        Option<std::result::Result<Vec<u64>, String>>,
        (reference::Nat, reference::Int),
    );
    let natural = BigUint::from(1u8) << 1024usize;
    let integer = -(BigInt::from(1) << 2048usize);
    for outcome in [
        None,
        Some(Ok(vec![])),
        Some(Ok(vec![0, 127, 128, u64::MAX])),
        Some(Err("denied 🌍".to_owned())),
    ] {
        let owned: Owned = (outcome.clone(), (natural.clone(), integer.clone()));
        let expected: Reference = (
            outcome,
            (
                natural.to_string().parse::<reference::Nat>().unwrap(),
                integer.to_string().parse::<reference::Int>().unwrap(),
            ),
        );
        let upstream_bytes = reference::encode_one(&expected).unwrap();
        assert_eq!(
            decode_one::<Owned>(&upstream_bytes, limits()).unwrap(),
            owned
        );
        let own_bytes = encode_one(&owned, 1_000_000).unwrap();
        let decoded = reference::decode_args_with_config::<(Reference,)>(
            &own_bytes,
            &reference::DecoderConfig::new(),
        )
        .unwrap()
        .0;
        assert_eq!(decoded, expected);
        for n in 0..upstream_bytes.len() {
            assert!(decode_one::<Owned>(&upstream_bytes[..n], limits()).is_err());
        }
    }
}

#[test]
#[ignore = "Explicit mixed-workload native throughput experiment."]
fn native_composite_throughput() {
    use std::{hint::black_box, time::Instant};
    type Owned = (
        Option<std::result::Result<Vec<u64>, String>>,
        (BigUint, BigInt),
    );
    let reference_limits = crate::tests::reference::DecodeLimits {
        reply_bytes: 1_000_000,
        header_bytes: 100_000,
        type_entries: 1024,
        decoding_quota: 10_000_000,
        skipping_quota: 10_000_000,
    };
    type Reference = (
        Option<std::result::Result<Vec<u64>, String>>,
        (reference::Nat, reference::Int),
    );
    println!("rows,integer_bits,iterations,sample,implementation,operation,total_ns");
    for (rows, bits) in [(0usize, 256usize), (16, 1024), (1024, 8192)] {
        let natural = BigUint::from(1u8) << bits;
        let integer = -BigInt::from(natural.clone()) - 1u8;
        let owned: Owned = (
            Some(Ok((0..rows as u64).collect())),
            (natural.clone(), integer.clone()),
        );
        let reference: Reference = (
            owned.0.clone(),
            (
                natural.to_string().parse::<reference::Nat>().unwrap(),
                integer.to_string().parse::<reference::Int>().unwrap(),
            ),
        );
        let bytes = reference::encode_one(&reference).unwrap();
        assert_eq!(decode_one::<Owned>(&bytes, limits()).unwrap(), owned);
        let own_bytes = encode_one(&owned, 1_000_000).unwrap();
        let (decoded,): (Reference,) =
            reference::decode_args_with_config(&own_bytes, &reference::DecoderConfig::new())
                .unwrap();
        assert_eq!(decoded, reference);
        for sample in 0..5 {
            for implementation in if sample % 2 == 0 { [0, 1] } else { [1, 0] } {
                let start = Instant::now();
                for _ in 0..1000 {
                    if implementation == 0 {
                        black_box(encode_one(black_box(&owned), 1_000_000).unwrap());
                    } else {
                        black_box(reference::encode_one(black_box(&reference)).unwrap());
                    }
                }
                println!(
                    "{rows},{bits},1000,{sample},{implementation},encode,{}",
                    start.elapsed().as_nanos()
                );
                let start = Instant::now();
                for _ in 0..1000 {
                    if implementation == 0 {
                        black_box(decode_one::<Owned>(black_box(&bytes), limits()).unwrap());
                    } else {
                        black_box(
                            crate::tests::reference::decode_one::<Reference>(
                                black_box(&bytes),
                                reference_limits,
                            )
                            .unwrap(),
                        );
                    }
                }
                println!(
                    "{rows},{bits},1000,{sample},{implementation},decode,{}",
                    start.elapsed().as_nanos()
                );
            }
        }
    }
}
