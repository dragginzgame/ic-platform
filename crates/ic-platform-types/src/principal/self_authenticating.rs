//! Protocol framing over the upstream SHA-224 implementation.

use super::Principal;
use sha2::{Digest, Sha224};

impl Principal {
    /// Derives an identity from SHA-224 of the supplied bytes followed by `0x02`.
    ///
    /// Callers supply the required public-key encoding. This does not parse or
    /// validate a key, prove possession, or grant authorization.
    #[must_use]
    pub fn self_authenticating<P: AsRef<[u8]>>(public_key: P) -> Self {
        let digest = Sha224::digest(public_key.as_ref());
        let mut bytes = [0; Self::MAX_LENGTH_IN_BYTES];
        bytes[..digest.len()].copy_from_slice(&digest);
        bytes[28] = 2;

        Self::from_slice(&bytes)
    }
}
