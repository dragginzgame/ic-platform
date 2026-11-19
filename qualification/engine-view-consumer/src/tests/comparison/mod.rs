//! Independent upstream schemas and fixed admission cases.

mod resources;

use crate::fixtures::{SubnetFrame, ViewFrame};
use ::candid::Principal;
use ic_platform_protocol::{candid, cloud_engine as new};

fn limits() -> candid::Limits {
    candid::Limits {
        bytes: 512 * 1024,
        header: 64 * 1024,
        types: 256,
        work: 2_000_000,
        allocation: 4 * 1024 * 1024,
        vector: 1024,
        skipping: 128 * 1024,
    }
}

#[test]
fn independently_declared_records_preserve_all_selected_values() {
    let principals = [
        Principal::management_canister(),
        Principal::anonymous(),
        Principal::from_slice(&[1; 29]),
    ];
    for timestamp in [None, Some(i64::MIN), Some(-1), Some(0), Some(i64::MAX)] {
        for maximum_state_size in [None, Some(0), Some(u64::MAX)] {
            for present in [false, true] {
                let frames = [ViewFrame {
                    extra: vec!["discarded\0é".into(); 3],
                    id: " engine\0é ".into(),
                    subnet_id: present.then_some(principals[0]),
                    engine_operator_id: Some(principals[1]),
                    subnet: present.then(|| SubnetFrame {
                        replica_version: String::new(),
                        maximum_state_size,
                        halted: true,
                        node_ids: principals.into_iter().rev().collect(),
                    }),
                    deleted_at: timestamp,
                    node_ids: principals.to_vec(),
                }];
                let frame = ::candid::encode_one(frames.as_slice()).unwrap();
                let view = new::decode_get_engine_views_response(&frame, limits())
                    .unwrap()
                    .pop()
                    .unwrap();
                assert_eq!(view.id, " engine\0é ");
                assert_eq!(view.deleted_at, timestamp);
                assert_eq!(
                    view.subnet_id.map(|p| p.as_slice().to_vec()),
                    present.then(Vec::new)
                );
                assert_eq!(view.engine_operator_id.unwrap().as_slice(), &[4]);
                assert_eq!(view.subnet.is_some(), present);
                if let Some(subnet) = view.subnet {
                    assert_eq!(subnet.maximum_state_size, maximum_state_size);
                    assert!(subnet.halted);
                    assert!(subnet.replica_version.is_empty());
                    assert_eq!(
                        subnet
                            .node_ids
                            .iter()
                            .map(|p| p.as_slice())
                            .collect::<Vec<_>>(),
                        principals
                            .iter()
                            .rev()
                            .map(|p| p.as_slice())
                            .collect::<Vec<_>>()
                    );
                }
                assert_eq!(
                    view.node_ids
                        .iter()
                        .map(|p| p.as_slice())
                        .collect::<Vec<_>>(),
                    principals.iter().map(|p| p.as_slice()).collect::<Vec<_>>()
                );
            }
        }
    }
}

#[test]
fn request_frames_match_upstream_encoder() {
    for ids in [
        vec![],
        vec![String::new()],
        vec!["a\0é".into(), "b".into()],
        vec!["x".repeat(128); 128],
    ] {
        let expected = ::candid::encode_one(&ids).unwrap();
        assert_eq!(
            new::encode_get_engine_views_args(&ids, expected.len()).unwrap(),
            expected
        );
        assert!(new::encode_get_engine_views_args(&ids, expected.len() - 1).is_err());
    }
}

#[test]
fn independent_literal_refusals_have_fixed_classes() {
    // One view, id="x", node_ids empty; optional fields omitted entirely.
    let valid = b"DIDL\x03\x6d\x01\x6c\x02\xdb\xb7\x01\x71\xbb\xf8\xfd\xed\x0f\x02\x6d\x68\x01\x00\x01\x01x\x00";
    assert!(new::decode_get_engine_views_response(valid, limits()).is_ok());
    for end in 0..valid.len() {
        assert_eq!(
            new::decode_get_engine_views_response(&valid[..end], limits()),
            Err(candid::Error::Malformed)
        );
    }
    let mut trailing = valid.to_vec();
    trailing.push(0);
    assert_eq!(
        new::decode_get_engine_views_response(&trailing, limits()),
        Err(candid::Error::Trailing)
    );
    let mut required = valid.to_vec();
    required[9] = 0xda;
    assert_eq!(
        new::decode_get_engine_views_response(&required, limits()),
        Err(candid::Error::Type)
    );
    assert_eq!(
        new::decode_get_engine_views_response(b"DIDL\0\0", limits()),
        Err(candid::Error::Arity)
    );
    assert_eq!(
        new::decode_get_engine_views_response(b"DIDL\0\x02\x7f\x7f", limits()),
        Err(candid::Error::Arity)
    );
    // An incompatible present optional payload must refuse, not become None.
    let wrong = b"DIDL\x04\x6d\x01\x6c\x03\xdb\xb7\x01\x71\xdd\xad\x9c\xd0\x09\x03\xbb\xf8\xfd\xed\x0f\x02\x6d\x68\x6e\x7e\x01\x00\x01\x01x\x01\x01\x00";
    assert!(new::decode_get_engine_views_response(wrong, limits()).is_err());
}

#[test]
fn empty_response_and_absent_optional_payload_match() {
    let empty = ::candid::encode_one(Vec::<ViewFrame>::new()).unwrap();
    assert!(
        new::decode_get_engine_views_response(&empty, limits())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn retained_mutation_admission_is_explicit() {
    let valid = b"DIDL\x03\x6d\x01\x6c\x02\xdb\xb7\x01\x71\xbb\xf8\xfd\xed\x0f\x02\x6d\x68\x01\x00\x01\x01x\x00";
    let rows: Vec<_> = include_str!("../../../fixtures/engine-mutations.tsv")
        .lines()
        .collect();
    assert_eq!(rows.len(), valid.len() * 7);
    for row in rows {
        let fields: Vec<_> = row.split('\t').collect();
        let offset: usize = fields[0].parse().unwrap();
        let byte: u8 = fields[1].parse().unwrap();
        let expected: bool = fields[2].parse().unwrap();
        let mut frame = valid.to_vec();
        frame[offset] = byte;
        assert_eq!(
            new::decode_get_engine_views_response(&frame, limits()).is_ok(),
            expected,
            "{row}"
        );
    }
}

#[test]
fn empty_vectors_have_value_based_admission() {
    // One argument of type vec bool, with no elements.
    let frame = b"DIDL\x01\x6d\x7e\x01\x00\x00";
    let candidate = new::decode_get_engine_views_response(frame, limits());
    assert!(candidate.unwrap().is_empty());
}

#[test]
fn absent_payload_has_strict_option_admission() {
    // subnet is declared opt bool rather than opt EngineSubnet, but is None.
    let frame = b"DIDL\x04\x6d\x01\x6c\x03\xdb\xb7\x01\x71\xdd\xad\x9c\xd0\x09\x03\xbb\xf8\xfd\xed\x0f\x02\x6d\x68\x6e\x7e\x01\x00\x01\x01x\x00\x00";
    let views = new::decode_get_engine_views_response(frame, limits()).unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].id, "x");
    assert!(views[0].subnet.is_none());
    // The same declared type with an actual bool payload must still refuse;
    // absent-payload admission must never become optional failure fallback.
    for boolean in [0, 1] {
        let mut present = frame[..frame.len() - 2].to_vec();
        present.extend_from_slice(&[1, boolean, 0]);
        assert_eq!(
            new::decode_get_engine_views_response(&present, limits()),
            Err(candid::Error::Type)
        );
    }
}
