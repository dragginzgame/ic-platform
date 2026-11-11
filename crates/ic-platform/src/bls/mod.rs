//! IC BLS byte admission and verification over external cryptographic primitives.
//!
//! This module owns byte admission and API composition, not cryptographic
//! algorithms, certificate policy, trust roots or application key caches.

use std::{fmt, sync::LazyLock};

use ic_bls12_381::{
    G1Affine, G1Projective, G2Affine, G2Prepared, Gt, Scalar,
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    multi_miller_loop,
};

mod sha256;

const CIPHERSUITE: &[u8] = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";
static NEGATIVE_GENERATOR: LazyLock<G2Prepared> =
    LazyLock::new(|| G2Prepared::from(-G2Affine::generator()));

/// A checked, subgroup-valid BLS public key.
#[derive(Clone, Eq, PartialEq)]
pub struct PublicKey(G2Affine);

/// Failure to admit a compressed public key.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InvalidPublicKey {
    /// The input is not 96 bytes.
    WrongLength,
    /// The input is not a valid compressed subgroup point.
    InvalidPoint,
}

impl PublicKey {
    /// Length of a compressed public key.
    pub const BYTES: usize = 96;

    /// Decode a compressed point, checking encoding, curve and subgroup validity.
    ///
    /// Identity points are admitted here, but rejected by verification.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, InvalidPublicKey> {
        let bytes = bytes
            .try_into()
            .map_err(|_| InvalidPublicKey::WrongLength)?;
        Option::from(G2Affine::from_compressed(bytes))
            .map(Self)
            .ok_or(InvalidPublicKey::InvalidPoint)
    }

    /// Encode this key in the supplier's canonical compressed representation.
    pub fn serialize(&self) -> [u8; Self::BYTES] {
        self.0.to_compressed()
    }

    /// Prepare immutable public pairing data for repeated verification.
    pub fn prepare(&self) -> PreparedPublicKey {
        PreparedPublicKey {
            point: G2Prepared::from(self.0),
            identity: bool::from(self.0.is_identity()),
        }
    }

    /// Verify the exact supplied message; mismatches and identity keys fail.
    #[expect(
        clippy::result_unit_err,
        reason = "The public BLS contract specifies opaque verification failure"
    )]
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), ()> {
        if bool::from(self.0.is_identity()) {
            return Err(());
        }
        self.prepare().verify(message, signature)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        encoded_debug(f, "PublicKey", &self.serialize())
    }
}

/// Caller-owned public-key pairing preparation; it carries no trust decision.
pub struct PreparedPublicKey {
    point: G2Prepared,
    identity: bool,
}

impl PreparedPublicKey {
    /// Verify with the same admission decisions as [`PublicKey::verify`].
    #[expect(
        clippy::result_unit_err,
        reason = "The public BLS contract specifies opaque verification failure"
    )]
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), ()> {
        if self.identity {
            return Err(());
        }
        let message_point = G1Affine::from(hash_message(message));
        // The supplier evaluates e(H(message), key) / e(signature, generator).
        let product = multi_miller_loop(&[
            (&message_point, &self.point),
            (&signature.0, &NEGATIVE_GENERATOR),
        ])
        .final_exponentiation();
        if product == Gt::identity() {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// A checked, subgroup-valid BLS signature point.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Signature(G1Affine);

/// Failure to admit a compressed signature.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InvalidSignature {
    /// The input is not 48 bytes.
    WrongLength,
    /// The input is not a valid compressed subgroup point.
    InvalidPoint,
}

impl Signature {
    /// Length of a compressed signature.
    pub const BYTES: usize = 48;

    /// Decode a compressed point, checking encoding, curve and subgroup validity.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, InvalidSignature> {
        let bytes = bytes
            .try_into()
            .map_err(|_| InvalidSignature::WrongLength)?;
        Option::from(G1Affine::from_compressed(bytes))
            .map(Self)
            .ok_or(InvalidSignature::InvalidPoint)
    }

    /// Encode this signature in the supplier's canonical compressed representation.
    pub fn serialize(&self) -> [u8; Self::BYTES] {
        self.0.to_compressed()
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        encoded_debug(f, "Signature", &self.serialize())
    }
}

/// A canonical scalar, including zero for byte compatibility.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PrivateKey(Scalar);

/// Failure to admit a private scalar.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InvalidPrivateKey {
    /// The input is not 32 bytes.
    WrongLength,
    /// The input is at least the scalar field order.
    OutOfRange,
}

impl PrivateKey {
    /// Length of a canonical big-endian private scalar.
    pub const BYTES: usize = 32;

    /// Decode a canonical big-endian scalar without reduction modulo its order.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, InvalidPrivateKey> {
        let mut bytes: [u8; Self::BYTES] = bytes
            .try_into()
            .map_err(|_| InvalidPrivateKey::WrongLength)?;
        bytes.reverse();
        Option::from(Scalar::from_bytes(&bytes))
            .map(Self)
            .ok_or(InvalidPrivateKey::OutOfRange)
    }

    /// Encode this scalar in canonical big-endian form.
    pub fn serialize(&self) -> [u8; Self::BYTES] {
        let mut bytes = self.0.to_bytes();
        bytes.reverse();
        bytes
    }

    /// Derive the public key using supplier scalar multiplication.
    ///
    /// Zero produces an identity key, which cannot verify signatures.
    pub fn public_key(&self) -> PublicKey {
        PublicKey(G2Affine::from(G2Affine::generator() * self.0))
    }

    /// Sign the exact supplied bytes using supplier hash-to-curve and multiplication.
    pub fn sign(&self, message: &[u8]) -> Signature {
        Signature(G1Affine::from(hash_message(message) * self.0))
    }
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PrivateKey(REDACTED)")
    }
}

/// Decode and verify a signature; malformed inputs, mismatches and identity keys fail.
#[expect(
    clippy::result_unit_err,
    reason = "The public BLS contract specifies opaque verification failure"
)]
pub fn verify_bls_signature(signature: &[u8], message: &[u8], public_key: &[u8]) -> Result<(), ()> {
    let signature = Signature::deserialize(signature).map_err(|_| ())?;
    let key = PublicKey::deserialize(public_key).map_err(|_| ())?;
    key.verify(message, &signature)
}

fn hash_message(message: &[u8]) -> G1Projective {
    <G1Projective as HashToCurve<ExpandMsgXmd<sha256::Sha256>>>::hash_to_curve(message, CIPHERSUITE)
}

fn encoded_debug(f: &mut fmt::Formatter<'_>, name: &str, bytes: &[u8]) -> fmt::Result {
    write!(f, "{name}(")?;
    for byte in bytes {
        write!(f, "{byte:02x}")?;
    }
    f.write_str(")")
}
