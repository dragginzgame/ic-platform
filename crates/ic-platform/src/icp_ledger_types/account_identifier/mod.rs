//! Selected ic-ledger-types 0.16.0 derivation; see NOTICE.md.
use candid::Principal;
use sha2::Digest;

struct Subaccount([u8; 32]);
struct AccountIdentifier([u8; 32]);

impl AccountIdentifier {
    pub fn new(owner: &Principal, subaccount: &Subaccount) -> Self {
        let mut hasher = sha2::Sha224::new();
        hasher.update(b"\x0Aaccount-id");
        hasher.update(owner.as_slice());
        hasher.update(&subaccount.0[..]);
        let hash: [u8; 28] = hasher.finalize().into();

        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&hash);
        let crc32_bytes = hasher.finalize().to_be_bytes();

        let mut result = [0u8; 32];
        result[0..4].copy_from_slice(&crc32_bytes[..]);
        result[4..32].copy_from_slice(hash.as_ref());
        Self(result)
    }
}

/// Derive the standard ICP ledger account identifier for an owner/subaccount.
///
/// Returns the unchanged upstream 32-byte identifier, including its CRC checksum.
/// This is distinct from ICRC Account text and performs no transfer, allocation,
/// account authorization or principal-to-subaccount policy. The subaccount must
/// contain exactly 32 bytes, enforced by its type. Available on native and Wasm
/// through the existing `icp-ledger-types` feature.
pub fn account_identifier(owner: crate::Principal, subaccount: &[u8; 32]) -> [u8; 32] {
    AccountIdentifier::new(&Principal::from(owner), &Subaccount(*subaccount)).0
}

#[cfg(test)]
mod tests;
