//! Independently selected compressed inputs, classified by upstream point admission.
#![cfg(feature = "certificate-authentication")]
#![forbid(unsafe_code)]

use candidate_types::Principal;
use ic_bls12_381::{G1Affine, G2Affine};
use ic_platform::certificate::{
    CertificateDecodeLimits, PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits,
    VerifyError, authenticate_certificate, authenticate_subnet_time, decode_certificate,
};

// Small canonical compressed x coordinates. All point decoding and subgroup
// classification stay upstream; no curve arithmetic or copied upstream vectors.
fn g1_non_subgroup() -> [u8; 48] {
    let mut bytes = [0; 48];
    bytes[0] = 0x80;
    bytes
}

fn g2_non_subgroup() -> [u8; 96] {
    let mut bytes = [0; 96];
    bytes[0] = 0x80;
    bytes[95] = 2;
    bytes
}

#[test]
fn fixtures_are_canonical_nonidentity_curve_points_outside_the_subgroup() {
    // Unchecked admission is confined to classification of test data. These
    // points are never used for signing, verification, pairing or public evidence.
    let g1 = g1_non_subgroup();
    let point = Option::<G1Affine>::from(G1Affine::from_compressed_unchecked(&g1)).unwrap();
    assert!(bool::from(point.is_on_curve()));
    assert!(!bool::from(point.is_identity()));
    assert!(!bool::from(point.is_torsion_free()));
    assert_eq!(point.to_compressed(), g1);
    assert!(bool::from(G1Affine::from_compressed(&g1).is_none()));

    let g2 = g2_non_subgroup();
    let point = Option::<G2Affine>::from(G2Affine::from_compressed_unchecked(&g2)).unwrap();
    assert!(bool::from(point.is_on_curve()));
    assert!(!bool::from(point.is_identity()));
    assert!(!bool::from(point.is_torsion_free()));
    assert_eq!(point.to_compressed(), g2);
    assert!(bool::from(G2Affine::from_compressed(&g2).is_none()));
}

#[test]
fn prepared_roots_reject_the_non_subgroup_public_key() {
    let mut der = *include_bytes!("fixtures/root.der");
    der[37..].copy_from_slice(&g2_non_subgroup());
    assert_eq!(
        PreparedRootKey::from_der(&der).unwrap_err(),
        VerifyError::InvalidRootKey
    );
}

#[test]
fn canister_and_subnet_authentication_reject_non_subgroup_signatures() {
    let root = PreparedRootKey::from_der(include_bytes!("fixtures/root.der")).unwrap();
    let root_subnet =
        Principal::from_text("hzklm-c4flk-lfdw4-v4532-s744u-jbwhv-6zh6w-be4il-apmtb-fosk7-wae")
            .unwrap();
    for (bytes, target, subnet, timestamp) in [
        (
            include_bytes!("fixtures/direct.cbor").as_slice(),
            "xp3jw-ot777-77777-aaaaa-cai",
            root_subnet,
            1_620_328_630_000_000_005,
        ),
        (
            include_bytes!("fixtures/delegated.cbor").as_slice(),
            "dllsh-pd777-77776-qaaaa-cai",
            Principal::from_text("sgier-p6fyz-ejpwm-jghgc-4ypgv-6ft53-5hns6-ojhae-wvlic-ifcah-eqe")
                .unwrap(),
            1_620_328_630_000_000_008,
        ),
    ] {
        let original =
            decode_certificate(bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
        let signature = &original.unverified().signature;
        let positions: Vec<_> = bytes
            .windows(signature.len())
            .enumerate()
            .filter(|(_, value)| *value == signature.as_slice())
            .map(|(offset, _)| offset)
            .collect();
        assert_eq!(positions.len(), 1);
        let mut changed = bytes.to_vec();
        changed[positions[0]..positions[0] + 48].copy_from_slice(&g1_non_subgroup());
        let decode = || {
            decode_certificate(
                &changed,
                CertificateDecodeLimits {
                    bytes: changed.len(),
                },
            )
            .unwrap()
        };
        let window = TimeWindow {
            reference_ns: timestamp,
            max_age_ns: 0,
            max_future_skew_ns: 0,
        };
        let time = TimePolicy {
            certificate: window,
            delegation: window,
        };
        let limits = VerificationLimits {
            tree_nodes: 4096,
            tree_depth: 64,
            delegation_bytes: 131072,
            canister_ranges: 256,
        };
        assert_eq!(
            authenticate_certificate(
                decode(),
                &root,
                Principal::from_text(target).unwrap(),
                time,
                limits
            )
            .unwrap_err(),
            VerifyError::Signature
        );
        assert_eq!(
            authenticate_subnet_time(decode(), &root, subnet, Some(root_subnet), time, limits)
                .unwrap_err(),
            VerifyError::Signature
        );
    }
}
