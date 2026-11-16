use candidate_platform::{ecdsa, ed25519};
use ecdsa::PrehashVerifier;

fn bytes(records: &str, name: &str) -> Vec<u8> {
    let hex = records
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once('\t')?;
            (key == name).then_some(value)
        })
        .unwrap();
    let (pairs, remainder) = hex.as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

pub fn run() {
    let vectors = include_str!("ed25519.tsv");
    let key = ed25519::VerifyingKey::from_bytes(&bytes(vectors, "public-key").try_into().unwrap())
        .unwrap();
    assert!(!key.is_weak());
    let mut digest = bytes(vectors, "digest");
    let signature = ed25519::Signature::from_slice(&bytes(vectors, "signature")).unwrap();
    assert!(bool::from(
        ed25519::Scalar::from_canonical_bytes(*signature.s_bytes()).is_some()
    ));
    key.verify_strict(&digest, &signature).unwrap();
    digest[0] ^= 1;
    assert!(key.verify_strict(&digest, &signature).is_err());
    let invalid =
        ed25519::Signature::from_slice(&bytes(vectors, "noncanonical-signature")).unwrap();
    assert!(bool::from(
        ed25519::Scalar::from_canonical_bytes(*invalid.s_bytes()).is_none()
    ));
    let mut identity = [0; 32];
    identity[0] = 1;
    assert!(
        ed25519::VerifyingKey::from_bytes(&identity)
            .unwrap()
            .is_weak()
    );
    let mut small_order_r = signature.to_bytes();
    small_order_r[..32].copy_from_slice(&identity);
    assert!(
        key.verify_strict(
            &bytes(vectors, "digest"),
            &ed25519::Signature::from_bytes(&small_order_r)
        )
        .is_err()
    );

    let vectors = include_str!("root.tsv");
    let mut digest = bytes(vectors, "header_hash");
    let signature = ecdsa::Signature::from_slice(&bytes(vectors, "signature")).unwrap();
    assert_eq!(signature.normalize_s(), signature);
    let high = ecdsa::Signature::from_slice(&bytes(vectors, "high_signature")).unwrap();
    assert_ne!(high.normalize_s(), high);
    for encoding in ["public_key", "public_key_uncompressed"] {
        let key = ecdsa::VerifyingKey::from_sec1_bytes(&bytes(vectors, encoding)).unwrap();
        key.verify_prehash(&digest, &signature).unwrap();
        digest[0] ^= 1;
        assert!(key.verify_prehash(&digest, &signature).is_err());
        digest[0] ^= 1;
    }
    assert!(ecdsa::VerifyingKey::from_sec1_bytes(&[0; 33]).is_err());
    assert!(ecdsa::Signature::from_slice(&[0; 64]).is_err());
}
