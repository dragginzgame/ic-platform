//! Platform-only BLS verification and deterministic fixture-key qualification.
use ic_platform::bls::{
    InvalidPrivateKey, InvalidPublicKey, InvalidSignature, PrivateKey, PublicKey, Signature,
    verify_bls_signature,
};

/// Preserve upstream key/signature serialization, verification and typed failures.
pub fn roundtrip() {
    let mut bytes = [0; 32];
    bytes[31] = 1;
    let key = PrivateKey::deserialize(&bytes).unwrap();
    assert_eq!(key.serialize(), bytes);
    assert_eq!(format!("{key:?}"), "PrivateKey(REDACTED)");
    let public = key.public_key();
    let signature = key.sign(b"platform-signature");
    let expected_public = hex(include_str!("fixtures/bls-public.hex"));
    let expected_signature = hex(include_str!("fixtures/bls-signature.hex"));
    assert_eq!(public.serialize().as_slice(), expected_public);
    assert_eq!(signature.serialize().as_slice(), expected_signature);
    assert_eq!(PublicKey::deserialize(&expected_public).unwrap(), public);
    assert_eq!(
        Signature::deserialize(&expected_signature).unwrap(),
        signature
    );
    assert_eq!(public.verify(b"platform-signature", &signature), Ok(()));
    assert_eq!(
        verify_bls_signature(&expected_signature, b"platform-signature", &expected_public),
        Ok(())
    );
    assert_eq!(public.verify(b"wrong message", &signature), Err(()));
    bytes[31] = 2;
    assert_eq!(
        key.public_key().verify(
            b"platform-signature",
            &PrivateKey::deserialize(&bytes)
                .unwrap()
                .sign(b"platform-signature")
        ),
        Err(())
    );
    assert_eq!(
        verify_bls_signature(&[], b"platform-signature", &expected_public),
        Err(())
    );
    assert_eq!(
        PublicKey::deserialize(&[]),
        Err(InvalidPublicKey::WrongLength)
    );
    assert_eq!(
        PublicKey::deserialize(&[0; 96]),
        Err(InvalidPublicKey::InvalidPoint)
    );
    assert_eq!(
        Signature::deserialize(&[]),
        Err(InvalidSignature::WrongLength)
    );
    assert_eq!(
        Signature::deserialize(&[0; 48]),
        Err(InvalidSignature::InvalidPoint)
    );
    assert_eq!(
        PrivateKey::deserialize(&[]),
        Err(InvalidPrivateKey::WrongLength)
    );
    assert_eq!(
        PrivateKey::deserialize(&[255; 32]),
        Err(InvalidPrivateKey::OutOfRange)
    );
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
fn bls_keys_signatures_and_errors_keep_upstream_identity() {
    roundtrip();
}
