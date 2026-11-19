//! Comparisons against independently declared upstream schemas.

mod resources;

use super::engine_view_limits as limits;
use crate::fixtures::{SubnetFrame, ViewFrame};
use ::candid::Principal;
use ic_platform_protocol::{candid, cloud_engine as new};

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
fn empty_response_and_absent_optional_payload_match() {
    let empty = ::candid::encode_one(Vec::<ViewFrame>::new()).unwrap();
    assert!(
        new::decode_get_engine_views_response(&empty, limits())
            .unwrap()
            .is_empty()
    );
}
