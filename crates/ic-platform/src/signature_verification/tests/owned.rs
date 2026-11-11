use super::super::*;
use super::{verify_canister_sig, verify_canister_sig_with_time};
use crate::canister_sig_creation::IC_ROOT_PK_DER_PREFIX;
use ic_certification::{empty, fork, labeled, leaf, pruned};
use ic_verify_bls_signature::PrivateKey;
use serde::Serialize;
use serde_cbor::Value;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct EncodedSignature<'a> {
    #[serde(with = "serde_bytes")]
    certificate: &'a [u8],
    tree: &'a HashTree,
}

fn tagged(value: &impl Serialize) -> Vec<u8> {
    let mut serializer = serde_cbor::Serializer::new(Vec::new());
    serializer.self_describe().unwrap();
    value.serialize(&mut serializer).unwrap();
    serializer.into_inner()
}

fn private_key(byte: u8) -> PrivateKey {
    PrivateKey::deserialize(&[byte; 32]).unwrap()
}

fn certificate(tree: HashTree, key: &PrivateKey, delegation: Option<Delegation>) -> Certificate {
    let mut message = b"\x0dic-state-root".to_vec();
    message.extend_from_slice(&tree.digest());
    Certificate {
        tree,
        signature: key.sign(&message).serialize().to_vec(),
        delegation,
    }
}

fn witness(key: &CanisterSigPublicKey, message: &[u8]) -> HashTree {
    labeled(
        b"sig".to_vec(),
        labeled(
            hash_bytes(&key.seed).to_vec(),
            labeled(hash_bytes(message).to_vec(), leaf(Vec::new())),
        ),
    )
}

fn signing_tree(issuer: &Principal, witness: &HashTree, time: HashTree) -> HashTree {
    fork(
        labeled(
            b"canister".to_vec(),
            labeled(
                issuer.as_slice().to_vec(),
                labeled(b"certified_data".to_vec(), leaf(witness.digest().to_vec())),
            ),
        ),
        time,
    )
}

fn envelope(certificate: &Certificate, witness: &HashTree) -> Vec<u8> {
    tagged(&EncodedSignature {
        certificate: &tagged(certificate),
        tree: witness,
    })
}

fn issuer_key() -> CanisterSigPublicKey {
    CanisterSigPublicKey::new(Principal::from_slice(&[7, 1]), vec![8, 9])
}

fn range_leaf(start: Principal, end: Principal) -> HashTree {
    leaf(serde_cbor::to_vec(&vec![(start, end)]).unwrap())
}

#[test]
fn direct_authentication_binds_message_key_witness_and_time() {
    let issuer = issuer_key();
    let root = private_key(1);
    let root_bytes = root.public_key().serialize();
    let witness = witness(&issuer, b"message");
    let cert = certificate(
        signing_tree(
            &issuer.canister_id,
            &witness,
            labeled(b"time".to_vec(), leaf(vec![0xc1, 2])),
        ),
        &root,
        None,
    );
    let signature = envelope(&cert, &witness);
    assert_eq!(
        verify_canister_sig(b"message", &signature, &issuer.to_der(), &root_bytes),
        Ok(())
    );
    assert_eq!(
        verify_canister_sig_with_time(b"message", &signature, &issuer.to_der(), &root_bytes),
        Ok(321)
    );
    assert!(verify_canister_sig(b"different", &signature, &issuer.to_der(), &root_bytes).is_err());
    let mut changed_issuer = issuer.clone();
    changed_issuer.canister_id = Principal::from_slice(&[7, 2]);
    assert!(
        verify_canister_sig(
            b"message",
            &signature,
            &changed_issuer.to_der(),
            &root_bytes
        )
        .is_err()
    );
    changed_issuer = issuer.clone();
    changed_issuer.seed.push(0);
    assert!(
        verify_canister_sig(
            b"message",
            &signature,
            &changed_issuer.to_der(),
            &root_bytes
        )
        .is_err()
    );
    assert!(matches!(
        verify_canister_sig_with_time(
            b"message",
            &signature,
            &issuer.to_der(),
            &private_key(2).public_key().serialize()
        ),
        Err(CanisterSigTimeError::Verification(_))
    ));
}

fn delegated_signature(
    root: &PrivateKey,
    subnet_key: &PrivateKey,
    nested: bool,
) -> (Vec<u8>, Vec<u8>) {
    let issuer = issuer_key();
    let witness = witness(&issuer, b"delegated");
    let subnet = vec![42];
    let mut subnet_der = IC_ROOT_PK_DER_PREFIX.to_vec();
    subnet_der.extend_from_slice(&subnet_key.public_key().serialize());
    let authority_tree = fork(
        labeled(
            b"subnet".to_vec(),
            labeled(
                subnet.clone(),
                fork(
                    labeled(
                        b"canister_ranges".to_vec(),
                        range_leaf(issuer.canister_id, issuer.canister_id),
                    ),
                    labeled(b"public_key".to_vec(), leaf(subnet_der)),
                ),
            ),
        ),
        labeled(b"time".to_vec(), leaf(vec![99])),
    );
    let authority = certificate(
        authority_tree,
        root,
        nested.then(|| Delegation {
            subnet_id: vec![1],
            certificate: Vec::new(),
        }),
    );
    let signing = certificate(
        signing_tree(
            &issuer.canister_id,
            &witness,
            labeled(b"time".to_vec(), leaf(vec![7])),
        ),
        subnet_key,
        Some(Delegation {
            subnet_id: subnet,
            certificate: tagged(&authority),
        }),
    );
    (envelope(&signing, &witness), issuer.to_der())
}

#[test]
fn delegation_authenticates_issuer_and_uses_only_signing_time() {
    let root = private_key(1);
    let subnet = private_key(2);
    let (signature, der) = delegated_signature(&root, &subnet, false);
    assert_eq!(
        verify_canister_sig_with_time(
            b"delegated",
            &signature,
            &der,
            &root.public_key().serialize()
        ),
        Ok(7)
    );
    let (nested, der) = delegated_signature(&root, &subnet, true);
    assert!(
        verify_canister_sig(b"delegated", &nested, &der, &root.public_key().serialize()).is_err()
    );
}

#[test]
fn identity_keys_are_refused_at_both_trust_levels() {
    for (root, subnet) in [
        (private_key(0), private_key(2)),
        (private_key(1), private_key(0)),
    ] {
        let (signature, der) = delegated_signature(&root, &subnet, false);
        assert!(
            verify_canister_sig(
                b"delegated",
                &signature,
                &der,
                &root.public_key().serialize()
            )
            .is_err()
        );
    }
}

#[test]
fn time_requires_one_visible_top_level_leaf() {
    assert_eq!(
        signing_time(&labeled(b"time".to_vec(), leaf(vec![0x80, 0]))),
        Some(0)
    );
    assert_eq!(
        signing_time(&fork(
            pruned([4; 32]),
            labeled(b"time".to_vec(), leaf(vec![1]))
        )),
        Some(1)
    );
    for tree in [
        empty(),
        pruned([0; 32]),
        labeled(b"time".to_vec(), empty()),
        labeled(b"time".to_vec(), pruned([0; 32])),
        labeled(b"nested".to_vec(), labeled(b"time".to_vec(), leaf(vec![1]))),
        fork(
            labeled(b"time".to_vec(), leaf(vec![1])),
            labeled(b"time".to_vec(), leaf(vec![2])),
        ),
    ] {
        assert_eq!(signing_time(&tree), None);
    }
}

#[test]
fn time_admits_nonminimal_u64_but_rejects_overflow_and_extra_bytes() {
    assert_eq!(decode_time(&[0]), Some(0));
    assert_eq!(decode_time(&[0x80, 1]), Some(128));
    assert_eq!(
        decode_time(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 1]),
        Some(u64::MAX)
    );
    assert_eq!(
        decode_time(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0]),
        Some(0)
    );
    for bytes in [
        vec![],
        vec![0x80],
        vec![0, 0],
        vec![0x80; 10],
        vec![0xff; 10],
        vec![0; 11],
    ] {
        assert_eq!(decode_time(&bytes), None);
    }
}

#[test]
fn missing_time_is_reported_only_after_authentication() {
    let issuer = issuer_key();
    let root = private_key(1);
    let witness = witness(&issuer, b"no time");
    let cert = certificate(
        signing_tree(&issuer.canister_id, &witness, empty()),
        &root,
        None,
    );
    let signature = envelope(&cert, &witness);
    assert_eq!(
        verify_canister_sig_with_time(
            b"no time",
            &signature,
            &issuer.to_der(),
            &root.public_key().serialize()
        ),
        Err(CanisterSigTimeError::InvalidCertificateTime)
    );
    assert!(matches!(
        verify_canister_sig_with_time(
            b"wrong",
            &signature,
            &issuer.to_der(),
            &root.public_key().serialize()
        ),
        Err(CanisterSigTimeError::Verification(_))
    ));
}

#[test]
fn sharded_routing_is_lexicographic_and_never_falls_back_from_visible_shards() {
    let issuer = Principal::from_slice(&[7, 1]);
    let subnet = [42];
    let legacy = labeled(
        b"subnet".to_vec(),
        labeled(
            subnet.to_vec(),
            labeled(b"canister_ranges".to_vec(), range_leaf(issuer, issuer)),
        ),
    );
    assert!(authorize_issuer(&legacy, &subnet, &issuer).is_ok());
    for selected in [
        empty(),
        leaf(vec![0]),
        range_leaf(Principal::anonymous(), Principal::anonymous()),
        pruned([0; 32]),
    ] {
        let shards = labeled(
            b"canister_ranges".to_vec(),
            labeled(
                subnet.to_vec(),
                fork(
                    labeled(vec![7], selected),
                    labeled(vec![8], range_leaf(issuer, issuer)),
                ),
            ),
        );
        assert!(authorize_issuer(&fork(shards, legacy.clone()), &subnet, &issuer).is_err());
    }
    let shards = labeled(
        b"canister_ranges".to_vec(),
        labeled(
            subnet.to_vec(),
            fork(
                labeled(vec![6, 255, 255], leaf(vec![0])),
                labeled(vec![7], range_leaf(issuer, issuer)),
            ),
        ),
    );
    assert!(authorize_issuer(&fork(shards, legacy), &subnet, &issuer).is_ok());
}

#[test]
fn range_membership_keeps_canonical_length_first_order() {
    let ranges = serde_cbor::to_vec(&vec![(
        Principal::from_slice(&[255]),
        Principal::from_slice(&[1, 0]),
    )])
    .unwrap();
    assert!(authorize_ranges(&ranges, &Principal::from_slice(&[0, 1])).is_ok());
    assert!(authorize_ranges(&ranges, &Principal::from_slice(&[254])).is_err());
    let overlong = serde_cbor::to_vec(&vec![(
        serde_bytes::ByteBuf::from(vec![0; 30]),
        serde_bytes::ByteBuf::from(vec![0; 30]),
    )])
    .unwrap();
    assert!(authorize_ranges(&overlong, &Principal::anonymous()).is_err());
}

#[test]
fn signature_and_certificate_decoding_preserve_serde_forms_and_reject_trailing_data() {
    let empty_tree = serde_cbor::value::to_value(empty()).unwrap();
    let cert_bytes = tagged(&certificate(empty(), &private_key(1), None));
    for value in [
        Value::Array(vec![Value::Bytes(cert_bytes.clone()), empty_tree.clone()]),
        Value::Map(BTreeMap::from([
            (Value::Integer(0), Value::Bytes(cert_bytes.clone())),
            (Value::Integer(1), empty_tree),
        ])),
    ] {
        let bytes = tagged(&value);
        assert!(decode_tagged::<SignatureEnvelope>(&bytes, "signature").is_ok());
        assert!(decode_tagged::<SignatureEnvelope>(&bytes[3..], "signature").is_err());
        let mut trailing = bytes;
        trailing.push(0);
        assert!(decode_tagged::<SignatureEnvelope>(&trailing, "signature").is_err());
    }
    let cert = certificate(empty(), &private_key(1), None);
    let tree = serde_cbor::value::to_value(&cert.tree).unwrap();
    for value in [
        Value::Array(vec![
            tree.clone(),
            Value::Bytes(cert.signature.clone()),
            Value::Null,
        ]),
        Value::Map(BTreeMap::from([
            (Value::Integer(0), tree),
            (Value::Integer(1), Value::Bytes(cert.signature)),
        ])),
    ] {
        assert!(decode_certificate(&tagged(&value)).is_ok());
    }
    assert!(decode_certificate(&cert_bytes[3..]).is_err());
}

#[test]
fn duplicate_required_cbor_fields_are_rejected() {
    let mut bytes = SELF_DESCRIBED_CBOR.to_vec();
    bytes.push(0xa3);
    for (name, value) in [
        ("certificate", Value::Bytes(Vec::new())),
        ("certificate", Value::Bytes(Vec::new())),
        ("tree", serde_cbor::value::to_value(empty()).unwrap()),
    ] {
        bytes.extend_from_slice(&serde_cbor::to_vec(&name).unwrap());
        bytes.extend_from_slice(&serde_cbor::to_vec(&value).unwrap());
    }
    assert!(decode_tagged::<SignatureEnvelope>(&bytes, "signature").is_err());
    let mut bytes = SELF_DESCRIBED_CBOR.to_vec();
    bytes.push(0xa3);
    for (name, value) in [
        ("tree", serde_cbor::value::to_value(empty()).unwrap()),
        ("tree", serde_cbor::value::to_value(empty()).unwrap()),
        ("signature", Value::Bytes(Vec::new())),
    ] {
        bytes.extend_from_slice(&serde_cbor::to_vec(&name).unwrap());
        bytes.extend_from_slice(&serde_cbor::to_vec(&value).unwrap());
    }
    assert!(decode_certificate(&bytes).is_err());
}
