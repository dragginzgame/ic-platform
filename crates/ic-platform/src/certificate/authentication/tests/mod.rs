use super::*;
use crate::certificate::{CertificateDecodeLimits, decode_certificate};
use ic_verify_bls_signature::PrivateKey;

fn signer(number: u8) -> PrivateKey {
    let mut scalar = [0; 32];
    scalar[31] = number;
    PrivateKey::deserialize(&scalar).unwrap()
}
fn root(number: u8) -> PreparedRootKey {
    let mut der = DER_PREFIX.to_vec();
    der.extend_from_slice(&signer(number).public_key().serialize());
    PreparedRootKey::from_der(&der).unwrap()
}
fn window() -> TimeWindow {
    TimeWindow {
        reference_ns: 10,
        max_age_ns: 1,
        max_future_skew_ns: 1,
    }
}
fn limits() -> VerificationLimits {
    VerificationLimits {
        tree_nodes: 2,
        tree_depth: 2,
        delegation_bytes: 0,
        canister_ranges: 0,
    }
}
fn envelope(tree: &[u8], signature: &[u8]) -> Vec<u8> {
    let mut bytes = b"\xa2\x64tree".to_vec();
    bytes.extend_from_slice(tree);
    bytes.extend_from_slice(b"\x69signature\x58");
    bytes.push(u8::try_from(signature.len()).unwrap());
    bytes.extend_from_slice(signature);
    bytes
}
fn signed(time: u8) -> Vec<u8> {
    signed_time(&[time])
}
fn signed_time(time: &[u8]) -> Vec<u8> {
    let tree = ic_certification::labeled(b"time".as_slice(), ic_certification::leaf(time));
    let mut message = b"\x0dic-state-root".to_vec();
    message.extend_from_slice(&tree.digest());
    let mut wire = b"\x83\x02\x44time\x82\x03".to_vec();
    wire.push(0x40 + u8::try_from(time.len()).unwrap());
    wire.extend_from_slice(time);
    envelope(&wire, &signer(1).sign(&message).serialize())
}
fn verify<'a>(
    bytes: &'a [u8],
    key: &PreparedRootKey,
    time: TimeWindow,
    cap: VerificationLimits,
) -> Result<AuthenticatedCertificate<'a>, VerifyError> {
    let decoded =
        decode_certificate(bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
    authenticate_direct_certificate(decoded, key, Principal::anonymous(), time, cap)
}

#[test]
fn direct_evidence_binds_original_bytes_root_target_and_window() {
    let bytes = signed(10);
    let key = root(1);
    let evidence = verify(&bytes, &key, window(), limits()).unwrap();
    assert!(std::ptr::eq(
        evidence.original_bytes().as_ptr(),
        bytes.as_ptr()
    ));
    assert_eq!(evidence.root_key_der(), key.as_der());
    assert_eq!(evidence.effective_canister(), Principal::anonymous());
    assert_eq!(evidence.time_window(), window());
    assert_eq!(evidence.time_ns(), 10);
    // Independently calculated with Node's SHA-256 and the IC tree hash domains.
    assert_eq!(
        evidence.tree().digest(),
        [
            0x9c, 0x66, 0xc2, 0x65, 0xb9, 0xa3, 0xb5, 0x2b, 0x2c, 0x83, 0x63, 0xab, 0x5e, 0xe6,
            0x79, 0x5e, 0x58, 0x0c, 0xd2, 0x5e, 0xde, 0x31, 0x47, 0xd1, 0x09, 0xb0, 0x45, 0xb7,
            0x19, 0xc4, 0xa3, 0x6f
        ]
    );
    assert_eq!(evidence.into_unverified().original_bytes(), bytes);
}

#[test]
fn inclusive_windows_and_overflow_refuse_without_clamping() {
    for time in [8, 9, 10, 11, 12] {
        let bytes = signed(time);
        let result = verify(&bytes, &root(1), window(), limits());
        if (9..=11).contains(&time) {
            assert_eq!(result.unwrap().time_ns(), u64::from(time));
        } else {
            assert_eq!(result.unwrap_err(), VerifyError::TimeOutsideWindow);
        }
    }
    for policy in [
        TimeWindow {
            reference_ns: 0,
            max_age_ns: 1,
            max_future_skew_ns: 0,
        },
        TimeWindow {
            reference_ns: u64::MAX,
            max_age_ns: 0,
            max_future_skew_ns: 1,
        },
    ] {
        assert_eq!(
            verify(&signed(10), &root(1), policy, limits()).unwrap_err(),
            VerifyError::InvalidWindow
        );
    }
}

#[test]
fn exact_der_and_checked_nonidentity_root_are_required() {
    let key = root(1);
    for end in 0..133 {
        assert!(PreparedRootKey::from_der(&key.as_der()[..end]).is_err());
    }
    let mut extra = key.as_der().to_vec();
    extra.push(0);
    assert!(PreparedRootKey::from_der(&extra).is_err());
    for index in 0..DER_PREFIX.len() {
        let mut bytes = *key.as_der();
        bytes[index] ^= 1;
        assert!(PreparedRootKey::from_der(&bytes).is_err());
    }
    let mut bytes = *key.as_der();
    bytes[37..].fill(0);
    bytes[37] = 0xc0;
    // Upstream admits the subgroup identity; Platform deliberately does not.
    assert!(PublicKey::deserialize(&bytes[37..]).is_ok());
    assert_eq!(
        PreparedRootKey::from_der(&bytes).unwrap_err(),
        VerifyError::InvalidRootKey
    );
    bytes[37] = 0xff;
    assert_eq!(
        PreparedRootKey::from_der(&bytes).unwrap_err(),
        VerifyError::InvalidRootKey
    );
}

#[test]
fn wrong_root_mutation_invalid_and_identity_signatures_refuse() {
    let mut bytes = signed(10);
    assert_eq!(
        verify(&bytes, &root(2), window(), limits()).unwrap_err(),
        VerifyError::Signature
    );
    *bytes.last_mut().unwrap() ^= 1;
    assert_eq!(
        verify(&bytes, &root(1), window(), limits()).unwrap_err(),
        VerifyError::Signature
    );
    let tree = b"\x83\x02\x44time\x82\x03\x41\x0a";
    for signature in [vec![], vec![0xff; 48], {
        let mut s = vec![0; 48];
        s[0] = 0xc0;
        s
    }] {
        assert_eq!(
            verify(&envelope(tree, &signature), &root(1), window(), limits()).unwrap_err(),
            VerifyError::Signature
        );
    }
    let mut bytes = signed(10);
    bytes[16] = 11; // Change the admitted time leaf, not its signature.
    assert_eq!(
        verify(&bytes, &root(1), window(), limits()).unwrap_err(),
        VerifyError::Signature
    );
}

#[test]
fn delegation_never_falls_back_to_direct_verification() {
    let mut bytes = signed(10);
    bytes[0] = 0xa3;
    bytes.extend_from_slice(b"\x6adelegation\xa2\x69subnet_id\x41\x04\x6bcertificate\x40");
    assert_eq!(
        verify(&bytes, &root(1), window(), limits()).unwrap_err(),
        VerifyError::DelegationUnsupported
    );
}

#[test]
fn traversal_limits_are_exact_and_ambiguous_forests_refuse() {
    for cap in [
        VerificationLimits {
            tree_nodes: 1,
            tree_depth: 2,
            delegation_bytes: 0,
            canister_ranges: 0,
        },
        VerificationLimits {
            tree_nodes: 2,
            tree_depth: 1,
            delegation_bytes: 0,
            canister_ranges: 0,
        },
    ] {
        assert_eq!(
            verify(&signed(10), &root(1), window(), cap).unwrap_err(),
            VerifyError::TreeLimit
        );
    }
    let time = b"\x83\x02\x44time\x82\x03\x41\x0a";
    for left in [time.as_slice(), b"\x83\x02\x41z\x81\x00", b"\x82\x03\x40"] {
        let mut tree = vec![0x83, 1];
        tree.extend_from_slice(left);
        tree.extend_from_slice(time);
        assert_eq!(
            verify(
                &envelope(&tree, &[0; 48]),
                &root(1),
                window(),
                VerificationLimits {
                    tree_nodes: 10,
                    tree_depth: 10,
                    delegation_bytes: 0,
                    canister_ranges: 0,
                }
            )
            .unwrap_err(),
            VerifyError::InvalidTree
        );
    }
}

#[test]
fn missing_pruned_trailing_and_overflow_time_refuse() {
    let signed = |tree: &[u8]| {
        let bytes = envelope(tree, &[0; 48]);
        let decoded =
            decode_certificate(&bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
        let message = [
            b"\x0dic-state-root".as_slice(),
            &decoded.unverified().tree.digest(),
        ]
        .concat();
        envelope(tree, &signer(1).sign(&message).serialize())
    };
    let cap = VerificationLimits {
        tree_nodes: 10,
        tree_depth: 10,
        delegation_bytes: 0,
        canister_ranges: 0,
    };
    for tree in [
        b"\x81\x00".as_slice(),
        b"\x83\x02\x44time\x82\x03\x40",
        b"\x83\x02\x44time\x82\x03\x41\x80",
        b"\x83\x02\x44time\x82\x03\x42\x0a\x00",
        b"\x83\x02\x44time\x82\x03\x4a\x80\x80\x80\x80\x80\x80\x80\x80\x80\x02",
    ] {
        assert_eq!(
            verify(&signed(tree), &root(1), window(), cap).unwrap_err(),
            VerifyError::InvalidTime
        );
    }
    let mut tree = b"\x83\x02\x44time\x82\x04\x58\x20".to_vec();
    tree.extend_from_slice(&[0; 32]);
    assert_eq!(
        verify(&signed(&tree), &root(1), window(), cap).unwrap_err(),
        VerifyError::InvalidTime
    );
}

#[test]
fn complete_u64_time_and_retained_nonminimal_encoding() {
    let bytes = signed_time(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 1]);
    let policy = TimeWindow {
        reference_ns: u64::MAX,
        max_age_ns: 0,
        max_future_skew_ns: 0,
    };
    assert_eq!(
        verify(&bytes, &root(1), policy, limits())
            .unwrap()
            .time_ns(),
        u64::MAX
    );
    assert_eq!(
        verify(&signed_time(&[0x8a, 0]), &root(1), window(), limits())
            .unwrap()
            .time_ns(),
        10
    );
}

#[test]
fn signature_without_ic_state_root_domain_refuses() {
    let tree = ic_certification::labeled(b"time".as_slice(), ic_certification::leaf([10]));
    let signature = signer(1).sign(&tree.digest()).serialize();
    let bytes = envelope(b"\x83\x02\x44time\x82\x03\x41\x0a", &signature);
    assert_eq!(
        verify(&bytes, &root(1), window(), limits()).unwrap_err(),
        VerifyError::Signature
    );
}

mod delegation_tests;

#[test]
fn subnet_root_binding_and_full_width_time_are_explicit() {
    let subnet = Principal::from_slice(&[1]);
    let bytes = signed_time(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 1]);
    let exact = TimeWindow {
        reference_ns: u64::MAX,
        max_age_ns: 0,
        max_future_skew_ns: 0,
    };
    let policy = TimePolicy {
        certificate: exact,
        delegation: exact,
    };
    let decode =
        || decode_certificate(&bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
    assert_eq!(
        authenticate_subnet_time(decode(), &root(1), subnet, None, policy, limits()).unwrap_err(),
        VerifyError::RootSubnetRequired
    );
    assert_eq!(
        authenticate_subnet_time(
            decode(),
            &root(1),
            subnet,
            Some(Principal::anonymous()),
            policy,
            limits()
        )
        .unwrap_err(),
        VerifyError::SubnetMismatch
    );
    let evidence =
        authenticate_subnet_time(decode(), &root(1), subnet, Some(subnet), policy, limits())
            .unwrap();
    assert_eq!(evidence.time_ns(), u64::MAX);
    assert_eq!(evidence.subnet(), subnet);
    assert_eq!(evidence.delegation(), None);
    assert_eq!(evidence.original_bytes().unwrap().as_ptr(), bytes.as_ptr());
    assert_eq!(
        authenticate_subnet_time(decode(), &root(2), subnet, Some(subnet), policy, limits())
            .unwrap_err(),
        VerifyError::Signature
    );
}

#[test]
fn subnet_time_retains_strict_leaf_and_inclusive_window_rules() {
    let subnet = Principal::from_slice(&[1]);
    for (time, accepted) in [(8, false), (9, true), (10, true), (11, true), (12, false)] {
        let bytes = signed(time);
        let decoded =
            decode_certificate(&bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
        let result = authenticate_subnet_time(
            decoded,
            &root(1),
            subnet,
            Some(subnet),
            TimePolicy {
                certificate: window(),
                delegation: window(),
            },
            limits(),
        );
        if accepted {
            assert!(result.is_ok());
        } else {
            assert_eq!(result.unwrap_err(), VerifyError::TimeOutsideWindow);
        }
    }
    for time in [vec![], vec![0x80], vec![10, 0], vec![0xff; 10]] {
        let bytes = signed_time(&time);
        let decoded =
            decode_certificate(&bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
        assert_eq!(
            authenticate_subnet_time(
                decoded,
                &root(1),
                subnet,
                Some(subnet),
                TimePolicy {
                    certificate: window(),
                    delegation: window()
                },
                limits()
            )
            .unwrap_err(),
            VerifyError::InvalidTime
        );
    }
    let bytes = signed_time(&[0x8a, 0]);
    let decoded =
        decode_certificate(&bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
    assert_eq!(
        authenticate_subnet_time(
            decoded,
            &root(1),
            subnet,
            Some(subnet),
            TimePolicy {
                certificate: window(),
                delegation: window()
            },
            limits()
        )
        .unwrap()
        .time_ns(),
        10
    );
}

#[test]
fn invalid_signature_precedes_malformed_time_diagnostics() {
    let mut bytes = signed_time(&[]);
    assert_eq!(
        verify(&bytes, &root(1), window(), limits()).unwrap_err(),
        VerifyError::InvalidTime
    );
    *bytes.last_mut().unwrap() ^= 1;
    assert_eq!(
        verify(&bytes, &root(1), window(), limits()).unwrap_err(),
        VerifyError::Signature
    );
}
