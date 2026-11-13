//! ICRC Account representation with canonical identity semantics.
//!
//! Serde retains the ledger's sequence-shaped subaccount field; textual
//! admission preserves this adapter's detailed external-format errors.

use crate::principal::{Principal, PrincipalError};
use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

/// Opaque application-defined subaccount bytes.
pub type Subaccount = [u8; 32];

/// The effective subaccount for an absent field.
pub const DEFAULT_SUBACCOUNT: &Subaccount = &[0; 32];

/// An ICRC account preserving the external owner/subaccount representation.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Account {
    /// The canonical Principal owning the account.
    pub owner: Principal,
    /// An explicit subaccount, or absence for the effective zero subaccount.
    pub subaccount: Option<Subaccount>,
}

impl Account {
    /// Borrows the explicit subaccount or the effective all-zero value.
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
        crate::Account::from(*self) == crate::Account::from(*other)
    }
}

impl Eq for Account {}

impl Ord for Account {
    fn cmp(&self, other: &Self) -> Ordering {
        crate::Account::from(*self).cmp(&crate::Account::from(*other))
    }
}

impl PartialOrd for Account {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Account {
    fn hash<H: Hasher>(&self, state: &mut H) {
        crate::Account::from(*self).hash(state);
    }
}

impl fmt::Display for Account {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&crate::Account::from(*self), formatter)
    }
}

/// A failed ICRC-1 textual account admission.
#[derive(Debug, PartialEq, Eq)]
pub enum ICRC1TextReprError {
    /// The explicit suffix denotes the default subaccount.
    DefaultSubaccountShouldBeOmitted,
    /// The supplied checksum differs from the canonical checksum.
    InvalidChecksum {
        /// The seven lowercase checksum characters required for this identity.
        expected: String,
    },
    /// The owner was refused by the canonical Principal parser.
    InvalidPrincipal(PrincipalError),
    /// Hexadecimal admission or the decoded subaccount length failed.
    InvalidSubaccount(String),
    /// The suffix starts with an ASCII zero.
    LeadingZeroesInSubaccount,
    /// The final dash does not delimit a seven-byte checksum.
    MissingChecksum,
}

impl fmt::Display for ICRC1TextReprError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DefaultSubaccountShouldBeOmitted => {
                formatter.write_str("default subaccount should be omitted")
            }
            Self::InvalidChecksum { expected } => {
                write!(formatter, "invalid checksum (expected: {expected})")
            }
            Self::InvalidPrincipal(error) => write!(formatter, "invalid principal: {error}"),
            Self::InvalidSubaccount(detail) => write!(formatter, "invalid subaccount: {detail}"),
            Self::LeadingZeroesInSubaccount => {
                formatter.write_str("subaccount should not have leading zeroes")
            }
            Self::MissingChecksum => formatter.write_str("missing checksum"),
        }
    }
}

impl std::error::Error for ICRC1TextReprError {}

fn decode_subaccount(suffix: &str) -> Result<Subaccount, ICRC1TextReprError> {
    // Only the first 64 scalar values matter to padding. Hex error indices are
    // byte offsets in that conceptual padding, not Unicode-character offsets.
    let padding = 64 - suffix.chars().take(64).count();
    let padded_len = padding + suffix.len();
    if !padded_len.is_multiple_of(2) {
        return Err(ICRC1TextReprError::InvalidSubaccount(format!(
            "subaccount is not hex-encoded: {}",
            hex::FromHexError::OddLength
        )));
    }
    let mut subaccount = [0_u8; 32];
    for (offset, byte) in suffix.bytes().enumerate() {
        let value = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => {
                let error = hex::FromHexError::InvalidHexCharacter {
                    c: char::from(byte),
                    index: padding + offset,
                };
                return Err(ICRC1TextReprError::InvalidSubaccount(format!(
                    "subaccount is not hex-encoded: {error}"
                )));
            }
        };
        // Oversized valid hex must be fully validated before the length error,
        // but it needs no proportional decoded buffer.
        if padded_len == 64 {
            let position = padding + offset;
            subaccount[position / 2] |= if position.is_multiple_of(2) {
                value << 4
            } else {
                value
            };
        }
    }
    if padded_len != 64 {
        return Err(ICRC1TextReprError::InvalidSubaccount(
            "subaccount is longer than 32 bytes".to_owned(),
        ));
    }
    if &subaccount == DEFAULT_SUBACCOUNT {
        return Err(ICRC1TextReprError::DefaultSubaccountShouldBeOmitted);
    }
    Ok(subaccount)
}

fn checksum(owner: &Principal, subaccount: &Subaccount) -> [u8; 7] {
    let owner_bytes = owner.as_slice();
    let mut payload = [0_u8; Principal::MAX_LENGTH_IN_BYTES + 32];
    payload[..owner_bytes.len()].copy_from_slice(owner_bytes);
    payload[owner_bytes.len()..owner_bytes.len() + 32].copy_from_slice(subaccount);
    let crc = crc32fast::hash(&payload[..owner_bytes.len() + 32]).to_be_bytes();
    let mut encoded = [0_u8; 7];
    BASE32_NOPAD.encode_mut(&crc, &mut encoded);
    encoded.make_ascii_lowercase();
    encoded
}

impl FromStr for Account {
    type Err = ICRC1TextReprError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let Some((prefix, suffix)) = text.split_once('.') else {
            return text
                .parse::<Principal>()
                .map(Self::from)
                .map_err(ICRC1TextReprError::InvalidPrincipal);
        };
        let (owner, supplied_checksum) = prefix
            .rsplit_once('-')
            .filter(|(_, checksum)| checksum.len() == 7)
            .ok_or(ICRC1TextReprError::MissingChecksum)?;
        if suffix.starts_with('0') {
            return Err(ICRC1TextReprError::LeadingZeroesInSubaccount);
        }
        let owner = owner
            .parse::<Principal>()
            .map_err(ICRC1TextReprError::InvalidPrincipal)?;
        let subaccount = decode_subaccount(suffix)?;
        let expected = checksum(&owner, &subaccount);
        if supplied_checksum.as_bytes() != expected {
            return Err(ICRC1TextReprError::InvalidChecksum {
                expected: expected.into_iter().map(char::from).collect(),
            });
        }
        Ok(Self {
            owner,
            subaccount: Some(subaccount),
        })
    }
}

#[cfg(test)]
mod tests;
