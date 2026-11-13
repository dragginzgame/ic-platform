//! Accounts combine a Principal owner with an optional 32-byte subaccount.
//!
//! Logical identity uses an effective zero subaccount when the field is absent;
//! serialization preserves whether that field was present.

use crate::principal::{Principal, PrincipalError};
use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

/// Opaque application-owned subaccount bytes.
pub type Subaccount = [u8; 32];

/// The effective subaccount when an Account's subaccount field is absent.
pub const DEFAULT_SUBACCOUNT: &Subaccount = &[0; 32];

/// A Principal owner and optional subaccount.
///
/// Absent and explicitly zero subaccounts compare and hash identically, while
/// Serde preserves the distinction. Ordering compares owner, then effective
/// subaccount bytes.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Account {
    /// The Principal that owns this account.
    pub owner: Principal,
    /// The explicit subaccount, or absence to select the effective zero value.
    #[serde(default, with = "serde_bytes")]
    pub subaccount: Option<Subaccount>,
}

/// A failed Account text admission.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AccountError {
    /// The input exceeds 136 UTF-8 bytes.
    #[error("account text is too long")]
    TextTooLong,
    /// The owner is not admitted as a Principal.
    #[error("invalid principal: {0}")]
    InvalidPrincipal(#[source] PrincipalError),
    /// A period was present without a final dash and seven-byte checksum.
    #[error("missing account checksum")]
    MissingChecksum,
    /// The checksum does not match the owner and subaccount bytes.
    #[error("invalid account checksum")]
    InvalidChecksum,
    /// The subaccount is not at most 64 ASCII hexadecimal digits.
    #[error("subaccount must contain at most 64 hex digits")]
    InvalidSubaccount,
    /// A nonempty subaccount component starts with an ASCII zero.
    #[error("subaccount should not have leading zeroes")]
    LeadingZeroesInSubaccount,
    /// An explicit subaccount component is empty.
    #[error("default subaccount should be omitted")]
    DefaultSubaccountShouldBeOmitted,
}

impl Account {
    /// The maximum admitted UTF-8 byte length of an Account text.
    pub const MAX_LENGTH_IN_TEXT: usize = 136;

    /// Borrows the explicit subaccount, or the effective zero value if absent.
    #[must_use]
    pub fn effective_subaccount(&self) -> &Subaccount {
        self.subaccount.as_ref().unwrap_or(DEFAULT_SUBACCOUNT)
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

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner && self.effective_subaccount() == other.effective_subaccount()
    }
}

impl Eq for Account {}

impl Ord for Account {
    fn cmp(&self, other: &Self) -> Ordering {
        self.owner.cmp(&other.owner).then_with(|| {
            self.effective_subaccount()
                .cmp(other.effective_subaccount())
        })
    }
}

impl PartialOrd for Account {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Account {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.owner.hash(state);
        self.effective_subaccount().hash(state);
    }
}

fn checksum(owner: &Principal, subaccount: &Subaccount) -> [u8; 7] {
    let owner_len = usize::from(owner.len());
    let mut payload = [0_u8; Principal::MAX_LENGTH_IN_BYTES + 32];
    payload[..owner_len].copy_from_slice(owner.as_slice());
    payload[owner_len..owner_len + 32].copy_from_slice(subaccount);
    let crc = crc32fast::hash(&payload[..owner_len + 32]).to_be_bytes();
    let mut symbols = [0_u8; 7];
    BASE32_NOPAD.encode_mut(&crc, &mut symbols);
    symbols.make_ascii_lowercase();
    symbols
}

impl fmt::Display for Account {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let subaccount = self.effective_subaccount();
        if subaccount == DEFAULT_SUBACCOUNT {
            return fmt::Display::fmt(&self.owner, formatter);
        }

        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut hex = [0_u8; 64];
        for (index, byte) in subaccount.iter().enumerate() {
            hex[index * 2] = DIGITS[usize::from(byte >> 4)];
            hex[index * 2 + 1] = DIGITS[usize::from(byte & 0x0f)];
        }
        let first = hex.iter().position(|byte| *byte != b'0').unwrap_or(63);
        let symbols = checksum(&self.owner, subaccount);
        // Both alphabets are ASCII; no input text is indexed or decoded here.
        let checksum = std::str::from_utf8(&symbols).expect("Base32 checksum is ASCII");
        let hex = std::str::from_utf8(&hex[first..]).expect("hexadecimal digits are ASCII");
        write!(formatter, "{}-{checksum}.{hex}", self.owner)
    }
}

impl FromStr for Account {
    type Err = AccountError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.len() > Self::MAX_LENGTH_IN_TEXT {
            return Err(AccountError::TextTooLong);
        }
        let Some((prefix, hex)) = text.split_once('.') else {
            return Principal::from_text(text)
                .map(Self::from)
                .map_err(AccountError::InvalidPrincipal);
        };
        let (owner_text, supplied_checksum) = prefix
            .rsplit_once('-')
            .filter(|(_, checksum)| checksum.len() == 7)
            .ok_or(AccountError::MissingChecksum)?;
        if hex.starts_with('0') {
            return Err(AccountError::LeadingZeroesInSubaccount);
        }
        let owner = Principal::from_text(owner_text).map_err(AccountError::InvalidPrincipal)?;
        if hex.len() > 64 {
            return Err(AccountError::InvalidSubaccount);
        }
        if hex.is_empty() {
            return Err(AccountError::DefaultSubaccountShouldBeOmitted);
        }

        let mut subaccount = [0_u8; 32];
        let offset = 64 - hex.len();
        for (index, byte) in hex.bytes().enumerate() {
            let nibble = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                b'A'..=b'F' => byte - b'A' + 10,
                _ => return Err(AccountError::InvalidSubaccount),
            };
            let position = offset + index;
            subaccount[position / 2] |= if position % 2 == 0 {
                nibble << 4
            } else {
                nibble
            };
        }
        if supplied_checksum.as_bytes() != checksum(&owner, &subaccount) {
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
