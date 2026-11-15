//! Native-only independent declarations using the published codec as an oracle.
//! None of these declarations come from the replacement's projection macro.

use ic_platform::{candid_codec, cloud_engine_types as old};
use ic_platform_protocol::{candid, cloud_engine as new};
use ic_platform_types::Principal;

ic_platform::candid_record! {
    struct SubnetFrame {
        1_133_613_777 => replica_version: String,
        1_588_007_470 => maximum_state_size: Option<u64>,
        3_065_459_808 => halted: bool,
        4_257_184_827 => node_ids: Vec<Principal>,
    }
}
ic_platform::candid_record! {
    struct ViewFrame {
        // An unrelated field exercises validated projection skipping.
        0 => extra: Vec<String>,
        23_515 => id: String,
        1_097_286_461 => subnet_id: Option<Principal>,
        1_645_921_689 => engine_operator_id: Option<Principal>,
        2_584_155_869 => subnet: Option<SubnetFrame>,
        2_866_115_289 => deleted_at: Option<i64>,
        4_257_184_827 => node_ids: Vec<Principal>,
    }
}

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
fn old_limits() -> candid_codec::decoder::Limits {
    let limit = limits();
    candid_codec::decoder::Limits {
        bytes: limit.bytes,
        header: limit.header,
        types: limit.types,
        work: limit.work,
        allocation: limit.allocation,
    }
}

// Compare data and refusal classes, not implementation-specific work counters.
fn compare(bytes: &[u8]) {
    let candidate = new::decode_get_engine_views_response(bytes, limits());
    let published = old::decode_get_engine_views_response_with_skip_limit(
        bytes,
        old_limits(),
        limits().skipping,
    );
    match (candidate, published) {
        (Ok(candidate), Ok(published)) => {
            assert_eq!(
                serde_json::to_value(candidate).unwrap(),
                serde_json::to_value(published).unwrap()
            );
        }
        (Err(candidate), Err(published)) => {
            use candid_codec::decoder::Error as Old;
            let expected = match published {
                Old::Malformed => candid::Error::Malformed,
                Old::Unsupported => candid::Error::Unsupported,
                Old::Type => candid::Error::Type,
                Old::Arity => candid::Error::Arity,
                Old::Trailing => candid::Error::Trailing,
                Old::Limit => candid::Error::Limit,
            };
            assert_eq!(candidate, expected);
        }
        (candidate, published) => {
            panic!("decoder disagreement: candidate={candidate:?}, published={published:?}")
        }
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
                let frame = candid_codec::encode_one(frames.as_slice(), 65536).unwrap();
                compare(&frame);
                let view = new::decode_get_engine_views_response(&frame, limits())
                    .unwrap()
                    .pop()
                    .unwrap();
                assert_eq!(view.id, " engine\0é ");
                assert_eq!(view.deleted_at, timestamp);
                assert_eq!(view.subnet.is_some(), present);
                if let Some(subnet) = view.subnet {
                    assert_eq!(subnet.maximum_state_size, maximum_state_size);
                    assert!(subnet.halted);
                    assert_eq!(
                        subnet.node_ids,
                        principals.into_iter().rev().collect::<Vec<_>>()
                    );
                }
                assert_eq!(view.node_ids, principals);
            }
        }
    }
}

#[test]
fn request_frames_match_published_encoder() {
    for ids in [
        vec![],
        vec![String::new()],
        vec!["a\0é".into(), "b".into()],
        vec!["x".repeat(128); 128],
    ] {
        let expected = old::encode_get_engine_views_args(&ids, 65536).unwrap();
        assert_eq!(
            new::encode_get_engine_views_args(&ids, expected.len()).unwrap(),
            expected
        );
        assert!(new::encode_get_engine_views_args(&ids, expected.len() - 1).is_err());
        assert!(old::encode_get_engine_views_args(&ids, expected.len() - 1).is_err());
    }
}

#[test]
fn independent_literal_refusals_match_published_projection() {
    // One view, id="x", node_ids empty; optional fields omitted entirely.
    let valid = b"DIDL\x03\x6d\x01\x6c\x02\xdb\xb7\x01\x71\xbb\xf8\xfd\xed\x0f\x02\x6d\x68\x01\x00\x01\x01x\x00";
    compare(valid);
    for end in 0..valid.len() {
        compare(&valid[..end]);
    }
    let mut trailing = valid.to_vec();
    trailing.push(0);
    compare(&trailing);
    let mut required = valid.to_vec();
    required[9] = 0xda;
    compare(&required);
    compare(b"DIDL\0\0");
    compare(b"DIDL\0\x02\x7f\x7f");
    // An incompatible present optional payload must refuse, not become None.
    let wrong = b"DIDL\x04\x6d\x01\x6c\x03\xdb\xb7\x01\x71\xdd\xad\x9c\xd0\x09\x03\xbb\xf8\xfd\xed\x0f\x02\x6d\x68\x6e\x7e\x01\x00\x01\x01x\x01\x01\x00";
    compare(wrong);
    assert!(new::decode_get_engine_views_response(wrong, limits()).is_err());
}

#[test]
fn empty_response_and_absent_optional_payload_match() {
    let empty = candid_codec::encode_one(&Vec::<ViewFrame>::new(), 65536).unwrap();
    compare(&empty);
    assert!(
        new::decode_get_engine_views_response(&empty, limits())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn mutated_literal_admission_matches_published_projection() {
    let valid = b"DIDL\x03\x6d\x01\x6c\x02\xdb\xb7\x01\x71\xbb\xf8\xfd\xed\x0f\x02\x6d\x68\x01\x00\x01\x01x\x00";
    for offset in 0..valid.len() {
        for byte in [0, 1, 2, 0x6d, 0x7f, 0x80, 0xff] {
            let mut frame = valid.to_vec();
            frame[offset] = byte;
            let candidate = new::decode_get_engine_views_response(&frame, limits());
            let published = old::decode_get_engine_views_response_with_skip_limit(
                &frame,
                old_limits(),
                limits().skipping,
            );
            assert_eq!(
                candidate.is_ok(),
                published.is_ok(),
                "offset {offset}, byte {byte:02x}: candidate={candidate:?}, published={published:?}"
            );
            if candidate.is_ok() {
                compare(&frame);
            }
        }
    }
}

mod resources;
