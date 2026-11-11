//! Self-authenticating Principal framing over upstream SHA-224.

use super::Principal;
use sha2_11::{Digest, Sha224};

impl Principal {
    /// Derive an identity as SHA-224 of the supplied bytes followed by `0x02`.
    ///
    /// The caller supplies the public key bytes. No key parsing, validation or
    /// authorization policy is applied.
    pub fn self_authenticating<P: AsRef<[u8]>>(public_key: P) -> Self {
        let mut bytes = [0; Self::MAX_LENGTH_IN_BYTES];
        bytes[..28].copy_from_slice(&Sha224::digest(public_key.as_ref()));
        bytes[28] = 0x02;
        Self::from_slice(&bytes)
    }
}

#[cfg(test)]
mod tests;
