//! Unchanged ic_principal 0.1.5 derivation; see ../NOTICE.md.
use super::Principal;
use sha2::{Digest, Sha224};
use sha2_11 as sha2;

impl Principal {
    const SELF_AUTHENTICATING_TAG: u8 = 2;

    /// Construct a self-authenticating ID from public key.
    pub fn self_authenticating<P: AsRef<[u8]>>(public_key: P) -> Self {
        let public_key = public_key.as_ref();
        let hash = Sha224::digest(public_key);
        let mut bytes = [0; Self::MAX_LENGTH_IN_BYTES];
        bytes[..Self::MAX_LENGTH_IN_BYTES - 1].copy_from_slice(hash.as_slice());
        bytes[Self::MAX_LENGTH_IN_BYTES - 1] = Self::SELF_AUTHENTICATING_TAG;

        Self {
            len: Self::MAX_LENGTH_IN_BYTES as u8,
            bytes,
        }
    }
}
