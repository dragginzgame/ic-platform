use super::*;
use crate::certification::cbor::encode_certificate_untagged;
#[cfg(feature = "ic-protocol-fixtures")]
use crate::certification::cbor::strict;
use crate::certification::{Certificate, empty};
use std::borrow::Cow;

#[test]
fn testkit_read_state_preserves_sdk_framing_and_original_certificate_bytes() {
    let content = EnvelopeContent::ReadState {
        ingress_expiry: u64::MAX,
        sender: ExternalPrincipal::anonymous(),
        paths: vec![vec![
            b"subnet".to_vec().into(),
            vec![0, 255].into(),
            b"metrics".to_vec().into(),
        ]],
    };
    let envelope = Envelope {
        content: Cow::Borrowed(&content),
        sender_pubkey: None,
        sender_sig: None,
        sender_delegation: None,
    };
    let expected = envelope.encode_bytes();
    let encoded = encode_envelope(&envelope, expected.len()).unwrap();
    assert_eq!(encoded, expected);
    assert_eq!(&encoded[..3], &[0xd9, 0xd9, 0xf7]);
    let certificate = Certificate {
        tree: empty(),
        signature: vec![0, 255, 128],
        delegation: None,
    };
    let raw_certificate = encode_certificate_untagged(&certificate, 1024).unwrap();
    let response = serde_cbor::to_vec(&ReadStateResponse {
        certificate: raw_certificate.clone(),
    })
    .unwrap();
    assert_eq!(
        decode_read_state_response(&response, response.len())
            .unwrap()
            .certificate,
        raw_certificate
    );
    assert!(matches!(
        decode_read_state_response(&response, response.len() - 1),
        Err(CborError::InputTooLarge)
    ));
    let mut trailing = response;
    trailing.push(0);
    assert!(matches!(
        decode_read_state_response(&trailing, trailing.len()),
        Err(CborError::Decode(_))
    ));
    assert!(matches!(
        decode_read_state_response(&[0xff], 1),
        Err(CborError::Decode(_))
    ));
}

#[test]
fn subnet_metrics_preserve_every_field_across_the_full_u128_range() {
    for cycles in [
        0,
        u128::from(u64::MAX),
        u128::from(u64::MAX) + 1,
        1u128 << 127,
        u128::MAX,
    ] {
        let expected = SubnetMetrics {
            num_canisters: u64::MAX,
            canister_state_bytes: 8,
            consumed_cycles_total: cycles,
            update_transactions_total: u64::MAX,
        };
        let encoded = serde_cbor::to_vec(&expected).unwrap();
        let decoded = decode_subnet_metrics(&encoded, encoded.len()).unwrap();
        assert_eq!(decoded.num_canisters, expected.num_canisters);
        assert_eq!(decoded.canister_state_bytes, expected.canister_state_bytes);
        assert_eq!(decoded.consumed_cycles_total, cycles);
        assert_eq!(
            decoded.update_transactions_total,
            expected.update_transactions_total
        );
        assert!(matches!(
            decode_subnet_metrics(&encoded, encoded.len() - 1),
            Err(CborError::InputTooLarge)
        ));
        let mut trailing = encoded;
        trailing.push(0);
        assert!(matches!(
            decode_subnet_metrics(&trailing, trailing.len()),
            Err(CborError::Decode(_))
        ));
    }
    assert!(matches!(
        decode_subnet_metrics(&[0xff], 1),
        Err(CborError::Decode(_))
    ));
}

#[test]
fn sdk_envelope_bytes_identity_and_budgets_are_preserved() {
    let content = EnvelopeContent::Call {
        ingress_expiry: u64::MAX,
        sender: ExternalPrincipal::anonymous(),
        canister_id: ExternalPrincipal::management_canister(),
        method_name: "opaque".into(),
        arg: vec![0, 255, 128],
        nonce: Some(vec![8]),
        sender_info: None,
    };
    let envelope = Envelope {
        content: Cow::Owned(content),
        sender_pubkey: Some(vec![1]),
        sender_sig: Some(vec![2]),
        sender_delegation: None,
    };
    let expected = envelope.encode_bytes();
    assert_eq!(
        encode_envelope(&envelope, expected.len()).unwrap(),
        expected
    );
    let restored = decode_envelope(&expected, expected.len()).unwrap();
    assert_eq!(
        ic_transport_types::to_request_id(&restored.content).unwrap(),
        ic_transport_types::to_request_id(&envelope.content).unwrap()
    );
    assert_eq!(restored.sender_sig, envelope.sender_sig);
    let strict = decode_call_content_strict(&expected, expected.len()).unwrap();
    assert_eq!(
        ic_transport_types::to_request_id(&strict).unwrap(),
        ic_transport_types::to_request_id(&envelope.content).unwrap()
    );
    assert!(matches!(
        decode_envelope(&expected, expected.len() - 1),
        Err(CborError::InputTooLarge)
    ));
    assert!(matches!(
        encode_envelope(&envelope, expected.len() - 1),
        Err(CborError::OutputTooLarge)
    ));
    let mut trailing = expected;
    trailing.push(0);
    assert!(matches!(
        decode_envelope(&trailing, trailing.len()),
        Err(CborError::Decode(_))
    ));
}

#[test]
fn strict_call_inspection_keeps_unknown_field_and_request_shape_refusal() {
    use serde_cbor::Value;
    let mut fields = std::collections::BTreeMap::from([
        (
            Value::Text("request_type".into()),
            Value::Text("call".into()),
        ),
        (
            Value::Text("ingress_expiry".into()),
            Value::Integer(u64::MAX.into()),
        ),
        (Value::Text("sender".into()), Value::Bytes(vec![4])),
        (Value::Text("canister_id".into()), Value::Bytes(vec![])),
        (
            Value::Text("method_name".into()),
            Value::Text("method".into()),
        ),
        (Value::Text("arg".into()), Value::Bytes(vec![0, 255])),
    ]);
    let packet = |fields| {
        serde_cbor::to_vec(&Value::Map(std::collections::BTreeMap::from([(
            Value::Text("content".into()),
            Value::Map(fields),
        )])))
        .unwrap()
    };
    let valid = packet(fields.clone());
    assert!(decode_call_content_strict(&valid, valid.len()).is_ok());
    for (key, value) in [
        ("extra", Value::Null),
        ("request_type", Value::Text("query".into())),
        ("sender", Value::Bytes(vec![1; 30])),
    ] {
        let mut changed = fields.clone();
        changed.insert(Value::Text(key.into()), value);
        let bytes = packet(changed);
        assert!(decode_call_content_strict(&bytes, bytes.len()).is_err());
    }
    fields.insert(
        Value::Text("sender_info".into()),
        serde_cbor::value::to_value(ic_transport_types::SenderInfo {
            info: vec![1],
            signer: vec![2],
            sig: vec![3],
        })
        .unwrap(),
    );
    let bytes = packet(fields);
    assert!(matches!(
        decode_call_content_strict(&bytes, bytes.len()).unwrap(),
        EnvelopeContent::Call {
            sender_info: Some(_),
            ..
        }
    ));
}

#[cfg(feature = "ic-protocol-fixtures")]
#[test]
fn typed_protocol_fixtures_keep_payload_ranges_and_strict_rejections() {
    let reply = vec![255, 0, 128];
    let bytes = fixtures::query_reply(&reply, 1024).unwrap();
    let decoded: ic_transport_types::QueryResponse = decode(&bytes, bytes.len()).unwrap();
    assert!(
        matches!(decoded, ic_transport_types::QueryResponse::Replied { reply: ic_transport_types::ReplyResponse { arg }, .. } if arg == reply)
    );
    let value = SubnetMetrics {
        num_canisters: u64::MAX,
        canister_state_bytes: 5,
        consumed_cycles_total: u128::MAX,
        update_transactions_total: 7,
    };
    let bytes = fixtures::subnet_metrics(&value, 1024).unwrap();
    assert_eq!(bytes, serde_cbor::to_vec(&value).unwrap());
    assert_eq!(
        decode_subnet_metrics(&bytes, bytes.len())
            .unwrap()
            .consumed_cycles_total,
        u128::MAX
    );
    let certificate = Certificate {
        tree: empty(),
        signature: vec![0; 48],
        delegation: None,
    };
    for bytes in [
        fixtures::certificate_with_unknown_field(&certificate, 1024).unwrap(),
        fixtures::certificate_with_duplicate_tree(&certificate, 1024).unwrap(),
    ] {
        assert!(
            strict::decode_certificate(&bytes, strict::DecodeLimits { bytes: bytes.len() })
                .is_err()
        );
    }
    let bytes = encode_certificate_untagged(&certificate, 1024).unwrap();
    assert_eq!(bytes, serde_cbor::to_vec(&certificate).unwrap());
}
