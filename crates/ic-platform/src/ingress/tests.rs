use super::*;
use std::borrow::Cow;

#[test]
fn bounded_construction_preserves_external_envelopes_without_authenticating() {
    let read = ic_transport_types::EnvelopeContent::ReadState {
        ingress_expiry: 0,
        sender: serde_cbor::from_slice(&[0x41, 4]).unwrap(),
        paths: vec![vec![
            b"request_status".to_vec().into(),
            vec![255; 32].into(),
        ]],
    };
    for content in [original(), read] {
        for delegation in [
            None,
            Some(vec![]),
            Some(vec![ic_transport_types::SignedDelegation {
                delegation: ic_transport_types::Delegation {
                    pubkey: vec![0, 255],
                    expiration: u64::MAX,
                    targets: Some(vec![]),
                    permissions: Some(ic_transport_types::DelegationPermissions::Queries),
                },
                signature: vec![128, 0],
            }]),
        ] {
            let upstream = ic_transport_types::Envelope {
                content: Cow::Borrowed(&content),
                sender_pubkey: Some(vec![]),
                sender_sig: Some(vec![0; 3]),
                sender_delegation: delegation,
            };
            let original = upstream.encode_bytes();
            let admitted = decode_envelope(&original, original.len()).unwrap();
            let encoded = encode_envelope(&admitted.value, 4096).unwrap();
            assert_eq!(
                encode_envelope(&admitted.value, encoded.len()).unwrap(),
                encoded
            );
            assert_eq!(
                encode_envelope(&admitted.value, encoded.len() - 1),
                Err(EncodeError::ByteLimit)
            );
            assert_eq!(
                encode_envelope(&admitted.value, 0),
                Err(EncodeError::ByteLimit)
            );
            // Independent upstream envelope serialization, including optional presence.
            let value = |bytes: &[u8]| serde_cbor::from_slice::<serde_cbor::Value>(bytes).unwrap();
            assert_eq!(value(&encoded), value(&original));
            let decoded = decode_envelope(&encoded, encoded.len()).unwrap();
            assert_eq!(
                decoded.value.content.request_id().unwrap(),
                *content.to_request_id()
            );
            assert_eq!(
                decoded.value.content.signable().unwrap(),
                content.to_request_id().signable()
            );
            assert_eq!(decoded.value.sender_sig, Some(vec![0; 3]));
        }
    }
}

#[test]
fn construction_admits_full_two_mib_output_and_preserves_sender_info() {
    const MAXIMUM: usize = 2 * 1024 * 1024;
    let mut envelope = Envelope {
        content: Content::Call {
            nonce: Some(vec![]),
            ingress_expiry: u64::MAX,
            sender: Principal::anonymous(),
            canister_id: Principal::management_canister(),
            method_name: "method".into(),
            arg: vec![255; MAXIMUM],
            sender_info: Some(SenderInfo {
                info: vec![],
                signer: vec![0],
                sig: vec![255],
            }),
        },
        sender_pubkey: None,
        sender_sig: None,
        sender_delegation: None,
    };
    assert_eq!(
        encode_envelope(&envelope, MAXIMUM),
        Err(EncodeError::ByteLimit)
    );
    let length = encode_envelope(&envelope, MAXIMUM + 4096).unwrap().len();
    if let Content::Call { arg, .. } = &mut envelope.content {
        arg.truncate(2 * MAXIMUM - length);
    }
    let bytes = encode_envelope(&envelope, MAXIMUM).unwrap();
    assert_eq!(bytes.len(), MAXIMUM);
    let decoded = decode_envelope(&bytes, MAXIMUM).unwrap();
    assert_eq!(
        decoded.value.content.request_id().unwrap(),
        envelope.content.request_id().unwrap()
    );
    let Content::Call {
        sender_info, nonce, ..
    } = decoded.value.content
    else {
        panic!()
    };
    assert_eq!(nonce, Some(vec![]));
    assert_eq!(sender_info.unwrap().sig, [255]);
    assert_eq!(
        encode_envelope(&envelope, MAXIMUM - 1),
        Err(EncodeError::ByteLimit)
    );
}

fn original() -> ic_transport_types::EnvelopeContent {
    ic_transport_types::EnvelopeContent::Call {
        nonce: Some(vec![]),
        ingress_expiry: u64::MAX,
        sender: serde_cbor::from_slice(&[0x41, 4]).unwrap(),
        canister_id: serde_cbor::from_slice(&[0x40]).unwrap(),
        method_name: "method\0é".into(),
        arg: vec![0, 255, 128],
        sender_info: None,
    }
}

#[test]
fn upstream_identity_exact_bytes_and_presence() {
    let original = original();
    let bytes = serde_cbor::to_vec(&original).unwrap();
    let packet = decode_call_content(&bytes, bytes.len()).unwrap();
    assert_eq!(packet.original.as_ptr(), bytes.as_ptr());
    assert_eq!(
        packet.value.request_id().unwrap(),
        *original.to_request_id()
    );
    assert_eq!(
        packet.value.signable().unwrap(),
        original.to_request_id().signable()
    );
    assert_eq!(
        decode_call_content(&bytes, bytes.len() - 1).unwrap_err(),
        Error::ByteLimit
    );
    assert_eq!(
        decode_read_state_content(&bytes, bytes.len()).unwrap_err(),
        Error::Kind
    );
    let Content::Call {
        nonce,
        sender,
        canister_id,
        method_name,
        arg,
        ..
    } = packet.value
    else {
        panic!()
    };
    assert_eq!(nonce, Some(vec![]));
    assert_eq!(sender, Principal::anonymous());
    assert_eq!(canister_id, Principal::management_canister());
    assert_eq!(method_name, "method\0é");
    assert_eq!(arg, [0, 255, 128]);
    for delegation in [None, Some(vec![])] {
        let envelope = ic_transport_types::Envelope {
            content: Cow::Borrowed(&original),
            sender_pubkey: Some(vec![]),
            sender_sig: None,
            sender_delegation: delegation.clone(),
        };
        let bytes = envelope.encode_bytes();
        let decoded = decode_envelope(&bytes, bytes.len()).unwrap();
        assert_eq!(decoded.original, bytes);
        assert_eq!(decoded.value.sender_pubkey, Some(vec![]));
        assert_eq!(decoded.value.sender_sig, None);
        assert_eq!(
            decoded.value.sender_delegation.is_some(),
            delegation.is_some()
        );
        for end in 0..bytes.len() {
            assert!(decode_envelope(&bytes[..end], bytes.len()).is_err());
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(decode_envelope(&trailing, trailing.len()).is_err());
    }
}

#[test]
fn read_state_labels_and_id_use_original_upstream_algorithm() {
    let original = ic_transport_types::EnvelopeContent::ReadState {
        ingress_expiry: 123,
        sender: serde_cbor::from_slice(&[0x41, 4]).unwrap(),
        paths: vec![vec![vec![0, 128, 255].into(), vec![].into()], vec![]],
    };
    let bytes = serde_cbor::to_vec(&original).unwrap();
    let decoded = decode_read_state_content(&bytes, bytes.len()).unwrap();
    assert_eq!(
        decoded.value.request_id().unwrap(),
        *original.to_request_id()
    );
    assert_eq!(
        decoded.value.signable().unwrap(),
        original.to_request_id().signable()
    );
    let Content::ReadState { paths, .. } = decoded.value else {
        panic!()
    };
    assert_eq!(paths, vec![vec![vec![0, 128, 255], vec![]], vec![]]);
}

fn content_map() -> std::collections::BTreeMap<serde_cbor::Value, serde_cbor::Value> {
    let serde_cbor::Value::Map(map) = serde_cbor::value::to_value(original()).unwrap() else {
        panic!()
    };
    map
}

#[test]
fn strict_content_refuses_unknown_duplicates_and_wrong_byte_kinds() {
    use serde_cbor::Value as V;
    for (key, value) in [
        ("extra", V::Null),
        ("arg", V::Array(vec![V::Integer(0)])),
        ("nonce", V::Null),
        ("sender", V::Text("2vxsx-fae".into())),
        ("canister_id", V::Bytes(vec![1; 30])),
        ("request_type", V::Text("query".into())),
    ] {
        let mut map = content_map();
        map.insert(V::Text(key.into()), value);
        let bytes = serde_cbor::to_vec(&map).unwrap();
        assert!(decode_call_content(&bytes, bytes.len()).is_err(), "{key}");
    }
    let bytes = serde_cbor::to_vec(&original()).unwrap();
    assert!(bytes[0] >= 0xa0 && bytes[0] < 0xb7);
    for (key, value) in content_map() {
        let mut duplicate = bytes.clone();
        duplicate[0] += 1;
        duplicate.extend(serde_cbor::to_vec(&key).unwrap());
        duplicate.extend(serde_cbor::to_vec(&value).unwrap());
        assert!(decode_call_content(&duplicate, duplicate.len()).is_err());
    }
    for end in 0..bytes.len() {
        assert!(decode_call_content(&bytes[..end], bytes.len()).is_err());
    }
}

#[test]
fn outcomes_preserve_opaque_certificates_and_refuse_tuple_maps() {
    let bytes = serde_cbor::to_vec(&ic_transport_types::TransportCallResponse::Replied {
        certificate: vec![0, 255, 1],
    })
    .unwrap();
    let CallResponse::Replied { certificate } =
        decode_call_response(&bytes, bytes.len()).unwrap().value
    else {
        panic!()
    };
    assert_eq!(certificate, [0, 255, 1]);
    let accepted =
        serde_cbor::to_vec(&ic_transport_types::TransportCallResponse::Accepted).unwrap();
    assert!(matches!(
        decode_call_response(&accepted, accepted.len())
            .unwrap()
            .value,
        CallResponse::Accepted
    ));
    let response = serde_cbor::to_vec(&ic_transport_types::ReadStateResponse {
        certificate: vec![],
    })
    .unwrap();
    assert!(
        decode_read_state_response(&response, response.len())
            .unwrap()
            .value
            .certificate
            .is_empty()
    );
    assert!(decode_read_state_response(b"\x81\x40", 2).is_err());
    assert!(decode_read_state_response(b"\xa1\x6bcertificate\x80", 100).is_err());
}

#[test]
fn delegation_and_sender_information_preserve_presence_and_reject_nested_arrays() {
    use serde_cbor::Value as V;
    let mut call = content_map();
    let info = V::Map(
        [
            (V::Text("info".into()), V::Bytes(vec![])),
            (V::Text("signer".into()), V::Bytes(vec![4])),
            (V::Text("sig".into()), V::Bytes(vec![255])),
        ]
        .into(),
    );
    call.insert(V::Text("sender_info".into()), info);
    let bytes = serde_cbor::to_vec(&call).unwrap();
    let decoded = decode_call_content(&bytes, bytes.len()).unwrap();
    let original: ic_transport_types::EnvelopeContent = serde_cbor::from_slice(&bytes).unwrap();
    assert_eq!(
        decoded.value.request_id().unwrap(),
        *original.to_request_id()
    );
    let Content::Call { sender_info, .. } = decoded.value else {
        panic!()
    };
    let info = sender_info.unwrap();
    assert_eq!(info.sig, [255]);
    assert!(info.info.is_empty());
    for bad in [
        V::Null,
        V::Array(vec![
            V::Bytes(vec![]),
            V::Bytes(vec![4]),
            V::Bytes(vec![255]),
        ]),
    ] {
        call.insert(V::Text("sender_info".into()), bad);
        let bytes = serde_cbor::to_vec(&call).unwrap();
        assert!(decode_call_content(&bytes, bytes.len()).is_err());
    }
    let delegation = ic_transport_types::SignedDelegation {
        delegation: ic_transport_types::Delegation {
            pubkey: vec![0, 255],
            expiration: u64::MAX,
            targets: Some(vec![]),
            permissions: Some(ic_transport_types::DelegationPermissions::Queries),
        },
        signature: vec![128, 0],
    };
    let envelope = ic_transport_types::Envelope {
        content: Cow::Owned(original),
        sender_pubkey: Some(vec![1]),
        sender_sig: Some(vec![]),
        sender_delegation: Some(vec![delegation]),
    };
    let bytes = envelope.encode_bytes();
    let packet = decode_envelope(&bytes, bytes.len()).unwrap();
    let chain = packet.value.sender_delegation.unwrap();
    assert_eq!(chain[0].signature, [128, 0]);
    assert_eq!(chain[0].delegation.targets, Some(vec![]));
    assert_eq!(chain[0].delegation.expiration, u64::MAX);
    assert!(matches!(
        chain[0].delegation.permissions,
        Some(Permissions::Queries)
    ));
    let oversized = vec![0; 2 * 1024 * 1024 + 1];
    assert_eq!(
        decode_envelope(&oversized, 2 * 1024 * 1024).unwrap_err(),
        Error::ByteLimit
    );
}

#[test]
fn synchronous_rejection_stays_distinct_from_certified_execution() {
    let original = ic_transport_types::TransportCallResponse::NonReplicatedRejection(
        ic_transport_types::RejectResponse {
            reject_code: ic_transport_types::RejectCode::CanisterReject,
            reject_message: "rejected\0é".into(),
            error_code: Some(String::new()),
        },
    );
    let bytes = serde_cbor::to_vec(&original).unwrap();
    let CallResponse::NonReplicatedRejection {
        reject_code,
        reject_message,
        error_code,
    } = decode_call_response(&bytes, bytes.len()).unwrap().value
    else {
        panic!()
    };
    assert_eq!(reject_code, 4);
    assert_eq!(reject_message, "rejected\0é");
    assert_eq!(error_code, Some(String::new()));
    for end in 0..bytes.len() {
        assert!(decode_call_response(&bytes[..end], bytes.len()).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(decode_call_response(&trailing, trailing.len()).is_err());
}
