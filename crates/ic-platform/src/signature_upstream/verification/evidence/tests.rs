use super::*;
use crate::signature_upstream::creation::IC_ROOT_PK_DER_PREFIX;
use ic_certification::{Certificate, Delegation, HashTree, empty, fork, labeled, leaf, pruned};
use ic_principal::Principal;
use ic_verify_bls_signature::PrivateKey;
use serde::Serialize;
use serde_bytes::ByteBuf;

fn key(value: u8) -> PrivateKey {
    let mut bytes = [0; 32];
    bytes[31] = value;
    PrivateKey::deserialize(&bytes).unwrap()
}
fn cbor<T: Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut serializer = serde_cbor::Serializer::new(&mut bytes);
    serializer.self_describe().unwrap();
    value.serialize(&mut serializer).unwrap();
    bytes
}
fn signer() -> Principal {
    Principal::from_slice(&[1; 10])
}
fn certificate(tree: HashTree, key: &PrivateKey, delegation: Option<Delegation>) -> Certificate {
    let mut message = super::super::IC_STATE_ROOT_DOMAIN_SEPARATOR.to_vec();
    message.extend_from_slice(&tree.digest());
    Certificate {
        tree,
        signature: key.sign(&message).serialize().to_vec(),
        delegation,
    }
}
fn envelope(time_tree: HashTree, delegation: Option<Delegation>) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let public_key = CanisterSigPublicKey::new(signer(), b"seed".to_vec());
    let tree = labeled(
        b"sig".to_vec(),
        labeled(
            super::super::hash_sha256(b"seed"),
            labeled(super::super::hash_sha256(b"message"), leaf(Vec::new())),
        ),
    );
    let certified = labeled(
        b"canister".to_vec(),
        labeled(
            signer().as_slice().to_vec(),
            labeled(b"certified_data".to_vec(), leaf(tree.digest())),
        ),
    );
    let signing_key = if delegation.is_some() { key(2) } else { key(1) };
    let cert = certificate(fork(certified, time_tree), &signing_key, delegation);
    (
        cbor(&super::super::CanisterSignature {
            certificate: ByteBuf::from(cbor(&cert)),
            tree,
        }),
        public_key.to_der(),
        key(1).public_key().serialize().to_vec(),
    )
}
fn time(bytes: Vec<u8>) -> HashTree {
    labeled(b"time".to_vec(), leaf(bytes))
}

#[test]
fn authenticated_times_cover_u64_and_existing_nonminimal_encodings() {
    for (bytes, expected) in [
        (vec![0], 0),
        (vec![0x80, 1], 128),
        (vec![0x80, 0], 0),
        (
            {
                let mut b = vec![255; 9];
                b.push(1);
                b
            },
            u64::MAX,
        ),
    ] {
        let (signature, public, root) = envelope(time(bytes), None);
        assert_eq!(
            verify_canister_sig_with_time(b"message", &signature, &public, &root),
            Ok(expected)
        );
        assert_eq!(
            super::super::verify_canister_sig(b"message", &signature, &public, &root),
            Ok(())
        );
        if expected == 128
            && let Some(path) = std::env::var_os("PLATFORM_SIGNATURE_TIME_FIXTURE")
        {
            let text: String = signature.iter().map(|byte| format!("{byte:02x}")).collect();
            std::fs::write(path, text + "\n").unwrap();
        }
    }
}

#[test]
fn malformed_time_is_separate_from_signature_verification() {
    for bytes in [
        vec![],
        vec![0x80],
        vec![255; 10],
        vec![0x80; 11],
        vec![0, 0],
        {
            let mut b = vec![0x80; 9];
            b.push(2);
            b
        },
    ] {
        let (signature, public, root) = envelope(time(bytes), None);
        assert_eq!(
            super::super::verify_canister_sig(b"message", &signature, &public, &root),
            Ok(())
        );
        assert_eq!(
            verify_canister_sig_with_time(b"message", &signature, &public, &root),
            Err(CanisterSigTimeError::InvalidCertificateTime)
        );
        assert!(matches!(
            verify_canister_sig_with_time(b"changed", &signature, &public, &root),
            Err(CanisterSigTimeError::Verification(_))
        ));
    }
    for tree in [
        empty(),
        pruned([0; 32]),
        labeled(b"time".to_vec(), fork(leaf(vec![1]), leaf(vec![2]))),
        fork(time(vec![1]), time(vec![2])),
    ] {
        let (signature, public, root) = envelope(tree, None);
        assert_eq!(
            verify_canister_sig_with_time(b"message", &signature, &public, &root),
            Err(CanisterSigTimeError::InvalidCertificateTime)
        );
    }
}

#[test]
fn original_failures_are_preserved_and_never_return_time() {
    let (signature, public, root) = envelope(time(vec![42]), None);
    let mut bad_key = public.clone();
    bad_key[2] ^= 1;
    let mut bad_signature = signature.clone();
    let last = bad_signature.len() - 1;
    bad_signature[last] ^= 1;
    for (message, signature, public, root) in [
        (
            b"changed".as_slice(),
            signature.as_slice(),
            public.as_slice(),
            root.as_slice(),
        ),
        (b"message", &signature, &bad_key, &root),
        (b"message", &signature, &public, &[0; 96]),
        (b"message", &bad_signature, &public, &root),
        (
            b"message",
            &signature[..signature.len() - 1],
            &public,
            &root,
        ),
    ] {
        let error =
            super::super::verify_canister_sig(message, signature, public, root).unwrap_err();
        assert_eq!(
            verify_canister_sig_with_time(message, signature, public, root),
            Err(CanisterSigTimeError::Verification(error))
        );
    }
}

#[test]
fn delegation_time_is_not_the_signing_time() {
    let subnet = vec![3; 10];
    let mut subnet_key = IC_ROOT_PK_DER_PREFIX.to_vec();
    subnet_key.extend_from_slice(&key(2).public_key().serialize());
    let delegation_tree = fork(
        labeled(
            b"subnet".to_vec(),
            labeled(
                subnet.clone(),
                fork(
                    labeled(
                        b"canister_ranges".to_vec(),
                        leaf(cbor(&vec![(signer(), signer())])),
                    ),
                    labeled(b"public_key".to_vec(), leaf(subnet_key)),
                ),
            ),
        ),
        time(vec![1]),
    );
    let delegation = Delegation {
        subnet_id: subnet,
        certificate: cbor(&certificate(delegation_tree, &key(1), None)),
    };
    let (signature, public, root) = envelope(time(vec![42]), Some(delegation));
    assert_eq!(
        verify_canister_sig_with_time(b"message", &signature, &public, &root),
        Ok(42)
    );
}
