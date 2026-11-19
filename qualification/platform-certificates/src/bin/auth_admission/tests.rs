//! Policy boundary cases using independent frozen replica observations.

use super::{ALL_TIMES, Policy, Refusal, message, prepare_root, verify};
use candidate_platform::certificate::{
    CanisterSignatureKey, CanisterSignatureLimits, CanisterSignatureVerifyError,
    CertificateDecodeLimits, TimePolicy, TimeWindow, VerificationLimits, VerifyError,
    authenticate_canister_signature, decode_canister_signature,
};
use candidate_types::Principal;
use sha2::{Digest, Sha256};

pub(crate) fn run() {
    let root_der = include_bytes!("../../../fixtures/signatures/root.der");
    let raw_root = &root_der[37..];
    assert_eq!(prepare_root(raw_root).unwrap().as_der(), root_der);
    assert_eq!(
        message([0x42; 32]),
        *b"\x1bic-auth-portable-credentialBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"
    );
    for (signature, key_der, timestamp, issuer, delegated) in [
        (
            include_bytes!("../../../fixtures/signatures/auth-direct.cbor").as_slice(),
            include_bytes!("../../../fixtures/signatures/auth-direct.der").as_slice(),
            1_620_328_630_000_000_011_u64,
            "xi2pc-dl777-77777-aaaaq-cai",
            false,
        ),
        (
            include_bytes!("../../../fixtures/signatures/auth-delegated.cbor").as_slice(),
            include_bytes!("../../../fixtures/signatures/auth-delegated.der").as_slice(),
            1_620_328_630_000_000_014_u64,
            "dmkut-c3777-77776-qaaaq-cai",
            true,
        ),
    ] {
        let key = CanisterSignatureKey::from_der(key_der).unwrap();
        let mut policy = Policy {
            issuer: Principal::from_text(issuer).unwrap(),
            // Independently computed fixture commitment, not read from proof input.
            seed_hash: [
                0x47, 0xe5, 0x25, 0x00, 0xac, 0xfa, 0x5a, 0x92, 0x6e, 0x4f, 0x26, 0xf1, 0xb9, 0xfb,
                0xb4, 0x55, 0x1e, 0xd1, 0xa8, 0x72, 0xe7, 0xf5, 0x5e, 0x51, 0x75, 0x23, 0x94, 0x4a,
                0x6a, 0xab, 0x47, 0xf6,
            ],
            root_raw: raw_root,
            now: timestamp,
            maximum_age: 0,
            signature_bytes: signature.len(),
            verification: VerificationLimits {
                tree_nodes: 4096,
                tree_depth: 64,
                delegation_bytes: 131_072,
                canister_ranges: 256,
            },
        };
        let check = |policy: &Policy<'_>| verify([0x42; 32], signature, key_der, policy);
        assert_eq!(check(&policy), Ok(timestamp));
        if delegated {
            // Delegation freshness is independently selectable. A narrow window
            // refuses this same otherwise-valid signature; the adapter's explicit
            // all-time delegation policy accepts it without selecting a default.
            let narrow_delegation = authenticate_canister_signature(
                decode_canister_signature(
                    signature,
                    CertificateDecodeLimits {
                        bytes: signature.len(),
                    },
                )
                .unwrap(),
                &message([0x42; 32]),
                key,
                &prepare_root(raw_root).unwrap(),
                TimePolicy {
                    certificate: ALL_TIMES,
                    delegation: TimeWindow {
                        reference_ns: timestamp + 1,
                        max_age_ns: 0,
                        max_future_skew_ns: 0,
                    },
                },
                CanisterSignatureLimits {
                    message_bytes: 60,
                    certificate_bytes: 131_072,
                    verification: policy.verification,
                },
            )
            .unwrap_err();
            assert!(matches!(
                narrow_delegation,
                CanisterSignatureVerifyError::Verification(VerifyError::TimeOutsideWindow)
            ));
        }
        policy.now += 10;
        policy.maximum_age = 10;
        assert_eq!(check(&policy), Ok(timestamp));
        policy.maximum_age = 9;
        assert_eq!(check(&policy), Err(Refusal::Stale));
        policy.now = timestamp - 1;
        policy.maximum_age = u64::MAX;
        assert_eq!(check(&policy), Err(Refusal::Future));
        policy.now = 0;
        assert_eq!(check(&policy), Err(Refusal::Future));
        policy.now = u64::MAX;
        assert_eq!(check(&policy), Ok(timestamp));
        policy.now = timestamp;
        policy.maximum_age = 0;
        policy.issuer = Principal::anonymous();
        assert_eq!(check(&policy), Err(Refusal::Issuer));
        policy.issuer = Principal::from_text(issuer).unwrap();
        policy.seed_hash[0] ^= 1;
        assert_eq!(check(&policy), Err(Refusal::Seed));
        policy.seed_hash[0] ^= 1;
        policy.root_raw = &raw_root[..95];
        assert_eq!(check(&policy), Err(Refusal::RootLength));
        policy.root_raw = &include_bytes!("../../../fixtures/mainnet-root.der")[37..];
        assert_eq!(check(&policy), Err(Refusal::Authentication));
        policy.root_raw = raw_root;
        assert_eq!(
            verify([0x43; 32], signature, key_der, &policy),
            Err(Refusal::Authentication)
        );
        assert_eq!(
            verify([0x42; 32], signature, &[0; 20], &policy),
            Err(Refusal::Key)
        );
        policy.signature_bytes -= 1;
        assert_eq!(check(&policy), Err(Refusal::SignatureSize));
        policy.signature_bytes = usize::MAX;
        assert_eq!(
            verify([0x42; 32], &vec![0; 131_073], &[], &policy),
            Err(Refusal::SignatureSize)
        );
        assert_eq!(
            verify([0x42; 32], &[0], key_der, &policy),
            Err(Refusal::Authentication)
        );
        policy.verification.tree_nodes = 0;
        assert_eq!(check(&policy), Err(Refusal::Authentication));
    }
    // A valid signature for another message domain cannot authenticate claims.
    let key_der = include_bytes!("../../../fixtures/signatures/direct.der");
    let policy = Policy {
        issuer: CanisterSignatureKey::from_der(key_der).unwrap().issuer(),
        seed_hash: Sha256::digest(b"qualification-seed").into(),
        root_raw: raw_root,
        now: 1_620_328_630_000_000_005,
        maximum_age: 0,
        signature_bytes: 131_072,
        verification: VerificationLimits {
            tree_nodes: 4096,
            tree_depth: 64,
            delegation_bytes: 131_072,
            canister_ranges: 256,
        },
    };
    assert_eq!(
        verify(
            [0x42; 32],
            include_bytes!("../../../fixtures/signatures/direct.cbor"),
            key_der,
            &policy
        ),
        Err(Refusal::Authentication)
    );
}
