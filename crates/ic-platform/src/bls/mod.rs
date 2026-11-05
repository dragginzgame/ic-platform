//! Upstream BLS12-381 cryptography with identity-public-key admission.
//!
//! Arithmetic, hashing, key encoding, signing and verification belong to the
//! registry dependencies. The adapters retain identity-key rejection while
//! ic-verify-bls-signature 0.6.0 lacks the upstream fix for Platform #127.
//! No RNG capability is selected; consumers retain entropy and trust policy.

use std::fmt;

pub use ic_verify_bls_signature::{
    InvalidPrivateKey, InvalidPublicKey, InvalidSignature, Signature,
};

/// An upstream BLS public key with cached identity-point admission.
#[derive(Clone, Eq, PartialEq)]
pub struct PublicKey {
    key: ic_verify_bls_signature::PublicKey,
    identity: bool,
}

impl PublicKey {
    /// Length of the upstream compressed public-key encoding.
    pub const BYTES: usize = ic_verify_bls_signature::PublicKey::BYTES;

    fn from_upstream(key: ic_verify_bls_signature::PublicKey) -> Self {
        // Both serialization and point admission are upstream operations. This
        // second parse is confined to construction, never repeated by verify.
        let point = ic_bls12_381::G2Affine::from_compressed(&key.serialize()).unwrap();
        Self {
            key,
            identity: point.is_identity().into(),
        }
    }

    /// Serialize using the upstream key encoding.
    pub fn serialize(&self) -> [u8; Self::BYTES] {
        self.key.serialize()
    }

    /// Parse upstream key bytes, preserving typed errors and identity parsing.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, InvalidPublicKey> {
        ic_verify_bls_signature::PublicKey::deserialize(bytes).map(Self::from_upstream)
    }

    /// Reject identity public keys, then delegate verification upstream.
    #[expect(
        clippy::result_unit_err,
        reason = "Preserve the existing BLS verification contract."
    )]
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), ()> {
        if self.identity {
            return Err(());
        }
        self.key.verify(message, signature)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.key.fmt(f)
    }
}

/// An upstream BLS private key whose public keys retain identity admission.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PrivateKey(ic_verify_bls_signature::PrivateKey);

impl PrivateKey {
    /// Length of the upstream private-key encoding.
    pub const BYTES: usize = ic_verify_bls_signature::PrivateKey::BYTES;

    /// Serialize using the upstream private-key encoding.
    pub fn serialize(&self) -> [u8; Self::BYTES] {
        self.0.serialize()
    }

    /// Parse upstream private-key bytes with the original typed errors.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, InvalidPrivateKey> {
        ic_verify_bls_signature::PrivateKey::deserialize(bytes).map(Self)
    }

    /// Delegate public-key derivation upstream and retain identity admission.
    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_upstream(self.0.public_key())
    }

    /// Delegate signing upstream.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.0.sign(message)
    }
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Verify upstream signature/key encodings with identity-public-key rejection.
#[expect(
    clippy::result_unit_err,
    reason = "Preserve the existing BLS verification contract."
)]
pub fn verify_bls_signature(signature: &[u8], message: &[u8], key: &[u8]) -> Result<(), ()> {
    let signature = Signature::deserialize(signature).map_err(|_| ())?;
    PublicKey::deserialize(key)
        .map_err(|_| ())?
        .verify(message, &signature)
}
