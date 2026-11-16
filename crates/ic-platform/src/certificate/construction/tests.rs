use super::*;
use crate::certificate::{CertificateDecodeLimits, decode_canister_signature, decode_certificate};
#[test]
fn tagged_construction_preserves_opaque_bytes_and_exact_limit() {
    let tree = labeled(b"time".to_vec(), leaf(vec![100]));
    let cert = Certificate {
        tree: tree.clone(),
        signature: vec![1; 48],
        delegation: None,
    };
    let bytes = encode_certificate(&cert, 4096).unwrap();
    assert_eq!(encode_certificate(&cert, bytes.len()).unwrap(), bytes);
    assert!(matches!(
        encode_certificate(&cert, bytes.len() - 1),
        Err(ConstructionError::OutputLimit)
    ));
    let decoded =
        decode_certificate(&bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
    assert_eq!(decoded.unverified().tree.digest(), tree.digest());
    assert_eq!(decoded.unverified().signature, vec![1; 48]);
    let signature = encode_canister_signature(&bytes, &tree, 4096).unwrap();
    let decoded = decode_canister_signature(
        &signature,
        CertificateDecodeLimits {
            bytes: signature.len(),
        },
    )
    .unwrap();
    assert_eq!(decoded.certificate_bytes(), bytes);
    assert_eq!(decoded.unverified_witness().digest(), tree.digest());
    assert!(matches!(
        encode_canister_signature(&bytes, &tree, signature.len() - 1),
        Err(ConstructionError::OutputLimit)
    ));
}
#[cfg(feature = "canister-signature-authentication")]
#[test]
fn key_framing_matches_literal_and_supported_boundaries() {
    use crate::certificate::CanisterSignatureKey;
    let key = canister_key_der(candidate_types::Principal::from_slice(&[1]), b"s").unwrap();
    assert_eq!(
        key,
        b"\x30\x14\x30\x0c\x06\x0a\x2b\x06\x01\x04\x01\x83\xb8\x43\x01\x02\x03\x04\x00\x01\x01\x73"
    );
    for n in [0, 1, 29] {
        let issuer = candidate_types::Principal::from_slice(&vec![1; n]);
        let seed = vec![9; 109 - n];
        let der = canister_key_der(issuer, &seed).unwrap();
        let key = CanisterSignatureKey::from_der(&der).unwrap();
        assert_eq!(key.issuer(), issuer);
        assert_eq!(key.seed(), seed);
        assert!(canister_key_der(issuer, &vec![9; 110 - n]).is_err());
    }
}
