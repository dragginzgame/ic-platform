//! Platform-only IC signature verification against original-owner certificate bytes.
use ic_platform::signature_verification::verify_canister_sig;

/// Preserve accepted signed bytes and upstream failures without adding trust policy.
pub fn roundtrip() {
    let signature = hex(include_str!("fixtures/signature-valid.hex"));
    let public = hex(include_str!("fixtures/signature-public.hex"));
    let root = hex(include_str!("fixtures/bls-public.hex"));
    assert_eq!(
        verify_canister_sig(b"platform-signature", &signature, &public, &root),
        Ok(())
    );
    assert!(verify_canister_sig(b"changed", &signature, &public, &root).is_err());
    assert!(verify_canister_sig(b"platform-signature", &signature, &public, &[0; 96]).is_err());
    let error: String =
        verify_canister_sig(b"platform-signature", &[], &public, &root).unwrap_err();
    assert_eq!(error, "signature CBOR doesn't have a self-describing tag");
    assert!(verify_canister_sig(b"platform-signature", &signature[3..], &public, &root).is_err());
    assert!(
        verify_canister_sig(
            b"platform-signature",
            &signature[..signature.len() - 1],
            &public,
            &root
        )
        .is_err()
    );
    // Full-length malformed DER is safe; upstream short-input admission stays with callers.
    let mut wrong_key = public.clone();
    wrong_key[2] ^= 1;
    assert!(verify_canister_sig(b"platform-signature", &signature, &wrong_key, &root).is_err());
    let mut bad_signature = signature;
    let last = bad_signature.len() - 1;
    bad_signature[last] ^= 1;
    assert!(verify_canister_sig(b"platform-signature", &bad_signature, &public, &root).is_err());
}

fn hex(input: &str) -> Vec<u8> {
    let (pairs, remainder) = input.trim().as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn signatures_keep_original_acceptance_and_refusal_contracts() {
    roundtrip();
}
