//! Owned ICRC account identity, independent of Candid and ledger clients.
//!
//! Account semantics and textual checksum derive from icrc-ledger-types 0.2.0;
//! see NOTICE.md. Serialization preserves absent versus explicit zero subaccounts,
//! although equality, ordering and hashing treat them as the same account.
//!
//! Convert to `ExternalAccount` before encoding external Candid DTOs:
//!

use crate::{Principal, PrincipalError};
use serde::{Deserialize, Serialize};
use std::{fmt, hash::Hash, str::FromStr};

/// The 32-byte ledger subaccount; no principal or Candid marker is embedded.
pub type Subaccount = [u8; 32];

/// Effective subaccount when the optional field is absent.
pub const DEFAULT_SUBACCOUNT: &Subaccount = &[0; 32];

/// An owned account for application protocols. External ledger DTOs remain separate.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Account {
    /// Account owner.
    pub owner: Principal,
    /// Explicit subaccount, or the effective all-zero default.
    pub subaccount: Option<Subaccount>,
}

impl Account {
    /// Maximum canonical account text length, including checksum and subaccount.
    pub const MAX_LENGTH_IN_TEXT: usize = Principal::MAX_LENGTH_IN_TEXT + 1 + 7 + 1 + 64;

    /// Return the explicit subaccount or the all-zero default.
    pub fn effective_subaccount(&self) -> &Subaccount {
        self.subaccount.as_ref().unwrap_or(DEFAULT_SUBACCOUNT)
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner && self.effective_subaccount() == other.effective_subaccount()
    }
}
impl Eq for Account {}
impl PartialOrd for Account {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Account {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.owner.cmp(&other.owner).then_with(|| {
            self.effective_subaccount()
                .cmp(other.effective_subaccount())
        })
    }
}
impl Hash for Account {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.owner.hash(state);
        self.effective_subaccount().hash(state);
    }
}
impl From<Principal> for Account {
    fn from(owner: Principal) -> Self {
        Self {
            owner,
            subaccount: None,
        }
    }
}

// Preserve the upstream CRC32(owner || subaccount) textual checksum algorithm.
fn checksum(owner: &[u8], subaccount: &Subaccount) -> [u8; 7] {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(owner);
    hasher.update(subaccount);
    let mut encoded = [0; 7];
    data_encoding::BASE32_NOPAD.encode_mut(&hasher.finalize().to_be_bytes(), &mut encoded);
    encoded.make_ascii_lowercase();
    encoded
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.owner, f)?;
        let subaccount = self.effective_subaccount();
        if subaccount == DEFAULT_SUBACCOUNT {
            return Ok(());
        }
        let checksum = checksum(self.owner.as_slice(), subaccount);
        let checksum = std::str::from_utf8(&checksum).map_err(|_| fmt::Error)?;
        f.write_str("-")?;
        f.write_str(checksum)?;
        f.write_str(".")?;
        let first = subaccount
            .iter()
            .position(|byte| *byte != 0)
            .ok_or(fmt::Error)?;
        // Encode once into a fixed buffer, omitting only the first leading nibble.
        // Repeated integer formatting here dominates non-default account Display.
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut hex = [0u8; 64];
        let mut written = 0;
        for (index, &byte) in subaccount[first..].iter().enumerate() {
            if index != 0 || byte >= 16 {
                hex[written] = HEX[(byte >> 4) as usize];
                written += 1;
            }
            hex[written] = HEX[(byte & 15) as usize];
            written += 1;
        }
        f.write_str(std::str::from_utf8(&hex[..written]).map_err(|_| fmt::Error)?)
    }
}

/// Failures admitting canonical account text. No input strings are retained.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AccountError {
    /// Input exceeds the account grammar's maximum length.
    #[error("account text is too long")]
    TextTooLong,
    /// The owner is not a valid principal.
    #[error("invalid principal: {0}")]
    InvalidPrincipal(#[source] PrincipalError),
    /// A non-default subaccount needs a seven-character checksum.
    #[error("missing account checksum")]
    MissingChecksum,
    /// The checksum differs from CRC32(owner || subaccount).
    #[error("invalid account checksum")]
    InvalidChecksum,
    /// Subaccount contains non-hex characters or exceeds 64 hex digits.
    #[error("subaccount must contain at most 64 hex digits")]
    InvalidSubaccount,
    /// Canonical subaccount text must omit leading zeroes.
    #[error("subaccount should not have leading zeroes")]
    LeadingZeroesInSubaccount,
    /// Default subaccounts must be represented by the owner alone.
    #[error("default subaccount should be omitted")]
    DefaultSubaccountShouldBeOmitted,
}
impl FromStr for Account {
    type Err = AccountError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.len() > Self::MAX_LENGTH_IN_TEXT {
            return Err(AccountError::TextTooLong);
        }
        let Some((prefix, hex)) = text.split_once('.') else {
            return text
                .parse::<Principal>()
                .map(Self::from)
                .map_err(AccountError::InvalidPrincipal);
        };
        let (owner, supplied_checksum) = prefix
            .rsplit_once('-')
            .filter(|(_, sum)| sum.len() == 7)
            .ok_or(AccountError::MissingChecksum)?;
        if hex.starts_with('0') {
            return Err(AccountError::LeadingZeroesInSubaccount);
        }
        let owner: Principal = owner
            .parse::<Principal>()
            .map_err(AccountError::InvalidPrincipal)?;
        if hex.len() > 64 {
            return Err(AccountError::InvalidSubaccount);
        }
        if hex.is_empty() {
            return Err(AccountError::DefaultSubaccountShouldBeOmitted);
        }
        let mut subaccount = [0; 32];
        for (index, byte) in hex.bytes().enumerate() {
            let nibble = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                b'A'..=b'F' => byte - b'A' + 10,
                _ => return Err(AccountError::InvalidSubaccount),
            };
            let offset = 64 - hex.len() + index;
            subaccount[offset / 2] |= nibble << (if offset % 2 == 0 { 4 } else { 0 });
        }
        if supplied_checksum.as_bytes() != checksum(owner.as_slice(), &subaccount) {
            return Err(AccountError::InvalidChecksum);
        }
        Ok(Self {
            owner,
            subaccount: Some(subaccount),
        })
    }
}

#[cfg(test)]
mod tests;
