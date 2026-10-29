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
fn nesting(depth: usize) -> NestingLimits {
    NestingLimits {
        depth,
        work: 1_000_000,
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
    for some in [0, 1, 62, 63] {
        check(&opts(some), wire(), nesting(64)).unwrap();
    }
    for some in [64, 65, 8192, 65536] {
        let bytes = opts(some);
        assert_eq!(
            check(&bytes, wire(), nesting(64)),
            Err(NestingError::ValueDepth { maximum: 64 })
        );
        let error = wire()
            .with_nesting(nesting(64))
            .decode_exactly_one::<contract::Reserved>(&bytes)
            .unwrap_err();
        assert_eq!(
            error.nesting_error(),
            Some(&NestingError::ValueDepth { maximum: 64 })
        );
    }
}
#[test]
fn skipped_arguments_and_record_fields_are_admitted_too() {
    let mut extra = b"DIDL\x01\x6e\0\x02\x7e\0\x01".to_vec();
    extra.extend_from_slice(&[1; 65]);
    extra.push(0);
    let error = wire()
        .with_nesting(nesting(64))
        .decode_args::<(bool,)>(&extra)
        .unwrap_err();
    assert!(matches!(
        error.nesting_error(),
        Some(NestingError::ValueDepth { .. })
    ));
    let mut record = b"DIDL\x02\x6e\0\x6c\x02\0\x7e\x01\0\x01\x01\x01".to_vec();
    record.extend_from_slice(&[1; 65]);
    record.push(0);
    assert!(matches!(
        check(&record, wire(), nesting(64)),
        Err(NestingError::ValueDepth { .. })
    ));
}
#[test]
fn byte_blobs_are_not_mistaken_for_nested_option_tags() {
    let value = vec![1u8; 8192];
    let bytes = contract::encode_one(&value).unwrap();
    check(&bytes, wire(), nesting(1)).unwrap();
    assert_eq!(
        wire()
            .with_nesting(nesting(1))
            .decode_exactly_one::<Vec<u8>>(&bytes)
            .unwrap(),
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
    check(&bytes, wire(), nesting(64)).unwrap();
    assert_eq!(
        wire()
            .with_nesting(nesting(64))
            .decode_exactly_one::<Tree>(&bytes)
            .unwrap(),
        value
    );
    for prefix in 0..bytes.len() {
        assert!(check(&bytes[..prefix], wire(), nesting(64)).is_err());
    }
    for index in 4..bytes.len() {
        for replacement in [0, 1, 0x7f, 0x80, 0xff] {
            let mut malformed = bytes.clone();
            malformed[index] = replacement;
            let _ = check(
                &malformed,
                wire(),
                NestingLimits {
                    depth: 32,
                    work: 1000,
                },
            );
        }
    }
}
#[test]
fn unused_header_paths_are_checked_before_upstream_normalization() {
    // Five record definitions form a chain; the actual argument is only bool.
    let bytes = b"DIDL\x05\x6c\x01\0\x01\x6c\x01\0\x02\x6c\x01\0\x03\x6c\x01\0\x04\x6c\x01\0\x7f\x01\x7e\x01";
    assert_eq!(
        check(bytes, wire(), nesting(4)),
        Err(NestingError::TypeDepth { maximum: 4 })
    );
    assert!(
        wire()
            .with_nesting(nesting(5))
            .decode_exactly_one::<bool>(bytes)
            .unwrap()
    );
}
#[test]
fn work_includes_zero_byte_values_and_graph_expansion() {
    // vec null, with u64::MAX entries but no element bytes.
    let bytes = b"DIDL\x01\x6d\x7f\x01\0\xff\xff\xff\xff\xff\xff\xff\xff\xff\x01";
    assert!(
        check(
            bytes,
            wire(),
            NestingLimits {
                depth: 4,
                work: 100
            }
        )
        .is_err()
    );
    assert_eq!(
        check(b"DIDL\0\0", wire(), NestingLimits { depth: 4, work: 0 }),
        Err(NestingError::Work { maximum: 0 })
    );
    // Small acyclic header with exponentially many simple paths. The work
    // budget caps path enumeration independently of the otherwise legal depth.
    let mut dag = b"DIDL\x14".to_vec();
    for index in 0..20 {
        let child = if index == 19 { 0x7f } else { index + 1 };
        dag.extend_from_slice(&[0x6c, 2, 0, child, 1, child]);
    }
    dag.push(0); // no arguments
    assert_eq!(
        check(
            &dag,
            wire(),
            NestingLimits {
                depth: 64,
                work: 1000
            }
        ),
        Err(NestingError::Work { maximum: 1000 })
    );
}
#[test]
fn zero_depth_admits_scalars_but_no_constructed_types() {
    check(b"DIDL\0\x01\x7e\x01", wire(), nesting(0)).unwrap();
    assert_eq!(
        check(&opts(0), wire(), nesting(0)),
        Err(NestingError::TypeDepth { maximum: 0 })
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
            nesting(64)
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
            nesting(64)
        ),
        Err(NestingError::Header { .. })
    ));
    assert!(matches!(
        check(b"DIDL\x01\x67", wire(), nesting(64)),
        Err(NestingError::Unsupported { opcode: -25 })
    ));
    assert!(check(b"DIDL\0\x01\x01", wire(), nesting(64)).is_err());
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
