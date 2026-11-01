//! Standalone consumer exercising Auth's strict compact-signature policy.
use ic_platform::ecdsa::{PrehashVerifier, Signature, SignatureError, VerifyingKey};

/// Preserve consumer-owned high-S refusal before prehash verification.
pub fn verify(key: &[u8], prehash: &[u8; 32], bytes: &[u8]) -> Result<(), SignatureError> {
    let key = VerifyingKey::from_sec1_bytes(key)?;
    let signature = Signature::from_slice(bytes)?;
    if signature.normalize_s() != signature {
        return Err(SignatureError::new());
    }
    key.verify_prehash(prehash, &signature)
}

#[test]
fn compact_signature_policy() {
    use ic_platform::ecdsa::{PrehashSigner, SigningKey};
    let key = SigningKey::from_slice(&[7; 32]).unwrap();
    let prehash = [42; 32];
    let signature: Signature = key.sign_prehash(&prehash).unwrap();
    let public = key.verifying_key().to_sec1_point(true);
    // Frozen outputs from K256 0.13.4: preserve prehash/signature and SEC1 bytes
    // across the upstream type upgrade, independently of a new/new roundtrip.
    assert_eq!(
        public.as_bytes(),
        &[
            0x02, 0x98, 0x9c, 0x0b, 0x76, 0xcb, 0x56, 0x39, 0x71, 0xfd, 0xc9, 0xbe, 0xf3, 0x1e,
            0xc0, 0x6c, 0x35, 0x60, 0xf3, 0x24, 0x9d, 0x6e, 0xe9, 0xe5, 0xd8, 0x3c, 0x57, 0x62,
            0x55, 0x96, 0xe0, 0x5f, 0x6f
        ]
    );
    assert_eq!(
        &signature.to_bytes()[..],
        &[
            0x58, 0x9f, 0xc8, 0x28, 0xe3, 0x93, 0x6c, 0x0a, 0x45, 0x26, 0x12, 0x96, 0xf0, 0x85,
            0x13, 0x39, 0xac, 0xf2, 0x4c, 0xc1, 0x03, 0x40, 0xb9, 0xf4, 0x9d, 0xa0, 0x96, 0x5a,
            0x8c, 0x04, 0x9d, 0xa6, 0x6a, 0x94, 0xed, 0x30, 0x66, 0x8c, 0x17, 0xc5, 0x1b, 0x4c,
            0xe4, 0xa6, 0x43, 0xa2, 0x06, 0x83, 0x04, 0xf1, 0xb8, 0x60, 0x7e, 0x0f, 0x8a, 0x36,
            0x89, 0x06, 0xa1, 0xa6, 0x09, 0xcc, 0x01, 0x11
        ]
    );
    verify(public.as_bytes(), &prehash, &signature.to_bytes()).unwrap();
    verify(
        key.verifying_key().to_sec1_point(false).as_bytes(),
        &prehash,
        &signature.to_bytes(),
    )
    .unwrap();
    assert!(verify(public.as_bytes(), &[43; 32], &signature.to_bytes()).is_err());
    assert!(verify(&[0; 33], &prehash, &signature.to_bytes()).is_err());
    assert!(verify(public.as_bytes(), &prehash, &[0; 64]).is_err());
    assert!(verify(public.as_bytes(), &prehash, &signature.to_bytes()[..63]).is_err());
    let (r, s) = signature.split_scalars();
    let high = Signature::from_scalars(r.to_bytes(), (-s).to_bytes()).unwrap();
    assert!(verify(public.as_bytes(), &prehash, &high.to_bytes()).is_err());
    assert_eq!(high.normalize_s().to_bytes(), signature.to_bytes());
    assert_eq!(signature.normalize_s(), signature);
}
