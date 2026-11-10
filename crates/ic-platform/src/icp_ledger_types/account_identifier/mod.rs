//! ICP ledger account framing using external SHA-224 and CRC32 implementations.

use sha2_11::{Digest, Sha224};

/// Derives the ICP account identifier for an owner and subaccount.
pub fn account_identifier(owner: crate::Principal, subaccount: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha224::new();
    hasher.update(b"\x0Aaccount-id");
    hasher.update(owner.as_slice());
    hasher.update(subaccount);
    let digest = hasher.finalize();

    let mut identifier = [0_u8; 32];
    identifier[..4].copy_from_slice(&crc32fast::hash(&digest).to_be_bytes());
    identifier[4..].copy_from_slice(&digest);
    identifier
}
