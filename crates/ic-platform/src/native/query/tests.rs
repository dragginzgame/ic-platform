use super::*;
use crate::certificate::{
    TimeWindow,
    construction::{Certificate, encode_certificate, fork, labeled, leaf},
};
use crate::ed25519::{Signer, SigningKey};
use crate::{
    certificate::{PreparedRootKey, VerificationLimits, VerifyError},
    native::ReadConfig,
};
use ic_transport_types::{NodeSignature, ReplyResponse, RequestId};
use ic_verify_bls_signature::PrivateKey;
use std::time::Duration;
fn fixtures() -> (
    ReadClient,
    SubnetObservation,
    QueryResponse,
    RequestId,
    TimeWindow,
) {
    let mut raw = [0; 32];
    raw[31] = 1;
    let signer = PrivateKey::deserialize(&raw).unwrap();
    let node = SigningKey::from_bytes(&[9; 32]);
    let key = [
        b"\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00".as_slice(),
        &node.verifying_key().to_bytes(),
    ]
    .concat();
    let subnet = Principal::from_slice(&[1]);
    let identity = Principal::from_slice(&[2]);
    let tree = fork(
        labeled(
            b"subnet".to_vec(),
            labeled(
                subnet.as_slice().to_vec(),
                fork(
                    labeled(
                        b"node".to_vec(),
                        labeled(
                            identity.as_slice().to_vec(),
                            labeled(b"public_key".to_vec(), leaf(key)),
                        ),
                    ),
                    labeled(b"type".to_vec(), leaf(b"future-subnet-kind".to_vec())),
                ),
            ),
        ),
        labeled(b"time".to_vec(), leaf(vec![100])),
    );
    let signed = [b"\x0dic-state-root".as_slice(), &tree.digest()].concat();
    let cert = Certificate {
        tree,
        signature: signer.sign(&signed).serialize().to_vec(),
        delegation: None,
    };
    let bytes = encode_certificate(&cert, 4096).unwrap();
    let client = ReadClient::new(ReadConfig {
        endpoint: "https://example.invalid".into(),
        root: PreparedRootKey::from_raw(&signer.public_key().serialize()).unwrap(),
        root_subnet: subnet,
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
    let time = TimeWindow {
        reference_ns: 100,
        max_age_ns: 0,
        max_future_skew_ns: 0,
    };
    let observation = client
        .admit_subnet(
            bytes,
            subnet,
            TimePolicy {
                certificate: time,
                delegation: time,
            },
        )
        .unwrap();
    assert_eq!(observation.subnet_type(), Some("future-subnet-kind"));
    assert!(observation.metrics().is_none());
    let id = RequestId::new(&[7; 32]);
    let mut response = QueryResponse::Replied {
        reply: ReplyResponse { arg: vec![0, 255] },
        signatures: vec![],
    };
    let signature = NodeSignature {
        timestamp: 100,
        identity: identity.to_string().parse().unwrap(),
        signature: node.sign(&response.signable(id, 100)).to_bytes().to_vec(),
    };
    if let QueryResponse::Replied { signatures, .. } = &mut response {
        signatures.push(signature)
    }
    (client, observation, response, id, time)
}
#[test]
fn response_bytes_request_identity_time_and_node_authority_are_bound() {
    let (_, nodes, response, id, time) = fixtures();
    assert_eq!(
        admit_response(response.clone(), id, &nodes, time).unwrap(),
        [0, 255]
    );
    assert!(matches!(
        admit_response(response.clone(), RequestId::new(&[8; 32]), &nodes, time),
        Err(ReadError::QuerySignature)
    ));
    let mut older = time;
    older.reference_ns = 101;
    assert!(matches!(
        admit_response(response.clone(), id, &nodes, older),
        Err(ReadError::QueryTime)
    ));
    for kind in 0..5 {
        let mut altered = response.clone();
        if let QueryResponse::Replied { reply, signatures } = &mut altered {
            match kind {
                0 => signatures.clear(),
                1 => signatures.push(signatures[0].clone()),
                2 => {
                    signatures[0].identity =
                        Principal::from_slice(&[3]).to_string().parse().unwrap()
                }
                3 => signatures[0].signature[0] ^= 1,
                _ => reply.arg.push(9),
            }
        }
        assert!(matches!(
            admit_response(altered, id, &nodes, time),
            Err(ReadError::QuerySignature)
        ));
    }
}
#[test]
fn selected_subnet_certificate_cannot_be_rebound_or_retimed() {
    let (client, nodes, _, _, time) = fixtures();
    let policy = TimePolicy {
        certificate: time,
        delegation: time,
    };
    assert!(matches!(
        client.admit_subnet(
            nodes.certificate_bytes().to_vec(),
            Principal::from_slice(&[3]),
            policy
        ),
        Err(ReadError::Verification(VerifyError::SubnetMismatch))
    ));
    let stale = TimePolicy {
        certificate: TimeWindow {
            reference_ns: 101,
            ..time
        },
        delegation: time,
    };
    assert!(matches!(
        client.admit_subnet(nodes.certificate_bytes().to_vec(), nodes.subnet(), stale),
        Err(ReadError::Verification(VerifyError::TimeOutsideWindow))
    ));
    let mut corrupt = nodes.certificate_bytes().to_vec();
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(
        client
            .admit_subnet(corrupt, nodes.subnet(), policy)
            .is_err()
    );
}
