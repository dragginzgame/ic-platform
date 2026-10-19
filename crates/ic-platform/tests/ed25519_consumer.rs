//! Standalone consumer: only Platform is declared in its manifest.
use ic_platform::ed25519::{Scalar, Signature, SignatureError, VerifyingKey};

/// Exercise the consumer's strict verification sequence without upstream imports.
pub fn verify(key: &[u8; 32], bytes: &[u8], message: &[u8]) -> Result<(), SignatureError> {
    let key = VerifyingKey::from_bytes(key)?;
    let signature = Signature::from_slice(bytes)?;
    if key.is_weak() || bool::from(Scalar::from_canonical_bytes(*signature.s_bytes()).is_none()) {
        return Err(SignatureError::new());
    }
    key.verify_strict(message, &signature)
}

#[test]
fn rfc8032_empty_message_and_rejections() {
    use ic_platform::ed25519::{Signer, SigningKey};
    // RFC 8032 section 7.1, test 1: independently specified seed, key and signature.
    fn hex<const N: usize>(s: &str) -> [u8; N] {
        std::array::from_fn(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap())
    }
    let seed = hex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60");
    let public = hex("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a");
    let expected: [u8; 64] = hex(concat!(
        "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555f",
        "b8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
    ));
    let key = SigningKey::from_bytes(&seed);
    assert_eq!(key.verifying_key().to_bytes(), public);
    assert_eq!(key.sign(b"").to_bytes(), expected);
    verify(&public, &expected, b"").unwrap();
    assert!(verify(&public, &expected, b"changed").is_err());
    assert!(verify(&public, &expected[..63], b"").is_err());
    let mut noncanonical = expected;
    noncanonical[32..].fill(0xff);
    assert!(verify(&public, &noncanonical, b"").is_err());
    let mut weak = [0; 32];
    weak[0] = 1;
    assert!(verify(&weak, &expected, b"").is_err());
}
