//! Independently observed PocketIC frames, replayed through public consumer APIs.
#![cfg(feature = "certificate-authentication")]
#![forbid(unsafe_code)]

use candidate_types::Principal;
use ic_certification::LookupResult;
use ic_platform::certificate::{
    CertificateDecodeLimits, PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits,
    VerifyError, authenticate_certificate, authenticate_direct_certificate, decode_certificate,
};

const ROOT: &[u8] = include_bytes!("fixtures/root.der");
const DIRECT: &[u8] = include_bytes!("fixtures/direct.cbor");
const DELEGATED: &[u8] = include_bytes!("fixtures/delegated.cbor");
const DIRECT_TARGET: &str = "xp3jw-ot777-77777-aaaaa-cai";
const DELEGATED_TARGET: &str = "dllsh-pd777-77776-qaaaa-cai";

fn limits() -> VerificationLimits {
    VerificationLimits {
        tree_nodes: 4096,
        tree_depth: 64,
        delegation_bytes: 131072,
        canister_ranges: 256,
    }
}

fn policy(timestamp: u64) -> TimePolicy {
    let window = TimeWindow {
        reference_ns: timestamp,
        max_age_ns: 0,
        max_future_skew_ns: 0,
    };
    TimePolicy {
        certificate: window,
        delegation: window,
    }
}

#[test]
fn replay_observed_direct_and_delegated_certificates() {
    let root = PreparedRootKey::from_der(ROOT).unwrap();
    for (bytes, target, timestamp, delegated) in [
        (DIRECT, DIRECT_TARGET, 1_620_328_630_000_000_005, false),
        (DELEGATED, DELEGATED_TARGET, 1_620_328_630_000_000_008, true),
    ] {
        let target = Principal::from_text(target).unwrap();
        let decoded =
            decode_certificate(bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
        let evidence =
            authenticate_certificate(decoded, &root, target, policy(timestamp), limits()).unwrap();
        assert_eq!(evidence.original_bytes().as_ptr(), bytes.as_ptr());
        assert_eq!(evidence.root_key_der(), root.as_der());
        assert_eq!(evidence.effective_canister(), target);
        assert_eq!(evidence.time_ns(), timestamp);
        assert_eq!(evidence.time_window(), policy(timestamp).certificate);
        assert_eq!(evidence.delegation().is_some(), delegated);
        if let Some((subnet, window, time)) = evidence.delegation() {
            assert_eq!(
                subnet,
                Principal::from_text(
                    "sgier-p6fyz-ejpwm-jghgc-4ypgv-6ft53-5hns6-ojhae-wvlic-ifcah-eqe"
                )
                .unwrap()
            );
            assert_eq!(window, policy(timestamp).delegation);
            assert_eq!(time, timestamp);
        }
        assert_eq!(
            evidence.tree().lookup_path([
                b"canister".as_slice(),
                target.as_slice(),
                b"certified_data"
            ]),
            LookupResult::Found(&[0; 32])
        );
    }
}

#[test]
fn replay_refuses_wrong_target_time_and_direct_only_path() {
    let root = PreparedRootKey::from_der(ROOT).unwrap();
    let target = Principal::from_text(DELEGATED_TARGET).unwrap();
    let decode = || {
        decode_certificate(
            DELEGATED,
            CertificateDecodeLimits {
                bytes: DELEGATED.len(),
            },
        )
        .unwrap()
    };
    let time = policy(1_620_328_630_000_000_008);
    assert_eq!(
        authenticate_certificate(
            decode(),
            &root,
            Principal::from_text(DIRECT_TARGET).unwrap(),
            time,
            limits()
        )
        .unwrap_err(),
        VerifyError::NotAuthorized
    );
    assert_eq!(
        authenticate_direct_certificate(decode(), &root, target, time.certificate, limits())
            .unwrap_err(),
        VerifyError::DelegationUnsupported
    );
    let mut stale = time;
    stale.certificate.reference_ns += 1;
    assert_eq!(
        authenticate_certificate(decode(), &root, target, stale, limits()).unwrap_err(),
        VerifyError::TimeOutsideWindow
    );
    let mut stale = time;
    stale.delegation.reference_ns += 1;
    assert_eq!(
        authenticate_certificate(decode(), &root, target, stale, limits()).unwrap_err(),
        VerifyError::TimeOutsideWindow
    );
}

#[test]
fn observed_signature_corruption_never_authenticates() {
    let root = PreparedRootKey::from_der(ROOT).unwrap();
    for (bytes, target, timestamp) in [
        (DIRECT, DIRECT_TARGET, 1_620_328_630_000_000_005),
        (DELEGATED, DELEGATED_TARGET, 1_620_328_630_000_000_008),
    ] {
        let original =
            decode_certificate(bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
        let signature = &original.unverified().signature;
        let offsets: Vec<_> = bytes
            .windows(signature.len())
            .enumerate()
            .filter(|(_, window)| *window == signature.as_slice())
            .map(|(offset, _)| offset)
            .collect();
        assert_eq!(
            offsets.len(),
            1,
            "fixture signature location must be unique"
        );
        for index in 0..signature.len() {
            let mut changed = bytes.to_vec();
            changed[offsets[0] + index] ^= 1;
            let decoded = decode_certificate(
                &changed,
                CertificateDecodeLimits {
                    bytes: changed.len(),
                },
            )
            .unwrap();
            assert_eq!(
                authenticate_certificate(
                    decoded,
                    &root,
                    Principal::from_text(target).unwrap(),
                    policy(timestamp),
                    limits()
                )
                .unwrap_err(),
                VerifyError::Signature
            );
        }
    }
}

#[test]
fn observed_subnet_projection_requires_exact_source_binding() {
    use ic_platform::certificate::authenticate_subnet_time;
    let root = PreparedRootKey::from_der(ROOT).unwrap();
    let root_subnet =
        Principal::from_text("hzklm-c4flk-lfdw4-v4532-s744u-jbwhv-6zh6w-be4il-apmtb-fosk7-wae")
            .unwrap();
    let app_subnet =
        Principal::from_text("sgier-p6fyz-ejpwm-jghgc-4ypgv-6ft53-5hns6-ojhae-wvlic-ifcah-eqe")
            .unwrap();
    for (bytes, subnet, timestamp) in [
        (DIRECT, root_subnet, 1_620_328_630_000_000_005),
        (DELEGATED, app_subnet, 1_620_328_630_000_000_008),
    ] {
        let decode =
            || decode_certificate(bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
        let time = policy(timestamp);
        let evidence =
            authenticate_subnet_time(decode(), &root, subnet, Some(root_subnet), time, limits())
                .unwrap();
        assert_eq!(evidence.time_ns(), timestamp);
        assert_eq!(evidence.subnet(), subnet);
        assert_eq!(evidence.root_key_der(), root.as_der());
        assert_eq!(evidence.original_bytes().unwrap().as_ptr(), bytes.as_ptr());
        let wrong = if subnet == root_subnet {
            app_subnet
        } else {
            root_subnet
        };
        assert_eq!(
            authenticate_subnet_time(decode(), &root, wrong, Some(root_subnet), time, limits())
                .unwrap_err(),
            VerifyError::SubnetMismatch
        );
    }
}

#[test]
fn parsed_subnet_projection_borrows_the_exact_certificate_without_reencoding() {
    use ic_platform::certificate::authenticate_subnet_time_from_certificate;
    let root = PreparedRootKey::from_der(ROOT).unwrap();
    let root_subnet =
        Principal::from_text("hzklm-c4flk-lfdw4-v4532-s744u-jbwhv-6zh6w-be4il-apmtb-fosk7-wae")
            .unwrap();
    let app_subnet =
        Principal::from_text("sgier-p6fyz-ejpwm-jghgc-4ypgv-6ft53-5hns6-ojhae-wvlic-ifcah-eqe")
            .unwrap();
    for (bytes, subnet, timestamp) in [
        (DIRECT, root_subnet, 1_620_328_630_000_000_005),
        (DELEGATED, app_subnet, 1_620_328_630_000_000_008),
    ] {
        let decoded =
            decode_certificate(bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
        let cert = decoded.unverified();
        let time = policy(timestamp);
        let evidence = authenticate_subnet_time_from_certificate(
            cert,
            &root,
            subnet,
            Some(root_subnet),
            time,
            limits(),
        )
        .unwrap();
        assert!(evidence.original_bytes().is_none());
        assert!(std::ptr::eq(evidence.certificate().unwrap(), cert));
        assert_eq!(evidence.time_ns(), timestamp);
        let wrong = if subnet == root_subnet {
            app_subnet
        } else {
            root_subnet
        };
        assert_eq!(
            authenticate_subnet_time_from_certificate(
                cert,
                &root,
                wrong,
                Some(root_subnet),
                time,
                limits()
            )
            .unwrap_err(),
            VerifyError::SubnetMismatch
        );
        let mut cap = limits();
        cap.tree_nodes = 0;
        assert_eq!(
            authenticate_subnet_time_from_certificate(
                cert,
                &root,
                subnet,
                Some(root_subnet),
                time,
                cap
            )
            .unwrap_err(),
            VerifyError::TreeLimit
        );
        let mut altered = cert.clone();
        altered.signature[0] ^= 1;
        assert_eq!(
            authenticate_subnet_time_from_certificate(
                &altered,
                &root,
                subnet,
                Some(root_subnet),
                time,
                limits()
            )
            .unwrap_err(),
            VerifyError::Signature
        );
        let mut stale = time;
        stale.certificate.reference_ns += 1;
        assert_eq!(
            authenticate_subnet_time_from_certificate(
                cert,
                &root,
                subnet,
                Some(root_subnet),
                stale,
                limits()
            )
            .unwrap_err(),
            VerifyError::TimeOutsideWindow
        );
    }
}
