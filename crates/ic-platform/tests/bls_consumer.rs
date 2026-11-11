//! Platform-only BLS verification and deterministic fixture-key qualification.
mod support;
use ic_platform::bls::{
    InvalidPrivateKey, InvalidPublicKey, InvalidSignature, PrivateKey, PublicKey, Signature,
    verify_bls_signature,
};

/// Preserve upstream key/signature serialization, verification and typed failures.
pub fn roundtrip() {
    // Upstream #127 regression: valid identity point encodings remain parseable,
    // but an identity public key must never verify an identity signature.
    let mut identity_key_bytes = [0; 96];
    identity_key_bytes[0] = 0xc0;
    let mut identity_signature_bytes = [0; 48];
    identity_signature_bytes[0] = 0xc0;
    let identity_key = PublicKey::deserialize(&identity_key_bytes).unwrap();
    let identity_signature = Signature::deserialize(&identity_signature_bytes).unwrap();
    for message in [b"".as_slice(), b"identity-key regression"] {
        assert_eq!(identity_key.verify(message, &identity_signature), Err(()));
        assert_eq!(
            verify_bls_signature(&identity_signature_bytes, message, &identity_key_bytes),
            Err(())
        );
    }
    // Derived public keys use the same admission as deserialized public keys.
    let zero_key = PrivateKey::deserialize(&[0; 32]).unwrap();
    let zero_signature = zero_key.sign(b"derived identity-key regression");
    assert_eq!(
        zero_key
            .public_key()
            .verify(b"derived identity-key regression", &zero_signature),
        Err(())
    );
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

use support::decode_hex as hex;

#[test]
fn bls_keys_signatures_and_errors_preserve_bytes_and_admission() {
    roundtrip();
}
