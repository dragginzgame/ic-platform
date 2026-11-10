//! One development-only catalogue for correctness and explicit native timings.
use crate::candid_codec::{
    Encode,
    decoder::{Arity, Decode, Error, Limits, decode_args, decode_one},
    encode_one,
};
use candid as reference;
use std::{collections::BTreeMap, fmt::Debug, hint::black_box, time::Instant};

fn limits() -> Limits {
    Limits {
        bytes: 1_000_000,
        header: 100_000,
        types: 1024,
        work: 10_000_000,
        allocation: 4_000_000,
    }
}

fn reference_limits() -> reference::DecoderConfig {
    let mut limits = reference::DecoderConfig::new();
    limits
        .set_decoding_quota(10_000_000)
        .set_skipping_quota(10_000_000)
        .set_max_type_len(1024)
        .set_max_header_len(100_000);
    limits
}

fn time(
    name: &str,
    bytes: usize,
    operation: &str,
    mut owned: impl FnMut(),
    mut upstream: impl FnMut(),
) {
    let iterations = (1_048_576 / bytes.max(1)).clamp(16, 1000);
    for sample in 0..5 {
        for implementation in if sample % 2 == 0 { [0, 1] } else { [1, 0] } {
            let start = Instant::now();
            for _ in 0..iterations {
                if implementation == 0 {
                    owned();
                } else {
                    upstream();
                }
            }
            println!(
                "{name},{bytes},{iterations},{sample},{implementation},{operation},{}",
                start.elapsed().as_nanos()
            );
        }
    }
}

fn pair<O, R>(name: &str, owned: &O, upstream: &R, measure: bool)
where
    O: Encode + for<'a> Decode<'a> + PartialEq + Debug,
    R: reference::CandidType + for<'a> serde::Deserialize<'a> + PartialEq + Debug,
{
    let bytes = reference::encode_one(upstream).unwrap();
    let own_bytes = encode_one(owned, limits().bytes).unwrap();
    assert_eq!(own_bytes, bytes, "{name}: independent encoder bytes");
    assert_eq!(&decode_one::<O>(&bytes, limits()).unwrap(), owned, "{name}");
    let (decoded,): (R,) =
        reference::decode_args_with_config(&own_bytes, &reference_limits()).unwrap();
    assert_eq!(&decoded, upstream, "{name}");
    assert_eq!(encode_one(owned, bytes.len()).unwrap(), bytes, "{name}");
    assert!(encode_one(owned, bytes.len() - 1).is_err(), "{name}");
    for length in [0, 4, bytes.len() / 2, bytes.len() - 1] {
        assert!(
            decode_one::<O>(&bytes[..length], limits()).is_err(),
            "{name}: prefix {length}"
        );
        assert!(
            reference::decode_args_with_config::<(R,)>(&bytes[..length], &reference_limits())
                .is_err(),
            "{name}: upstream prefix {length}"
        );
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        decode_one::<O>(&trailing, limits()),
        Err(Error::Trailing),
        "{name}"
    );
    assert!(
        reference::decode_args_with_config::<(R,)>(&trailing, &reference_limits()).is_err(),
        "{name}"
    );
    let mut bounded = limits();
    bounded.bytes = bytes.len() - 1;
    assert_eq!(
        decode_one::<O>(&bytes, bounded),
        Err(Error::Limit),
        "{name}"
    );
    bounded = limits();
    bounded.work = 0;
    assert_eq!(
        decode_one::<O>(&bytes, bounded),
        Err(Error::Limit),
        "{name}"
    );
    if measure {
        time(
            name,
            bytes.len(),
            "encode",
            || {
                black_box(encode_one(black_box(owned), limits().bytes).unwrap());
            },
            || {
                black_box(reference::encode_one(black_box(upstream)).unwrap());
            },
        );
        let config = reference_limits();
        time(
            name,
            bytes.len(),
            "decode",
            || {
                black_box(decode_one::<O>(black_box(&bytes), limits()).unwrap());
            },
            || {
                black_box(
                    reference::decode_args_with_config::<(R,)>(black_box(&bytes), &config).unwrap(),
                );
            },
        );
    }
}

fn same<T>(name: &str, value: T, measure: bool)
where
    T: Encode
        + for<'a> Decode<'a>
        + reference::CandidType
        + for<'a> serde::Deserialize<'a>
        + PartialEq
        + Debug,
{
    pair(name, &value, &value, measure);
}

fn catalogue(measure: bool) -> usize {
    let mut count = 0;
    macro_rules! case {
        ($name:expr, $value:expr) => {{
            same($name, $value, measure);
            count += 1;
        }};
    }
    macro_rules! pair_case {
        ($name:expr, $owned:expr, $reference:expr) => {{
            pair($name, &$owned, &$reference, measure);
            count += 1;
        }};
    }
    case!("scalar-unit", ());
    case!("scalar-false", false);
    case!("scalar-true", true);
    macro_rules! boundaries {
        ($($t:ty),*) => {$({
            case!(concat!("scalar-", stringify!($t), "-min"), <$t>::MIN);
            case!(concat!("scalar-", stringify!($t), "-max"), <$t>::MAX);
        })*};
    }
    boundaries!(u8, u16, u32, u64, i8, i16, i32, i64);
    case!("scalar-f32-negative-zero", -0.0f32);
    case!("scalar-f64-infinity", f64::INFINITY);
    for length in [0, 1, 31, 127, 128, 4096, 65_536] {
        case!(&format!("text-ascii-{length}"), "x".repeat(length));
        case!(
            &format!("text-unicode-repeats-{length}"),
            "é🌍".repeat(length)
        );
    }
    for length in [0, 1, 31, 127, 128, 1024, 4096, 65_536] {
        case!(
            &format!("blob-{length}"),
            (0..length).map(|i| i as u8).collect::<Vec<_>>()
        );
    }
    for length in [0, 1, 16, 128, 1024] {
        case!(
            &format!("words-{length}"),
            (0..length as u64).collect::<Vec<_>>()
        );
        case!(&format!("bools-{length}"), vec![true; length]);
        case!(&format!("zero-sized-{length}"), vec![(); length]);
        let map: BTreeMap<_, _> = (0..length)
            .rev()
            .map(|i| (format!("key-{i:04}"), i as u64))
            .collect();
        case!(&format!("map-{length}"), map);
    }
    case!("array-empty", [0u8; 0]);
    case!("array-32", [255u8; 32]);
    case!("array-16-words", [u64::MAX; 16]);
    case!("option-none", None::<u64>);
    case!("option-some", Some(u64::MAX));
    case!("option-nested-none", Some(None::<u64>));
    case!("option-nested-some", Some(Some(u64::MAX)));
    type Branch = Option<Result<Vec<u64>, String>>;
    for (name, branch) in [
        ("branch-none", None),
        ("branch-empty-ok", Some(Ok(Vec::new()))),
        ("branch-ok", Some(Ok(vec![0, u64::MAX]))),
        ("branch-empty-error", Some(Err(String::new()))),
        ("branch-error", Some(Err("refused 🌍".repeat(128)))),
    ] {
        let branch: Branch = branch;
        case!(name, branch);
    }
    for length in [0, 1, 16, 128] {
        let records: Vec<_> = (0..length)
            .map(|i| (i as u64, Some(format!("row-{i}-🌍"))))
            .collect();
        case!(&format!("records-{length}"), records);
    }
    for bits in [0, 7, 64, 256, 1024, 8192] {
        let value = if bits == 0 {
            num_bigint::BigUint::from(0u8)
        } else {
            (num_bigint::BigUint::from(1u8) << bits) - 1u8
        };
        pair_case!(
            &format!("nat-{bits}"),
            value,
            value.to_string().parse::<reference::Nat>().unwrap()
        );
        for negative in [false, true] {
            let integer = num_bigint::BigInt::from(value.clone());
            let integer = if negative { -integer - 1u8 } else { integer };
            pair_case!(
                &format!("int-{bits}-{negative}"),
                integer,
                integer.to_string().parse::<reference::Int>().unwrap()
            );
        }
    }
    for length in [0, 1, 28, 29] {
        let raw = vec![42u8; length];
        pair_case!(
            &format!("principal-{length}"),
            crate::Principal::try_from_slice(&raw).unwrap(),
            reference::Principal::from_slice(&raw)
        );
    }
    count
}

fn discarded(name: &str, bytes: &[u8], expected: (bool, bool), measure: bool) {
    let config = reference_limits();
    let own = || decode_args::<()>(black_box(bytes), limits(), Arity::Compatible);
    let upstream = || reference::decode_args_with_config::<()>(black_box(bytes), &config);
    assert_eq!(own().is_ok(), expected.0, "{name}: owned admission");
    assert_eq!(upstream().is_ok(), expected.1, "{name}: upstream admission");
    if measure {
        time(
            name,
            bytes.len(),
            match expected {
                (true, true) => "skip",
                (false, false) => "reject",
                _ => "admission-difference",
            },
            || {
                let _ = black_box(own());
            },
            || {
                let _ = black_box(upstream());
            },
        );
    }
}

fn discarded_catalogue(measure: bool) -> usize {
    let mut count = 0;
    for length in [0, 1, 128, 4096] {
        let bytes =
            reference::encode_args((vec!["🌍".to_owned(); length], vec![(); length])).unwrap();
        discarded(&format!("skip-{length}"), &bytes, (true, true), measure);
        count += 1;
    }
    for (name, bytes) in [
        ("reject-magic", b"XXXX\0\0".as_slice()),
        ("reject-bool", b"DIDL\0\x01\x7e\x02"),
        ("reject-utf8", b"DIDL\0\x01\x71\x01\xff"),
        ("reject-reference", b"DIDL\x01\x6d\x05\x01\x7f"),
        (
            "reject-field-order",
            b"DIDL\x01\x6c\x02\x01\x7f\x00\x7f\x01\x00",
        ),
        ("reject-mode", b"DIDL\x01\x6a\0\0\x01\x04\x01\x7f"),
        ("reject-option-tag", b"DIDL\x01\x6e\x7e\x01\x00\x02"),
        (
            "reject-variant-index",
            b"DIDL\x01\x6b\x01\x00\x7f\x01\x00\x01",
        ),
        ("reject-missing-value", b"DIDL\0\x01\x7e"),
        ("reject-trailing", b"DIDL\0\x01\x7f\x00"),
        (
            "reject-length-overflow",
            b"DIDL\xff\xff\xff\xff\xff\xff\xff\xff\xff\x02",
        ),
    ] {
        discarded(name, bytes, (false, false), measure);
        count += 1;
    }
    // Upstream accepts this unused oneway declaration with a nonempty result.
    // Platform deliberately rejects it; never weaken admission to force parity.
    discarded(
        "admission-oneway-result",
        b"DIDL\x01\x6a\0\x01\x7f\x01\x02\x01\x7f",
        (false, true),
        measure,
    );
    count + 1
}

#[test]
fn comparison_matrix_cross_decodes_and_preserves_bytes() {
    println!("{} valid codec cases", catalogue(false));
}

#[test]
fn comparison_matrix_validates_skipped_and_malformed_inputs() {
    println!("{} skip/rejection cases", discarded_catalogue(false));
}

#[test]
#[ignore = "Explicit comparison matrix timings; no IC instruction or allocation claim."]
fn comparison_matrix_native_throughput() {
    println!("case,wire_bytes,iterations,sample,implementation,operation,total_ns");
    catalogue(true);
    discarded_catalogue(true);
}
