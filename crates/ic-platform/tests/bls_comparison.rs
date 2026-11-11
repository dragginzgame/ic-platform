//! Independent public-boundary comparisons; no supplier implementation or vector suite is copied.
use ic_platform::bls::{
    InvalidPrivateKey, InvalidPublicKey, InvalidSignature, PrivateKey, PublicKey, Signature,
    verify_bls_signature,
};

fn secret(value: u8) -> [u8; 32] {
    let mut encoded = [0; 32];
    encoded[31] = value;
    encoded
}

#[test]
fn bytes_and_verification_match_independent_supplier_calls() {
    for number in [1, 7, 253] {
        let bytes = secret(number);
        let candidate = PrivateKey::deserialize(&bytes).unwrap();
        let oracle = ic_verify_bls_signature::PrivateKey::deserialize(&bytes).unwrap();
        assert_eq!(candidate.serialize(), bytes);
        assert_eq!(
            candidate.public_key().serialize(),
            oracle.public_key().serialize()
        );
        let public = PublicKey::deserialize(&oracle.public_key().serialize()).unwrap();
        let prepared = public.prepare();
        for length in [0, 1, 31, 55, 56, 63, 64, 65, 127, 128, 129, 1024] {
            let message: Vec<u8> = (0..length).map(|i| (i as u8).wrapping_mul(37)).collect();
            let reference = oracle.sign(&message);
            let signature = candidate.sign(&message);
            assert_eq!(
                signature.serialize(),
                reference.serialize(),
                "key {number}, length {length}"
            );
            let decoded = Signature::deserialize(&reference.serialize()).unwrap();
            assert_eq!(public.verify(&message, &decoded), Ok(()));
            assert_eq!(prepared.verify(&message, &decoded), Ok(()));
            assert_eq!(
                oracle.public_key().verify(
                    &message,
                    &ic_verify_bls_signature::Signature::deserialize(&signature.serialize())
                        .unwrap()
                ),
                Ok(())
            );
            assert_eq!(
                verify_bls_signature(&reference.serialize(), &message, &public.serialize()),
                Ok(())
            );
            let mut wrong_message = message.clone();
            wrong_message.push(1);
            assert!(public.verify(&wrong_message, &decoded).is_err());
            assert!(prepared.verify(&wrong_message, &decoded).is_err());
        }
    }
}

#[test]
fn malformed_inputs_and_cross_key_signatures_fail_closed() {
    for length in [0, 1, 47, 49, 95, 97] {
        let bytes = vec![0; length];
        assert_eq!(
            PublicKey::deserialize(&bytes),
            Err(InvalidPublicKey::WrongLength)
        );
        assert_eq!(
            Signature::deserialize(&bytes),
            Err(InvalidSignature::WrongLength)
        );
        assert!(verify_bls_signature(&bytes, b"message", &bytes).is_err());
    }
    for byte in [0, 255] {
        assert_eq!(
            PublicKey::deserialize(&[byte; 96]),
            Err(InvalidPublicKey::InvalidPoint)
        );
        assert_eq!(
            Signature::deserialize(&[byte; 48]),
            Err(InvalidSignature::InvalidPoint)
        );
    }
    for length in [0, 31, 33] {
        assert_eq!(
            PrivateKey::deserialize(&vec![0; length]),
            Err(InvalidPrivateKey::WrongLength)
        );
    }
    assert_eq!(
        PrivateKey::deserialize(&[255; 32]),
        Err(InvalidPrivateKey::OutOfRange)
    );
    let first = PrivateKey::deserialize(&secret(1)).unwrap();
    let second = PrivateKey::deserialize(&secret(2)).unwrap();
    let signature = first.sign(b"message");
    assert!(second.public_key().verify(b"message", &signature).is_err());
    assert!(
        second
            .public_key()
            .prepare()
            .verify(b"message", &signature)
            .is_err()
    );
    let changed = first.sign(b"different");
    assert!(first.public_key().verify(b"message", &changed).is_err());
}

#[test]
fn identity_keys_remain_parseable_but_never_authenticate() {
    let zero = PrivateKey::deserialize(&[0; 32]).unwrap();
    assert_eq!(zero.serialize(), [0; 32]);
    let public = zero.public_key();
    let encoded = public.serialize();
    let parsed = PublicKey::deserialize(&encoded).unwrap();
    assert_eq!(parsed, public);
    let signature = zero.sign(b"identity-key-refusal");
    assert_eq!(
        Signature::deserialize(&signature.serialize()).unwrap(),
        signature
    );
    assert!(parsed.verify(b"identity-key-refusal", &signature).is_err());
    assert!(
        parsed
            .prepare()
            .verify(b"identity-key-refusal", &signature)
            .is_err()
    );
    assert!(
        verify_bls_signature(&signature.serialize(), b"identity-key-refusal", &encoded).is_err()
    );
    assert!(!format!("{zero:?}").contains(&"00".repeat(32)));
}

#[test]
fn checked_decoding_rejects_supplier_identified_non_subgroup_points() {
    // Ask the external codec to find curve points outside the prime-order
    // subgroup. This tests our admission boundary, not the curve algorithms.
    let mut g1 = None;
    let mut g2 = None;
    for number in 1_u64..128 {
        let mut encoded = [0; 48];
        encoded[0] = 0x80;
        encoded[40..].copy_from_slice(&number.to_be_bytes());
        if let Some(point) = Option::<ic_bls12_381::G1Affine>::from(
            ic_bls12_381::G1Affine::from_compressed_unchecked(&encoded),
        ) && !bool::from(point.is_torsion_free())
        {
            g1 = Some(encoded);
        }
        let mut encoded = [0; 96];
        encoded[0] = 0x80;
        encoded[88..].copy_from_slice(&number.to_be_bytes());
        if let Some(point) = Option::<ic_bls12_381::G2Affine>::from(
            ic_bls12_381::G2Affine::from_compressed_unchecked(&encoded),
        ) && !bool::from(point.is_torsion_free())
        {
            g2 = Some(encoded);
        }
        if g1.is_some() && g2.is_some() {
            break;
        }
    }
    assert_eq!(
        Signature::deserialize(&g1.expect("supplier should decode a non-subgroup G1 point")),
        Err(InvalidSignature::InvalidPoint)
    );
    assert_eq!(
        PublicKey::deserialize(&g2.expect("supplier should decode a non-subgroup G2 point")),
        Err(InvalidPublicKey::InvalidPoint)
    );
}
