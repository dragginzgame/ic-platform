use super::*;
use crate::certificate::{
    TimeWindow,
    construction::{
        Certificate, Delegation, HashTree, encode_certificate, fork, labeled, leaf, pruned,
    },
};
use ic_verify_bls_signature::PrivateKey;

fn key(n: u8) -> PrivateKey {
    let mut bytes = [0; 32];
    bytes[31] = n;
    PrivateKey::deserialize(&bytes).unwrap()
}

fn policy() -> TimePolicy {
    let w = TimeWindow {
        reference_ns: 100,
        max_age_ns: 0,
        max_future_skew_ns: 0,
    };
    TimePolicy {
        certificate: w,
        delegation: w,
    }
}

fn client() -> ReadClient {
    ReadClient::new(ReadConfig {
        endpoint: "http://127.0.0.1:1".into(),
        root: PreparedRootKey::from_raw(&key(1).public_key().serialize()).unwrap(),
        root_subnet: Principal::from_slice(&[2]),
        message_bytes: 8192,
        request_timeout: Duration::from_secs(1),
        verification: VerificationLimits {
            tree_nodes: 128,
            tree_depth: 32,
            delegation_bytes: 4096,
            canister_ranges: 8,
        },
    })
    .unwrap()
}

fn signed(tree: HashTree, signer: &PrivateKey, delegation: Option<Delegation>) -> Vec<u8> {
    let signature = signer
        .sign(&[b"\x0dic-state-root".as_slice(), &tree.digest()].concat())
        .serialize()
        .to_vec();
    encode_certificate(
        &Certificate {
            tree,
            signature,
            delegation,
        },
        8192,
    )
    .unwrap()
}

fn proof(controllers: HashTree, delegated: bool, target: u8, range: u8) -> Vec<u8> {
    let tree = fork(
        labeled(b"canister".to_vec(), labeled(vec![target], controllers)),
        labeled(b"time".to_vec(), leaf(vec![100])),
    );
    let delegation = delegated.then(|| {
        let ranges = serde_cbor::to_vec(&vec![(
            serde_bytes::ByteBuf::from(vec![range]),
            serde_bytes::ByteBuf::from(vec![range]),
        )])
        .unwrap();
        let tree = fork(
            labeled(
                b"subnet".to_vec(),
                labeled(
                    vec![3],
                    fork(
                        labeled(b"canister_ranges".to_vec(), leaf(ranges)),
                        labeled(
                            b"public_key".to_vec(),
                            leaf(
                                PreparedRootKey::from_raw(&key(2).public_key().serialize())
                                    .unwrap()
                                    .as_der()
                                    .to_vec(),
                            ),
                        ),
                    ),
                ),
            ),
            labeled(b"time".to_vec(), leaf(vec![100])),
        );
        Delegation {
            subnet_id: vec![3],
            certificate: signed(tree, &key(1), None),
        }
    });
    signed(tree, &key(if delegated { 2 } else { 1 }), delegation)
}

fn controllers(raw: Vec<u8>) -> HashTree {
    labeled(b"controllers".to_vec(), leaf(raw))
}

#[test]
fn admitted_controllers_and_placement_retain_original_proof() {
    for delegated in [false, true] {
        let raw = vec![0x82, 0x41, 1, 0x41, 4];
        let bytes = proof(controllers(raw), delegated, 7, 7);
        let c = client();
        let target = Principal::from_slice(&[7]);
        let evidence = c
            .admit_canister(bytes.clone(), target, policy(), 2)
            .unwrap();
        assert_eq!(
            evidence.controllers(),
            [Principal::from_slice(&[1]), Principal::from_slice(&[4])]
        );
        assert_eq!(evidence.canister(), target);
        assert_eq!(
            evidence.subnet(),
            Principal::from_slice(&[if delegated { 3 } else { 2 }])
        );
        assert_eq!(evidence.time_ns(), 100);
        assert_eq!(evidence.certificate_bytes(), bytes);
        assert!(matches!(
            c.admit_canister(bytes.clone(), target, policy(), 1),
            Err(CanisterReadError::ControllerLimit)
        ));
        let mut stale = policy();
        stale.certificate.reference_ns = 101;
        assert!(matches!(
            c.admit_canister(bytes.clone(), target, stale, 2),
            Err(CanisterReadError::Read(ReadError::Verification(
                VerifyError::TimeOutsideWindow
            )))
        ));
        if delegated {
            let mut old = policy();
            old.delegation.reference_ns = 101;
            assert!(matches!(
                c.admit_canister(bytes.clone(), target, old, 2),
                Err(CanisterReadError::Read(ReadError::Verification(
                    VerifyError::TimeOutsideWindow
                )))
            ));
        }
        let mut wrong = client();
        wrong.config.root = PreparedRootKey::from_raw(&key(3).public_key().serialize()).unwrap();
        assert!(matches!(
            wrong.admit_canister(bytes.clone(), target, policy(), 2),
            Err(CanisterReadError::Read(ReadError::Verification(_)))
        ));
        assert!(
            c.admit_canister(bytes, Principal::from_slice(&[8]), policy(), 2)
                .is_err()
        );
    }
}

#[test]
fn missing_pruned_wrong_range_and_malformed_controller_leaves_refuse() {
    let c = client();
    let target = Principal::from_slice(&[7]);
    let b = proof(
        labeled(b"module_hash".to_vec(), leaf(vec![0; 32])),
        false,
        7,
        7,
    );
    assert!(matches!(
        c.admit_canister(b, target, policy(), 10),
        Err(CanisterReadError::Read(ReadError::MissingPath))
    ));
    let b = proof(pruned([0; 32]), false, 7, 7);
    assert!(matches!(
        c.admit_canister(b, target, policy(), 10),
        Err(CanisterReadError::PrunedPath)
    ));
    let b = proof(controllers(vec![0x80]), true, 7, 8);
    assert!(matches!(
        c.admit_canister(b, target, policy(), 10),
        Err(CanisterReadError::Read(ReadError::Verification(
            VerifyError::NotAuthorized
        )))
    ));
    for raw in [
        vec![0x81, 0x61, b'a'],
        vec![0x81, 0x81, 1],
        vec![0x82, 0x41, 1, 0x41, 1],
        vec![0x80, 0],
        vec![0x81],
        [vec![0x81, 0x58, 30], vec![1; 30]].concat(),
    ] {
        assert!(matches!(
            c.admit_canister(proof(controllers(raw), false, 7, 7), target, policy(), 10),
            Err(CanisterReadError::Read(ReadError::Packet))
        ));
    }
    let empty = c
        .admit_canister(
            proof(controllers(vec![0x80]), false, 7, 7),
            target,
            policy(),
            0,
        )
        .unwrap();
    assert!(empty.controllers().is_empty());
    let b = proof(controllers(vec![0x9f, 0x41, 1, 0xff]), false, 7, 7);
    assert!(matches!(
        c.admit_canister(b, target, policy(), 0),
        Err(CanisterReadError::ControllerLimit)
    ));
}

#[test]
fn canister_proof_input_and_tree_budgets_are_retained() {
    let bytes = proof(controllers(vec![0x81, 0x41, 1]), true, 7, 7);
    let target = Principal::from_slice(&[7]);
    let mut c = client();
    c.config.message_bytes = bytes.len();
    assert!(c.admit_canister(bytes.clone(), target, policy(), 1).is_ok());
    c.config.message_bytes -= 1;
    assert!(
        c.admit_canister(bytes.clone(), target, policy(), 1)
            .is_err()
    );
    let mut c = client();
    c.config.verification.tree_nodes = 0;
    assert!(
        c.admit_canister(bytes.clone(), target, policy(), 1)
            .is_err()
    );
    let c = client();
    for end in 0..bytes.len() {
        assert!(
            c.admit_canister(bytes[..end].to_vec(), target, policy(), 1)
                .is_err()
        );
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(
        client()
            .admit_canister(trailing, target, policy(), 1)
            .is_err()
    );
}
