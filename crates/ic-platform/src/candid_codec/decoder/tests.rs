use super::*;
use crate::candid_codec as codec;
use candid as upstream;

fn limits() -> Limits {
    Limits {
        bytes: 1_000_000,
        header: 100_000,
        types: 1024,
        work: 10_000_000,
        allocation: 4_000_000,
    }
}
fn check<T>(value: T)
where
    T: Encode
        + for<'a> Decode<'a>
        + upstream::CandidType
        + for<'a> serde::Deserialize<'a>
        + PartialEq
        + std::fmt::Debug,
{
    let bytes = upstream::encode_one(&value).unwrap();
    assert_eq!(decode_one::<T>(&bytes, limits()).unwrap(), value);
    let own = codec::encode_one(&value, 1_000_000).unwrap();
    assert_eq!(decode_one::<T>(&own, limits()).unwrap(), value);
    for n in 0..bytes.len() {
        assert!(
            decode_one::<T>(&bytes[..n], limits()).is_err(),
            "accepted prefix {n}"
        );
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(decode_one::<T>(&trailing, limits()), Err(Error::Trailing));
}
fn conversion<S, T>(source: S)
where
    S: upstream::CandidType,
    T: Encode
        + for<'a> Decode<'a>
        + upstream::CandidType
        + for<'a> serde::Deserialize<'a>
        + PartialEq
        + std::fmt::Debug,
{
    let bytes = upstream::encode_one(source).unwrap();
    let expected =
        upstream::decode_args_with_config::<(T,)>(&bytes, &upstream::DecoderConfig::new())
            .unwrap()
            .0;
    assert_eq!(decode_one::<T>(&bytes, limits()).unwrap(), expected);
}
#[test]
fn scalar_and_container_differential() {
    check(());
    check(true);
    check(false);
    check(u8::MAX);
    check(u16::MAX);
    check(u32::MAX);
    check(u64::MAX);
    check(i8::MIN);
    check(i16::MIN);
    check(i32::MIN);
    check(i64::MIN);
    check(-0.0f32);
    check(f64::INFINITY);
    check("héllo 🌍".to_owned());
    check(vec![
        Some((1u64, "x".to_owned())),
        None,
        Some((u64::MAX, "".to_owned())),
    ]);
    check(vec![(); 100]);
    check(Vec::<String>::new());
    check(Some(None::<bool>));
}
#[test]
fn optional_fallback_and_promotion() {
    conversion::<_, Option<u64>>(42u64);
    conversion::<_, Option<u64>>(true);
    conversion::<_, Option<u64>>(Some(true));
    conversion::<_, Option<u64>>(None::<bool>);
    conversion::<_, Option<u64>>(());
    conversion::<_, Option<Option<u64>>>(Some(42u64));
    conversion::<_, Option<Option<u64>>>(42u64);
    conversion::<_, Option<Option<u64>>>(Some(true));
    conversion::<_, Option<()>>(());
}
#[test]
fn record_extension_and_missing_optional_fields() {
    conversion::<_, (u64, Option<String>)>((42u64,));
    conversion::<_, (u64, Option<String>)>((42u64, Some("hello".to_owned()), vec![true, false]));
    conversion::<_, (u64, ())>((42u64,));
    conversion::<_, (u64, Option<String>)>((42u64, true, "extra".to_owned()));
}
#[test]
fn malformed_framing_and_values_fail_closed() {
    for bytes in [
        b"".as_slice(),
        b"DIDL",
        b"DIDL\0\x01\x7e\x02",
        b"DIDL\0\x01\x71\x01\xff",
        b"DIDL\x01\x6d\x05\x01\x7f",
        b"DIDL\x01\x6c\x02\x01\x7f\x01\x7f\x01\x00",
    ] {
        assert!(decode_one::<Option<bool>>(bytes, limits()).is_err());
    }
    // An invalid reference in an unused type still refuses the whole message.
    assert!(decode_one::<()>(b"DIDL\x01\x6d\x05\x01\x7f", limits()).is_err());
    assert_eq!(
        decode_one::<u64>(&upstream::encode_one(true).unwrap(), limits()),
        Err(Error::Type)
    );
}
#[test]
fn caller_budgets_are_enforced() {
    let bytes = upstream::encode_one(vec!["hello".to_owned(); 10]).unwrap();
    let mut budget = limits();
    budget.bytes = bytes.len() - 1;
    assert_eq!(decode_one::<Vec<String>>(&bytes, budget), Err(Error::Limit));
    let mut budget = limits();
    budget.header = 4;
    assert_eq!(decode_one::<Vec<String>>(&bytes, budget), Err(Error::Limit));
    let mut budget = limits();
    budget.types = 0;
    assert_eq!(decode_one::<Vec<String>>(&bytes, budget), Err(Error::Limit));
    let mut budget = limits();
    budget.work = 10;
    assert_eq!(decode_one::<Vec<String>>(&bytes, budget), Err(Error::Limit));
    let mut budget = limits();
    budget.allocation = 1;
    assert_eq!(decode_one::<Vec<String>>(&bytes, budget), Err(Error::Limit));
    // Small input must not expand into an unbounded zero-sized collection.
    let bomb = b"DIDL\x01\x6d\x7f\x01\x00\xff\xff\xff\xff\x0f";
    assert_eq!(decode_one::<Vec<()>>(bomb, limits()), Err(Error::Limit));
}
#[test]
fn arbitrary_byte_mutations_do_not_panic() {
    let original = upstream::encode_one(vec![Some((1u64, "hello".to_owned()))]).unwrap();
    for position in 0..original.len() {
        for replacement in [0, 1, 0x7f, 0x80, 0xff] {
            let mut bytes = original.clone();
            bytes[position] = replacement;
            let _ = decode_one::<Vec<Option<(u64, String)>>>(&bytes, limits());
        }
    }
}

#[test]
fn nesting_boundary_including_skipped_and_unused_types() {
    for depth in [16u8, 17] {
        let mut bytes = b"DIDL".to_vec();
        bytes.push(depth);
        for index in 0..depth {
            bytes.extend_from_slice(&[0x6d, if index + 1 == depth { 0x7f } else { index + 1 }]);
        }
        // All table entries are unused, but still subject to admission.
        bytes.extend_from_slice(&[1, 0x7f]);
        assert_eq!(
            decode_one::<()>(&bytes, limits()),
            if depth == 16 {
                Ok(())
            } else {
                Err(Error::Limit)
            }
        );
        bytes.truncate(bytes.len() - 2);
        bytes.extend_from_slice(&[1, 0]);
        bytes.extend(std::iter::repeat_n(1, usize::from(depth)));
        assert_eq!(
            decode_one::<Option<bool>>(&bytes, limits()),
            if depth == 16 {
                Ok(None)
            } else {
                Err(Error::Limit)
            }
        );
    }
    for depth in [16, 17] {
        // One recursive opt entry; finite values still consume the depth budget.
        let mut bytes = b"DIDL\x01\x6e\x00\x01\x00".to_vec();
        bytes.extend(std::iter::repeat_n(1, depth - 1));
        bytes.push(0);
        assert_eq!(
            decode_one::<Option<bool>>(&bytes, limits()),
            if depth == 16 {
                Ok(None)
            } else {
                Err(Error::Limit)
            }
        );
    }
}

#[test]
fn accepted_mutations_match_upstream_values() {
    type Value = Vec<Option<(u64, String)>>;
    let source = upstream::encode_one(vec![Some((1u64, "hello".to_owned()))]).unwrap();
    for position in 0..source.len() {
        for replacement in [0, 1, 2, 0x7f, 0x80, 0xff] {
            let mut bytes = source.clone();
            bytes[position] = replacement;
            if let Ok(actual) = decode_one::<Value>(&bytes, limits()) {
                let reference = upstream::decode_args_with_config::<(Value,)>(
                    &bytes,
                    &upstream::DecoderConfig::new(),
                )
                .unwrap();
                assert_eq!(actual, reference.0, "mutation {position}/{replacement}");
            }
        }
    }
}

#[test]
fn floating_point_payloads_preserve_bits() {
    for bits in [0u64, 1, 1 << 63, 0x7ff8000000000001, u64::MAX] {
        let bytes = upstream::encode_one(f64::from_bits(bits)).unwrap();
        assert_eq!(decode_one::<f64>(&bytes, limits()).unwrap().to_bits(), bits);
    }
}

#[test]
fn unused_function_modes_preserve_admission_and_allocation_charges() {
    for modes in [&[][..], &[1], &[2], &[3], &[0], &[4], &[255], &[1, 1]] {
        // One unused function type with no arguments/results, then one null.
        let mut bytes = b"DIDL\x01\x6a\0\0".to_vec();
        bytes.push(modes.len() as u8);
        bytes.extend_from_slice(modes);
        bytes.extend_from_slice(&[1, 0x7f]);
        let valid = matches!(modes, [] | [1] | [2] | [3]);
        assert_eq!(
            decode_one::<()>(&bytes, limits()),
            if valid { Ok(()) } else { Err(Error::Malformed) }
        );
        if valid {
            let charge = size_of::<Wire>() + size_of::<Id>() + modes.len();
            let mut budget = limits();
            budget.allocation = charge;
            assert_eq!(decode_one::<()>(&bytes, budget), Ok(()));
            budget.allocation -= 1;
            assert_eq!(decode_one::<()>(&bytes, budget), Err(Error::Limit));
            assert!(upstream::decode_one::<()>(&bytes).is_ok());
            for length in 0..bytes.len() {
                assert!(decode_one::<()>(&bytes[..length], limits()).is_err());
            }
        }
    }
    // Oneway with a nonempty result list is malformed even when unused.
    let bytes = b"DIDL\x01\x6a\0\x01\x7f\x01\x02\x01\x7f";
    assert_eq!(decode_one::<()>(bytes, limits()), Err(Error::Malformed));
}

#[test]
fn typed_tuple_requires_tuple_wire_layout() {
    let mut bytes = b"DIDL\x01\x6c\x02\x00\x78\x02\x7f\x01\x00".to_vec();
    bytes.extend_from_slice(&42u64.to_le_bytes());
    type Pair = (u64, Option<String>);
    assert!(
        upstream::decode_args_with_config::<(Pair,)>(&bytes, &upstream::DecoderConfig::new())
            .is_err()
    );
    assert_eq!(decode_one::<Pair>(&bytes, limits()), Err(Error::Type));
}

#[test]
#[ignore = "Explicit native throughput experiment; not a correctness or IC instruction gate."]
fn native_text_throughput() {
    use std::{hint::black_box, time::Instant};
    let reference_limits = crate::tests::reference::DecodeLimits {
        reply_bytes: 1_000_000,
        header_bytes: 100_000,
        type_entries: 1024,
        decoding_quota: 10_000_000,
        skipping_quota: 10_000_000,
    };
    println!("payload_bytes,iterations,sample,implementation,operation,total_ns");
    for length in [0, 32, 4096, 65_536] {
        let text = "x".repeat(length);
        let bytes = upstream::encode_one(&text).unwrap();
        assert_eq!(codec::encode_one(&text, 1_000_000).unwrap(), bytes);
        assert_eq!(decode_one::<String>(&bytes, limits()).unwrap(), text);
        let iterations = (16_777_216 / length.max(1)).clamp(256, 10_000);
        for sample in 0..5 {
            // Alternate order between samples to reduce a fixed warm-cache bias.
            for implementation in if sample % 2 == 0 { [0, 1] } else { [1, 0] } {
                let start = Instant::now();
                for _ in 0..iterations {
                    if implementation == 0 {
                        black_box(codec::encode_one(black_box(&text), 1_000_000).unwrap());
                    } else {
                        black_box(upstream::encode_one(black_box(&text)).unwrap());
                    }
                }
                println!(
                    "{length},{iterations},{sample},{implementation},encode,{}",
                    start.elapsed().as_nanos()
                );
                let start = Instant::now();
                for _ in 0..iterations {
                    if implementation == 0 {
                        black_box(decode_one::<String>(black_box(&bytes), limits()).unwrap());
                    } else {
                        black_box(
                            crate::tests::reference::decode_one::<String>(
                                black_box(&bytes),
                                reference_limits,
                            )
                            .unwrap(),
                        );
                    }
                }
                println!(
                    "{length},{iterations},{sample},{implementation},decode,{}",
                    start.elapsed().as_nanos()
                );
            }
        }
    }
}

#[test]
fn result_variants_and_optional_fallback() {
    check(std::result::Result::<u64, String>::Ok(42));
    check(std::result::Result::<u64, String>::Err("denied".to_owned()));
    check(vec![Ok(vec![1u64, 2]), Err("bad".to_owned())]);
    conversion::<_, Option<std::result::Result<u64, String>>>(Ok::<u64, bool>(42));
    conversion::<_, Option<std::result::Result<u64, String>>>(Err::<u64, bool>(true));
    conversion::<_, Option<std::result::Result<Vec<u64>, String>>>(Ok::<Vec<bool>, String>(vec![]));
}

#[test]
fn bounded_vectors_refuse_before_storage_or_element_reads() {
    for count in [0, 31, 32, 255, 256, 257, 1024, 1025, 4096, 4097] {
        let bytes = upstream::encode_one(vec![7u8; count]).unwrap();
        for cap in [0, 32, 256, 1024, 4096] {
            let (mut reader, args) = message(&bytes, limits()).unwrap();
            let allocation = reader.allocation;
            let result = reader.bounded_vec::<u8>(args[0], cap);
            if count > cap {
                assert_eq!(result, Err(Error::Limit));
                assert_eq!(reader.allocation, allocation);
                assert_eq!(reader.input.len() - reader.pos, count);
            } else {
                assert_eq!(result.unwrap(), vec![7; count]);
                assert_eq!(reader.pos, bytes.len());
            }
        }
    }
    // A huge announced zero-sized vector is rejected without entering its loop.
    let bytes = b"DIDL\x01\x6d\x7f\x01\x00\xff\xff\xff\xff\x0f";
    let (mut reader, args) = message(bytes, limits()).unwrap();
    let allocation = reader.allocation;
    assert_eq!(reader.bounded_vec::<()>(args[0], 1024), Err(Error::Limit));
    assert_eq!(reader.allocation, allocation);
}

#[test]
fn array_lengths_are_checked_before_storage_or_element_reads() {
    for count in [0, 1, 2, 3, 128, 4096] {
        let bytes = upstream::encode_one(vec![7u8; count]).unwrap();
        let (mut reader, args) = message(&bytes, limits()).unwrap();
        let allocation = reader.allocation;
        let result = reader.value::<[u8; 2]>(args[0]);
        if count == 2 {
            assert_eq!(result, Ok([7; 2]));
        } else {
            assert_eq!(result, Err(Error::Malformed));
            assert_eq!(reader.allocation, allocation);
            assert_eq!(reader.input.len() - reader.pos, count);
        }
        let expected = upstream::decode_one::<[u8; 2]>(&bytes);
        assert_eq!(result.is_ok(), expected.is_ok());
    }
    check([1u8, 2]);
    check([0u8; 0]);
    // Invalid fixed-array lengths must not disappear through optional fallback.
    let bytes = upstream::encode_one(vec![7u8; 3]).unwrap();
    assert_eq!(
        decode_one::<Option<[u8; 2]>>(&bytes, limits()),
        Err(Error::Malformed)
    );
    let mut bytes = upstream::encode_one(vec![true; 3]).unwrap();
    *bytes.last_mut().unwrap() = 2;
    assert_eq!(
        decode_one::<Option<[bool; 2]>>(&bytes, limits()),
        Err(Error::Malformed)
    );
    let bytes = b"DIDL\x01\x6d\x7f\x01\x00\xff\xff\xff\xff\x0f";
    let (mut reader, args) = message(bytes, limits()).unwrap();
    let allocation = reader.allocation;
    assert_eq!(reader.value::<[(); 2]>(args[0]), Err(Error::Malformed));
    assert_eq!(reader.allocation, allocation);
}

#[test]
fn array_schema_and_bytes_match_the_independent_dynamic_oracle() {
    crate::tests::reference::roundtrip(&[Some("héllo".to_owned()), None]);
    crate::tests::reference::roundtrip(&[0u8; 0]);
}

#[test]
fn optional_fallback_retains_failed_attempt_budgets() {
    type Target = Option<(String, u64)>;
    let text = "héllo 🌍";
    let bytes = upstream::encode_one((text, true)).unwrap();
    // Reading the first field allocates; the incompatible second field triggers
    // fallback. Its cursor rollback must not refund the first field's storage.
    let (mut reader, args) = message(&bytes, limits()).unwrap();
    reader.allocation = text.len();
    assert_eq!(reader.value::<Target>(args[0]), Ok(None));
    assert_eq!(reader.allocation, 0);
    assert_eq!(reader.pos, bytes.len());
    assert!(!reader.in_skip);
    for remaining in 0..text.len() {
        let (mut reader, args) = message(&bytes, limits()).unwrap();
        reader.allocation = remaining;
        assert_eq!(reader.value::<Target>(args[0]), Err(Error::Limit));
    }
    let (mut reader, args) = message(&bytes, limits()).unwrap();
    reader.work = 1;
    assert_eq!(reader.value::<Target>(args[0]), Err(Error::Limit));
    assert!(!reader.in_skip);
    let (mut reader, args) = message(&bytes, limits()).unwrap();
    reader.skipping = 0;
    assert_eq!(reader.value::<Target>(args[0]), Err(Error::Limit));
    assert!(!reader.in_skip);
}

#[test]
fn skipped_work_is_separate_cumulative_and_charged_once() {
    let bytes = upstream::encode_one(vec![true, false]).unwrap();
    // skip vector: value + enter + length byte + element count + 2*(value + byte).
    for budget in 0..9 {
        let (mut reader, args) = message(&bytes, limits()).unwrap();
        reader.skipping = budget;
        assert_eq!(reader.skip(args[0]), Err(Error::Limit));
        assert!(!reader.in_skip);
    }
    let (mut reader, args) = message(&bytes, limits()).unwrap();
    reader.skipping = 9;
    let work = reader.work;
    reader.skip(args[0]).unwrap();
    assert_eq!(reader.skipping, 0);
    assert_eq!(work - reader.work, 9);

    let bytes = upstream::encode_args(((), ())).unwrap();
    assert_eq!(
        decode_args_with_skip_limit::<()>(&bytes, limits(), Arity::Compatible, 1),
        Err(Error::Limit)
    );
    assert_eq!(
        decode_args_with_skip_limit::<()>(&bytes, limits(), Arity::Compatible, 2),
        Ok(())
    );
    let mut tight = limits();
    tight.work = 1;
    assert_eq!(
        decode_args_with_skip_limit::<()>(&bytes, tight, Arity::Compatible, usize::MAX),
        Err(Error::Limit)
    );
}

#[test]
fn zero_skip_refuses_unknown_fields_reserved_and_optional_fallback() {
    let bytes = upstream::encode_one((42u64, true, false)).unwrap();
    assert_eq!(
        decode_one_with_skip_limit::<(u64, bool)>(&bytes, limits(), 0),
        Err(Error::Limit)
    );
    assert_eq!(
        decode_one_with_skip_limit::<(u64, bool)>(
            &upstream::encode_one((42u64, true)).unwrap(),
            limits(),
            0
        ),
        Ok((42, true))
    );
    let bytes = upstream::encode_one(true).unwrap();
    assert_eq!(
        decode_one_with_skip_limit::<Option<u64>>(&bytes, limits(), 0),
        Err(Error::Limit)
    );
    assert_eq!(
        decode_one_with_skip_limit::<Option<u64>>(&bytes, limits(), 2),
        Ok(None)
    );
    assert_eq!(
        decode_one_with_skip_limit::<codec::Reserved>(&bytes, limits(), 0),
        Err(Error::Limit)
    );
    // A sufficient skip budget still validates malformed discarded bytes.
    let bytes = b"DIDL\0\x01\x7e\x02";
    assert_eq!(
        decode_one_with_skip_limit::<Option<u64>>(bytes, limits(), 2),
        Err(Error::Malformed)
    );
}

#[test]
fn strict_options_do_not_promote_or_discard_incompatible_values() {
    for value in [None, Some(u64::MAX)] {
        let bytes = upstream::encode_one(value).unwrap();
        let (mut reader, args) = message(&bytes, limits()).unwrap();
        reader.skipping = 0;
        assert_eq!(reader.strict_option::<u64>(args[0]), Ok(value));
        assert_eq!(reader.pos, bytes.len());
    }
    for bytes in [
        upstream::encode_one(42u64).unwrap(),
        upstream::encode_one(Some(true)).unwrap(),
        upstream::encode_one(()).unwrap(),
    ] {
        let (mut reader, args) = message(&bytes, limits()).unwrap();
        assert_eq!(reader.strict_option::<u64>(args[0]), Err(Error::Type));
    }
}
