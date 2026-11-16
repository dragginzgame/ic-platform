use super::signed::*;
use super::*;
use crate::certificate::{
    TimeWindow,
    construction::{Certificate, encode_certificate, fork, labeled, leaf},
};
use crate::ed25519::{Signer, SigningKey};
use crate::ingress;
use crate::ingress::Content;
use ed25519_dalek::pkcs8::EncodePrivateKey;
use ic_verify_bls_signature::PrivateKey;
fn identity() -> Ed25519Identity {
    let key = SigningKey::from_bytes(&[9; 32]);
    let pem = key.to_pkcs8_pem(Default::default()).unwrap();
    Ed25519Identity::from_pem(&pem, 4096).unwrap()
}
fn update() -> PreparedUpdate {
    identity()
        .prepare_update(
            UpdateArgs {
                recipient: Principal::management_canister(),
                effective: Principal::from_slice(&[1]),
                method: "canister_status",
                arg: b"original",
                expiry: 1,
                nonce: Some(b"nonce"),
            },
            4096,
        )
        .unwrap()
}
fn fixture() -> (ReadClient, PrivateKey, TimePolicy) {
    let mut raw = [0; 32];
    raw[31] = 1;
    let key = PrivateKey::deserialize(&raw).unwrap();
    let client = ReadClient::new(ReadConfig {
        endpoint: "http://127.0.0.1:1".into(),
        root: PreparedRootKey::from_raw(&key.public_key().serialize()).unwrap(),
        root_subnet: Principal::from_slice(&[2]),
        message_bytes: 4096,
        request_timeout: Duration::from_secs(1),
        verification: VerificationLimits {
            tree_nodes: 128,
            tree_depth: 32,
            delegation_bytes: 4096,
            canister_ranges: 16,
        },
    })
    .unwrap();
    let window = TimeWindow {
        reference_ns: 100,
        max_age_ns: 0,
        max_future_skew_ns: 0,
    };
    (
        client,
        key,
        TimePolicy {
            certificate: window,
            delegation: window,
        },
    )
}
fn proof(key: &PrivateKey, id: &[u8; 32], status: &[u8], reply: &[u8]) -> Vec<u8> {
    let tree = fork(
        labeled(
            b"request_status".to_vec(),
            labeled(
                id.to_vec(),
                fork(
                    labeled(b"reply".to_vec(), leaf(reply.to_vec())),
                    labeled(b"status".to_vec(), leaf(status.to_vec())),
                ),
            ),
        ),
        labeled(b"time".to_vec(), leaf(vec![100])),
    );
    let sig = key.sign(&[b"\x0dic-state-root".as_slice(), &tree.digest()].concat());
    encode_certificate(
        &Certificate {
            tree,
            signature: sig.serialize().to_vec(),
            delegation: None,
        },
        4096,
    )
    .unwrap()
}
#[test]
fn offline_signing_and_exact_original_recovery() {
    let original = update();
    assert_eq!(original.sender(), identity().principal());
    assert_eq!(original.expiry(), 1);
    assert_ne!(original.recipient(), original.effective_canister());
    // Noncanonical-but-valid outer CBOR self-description remains byte-for-byte.
    let bytes = [b"\xd9\xd9\xf7".as_slice(), original.envelope_bytes()].concat();
    let loaded = PreparedUpdate::from_bytes(&bytes, original.effective_canister(), 4096).unwrap();
    assert_eq!(loaded.envelope_bytes(), bytes);
    assert_eq!(loaded.request_id(), original.request_id());
    assert!(
        PreparedUpdate::from_bytes(&bytes, original.effective_canister(), bytes.len() - 1).is_err()
    );
    let mut envelope = ingress::decode_envelope(original.envelope_bytes(), 4096)
        .unwrap()
        .value;
    envelope.sender_sig.as_mut().unwrap()[0] ^= 1;
    assert!(
        PreparedUpdate::from_bytes(
            &serde_cbor::to_vec(&envelope).unwrap(),
            original.effective_canister(),
            4096
        )
        .is_err()
    );
    assert!(Ed25519Identity::from_pem("garbage", 4096).is_err());
    assert!(
        identity()
            .prepare_update(
                UpdateArgs {
                    recipient: original.recipient(),
                    effective: original.effective_canister(),
                    method: "x",
                    arg: b"",
                    expiry: 1,
                    nonce: None
                },
                1
            )
            .is_err()
    );
}
#[test]
fn certified_original_status_is_bound_and_historical_policy_is_explicit() {
    let (client, key, time) = fixture();
    let original = update();
    let bytes = proof(&key, original.request_id(), b"replied", b"original reply");
    let observed = client
        .authenticate_request(&original, &bytes, time)
        .unwrap();
    assert_eq!(observed.certificate_bytes(), bytes);
    assert_eq!(
        observed.status(),
        &RequestStatus::Replied(b"original reply".to_vec())
    );
    assert_eq!(observed.request_id(), original.request_id());
    assert_eq!(observed.time_ns(), 100); // Expired original update remains recoverable.
    let wrong = proof(&key, &[8; 32], b"replied", b"other reply");
    assert_eq!(
        client
            .authenticate_request(&original, &wrong, time)
            .unwrap()
            .status(),
        &RequestStatus::Unknown
    );
    for (name, expected) in [
        (b"received".as_slice(), RequestStatus::Received),
        (b"processing", RequestStatus::Processing),
        (b"done", RequestStatus::Done),
    ] {
        assert_eq!(
            client
                .authenticate_request(
                    &original,
                    &proof(&key, original.request_id(), name, b""),
                    time
                )
                .unwrap()
                .status(),
            &expected
        );
    }
    assert!(
        client
            .authenticate_request(
                &original,
                &proof(&key, original.request_id(), b"invented", b""),
                time
            )
            .is_err()
    );
    let future = TimePolicy {
        certificate: TimeWindow {
            reference_ns: 101,
            ..time.certificate
        },
        ..time
    };
    assert!(
        client
            .authenticate_request(&original, &bytes, future)
            .is_err()
    );
    let mut raw = [0; 32];
    raw[31] = 2;
    let wrong_key = PrivateKey::deserialize(&raw).unwrap();
    assert!(
        client
            .authenticate_request(
                &original,
                &proof(&wrong_key, original.request_id(), b"replied", b""),
                time
            )
            .is_err()
    );
    for end in 0..bytes.len() {
        assert!(
            client
                .authenticate_request(&original, &bytes[..end], time)
                .is_err()
        );
    }
    let mut changed = bytes;
    let index = changed
        .windows(14)
        .position(|w| w == b"original reply")
        .unwrap();
    changed[index] ^= 1;
    assert!(
        client
            .authenticate_request(&original, &changed, time)
            .is_err()
    );
}

#[test]
fn delegated_original_request_requires_effective_target_and_both_time_windows() {
    use crate::certificate::construction::Delegation;
    let (client, root, time) = fixture();
    let original = update();
    let mut raw = [0; 32];
    raw[31] = 2;
    let subnet_key = PrivateKey::deserialize(&raw).unwrap();
    let subnet = Principal::from_slice(&[2]);
    let public = PreparedRootKey::from_raw(&subnet_key.public_key().serialize()).unwrap();
    let tree = fork(
        labeled(
            b"subnet".to_vec(),
            labeled(
                subnet.as_slice().to_vec(),
                fork(
                    labeled(
                        b"canister_ranges".to_vec(),
                        leaf(b"\x81\x82\x41\x01\x41\x01".to_vec()),
                    ),
                    labeled(b"public_key".to_vec(), leaf(public.as_der().to_vec())),
                ),
            ),
        ),
        labeled(b"time".to_vec(), leaf(vec![100])),
    );
    let signature = root
        .sign(&[b"\x0dic-state-root".as_slice(), &tree.digest()].concat())
        .serialize()
        .to_vec();
    let authority = encode_certificate(
        &Certificate {
            tree,
            signature,
            delegation: None,
        },
        4096,
    )
    .unwrap();
    let direct = proof(
        &subnet_key,
        original.request_id(),
        b"replied",
        b"delegated reply",
    );
    let mut cert: Certificate = serde_cbor::from_slice(&direct).unwrap();
    cert.delegation = Some(Delegation {
        subnet_id: subnet.as_slice().to_vec(),
        certificate: authority,
    });
    let bytes = encode_certificate(&cert, 4096).unwrap();
    assert_eq!(
        client
            .authenticate_request(&original, &bytes, time)
            .unwrap()
            .status(),
        &RequestStatus::Replied(b"delegated reply".to_vec())
    );
    let wrong =
        PreparedUpdate::from_bytes(original.envelope_bytes(), Principal::from_slice(&[3]), 4096)
            .unwrap();
    assert!(client.authenticate_request(&wrong, &bytes, time).is_err());
    assert!(
        client
            .authenticate_request(
                &original,
                &bytes,
                TimePolicy {
                    delegation: TimeWindow {
                        reference_ns: 101,
                        ..time.delegation
                    },
                    ..time
                }
            )
            .is_err()
    );
}

fn status_envelope(update: &PreparedUpdate) -> ingress::Envelope {
    ingress::Envelope {
        content: ingress::Content::ReadState {
            ingress_expiry: 777,
            sender: update.sender(),
            paths: vec![vec![
                b"request_status".to_vec(),
                update.request_id().to_vec(),
            ]],
        },
        sender_pubkey: None,
        sender_sig: None,
        sender_delegation: None,
    }
}
fn sign_status(mut envelope: ingress::Envelope, seed: u8) -> Vec<u8> {
    let key = SigningKey::from_bytes(&[seed; 32]);
    envelope.sender_pubkey = Some(
        [
            b"\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00".as_slice(),
            &key.verifying_key().to_bytes(),
        ]
        .concat(),
    );
    envelope.sender_sig = Some(
        key.sign(&envelope.content.signable().unwrap())
            .to_bytes()
            .to_vec(),
    );
    serde_cbor::to_vec(&envelope).unwrap()
}
#[test]
fn external_status_admission_preserves_bytes_and_refuses_binding_changes() {
    let update = update();
    let bytes = [
        b"\xd9\xd9\xf7".as_slice(),
        &sign_status(status_envelope(&update), 9),
    ]
    .concat();
    let status = PreparedRequestStatus::from_bytes(&bytes, &update, bytes.len()).unwrap();
    assert_eq!(status.envelope_bytes(), bytes);
    assert_eq!(status.expiry(), 777);
    assert_eq!(update.expiry(), 1);
    assert!(PreparedRequestStatus::from_bytes(&bytes, &update, bytes.len() - 1).is_err());
    for index in 0..6 {
        let mut envelope = status_envelope(&update);
        if let Content::ReadState { sender, paths, .. } = &mut envelope.content {
            match index {
                0 => *sender = Principal::anonymous(),
                1 => paths[0][0] = b"time".to_vec(),
                2 => paths[0][1][0] ^= 1,
                3 => paths.push(vec![b"time".to_vec()]),
                4 => paths[0].push(b"reply".to_vec()),
                _ => paths.clear(),
            }
        }
        assert!(
            PreparedRequestStatus::from_bytes(&sign_status(envelope, 9), &update, 4096).is_err()
        );
    }
    assert!(
        PreparedRequestStatus::from_bytes(&sign_status(status_envelope(&update), 8), &update, 4096)
            .is_err()
    );
    let mut envelope = ingress::decode_envelope(&bytes, 4096).unwrap().value;
    envelope.sender_delegation = Some(vec![]);
    assert!(
        PreparedRequestStatus::from_bytes(&serde_cbor::to_vec(&envelope).unwrap(), &update, 4096)
            .is_err()
    );
    for length in 0..bytes.len() {
        assert!(PreparedRequestStatus::from_bytes(&bytes[..length], &update, 4096).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(PreparedRequestStatus::from_bytes(&trailing, &update, 4096).is_err());
}

#[cfg(feature = "native-signed-secp256k1")]
#[test]
fn direct_secp256k1_external_profile_uses_upstream_verifier() {
    let key = crate::ecdsa::SigningKey::from_slice(&[7; 32]).unwrap();
    let public = [b"\x30\x56\x30\x10\x06\x07\x2a\x86\x48\xce\x3d\x02\x01\x06\x05\x2b\x81\x04\x00\x0a\x03\x42\x00".as_slice(), key.verifying_key().to_sec1_point(false).as_bytes()].concat();
    let mut envelope = ingress::decode_envelope(update().envelope_bytes(), 4096)
        .unwrap()
        .value;
    if let Content::Call { sender, .. } = &mut envelope.content {
        *sender = Principal::self_authenticating(&public);
    }
    envelope.sender_pubkey = Some(public);
    let signature: crate::ecdsa::Signature = key.sign(&envelope.content.signable().unwrap());
    envelope.sender_sig = Some(signature.to_bytes().to_vec());
    let bytes = serde_cbor::to_vec(&envelope).unwrap();
    let admitted = PreparedUpdate::from_bytes(&bytes, Principal::from_slice(&[1]), 4096).unwrap();
    assert_eq!(admitted.envelope_bytes(), bytes);
    envelope.sender_sig.as_mut().unwrap()[0] ^= 1;
    assert!(
        PreparedUpdate::from_bytes(
            &serde_cbor::to_vec(&envelope).unwrap(),
            Principal::from_slice(&[1]),
            4096
        )
        .is_err()
    );
}
