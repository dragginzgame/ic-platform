mod owned;

use super::*;
use crate::bls::PrivateKey;
use crate::canister_sig_creation::CanisterSigPublicKey;
use crate::canister_sig_creation::IC_ROOT_PK_DER_PREFIX;
use crate::certification::{Certificate, Delegation, HashTree, empty, fork, labeled, leaf, pruned};
use crate::principal::Principal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CanisterSignature {
    certificate: ByteBuf,
    tree: HashTree,
}
use serde_bytes::ByteBuf;

// Exercise both public root forms against the same acceptance, rejection and
// authenticated-time corpus, including identity keys and delegated signatures.
fn verify_canister_sig(
    message: &[u8],
    signature: &[u8],
    public: &[u8],
    root: &[u8],
) -> Result<(), String> {
    let raw = super::verify_canister_sig(message, signature, public, root);
    if let Ok(key) = crate::bls::PublicKey::deserialize(root) {
        assert_eq!(
            raw,
            verify_canister_sig_with_prepared_root(message, signature, public, &key.prepare())
        );
    }
    raw
}
fn verify_canister_sig_with_time(
    message: &[u8],
    signature: &[u8],
    public: &[u8],
    root: &[u8],
) -> Result<u64, CanisterSigTimeError> {
    let raw = super::verify_canister_sig_with_time(message, signature, public, root);
    if let Ok(key) = crate::bls::PublicKey::deserialize(root) {
        assert_eq!(
            raw,
            verify_canister_sig_with_time_and_prepared_root(
                message,
                signature,
                public,
                &key.prepare()
            )
        );
    }
    raw
}

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
    let mut message = b"\x0Dic-state-root".to_vec();
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
            crate::canister_sig_creation::hash_bytes(b"seed"),
            labeled(
                crate::canister_sig_creation::hash_bytes(b"message"),
                leaf(Vec::new()),
            ),
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
        cbor(&CanisterSignature {
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
fn upstream_envelope_forms_preserve_authenticated_time() {
    let (signature, public, root) = envelope(time(vec![42]), None);
    let decoded: CanisterSignature = serde_cbor::from_slice(&signature).unwrap();
    let mut packed = Vec::new();
    let mut serializer = serde_cbor::Serializer::new(&mut packed).packed_format();
    serializer.self_describe().unwrap();
    decoded.serialize(&mut serializer).unwrap();
    // Serde structs also accept sequences. Only the outer representation changes;
    // the certificate, witness, public key and all signed bytes remain identical.
    let sequence = cbor(&(&decoded.certificate, &decoded.tree));
    for signature in [&signature, &packed, &sequence] {
        assert_eq!(
            ic_signature_verification::verify_canister_sig(b"message", signature, &public, &root),
            Ok(())
        );
        assert_eq!(
            verify_canister_sig(b"message", signature, &public, &root),
            Ok(())
        );
        assert_eq!(
            verify_canister_sig_with_time(b"message", signature, &public, &root),
            Ok(42)
        );
        assert!(verify_canister_sig(b"changed", signature, &public, &root).is_err());
    }
}

#[test]
fn certified_range_decoding_and_order_match_the_registry_reference() {
    use crate::principal::Principal;
    type OwnedRanges = Vec<(Principal, Principal)>;
    type ExternalRanges = Vec<(candid::Principal, candid::Principal)>;

    let principals: Vec<_> = (0..=29)
        .flat_map(|length| {
            [0, 2, 255].map(move |value| Principal::from_slice(&vec![value; length]))
        })
        .collect();
    for left in &principals {
        for right in &principals {
            let external_left = candid::Principal::from_slice(left.as_slice());
            let external_right = candid::Principal::from_slice(right.as_slice());
            assert_eq!(left.cmp(right), external_left.cmp(&external_right));
            let owned = vec![(*left, *right)];
            let external = vec![(external_left, external_right)];
            let bytes = serde_cbor::to_vec(&external).unwrap();
            assert_eq!(serde_cbor::to_vec(&owned).unwrap(), bytes);
            assert_eq!(
                serde_cbor::from_slice::<OwnedRanges>(&bytes).unwrap(),
                owned
            );
        }
    }

    // Definite/indefinite bytes and arrays, the self-describing tag, empty and
    // marker-looking principal bytes, and malformed/overlong range components.
    let cases = [
        vec![0x81, 0x82, 0x41, 0xff, 0x42, 0, 0],
        vec![
            0x81, 0x82, 0x5f, 0x41, 0xff, 0xff, 0x5f, 0x41, 0, 0x41, 0, 0xff,
        ],
        vec![0x9f, 0x9f, 0x40, 0x41, 2, 0xff, 0xff],
        vec![0xd9, 0xd9, 0xf7, 0x81, 0x82, 0x40, 0x41, 2],
        vec![0x81, 0x81, 0x40],
        vec![0x81, 0x82, 0, 0x40],
        vec![0x81, 0x82, 0x41],
        [vec![0x81, 0x82, 0x58, 30], vec![0; 30], vec![0x40]].concat(),
    ];
    for bytes in cases {
        let owned = serde_cbor::from_slice::<OwnedRanges>(&bytes);
        let external = serde_cbor::from_slice::<ExternalRanges>(&bytes);
        assert_eq!(owned.is_ok(), external.is_ok(), "{bytes:x?}");
        if let (Ok(owned), Ok(external)) = (owned, external) {
            let converted: OwnedRanges = external
                .into_iter()
                .map(|(left, right)| {
                    (
                        Principal::from_slice(left.as_slice()),
                        Principal::from_slice(right.as_slice()),
                    )
                })
                .collect();
            assert_eq!(owned, converted);
            assert_eq!(
                serde_cbor::from_reader::<OwnedRanges, _>(&bytes[..]).unwrap(),
                owned
            );
            let external: ExternalRanges = serde_cbor::from_reader(&bytes[..]).unwrap();
            assert_eq!(
                serde_cbor::to_vec(&external).unwrap(),
                serde_cbor::to_vec(&owned).unwrap()
            );
        }
    }
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
            verify_canister_sig(b"message", &signature, &public, &root),
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
            verify_canister_sig(b"message", &signature, &public, &root),
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
        let error = verify_canister_sig(message, signature, public, root).unwrap_err();
        assert_eq!(
            verify_canister_sig_with_time(message, signature, public, root),
            Err(CanisterSigTimeError::Verification(error))
        );
    }
}

#[test]
fn identity_root_cannot_authenticate_a_certificate() {
    let (signature, public, root) = envelope(time(vec![42]), None);
    assert_eq!(
        verify_canister_sig_with_time(b"message", &signature, &public, &root),
        Ok(42)
    );
    let mut envelope: CanisterSignature = serde_cbor::from_slice(&signature).unwrap();
    let cert: Certificate = serde_cbor::from_slice(&envelope.certificate).unwrap();
    let identity = key(0);
    let cert = certificate(cert.tree, &identity, None);
    envelope.certificate = ByteBuf::from(cbor(&cert));
    let forged = cbor(&envelope);
    let root = identity.public_key().serialize();
    let error = verify_canister_sig(b"message", &forged, &public, &root).unwrap_err();
    assert_eq!(
        verify_canister_sig_with_time(b"message", &forged, &public, &root),
        Err(CanisterSigTimeError::Verification(error))
    );
    if let Some(path) = std::env::var_os("PLATFORM_SIGNATURE_IDENTITY_FIXTURE") {
        let text: String = forged.iter().map(|byte| format!("{byte:02x}")).collect();
        std::fs::write(path, text + "\n").unwrap();
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

#[test]
fn delegated_range_formats_preserve_owned_principal_admission() {
    let subnet = vec![3; 10];
    let mut subnet_key = IC_ROOT_PK_DER_PREFIX.to_vec();
    subnet_key.extend_from_slice(&key(2).public_key().serialize());
    for sharded in [false, true] {
        for allowed in [true, false] {
            // Literal protocol bytes: one range with equal 10-byte endpoints.
            // Avoid using the owned serializer to generate its own acceptance oracle.
            let endpoint = if allowed { 1 } else { 2 };
            let ranges = [
                vec![0x81, 0x82, 0x4a],
                vec![endpoint; 10],
                vec![0x4a],
                vec![endpoint; 10],
            ]
            .concat();
            let public_key_tree = labeled(b"public_key".to_vec(), leaf(subnet_key.clone()));
            let delegation_tree = if sharded {
                fork(
                    labeled(
                        b"canister_ranges".to_vec(),
                        labeled(subnet.clone(), labeled(vec![0; 10], leaf(ranges))),
                    ),
                    labeled(b"subnet".to_vec(), labeled(subnet.clone(), public_key_tree)),
                )
            } else {
                labeled(
                    b"subnet".to_vec(),
                    labeled(
                        subnet.clone(),
                        fork(
                            labeled(b"canister_ranges".to_vec(), leaf(ranges)),
                            public_key_tree,
                        ),
                    ),
                )
            };
            let delegation = Delegation {
                subnet_id: subnet.clone(),
                certificate: cbor(&certificate(delegation_tree, &key(1), None)),
            };
            let (signature, public, root) = envelope(time(vec![42]), Some(delegation));
            let result = verify_canister_sig_with_time(b"message", &signature, &public, &root);
            if allowed {
                assert_eq!(result, Ok(42));
            } else {
                assert_eq!(
                    result,
                    Err(CanisterSigTimeError::Verification(
                        "signing canister id not in canister_ranges".into()
                    ))
                );
            }
            if let Some(directory) = std::env::var_os("PLATFORM_SIGNATURE_RANGE_FIXTURES") {
                let name = format!(
                    "signature-range-{}-{}.hex",
                    if sharded { "sharded" } else { "legacy" },
                    if allowed { "valid" } else { "outside" }
                );
                let text: String = signature.iter().map(|byte| format!("{byte:02x}")).collect();
                std::fs::write(std::path::Path::new(&directory).join(name), text + "\n").unwrap();
            }
        }
    }
}

#[test]
fn identity_keys_cannot_authenticate_either_delegation_level() {
    // Each envelope is accepted by registry 0.3.0/0.6.0. The integration guard
    // must reject identity keys both at the trust root and in the subnet leaf.
    for (root_key, signing_key) in [(key(0), key(2)), (key(1), key(0)), (key(0), key(0))] {
        let subnet = vec![3; 10];
        let mut subnet_key = IC_ROOT_PK_DER_PREFIX.to_vec();
        subnet_key.extend_from_slice(&signing_key.public_key().serialize());
        let delegation_tree = labeled(
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
        );
        let delegation = Delegation {
            subnet_id: subnet,
            certificate: cbor(&certificate(delegation_tree, &root_key, None)),
        };
        let (signature, public, _) = envelope(time(vec![42]), Some(delegation));
        let mut envelope: CanisterSignature = serde_cbor::from_slice(&signature).unwrap();
        let cert: Certificate = serde_cbor::from_slice(&envelope.certificate).unwrap();
        envelope.certificate =
            ByteBuf::from(cbor(&certificate(cert.tree, &signing_key, cert.delegation)));
        let signature = cbor(&envelope);
        let root = root_key.public_key().serialize();
        assert_eq!(
            ic_signature_verification::verify_canister_sig(b"message", &signature, &public, &root),
            Ok(())
        );
        assert_eq!(
            verify_canister_sig(b"message", &signature, &public, &root),
            Err("invalid BLS signature".into())
        );
        assert_eq!(
            verify_canister_sig_with_time(b"message", &signature, &public, &root),
            Err(CanisterSigTimeError::Verification(
                "invalid BLS signature".into()
            ))
        );
    }
}

#[test]
fn signing_time_errors_preserve_display_and_absent_source() {
    use std::error::Error as _;
    for (error, message) in [
        (
            CanisterSigTimeError::Verification("upstream refusal".into()),
            "upstream refusal",
        ),
        (
            CanisterSigTimeError::InvalidCertificateTime,
            "invalid signing certificate time",
        ),
    ] {
        assert_eq!(error.to_string(), message);
        assert!(error.source().is_none());
    }
}
