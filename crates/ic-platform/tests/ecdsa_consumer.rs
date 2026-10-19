//! Standalone consumer exercising Auth's strict compact-signature policy.
use ic_platform::ecdsa::{PrehashVerifier, Signature, SignatureError, VerifyingKey};

/// Preserve consumer-owned high-S refusal before prehash verification.
pub fn verify(key: &[u8], prehash: &[u8; 32], bytes: &[u8]) -> Result<(), SignatureError> {
    let key = VerifyingKey::from_sec1_bytes(key)?;
    let signature = Signature::from_slice(bytes)?;
    if signature.normalize_s().is_some() {
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
    let public = key.verifying_key().to_encoded_point(true);
    verify(public.as_bytes(), &prehash, &signature.to_bytes()).unwrap();
    verify(
        key.verifying_key().to_encoded_point(false).as_bytes(),
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
    assert_eq!(high.normalize_s().unwrap().to_bytes(), signature.to_bytes());
    assert_eq!(signature.normalize_s(), None);
}
