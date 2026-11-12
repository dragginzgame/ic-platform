use super::*;
use crate::certification::{
    cbor::{encode_certificate_untagged, encode_hash_tree},
    empty, labeled, leaf,
};
#[test]
fn witness_matching_requires_the_exact_leaf_and_canister() {
    let canister = Principal::from_slice(&[1, 2, 3]);
    let witness = leaf(vec![9]);
    let certificate = Certificate {
        tree: labeled(
            b"canister".to_vec(),
            labeled(
                canister.as_slice().to_vec(),
                labeled(b"certified_data".to_vec(), leaf(witness.digest().to_vec())),
            ),
        ),
        signature: vec![],
        delegation: None,
    };
    verify_witness(&certificate, canister, &witness).unwrap();
    assert!(matches!(
        verify_witness(&certificate, Principal::anonymous(), &witness),
        Err(EvidenceError::WitnessMismatch)
    ));
    assert!(matches!(
        verify_witness(&certificate, canister, &leaf(vec![8])),
        Err(EvidenceError::WitnessMismatch)
    ));
    let missing = Certificate {
        tree: empty(),
        ..certificate
    };
    assert!(matches!(
        verify_witness(&missing, canister, &witness),
        Err(EvidenceError::WitnessMismatch)
    ));
}
#[test]
fn admission_precedes_authentication_and_unsigned_pairs_never_authenticate() {
    let agent = Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    let canister = Principal::anonymous();
    assert!(matches!(
        authenticate_hash_tree(&agent, canister, &[0], &[0], 1),
        Err(EvidenceError::Budget)
    ));
    assert!(matches!(
        authenticate_hash_tree(&agent, canister, &[0], &[0], 2),
        Err(EvidenceError::Decode(_))
    ));
    let certificate = Certificate {
        tree: empty(),
        signature: vec![],
        delegation: None,
    };
    let bytes = encode_certificate_untagged(&certificate, 1024).unwrap();
    let witness = encode_hash_tree(&empty(), 1024).unwrap();
    assert!(matches!(
        authenticate_hash_tree(&agent, canister, &bytes, &witness, 2048),
        Err(EvidenceError::Authentication(_))
    ));
    let mut trailing = bytes;
    trailing.push(0);
    assert!(matches!(
        authenticate_hash_tree(&agent, canister, &trailing, &witness, 2048),
        Err(EvidenceError::Decode(DecodeError::TrailingData))
    ));
}

#[cfg(feature = "bls")]
#[test]
fn signed_evidence_uses_the_selected_root_and_preserves_witness_admission() {
    let key = crate::bls::PrivateKey::deserialize(&[1; 32]).unwrap();
    let mut der = vec![
        48, 129, 130, 48, 29, 6, 13, 43, 6, 1, 4, 1, 130, 220, 124, 5, 3, 1, 2, 1, 6, 12, 43, 6, 1,
        4, 1, 130, 220, 124, 5, 3, 2, 1, 3, 97, 0,
    ];
    der.extend_from_slice(&key.public_key().serialize());
    let agent = Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    agent.set_root_key(der);
    let canister = Principal::from_slice(&[1, 2, 3]);
    let witness = leaf(vec![7]);
    let mut now = u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    )
    .unwrap();
    let mut time = Vec::new();
    loop {
        let byte = (now & 127) as u8;
        now >>= 7;
        time.push(if now == 0 { byte } else { byte | 128 });
        if now == 0 {
            break;
        }
    }
    let tree = crate::certification::fork(
        labeled(
            b"canister".to_vec(),
            labeled(
                canister.as_slice().to_vec(),
                labeled(b"certified_data".to_vec(), leaf(witness.digest().to_vec())),
            ),
        ),
        labeled(b"time".to_vec(), leaf(time)),
    );
    let mut message = vec![13];
    message.extend_from_slice(b"ic-state-root");
    message.extend_from_slice(&tree.digest());
    let certificate = Certificate {
        tree,
        signature: key.sign(&message).serialize().to_vec(),
        delegation: None,
    };
    let bytes = encode_certificate_untagged(&certificate, 4096).unwrap();
    assert_eq!(
        authenticate_certificate(&agent, canister, &bytes, bytes.len()).unwrap(),
        certificate
    );
    let encoded = encode_hash_tree(&witness, 4096).unwrap();
    let exact = bytes.len() + encoded.len();
    assert_eq!(
        authenticate_hash_tree(&agent, canister, &bytes, &encoded, exact).unwrap(),
        witness
    );
    assert!(matches!(
        authenticate_hash_tree(&agent, canister, &bytes, &encoded, exact - 1),
        Err(EvidenceError::Budget)
    ));
    assert!(matches!(
        authenticate_hash_tree(&agent, Principal::anonymous(), &bytes, &encoded, exact),
        Err(EvidenceError::WitnessMismatch)
    ));
    let other = Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    assert!(matches!(
        authenticate_hash_tree(&other, canister, &bytes, &encoded, exact),
        Err(EvidenceError::Authentication(_))
    ));
    let mut trailing = encoded;
    trailing.push(0);
    assert!(matches!(
        authenticate_hash_tree(&agent, canister, &bytes, &trailing, exact + 1),
        Err(EvidenceError::Decode(DecodeError::TrailingData))
    ));

    // The same authenticated-byte boundary must retain Agent's signature and
    // timestamp checks, including when no witness is requested.
    let mut stale = certificate;
    stale.tree = labeled(b"time".to_vec(), leaf(vec![0]));
    let tampered = encode_certificate_untagged(&stale, 4096).unwrap();
    assert!(matches!(
        authenticate_certificate(&agent, canister, &tampered, tampered.len()),
        Err(EvidenceError::Authentication(_))
    ));
    let mut message = vec![13];
    message.extend_from_slice(b"ic-state-root");
    message.extend_from_slice(&stale.tree.digest());
    stale.signature = key.sign(&message).serialize().to_vec();
    let bytes = encode_certificate_untagged(&stale, 4096).unwrap();
    assert!(matches!(
        authenticate_certificate(&agent, canister, &bytes, bytes.len()),
        Err(EvidenceError::Authentication(source))
            if matches!(*source, AgentError::CertificateOutdated(_))
    ));
}

#[test]
fn witness_rejects_unknown_and_nonleaf_commitments() {
    let canister = Principal::anonymous();
    let witness = empty();
    for tree in [
        crate::certification::pruned([1; 32]),
        labeled(
            b"canister".to_vec(),
            labeled(
                canister.as_slice().to_vec(),
                labeled(
                    b"certified_data".to_vec(),
                    labeled(b"extra".to_vec(), leaf(witness.digest().to_vec())),
                ),
            ),
        ),
    ] {
        let certificate = Certificate {
            tree,
            signature: vec![],
            delegation: None,
        };
        assert!(matches!(
            verify_witness(&certificate, canister, &witness),
            Err(EvidenceError::WitnessMismatch)
        ));
    }
}
