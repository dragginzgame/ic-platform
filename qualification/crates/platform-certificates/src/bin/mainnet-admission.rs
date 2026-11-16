//! Replay an independently observed mainnet certificate with fixed historical policy.
#![forbid(unsafe_code)]

use candidate_platform::certificate::{
    CertificateDecodeLimits, PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits,
    VerifyError, authenticate_certificate, authenticate_direct_certificate,
    authenticate_subnet_time, authenticate_subnet_time_from_certificate, decode_certificate,
};
use candidate_types::Principal;
use ic_certification::LookupResult;

fn main() {
    direct_root();
    let bytes = std::hint::black_box(include_bytes!("../../fixtures/mainnet.cbor").as_slice());
    let root =
        PreparedRootKey::from_der(include_bytes!("../../fixtures/mainnet-root.der")).unwrap();
    let wrong_root = PreparedRootKey::from_der(include_bytes!(
        "../../../../../crates/ic-platform/tests/fixtures/root.der"
    ))
    .unwrap();
    let target = Principal::from_text("a2gzu-ziaaa-aaaam-ae66a-cai").unwrap();
    let subnet = Principal::from_slice(&[
        16, 182, 71, 51, 74, 84, 6, 152, 119, 150, 178, 248, 182, 177, 76, 211, 47, 8, 118, 211,
        253, 79, 200, 69, 33, 5, 131, 37, 2,
    ]);
    let signing_ns = 1_794_829_035_349_777_381;
    let delegation_ns = 1_794_828_784_203_552_976;
    let exact = |reference_ns| TimeWindow {
        reference_ns,
        max_age_ns: 0,
        max_future_skew_ns: 0,
    };
    // Frozen historical expectations, never inferred from the input being verified.
    let time = TimePolicy {
        certificate: exact(signing_ns),
        delegation: exact(delegation_ns),
    };
    let limits = VerificationLimits {
        tree_nodes: 4096,
        tree_depth: 64,
        delegation_bytes: 131072,
        canister_ranges: 256,
    };
    let decode =
        || decode_certificate(bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
    let evidence = authenticate_certificate(decode(), &root, target, time, limits).unwrap();
    assert_eq!(evidence.time_ns(), signing_ns);
    assert_eq!(
        evidence.delegation(),
        Some((subnet, time.delegation, delegation_ns))
    );
    assert_eq!(evidence.original_bytes().as_ptr(), bytes.as_ptr());
    assert!(matches!(evidence.tree().lookup_path([
        b"canister".as_slice(), target.as_slice(), b"certified_data"
    ]), LookupResult::Found(value) if value.len() == 32));
    assert_eq!(
        authenticate_certificate(decode(), &wrong_root, target, time, limits).unwrap_err(),
        VerifyError::Signature
    );
    assert_eq!(
        authenticate_certificate(
            decode(),
            &root,
            Principal::management_canister(),
            time,
            limits
        )
        .unwrap_err(),
        VerifyError::NotAuthorized
    );
    let subnet_evidence =
        authenticate_subnet_time(decode(), &root, subnet, None, time, limits).unwrap();
    assert_eq!(subnet_evidence.time_ns(), signing_ns);
    assert_eq!(
        subnet_evidence.delegation(),
        Some((time.delegation, delegation_ns))
    );
    assert_eq!(
        subnet_evidence.original_bytes().unwrap().as_ptr(),
        bytes.as_ptr()
    );
    let decoded = decode();
    let certificate = decoded.unverified();
    let verify = |cert, selected, policy| {
        authenticate_subnet_time_from_certificate(cert, &root, selected, None, policy, limits)
    };
    let parsed = verify(certificate, subnet, time).unwrap();
    assert_eq!(parsed.time_ns(), signing_ns);
    assert!(std::ptr::eq(parsed.certificate().unwrap(), certificate));
    assert!(parsed.original_bytes().is_none());
    assert_eq!(
        verify(certificate, Principal::anonymous(), time).unwrap_err(),
        VerifyError::SubnetMismatch
    );
    let mut wrong_signing_time = time;
    wrong_signing_time.certificate = exact(delegation_ns);
    assert_eq!(
        verify(certificate, subnet, wrong_signing_time).unwrap_err(),
        VerifyError::TimeOutsideWindow
    );
    let mut wrong_delegation_time = time;
    wrong_delegation_time.delegation = exact(signing_ns);
    assert_eq!(
        verify(certificate, subnet, wrong_delegation_time).unwrap_err(),
        VerifyError::TimeOutsideWindow
    );
    let mut altered = certificate.clone();
    altered.signature[0] ^= 1;
    assert_eq!(
        verify(&altered, subnet, time).unwrap_err(),
        VerifyError::Signature
    );
}

fn direct_root() {
    let bytes =
        std::hint::black_box(include_bytes!("../../fixtures/mainnet-direct.cbor").as_slice());
    let root =
        PreparedRootKey::from_der(include_bytes!("../../fixtures/mainnet-root.der")).unwrap();
    let target = Principal::from_text("qoctq-giaaa-aaaaa-aaaea-cai").unwrap();
    let window = TimeWindow {
        reference_ns: 1_794_829_929_737_402_286,
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
        delegation_bytes: 0,
        canister_ranges: 0,
    };
    let decode =
        || decode_certificate(bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
    let evidence = authenticate_certificate(decode(), &root, target, time, limits).unwrap();
    assert!(evidence.delegation().is_none());
    assert_eq!(evidence.time_ns(), window.reference_ns);
    assert_eq!(evidence.original_bytes().as_ptr(), bytes.as_ptr());
    assert!(matches!(evidence.tree().lookup_path([
        b"canister".as_slice(), target.as_slice(), b"certified_data"
    ]), LookupResult::Found(value) if value.len() == 32));
    assert!(authenticate_direct_certificate(decode(), &root, target, window, limits).is_ok());
    // Network identity is selected from the system-canister specification, not the response.
    let nns =
        Principal::from_text("tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe")
            .unwrap();
    let subnet = authenticate_subnet_time(decode(), &root, nns, Some(nns), time, limits).unwrap();
    assert_eq!(subnet.subnet(), nns);
    assert_eq!(subnet.time_ns(), window.reference_ns);
    assert!(subnet.delegation().is_none());
    assert_eq!(
        authenticate_subnet_time(decode(), &root, nns, None, time, limits).unwrap_err(),
        VerifyError::RootSubnetRequired
    );
    assert_eq!(
        authenticate_subnet_time(
            decode(),
            &root,
            Principal::anonymous(),
            Some(nns),
            time,
            limits
        )
        .unwrap_err(),
        VerifyError::SubnetMismatch
    );
    let decoded = decode();
    let certificate = decoded.unverified();
    let verify = |cert, selected, binding, policy| {
        authenticate_subnet_time_from_certificate(cert, &root, selected, binding, policy, limits)
    };
    let parsed = verify(certificate, nns, Some(nns), time).unwrap();
    assert_eq!(parsed.time_ns(), window.reference_ns);
    assert!(std::ptr::eq(parsed.certificate().unwrap(), certificate));
    assert!(parsed.original_bytes().is_none());
    assert_eq!(
        verify(certificate, nns, None, time).unwrap_err(),
        VerifyError::RootSubnetRequired
    );
    assert_eq!(
        verify(certificate, Principal::anonymous(), Some(nns), time).unwrap_err(),
        VerifyError::SubnetMismatch
    );
    let mut altered = certificate.clone();
    altered.signature[0] ^= 1;
    assert_eq!(
        verify(&altered, nns, Some(nns), time).unwrap_err(),
        VerifyError::Signature
    );
    let mut stale = time;
    stale.certificate.reference_ns += 1;
    assert_eq!(
        verify(certificate, nns, Some(nns), stale).unwrap_err(),
        VerifyError::TimeOutsideWindow
    );
}
