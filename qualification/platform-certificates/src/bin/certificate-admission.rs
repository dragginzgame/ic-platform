//! Execute replica certificate admission on native and 32-bit Wasm targets.
#![forbid(unsafe_code)]

use candidate_platform::certificate::{
    CertificateDecodeLimits, PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits,
    VerifyError, authenticate_certificate, authenticate_subnet_time_from_certificate,
    decode_certificate,
};
use candidate_types::Principal;
use ic_certification::LookupResult;

fn main() {
    let root = PreparedRootKey::from_der(include_bytes!(
        "../../../../crates/ic-platform/tests/fixtures/root.der"
    ))
    .unwrap();
    let root_subnet =
        Principal::from_text("hzklm-c4flk-lfdw4-v4532-s744u-jbwhv-6zh6w-be4il-apmtb-fosk7-wae")
            .unwrap();
    let app_subnet =
        Principal::from_text("sgier-p6fyz-ejpwm-jghgc-4ypgv-6ft53-5hns6-ojhae-wvlic-ifcah-eqe")
            .unwrap();
    let limits = VerificationLimits {
        tree_nodes: 4096,
        tree_depth: 64,
        delegation_bytes: 131072,
        canister_ranges: 256,
    };
    for (bytes, target, subnet, timestamp) in [
        (
            include_bytes!("../../../../crates/ic-platform/tests/fixtures/direct.cbor").as_slice(),
            "xp3jw-ot777-77777-aaaaa-cai",
            root_subnet,
            1_620_328_630_000_000_005,
        ),
        (
            include_bytes!("../../../../crates/ic-platform/tests/fixtures/delegated.cbor")
                .as_slice(),
            "dllsh-pd777-77776-qaaaa-cai",
            app_subnet,
            1_620_328_630_000_000_008,
        ),
    ] {
        let bytes = std::hint::black_box(bytes);
        let target = Principal::from_text(target).unwrap();
        let window = TimeWindow {
            reference_ns: timestamp,
            max_age_ns: 0,
            max_future_skew_ns: 0,
        };
        let time = TimePolicy {
            certificate: window,
            delegation: window,
        };
        let decode =
            || decode_certificate(bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
        let evidence = authenticate_certificate(decode(), &root, target, time, limits).unwrap();
        assert_eq!(evidence.time_ns(), timestamp);
        assert_eq!(evidence.original_bytes().as_ptr(), bytes.as_ptr());
        assert_eq!(
            evidence.tree().lookup_path([
                b"canister".as_slice(),
                target.as_slice(),
                b"certified_data"
            ]),
            LookupResult::Found(&[0; 32])
        );
        let decoded = decode();
        let certificate = decoded.unverified();
        let verify = |cert, selected, policy, bounds| {
            authenticate_subnet_time_from_certificate(
                cert,
                &root,
                selected,
                Some(root_subnet),
                policy,
                bounds,
            )
        };
        let subnet_evidence = verify(certificate, subnet, time, limits).unwrap();
        assert_eq!(subnet_evidence.time_ns(), timestamp);
        assert!(std::ptr::eq(
            subnet_evidence.certificate().unwrap(),
            certificate
        ));
        assert!(subnet_evidence.original_bytes().is_none());
        let wrong = if subnet == root_subnet {
            app_subnet
        } else {
            root_subnet
        };
        assert_eq!(
            verify(certificate, wrong, time, limits).unwrap_err(),
            VerifyError::SubnetMismatch
        );
        let mut stale = time;
        stale.certificate.reference_ns += 1;
        assert_eq!(
            verify(certificate, subnet, stale, limits).unwrap_err(),
            VerifyError::TimeOutsideWindow
        );
        let mut bounded = limits;
        bounded.tree_nodes = 0;
        assert_eq!(
            verify(certificate, subnet, time, bounded).unwrap_err(),
            VerifyError::TreeLimit
        );
        let mut corrupted = certificate.clone();
        corrupted.signature[0] ^= 1;
        assert_eq!(
            verify(&corrupted, subnet, time, limits).unwrap_err(),
            VerifyError::Signature
        );
        assert!(
            decode_certificate(
                bytes,
                CertificateDecodeLimits {
                    bytes: bytes.len() - 1
                }
            )
            .is_err()
        );
    }
}
