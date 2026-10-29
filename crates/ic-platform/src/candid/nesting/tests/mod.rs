use super::*;
use crate::candid::{self, contract};
fn wire() -> DecodeLimits {
    DecodeLimits {
        reply_bytes: 1_000_000,
        decoding_quota: 1_000_000,
        skipping_quota: 1_000_000,
        type_entries: 10_000,
        header_bytes: 100_000,
    }
}
fn opts(some: usize) -> Vec<u8> {
    let mut bytes = b"DIDL\x01\x6e\0\x01\0".to_vec();
    bytes.resize(bytes.len() + some, 1);
    bytes.push(0);
    bytes
}
#[test]
fn recursive_optional_boundary_refuses_the_installed_trap_before_decode() {
    for some in [0, 1, 14, 15] {
        check(&opts(some), wire()).unwrap();
    }
    for some in [16, 17, 8192, 65536] {
        let bytes = opts(some);
        assert_eq!(
            check(&bytes, wire()),
            Err(NestingError::ValueDepth { maximum: 16 })
        );
        let error = candid::decode_exactly_one::<contract::Reserved>(&bytes, wire()).unwrap_err();
        assert_eq!(
            error.nesting_error(),
            Some(&NestingError::ValueDepth { maximum: 16 })
        );
    }
}
#[test]
fn skipped_arguments_and_record_fields_are_admitted_too() {
    let mut extra = b"DIDL\x01\x6e\0\x02\x7e\0\x01".to_vec();
    extra.extend_from_slice(&[1; 65]);
    extra.push(0);
    let error = candid::decode_args::<(bool,)>(&extra, wire()).unwrap_err();
    assert!(matches!(
        error.nesting_error(),
        Some(NestingError::ValueDepth { .. })
    ));
    let mut record = b"DIDL\x02\x6e\0\x6c\x02\0\x7e\x01\0\x01\x01\x01".to_vec();
    record.extend_from_slice(&[1; 65]);
    record.push(0);
    assert!(matches!(
        check(&record, wire()),
        Err(NestingError::ValueDepth { .. })
    ));
}
#[test]
fn byte_blobs_are_not_mistaken_for_nested_option_tags() {
    let value = vec![1u8; 8192];
    let bytes = contract::encode_one(&value).unwrap();
    check(&bytes, wire()).unwrap();
    assert_eq!(
        candid::decode_exactly_one::<Vec<u8>>(&bytes, wire()).unwrap(),
        value
    );
}
#[derive(Debug, PartialEq, contract::CandidType, serde::Deserialize)]
#[candid_path("crate::candid::contract")]
enum Tree {
    Leaf(u64),
    Branch(Vec<Tree>),
}
#[test]
fn recursive_type_tables_and_ordinary_nested_values_stay_valid() {
    let value = Tree::Branch(vec![Tree::Leaf(7), Tree::Branch(vec![Tree::Leaf(8)])]);
    let bytes = contract::encode_one(&value).unwrap();
    check(&bytes, wire()).unwrap();
    assert_eq!(
        candid::decode_exactly_one::<Tree>(&bytes, wire()).unwrap(),
        value
    );
    for prefix in 0..bytes.len() {
        assert!(check(&bytes[..prefix], wire()).is_err());
    }
    for index in 4..bytes.len() {
        for replacement in [0, 1, 0x7f, 0x80, 0xff] {
            let mut malformed = bytes.clone();
            malformed[index] = replacement;
            let _ = check(
                &malformed,
                DecodeLimits {
                    decoding_quota: 1000,
                    skipping_quota: 0,
                    ..wire()
                },
            );
        }
    }
}
#[test]
fn unused_header_paths_are_checked_before_upstream_normalization() {
    for depth in [16u8, 17] {
        let mut bytes = b"DIDL".to_vec();
        bytes.push(depth);
        for index in 0..depth {
            bytes.extend_from_slice(&[
                0x6c,
                1,
                0,
                if index + 1 == depth { 0x7f } else { index + 1 },
            ]);
        }
        bytes.extend_from_slice(b"\x01\x7e\x01");
        if depth == 16 {
            assert!(candid::decode_exactly_one::<bool>(&bytes, wire()).unwrap());
        } else {
            assert_eq!(
                check(&bytes, wire()),
                Err(NestingError::TypeDepth { maximum: 16 })
            );
        }
    }
}
#[test]
fn work_includes_zero_byte_values_and_graph_expansion() {
    // vec null, with u64::MAX entries but no element bytes.
    let bytes = b"DIDL\x01\x6d\x7f\x01\0\xff\xff\xff\xff\xff\xff\xff\xff\xff\x01";
    assert!(
        check(
            bytes,
            DecodeLimits {
                decoding_quota: 100,
                skipping_quota: 0,
                ..wire()
            }
        )
        .is_err()
    );
    assert_eq!(
        check(
            b"DIDL\0\0",
            DecodeLimits {
                decoding_quota: 0,
                skipping_quota: 0,
                ..wire()
            }
        ),
        Err(NestingError::Work { maximum: 0 })
    );
    // Small acyclic header with exponentially many simple paths. The work
    // budget caps path enumeration independently of the otherwise legal depth.
    let mut dag = b"DIDL\x0c".to_vec();
    for index in 0..12 {
        let child = if index == 11 { 0x7f } else { index + 1 };
        dag.extend_from_slice(&[0x6c, 2, 0, child, 1, child]);
    }
    dag.push(0); // no arguments
    assert_eq!(
        check(
            &dag,
            DecodeLimits {
                decoding_quota: 1000,
                skipping_quota: 0,
                ..wire()
            }
        ),
        Err(NestingError::Work { maximum: 1000 })
    );
}
#[test]
fn original_envelope_budgets_and_future_constructors_fail_closed() {
    let bytes = b"DIDL\0\x01\x7e\x01";
    assert!(matches!(
        check(
            bytes,
            DecodeLimits {
                reply_bytes: 1,
                ..wire()
            },
        ),
        Err(NestingError::Bytes { .. })
    ));
    assert!(matches!(
        check(
            bytes,
            DecodeLimits {
                header_bytes: 3,
                ..wire()
            },
        ),
        Err(NestingError::Header { .. })
    ));
    assert!(matches!(
        check(b"DIDL\x01\x67", wire()),
        Err(NestingError::Unsupported { opcode: -25 })
    ));
    assert!(check(b"DIDL\0\x01\x01", wire()).is_err());
}

#[test]
fn existing_function_pointer_and_error_matches_remain_source_compatible() {
    let decode: fn(&'static [u8], DecodeLimits) -> std::result::Result<u64, candid::DecodeError> =
        candid::decode_one::<u64>;
    assert_eq!(
        decode(b"DIDL\0\x01\x78\x07\0\0\0\0\0\0\0", wire()).unwrap(),
        7
    );
}
