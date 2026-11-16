use super::{hex, limits};
use crate::{
    candid::Error,
    cloud_engine::{
        AvailableNodesPageResult as Reply, EngineError, NodeType,
        decode_list_available_nodes_page_response as decode,
    },
};
use candidate_types::Principal;

// Independently assembled from #166's selected IDL, including unselected IP text.
const HEADER: &str = "44 49 44 4c 09 6b 02 9c c2 01 01 e5 8e b4 02 08 6e 02 6c 03 f3 89 8a c8 04 03 d6 ee f5 a0 06 78 b1 f0 fa fd 09 04 6e 68 6d 05 6c 07 ff ae 01 06 d7 fa 89 89 01 07 b8 aa cc b7 01 68 b1 e2 e7 99 02 68 a4 b7 cc a3 04 68 a6 fb ec a3 0b 7e 9c cc b3 f7 0f 71 6e 71 6b 06 ba e0 ad d6 01 7f ac 8b cf c5 0f 7f ad 8b cf c5 0f 7f ae 8b cf c5 0f 7f af 8b cf c5 0f 7f b0 8b cf c5 0f 7f 6b 09 dd c6 a0 11 71 ec ea a7 7f 71 cc ce f0 c6 06 71 bd b1 98 ef 06 71 cf a0 de f2 06 7f d4 b4 c5 9a 09 7f f4 f6 d8 a2 09 7f e1 8b d8 8a 0c 7f aa 98 d9 d1 0e 71 01 00";

fn message(payload: &[u8]) -> Vec<u8> {
    let mut bytes = hex(HEADER);
    bytes.extend_from_slice(payload);
    bytes
}

fn page(class: u8) -> Vec<u8> {
    let mut payload = vec![0, 1, 1, 1, 1, 4]; // ok Some, cursor anonymous
    payload.extend_from_slice(&u64::MAX.to_le_bytes());
    payload.extend_from_slice(&[1, 1, 0, class, 1, 1, 4, 1, 0, 1, 1, 4, 1, 2, b'i', b'p']);
    message(&payload)
}

#[test]
fn unready_empty_and_present_pages_stay_distinct() {
    assert_eq!(
        decode(&message(&[0, 0]), limits()).unwrap(),
        Reply::Ok(None)
    );
    let mut empty = vec![0, 1, 0];
    empty.extend_from_slice(&0u64.to_le_bytes());
    empty.push(0);
    let Reply::Ok(Some(value)) = decode(&message(&empty), limits()).unwrap() else {
        panic!("expected page")
    };
    assert!(value.nodes.is_empty());
    assert_eq!(value.next, None);
    assert_eq!(value.registry_version, 0);
    for (index, class) in [
        NodeType::Type4,
        NodeType::Type4_1,
        NodeType::Type4_2,
        NodeType::Type4_3,
        NodeType::Type4_4,
        NodeType::Type4_5,
    ]
    .into_iter()
    .enumerate()
    {
        let Reply::Ok(Some(value)) = decode(&page(index as u8), limits()).unwrap() else {
            panic!("expected page")
        };
        assert_eq!(value.registry_version, u64::MAX);
        assert_eq!(value.next, Some(Principal::anonymous()));
        assert_eq!(value.nodes.len(), 1);
        let node = &value.nodes[0];
        assert_eq!(node.node_type, class);
        assert_eq!(node.dc.as_deref(), Some(""));
        assert_eq!(node.node_id, Principal::anonymous());
        assert_eq!(node.provider, Principal::management_canister());
        assert_eq!(node.operator, Principal::anonymous());
        assert!(node.sev_enabled);
    }
}

#[test]
fn all_application_errors_preserve_raw_details() {
    let text = " x\0é ";
    for (index, expected) in [
        EngineError::Internal(text.into()),
        EngineError::ControllerError(text.into()),
        EngineError::InvalidTransition(text.into()),
        EngineError::SlugConflict(text.into()),
        EngineError::NotFound,
        EngineError::Unauthorized,
        EngineError::AlreadyExists,
        EngineError::NotVetted,
        EngineError::BadRequest(text.into()),
    ]
    .into_iter()
    .enumerate()
    {
        let mut payload = vec![1, index as u8];
        if !matches!(index, 4..=7) {
            payload.push(text.len() as u8);
            payload.extend_from_slice(text.as_bytes());
        }
        assert_eq!(
            decode(&message(&payload), limits()).unwrap(),
            Reply::Err(expected)
        );
    }
}

#[test]
fn invalid_present_pages_never_fall_back_to_unready() {
    let bytes = page(0);
    for end in 0..bytes.len() {
        assert_eq!(decode(&bytes[..end], limits()), Err(Error::Malformed));
    }
    let mut unknown = page(5);
    let offset = unknown
        .windows(6)
        .position(|w| w == [0xb0, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f])
        .unwrap();
    unknown[offset] += 1;
    assert_eq!(decode(&unknown, limits()), Err(Error::Type));
    assert_eq!(decode(&page(6), limits()), Err(Error::Malformed));
    assert_eq!(decode(&message(&[0, 2]), limits()), Err(Error::Malformed));
    assert_eq!(decode(&message(&[2]), limits()), Err(Error::Malformed));
    assert_eq!(decode(&message(&[1, 9]), limits()), Err(Error::Malformed));
    let mut unknown_error = message(&[1, 8, 0]);
    let offset = unknown_error
        .windows(6)
        .position(|w| w == [0xaa, 0x98, 0xd9, 0xd1, 0x0e, 0x71])
        .unwrap();
    unknown_error[offset] += 1;
    assert_eq!(decode(&unknown_error, limits()), Err(Error::Type));
    let mut wrong_payload = message(&[1, 4]);
    let offset = wrong_payload
        .windows(6)
        .position(|w| w == [0xcf, 0xa0, 0xde, 0xf2, 6, 0x7f])
        .unwrap();
    wrong_payload[offset + 5] = 0x71;
    assert_eq!(decode(&wrong_payload, limits()), Err(Error::Type));
    let mut invalid_text = bytes.clone();
    *invalid_text.last_mut().unwrap() = 255;
    assert_eq!(decode(&invalid_text, limits()), Err(Error::Malformed));
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(decode(&trailing, limits()), Err(Error::Trailing));
    assert_eq!(decode(b"DIDL\0\0", limits()), Err(Error::Arity));
    assert_eq!(decode(b"DIDL\0\x02\x7f\x7f", limits()), Err(Error::Arity));
    assert_eq!(
        decode(b"DIDL\x01\x6b\x01\0\x7f\x01\0\0", limits()),
        Err(Error::Type)
    );
    for budget in 0..7 {
        let mut cap = limits();
        match budget {
            0 => cap.bytes = bytes.len() - 1,
            1 => cap.header = 179,
            2 => cap.types = 8,
            3 => cap.work = 0,
            4 => cap.allocation = 0,
            5 => cap.vector = 0,
            _ => cap.skipping = 0,
        }
        assert_eq!(decode(&bytes, cap), Err(Error::Limit));
    }
    let mut cap = limits();
    cap.skipping = 4; // discarded value traversal, length and two text bytes
    assert!(decode(&bytes, cap).is_ok());
    cap.skipping = 3;
    assert_eq!(decode(&bytes, cap), Err(Error::Limit));
}
