use super::*;
use crate::certificate::decode_canister_signature;
use ic_certification::{HashTree, HashTreeNode, empty, fork, labeled, leaf, pruned};
use ic_verify_bls_signature::PrivateKey;
use serde_cbor::Value;

const KEY: &[u8] =
    b"\x30\x14\x30\x0c\x06\x0a\x2b\x06\x01\x04\x01\x83\xb8\x43\x01\x02\x03\x04\x00\x01\x01\x73";
fn signing_key(n: u8) -> PrivateKey {
    let mut bytes = [0; 32];
    bytes[31] = n;
    PrivateKey::deserialize(&bytes).unwrap()
}
fn root(n: u8) -> PreparedRootKey {
    let mut der = super::super::DER_PREFIX.to_vec();
    der.extend_from_slice(&signing_key(n).public_key().serialize());
    PreparedRootKey::from_der(&der).unwrap()
}
fn wire(tree: &HashTreeNode) -> Value {
    let values = match tree {
        HashTreeNode::Empty() => vec![Value::Integer(0)],
        HashTreeNode::Fork(pair) => vec![Value::Integer(1), wire(&pair.0), wire(&pair.1)],
        HashTreeNode::Labeled(label, child) => vec![
            Value::Integer(2),
            Value::Bytes(label.as_bytes().to_vec()),
            wire(child),
        ],
        HashTreeNode::Leaf(bytes) => vec![Value::Integer(3), Value::Bytes(bytes.clone())],
        HashTreeNode::Pruned(hash) => vec![Value::Integer(4), Value::Bytes(hash.to_vec())],
    };
    Value::Array(values)
}
fn tagged(value: &Value) -> Vec<u8> {
    let mut bytes = vec![0xd9, 0xd9, 0xf7];
    bytes.extend_from_slice(&serde_cbor::to_vec(value).unwrap());
    bytes
}
fn cert(tree: &HashTree, signer: u8, authority: Option<Vec<u8>>) -> Vec<u8> {
    let mut input = b"\x0dic-state-root".to_vec();
    input.extend_from_slice(&tree.digest());
    let mut fields = std::collections::BTreeMap::from([
        (Value::Text("tree".into()), wire(tree.as_ref())),
        (
            Value::Text("signature".into()),
            Value::Bytes(signing_key(signer).sign(&input).serialize().to_vec()),
        ),
    ]);
    if let Some(authority) = authority {
        fields.insert(
            Value::Text("delegation".into()),
            Value::Map(std::collections::BTreeMap::from([
                (Value::Text("subnet_id".into()), Value::Bytes(vec![9])),
                (Value::Text("certificate".into()), Value::Bytes(authority)),
            ])),
        );
    }
    tagged(&Value::Map(fields))
}
fn witness(value: Vec<u8>) -> HashTree {
    labeled(
        b"sig".to_vec(),
        labeled(
            Sha256::digest(b"s").to_vec(),
            labeled(Sha256::digest(b"message").to_vec(), leaf(value)),
        ),
    )
}
fn envelope(witness: &HashTree, digest: [u8; 32], kind: Option<Option<&[u8]>>) -> Vec<u8> {
    let authority = kind.map(|kind| {
        let mut fields = fork(
            labeled(
                b"canister_ranges".to_vec(),
                leaf(vec![0x81, 0x82, 0x41, 1, 0x41, 1]),
            ),
            labeled(b"public_key".to_vec(), leaf(root(2).as_der().to_vec())),
        );
        if let Some(kind) = kind {
            fields = fork(fields, labeled(b"type".to_vec(), leaf(kind.to_vec())));
        }
        cert(
            &fork(
                labeled(b"subnet".to_vec(), labeled(vec![9], fields)),
                labeled(b"time".to_vec(), leaf(vec![5])),
            ),
            1,
            None,
        )
    });
    let tree = fork(
        labeled(
            b"canister".to_vec(),
            labeled(
                vec![1],
                labeled(b"certified_data".to_vec(), leaf(digest.to_vec())),
            ),
        ),
        labeled(b"time".to_vec(), leaf(vec![10])),
    );
    let signer = if authority.is_some() { 2 } else { 1 };
    tagged(&Value::Map(std::collections::BTreeMap::from([
        (
            Value::Text("certificate".into()),
            Value::Bytes(cert(&tree, signer, authority)),
        ),
        (Value::Text("tree".into()), wire(witness.as_ref())),
    ])))
}
fn policy() -> TimePolicy {
    let exact = |reference_ns| TimeWindow {
        reference_ns,
        max_age_ns: 0,
        max_future_skew_ns: 0,
    };
    TimePolicy {
        certificate: exact(10),
        delegation: exact(5),
    }
}
fn limits() -> CanisterSignatureLimits {
    CanisterSignatureLimits {
        message_bytes: 7,
        certificate_bytes: 4096,
        verification: VerificationLimits {
            tree_nodes: 100,
            tree_depth: 20,
            delegation_bytes: 4096,
            canister_ranges: 1,
        },
    }
}
fn verify<'a>(
    bytes: &'a [u8],
    message: &'a [u8],
    key: &'a [u8],
    time: TimePolicy,
    limits: CanisterSignatureLimits,
) -> Result<AuthenticatedCanisterSignature<'a>, CanisterSignatureVerifyError> {
    authenticate_canister_signature(
        decode_canister_signature(bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap(),
        message,
        CanisterSignatureKey::from_der(key)?,
        &root(1),
        time,
        limits,
    )
}

#[test]
fn key_admission_is_bounded_canonical_and_preserves_identity() {
    let key = CanisterSignatureKey::from_der(KEY).unwrap();
    assert_eq!(key.issuer().as_slice(), [1]);
    assert_eq!(key.seed(), b"s");
    assert!(std::ptr::eq(key.as_der(), KEY));
    for length in 0..KEY.len() {
        assert!(CanisterSignatureKey::from_der(&KEY[..length]).is_err());
    }
    for (offset, value) in [(0, 0x31), (1, 0x15), (15, 3), (16, 4), (18, 1), (19, 30)] {
        let mut changed = KEY.to_vec();
        changed[offset] = value;
        assert!(CanisterSignatureKey::from_der(&changed).is_err());
    }
    let mut trailing = KEY.to_vec();
    trailing.push(0);
    assert!(CanisterSignatureKey::from_der(&trailing).is_err());
    let mut maximum = KEY.to_vec();
    maximum.resize(129, 0);
    maximum[1] = 127;
    maximum[17] = 111;
    assert!(CanisterSignatureKey::from_der(&maximum).is_ok());
    maximum.push(0);
    assert!(CanisterSignatureKey::from_der(&maximum).is_err());
}
#[test]
fn direct_and_delegated_evidence_bind_exact_inputs_and_distinct_times() {
    for kind in [
        None,
        Some(Some(b"application".as_slice())),
        Some(Some(b"system".as_slice())),
        Some(Some(b"verified_application".as_slice())),
    ] {
        let tree = witness(vec![]);
        let bytes = envelope(&tree, tree.digest(), kind);
        let evidence = verify(&bytes, b"message", KEY, policy(), limits()).unwrap();
        assert!(std::ptr::eq(evidence.original_bytes(), bytes.as_slice()));
        assert_eq!(evidence.message(), b"message");
        assert_eq!(evidence.key().as_der(), KEY);
        assert_eq!(evidence.root_key_der(), root(1).as_der());
        assert_eq!(evidence.time_ns(), 10);
        assert_eq!(evidence.time_window(), policy().certificate);
        assert_eq!(
            evidence.delegation(),
            kind.map(|_| (Principal::from_slice(&[9]), policy().delegation, 5))
        );
    }
}
#[test]
fn wrong_message_seed_and_nonempty_or_unknown_witness_refuse() {
    let tree = witness(vec![]);
    let bytes = envelope(&tree, tree.digest(), None);
    assert!(matches!(
        verify(&bytes, b"changed", KEY, policy(), limits()),
        Err(CanisterSignatureVerifyError::Witness)
    ));
    let mut key = KEY.to_vec();
    *key.last_mut().unwrap() = b't';
    assert!(matches!(
        verify(&bytes, b"message", &key, policy(), limits()),
        Err(CanisterSignatureVerifyError::Witness)
    ));
    for tree in [witness(vec![0]), pruned([0; 32]), empty()] {
        let bytes = envelope(&tree, tree.digest(), None);
        assert!(matches!(
            verify(&bytes, b"message", KEY, policy(), limits()),
            Err(CanisterSignatureVerifyError::Witness)
        ));
    }
}
#[test]
fn witness_digest_and_issuer_must_match_certified_data() {
    let tree = witness(vec![]);
    let bytes = envelope(&tree, [0; 32], None);
    assert!(matches!(
        verify(&bytes, b"message", KEY, policy(), limits()),
        Err(CanisterSignatureVerifyError::CertifiedData)
    ));
    let bytes = envelope(&tree, tree.digest(), None);
    let mut key = KEY.to_vec();
    key[20] = 2;
    assert!(matches!(
        verify(&bytes, b"message", &key, policy(), limits()),
        Err(CanisterSignatureVerifyError::CertifiedData)
    ));
    let bytes = envelope(&tree, tree.digest(), Some(Some(b"system")));
    assert!(matches!(
        verify(&bytes, b"message", &key, policy(), limits()),
        Err(CanisterSignatureVerifyError::Verification(
            VerifyError::NotAuthorized
        ))
    ));
}
#[test]
fn delegation_requires_certified_non_cloud_subnet_type() {
    for kind in [
        None,
        Some(b"cloud_engine".as_slice()),
        Some(b"future_type".as_slice()),
        Some(b"".as_slice()),
    ] {
        let tree = witness(vec![]);
        let bytes = envelope(&tree, tree.digest(), Some(kind));
        assert!(matches!(
            verify(&bytes, b"message", KEY, policy(), limits()),
            Err(CanisterSignatureVerifyError::Verification(
                VerifyError::MissingDelegationPath | VerifyError::NotAuthorized
            ))
        ));
    }
}
#[test]
fn signing_and_delegation_windows_remain_independent() {
    let tree = witness(vec![]);
    let bytes = envelope(&tree, tree.digest(), Some(Some(b"system")));
    for is_delegation in [false, true] {
        let mut time = policy();
        if is_delegation {
            time.delegation.reference_ns += 1;
        } else {
            time.certificate.reference_ns += 1;
        }
        assert!(matches!(
            verify(&bytes, b"message", KEY, time, limits()),
            Err(CanisterSignatureVerifyError::Verification(
                VerifyError::TimeOutsideWindow
            ))
        ));
    }
}
#[test]
fn resource_bounds_and_ambiguous_witnesses_refuse() {
    let tree = witness(vec![]);
    let bytes = envelope(&tree, tree.digest(), None);
    let mut cap = limits();
    cap.message_bytes = 6;
    assert!(matches!(
        verify(&bytes, b"message", KEY, policy(), cap),
        Err(CanisterSignatureVerifyError::MessageLimit)
    ));
    cap = limits();
    cap.certificate_bytes = 0;
    assert!(matches!(
        verify(&bytes, b"message", KEY, policy(), cap),
        Err(CanisterSignatureVerifyError::Decode(
            DecodeError::InputLimit { .. }
        ))
    ));
    cap = limits();
    cap.verification.tree_depth = 3;
    assert!(matches!(
        verify(&bytes, b"message", KEY, policy(), cap),
        Err(CanisterSignatureVerifyError::Verification(
            VerifyError::TreeLimit
        ))
    ));
    // Witness has four nodes, signing tree seven: one allowance covers both.
    cap = limits();
    cap.verification.tree_nodes = 11;
    assert!(verify(&bytes, b"message", KEY, policy(), cap).is_ok());
    cap.verification.tree_nodes = 10;
    assert!(matches!(
        verify(&bytes, b"message", KEY, policy(), cap),
        Err(CanisterSignatureVerifyError::Verification(
            VerifyError::TreeLimit
        ))
    ));
    let ambiguous = fork(tree.clone(), tree);
    let bytes = envelope(&ambiguous, ambiguous.digest(), None);
    assert!(matches!(
        verify(&bytes, b"message", KEY, policy(), limits()),
        Err(CanisterSignatureVerifyError::Verification(
            VerifyError::InvalidTree
        ))
    ));
}

// Change only an embedded wire field; signatures are never recomputed here.
fn change_certificate(
    bytes: &[u8],
    mutate: impl FnOnce(&mut std::collections::BTreeMap<Value, Value>),
) -> Vec<u8> {
    let Value::Map(mut outer) = serde_cbor::from_slice(bytes).unwrap() else {
        panic!("map")
    };
    let Value::Bytes(encoded) = outer.get_mut(&Value::Text("certificate".into())).unwrap() else {
        panic!("bytes")
    };
    let Value::Map(mut certificate) = serde_cbor::from_slice(encoded).unwrap() else {
        panic!("map")
    };
    mutate(&mut certificate);
    *encoded = tagged(&Value::Map(certificate));
    tagged(&Value::Map(outer))
}

#[test]
fn wrong_root_and_both_certificate_signatures_refuse() {
    for kind in [None, Some(Some(b"system".as_slice()))] {
        let tree = witness(vec![]);
        let bytes = envelope(&tree, tree.digest(), kind);
        let decoded =
            decode_canister_signature(&bytes, CertificateDecodeLimits { bytes: bytes.len() })
                .unwrap();
        assert!(matches!(
            authenticate_canister_signature(
                decoded,
                b"message",
                CanisterSignatureKey::from_der(KEY).unwrap(),
                &root(3),
                policy(),
                limits()
            ),
            Err(CanisterSignatureVerifyError::Verification(
                VerifyError::Signature
            ))
        ));
        let altered = change_certificate(&bytes, |fields| {
            let Value::Bytes(signature) = fields.get_mut(&Value::Text("signature".into())).unwrap()
            else {
                panic!("signature")
            };
            signature[0] ^= 1;
        });
        assert!(matches!(
            verify(&altered, b"message", KEY, policy(), limits()),
            Err(CanisterSignatureVerifyError::Verification(
                VerifyError::Signature
            ))
        ));
    }
    let tree = witness(vec![]);
    let bytes = envelope(&tree, tree.digest(), Some(Some(b"system")));
    let altered = change_certificate(&bytes, |fields| {
        let Value::Map(delegation) = fields.get_mut(&Value::Text("delegation".into())).unwrap()
        else {
            panic!("delegation")
        };
        let Value::Bytes(authority) = delegation
            .get_mut(&Value::Text("certificate".into()))
            .unwrap()
        else {
            panic!("bytes")
        };
        let Value::Map(mut authority_fields) = serde_cbor::from_slice(authority).unwrap() else {
            panic!("map")
        };
        let Value::Bytes(signature) = authority_fields
            .get_mut(&Value::Text("signature".into()))
            .unwrap()
        else {
            panic!("signature")
        };
        signature[0] ^= 1;
        *authority = tagged(&Value::Map(authority_fields));
    });
    assert!(matches!(
        verify(&altered, b"message", KEY, policy(), limits()),
        Err(CanisterSignatureVerifyError::Verification(
            VerifyError::Signature
        ))
    ));
    let mut cap = limits();
    cap.verification.tree_nodes = 24;
    assert!(verify(&bytes, b"message", KEY, policy(), cap).is_ok());
    cap.verification.tree_nodes -= 1;
    assert!(matches!(
        verify(&bytes, b"message", KEY, policy(), cap),
        Err(CanisterSignatureVerifyError::Verification(
            VerifyError::TreeLimit
        ))
    ));
}

#[test]
fn certificate_tags_and_key_parameters_are_not_optional() {
    let mut key = KEY.to_vec();
    key.splice(16..16, [5, 0]);
    key[1] += 2;
    key[3] += 2;
    assert!(CanisterSignatureKey::from_der(&key).is_err());
    let tree = witness(vec![]);
    let bytes = envelope(&tree, tree.digest(), Some(Some(b"system")));
    let altered = change_certificate(&bytes, |fields| {
        let Value::Map(delegation) = fields.get_mut(&Value::Text("delegation".into())).unwrap()
        else {
            panic!("delegation")
        };
        let Value::Bytes(authority) = delegation
            .get_mut(&Value::Text("certificate".into()))
            .unwrap()
        else {
            panic!("bytes")
        };
        authority.drain(..3);
    });
    assert!(matches!(
        verify(&altered, b"message", KEY, policy(), limits()),
        Err(CanisterSignatureVerifyError::Verification(
            VerifyError::InvalidDelegationEncoding
        ))
    ));
    let Value::Map(mut outer) = serde_cbor::from_slice(&bytes).unwrap() else {
        panic!("map")
    };
    let Value::Bytes(certificate) = outer.get_mut(&Value::Text("certificate".into())).unwrap()
    else {
        panic!("bytes")
    };
    certificate.drain(..3);
    let altered = tagged(&Value::Map(outer));
    assert!(matches!(
        verify(&altered, b"message", KEY, policy(), limits()),
        Err(CanisterSignatureVerifyError::MissingCertificateTag)
    ));
}
