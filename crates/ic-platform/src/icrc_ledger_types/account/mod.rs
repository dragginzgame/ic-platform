//! Selected upstream ICRC implementation; see ../NOTICE.md.
#![allow(
    missing_docs,
    reason = "Retained upstream declarations and documentation"
)]

use std::{
    error::Error,
    fmt::{self, Display},
    str::FromStr,
};

use candid::{CandidType, Deserialize, Principal, types::principal::PrincipalError};
use serde::Serialize;

pub type Subaccount = [u8; 32];

pub const DEFAULT_SUBACCOUNT: &Subaccount = &[0; 32];

/// [Account](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-3/README.md#value)
/// representation of ledgers supporting the ICRC-1 standard.
#[derive(Serialize, CandidType, Deserialize, Clone, Debug, Copy)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Subaccount>,
}

impl Account {
    /// The effective subaccount of an account - the subaccount if it is set, otherwise the default
    /// subaccount of all zeroes.
    #[inline]
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

impl std::cmp::PartialOrd for Account {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Account {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.owner.cmp(&other.owner).then_with(|| {
            self.effective_subaccount()
                .cmp(other.effective_subaccount())
        })
    }
}

impl std::hash::Hash for Account {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.owner.hash(state);
        self.effective_subaccount().hash(state);
    }
}

fn full_account_checksum(owner: &[u8], subaccount: &[u8]) -> String {
    let mut crc32hasher = crc32fast::Hasher::new();
    crc32hasher.update(owner);
    crc32hasher.update(subaccount);
    let checksum = crc32hasher.finalize().to_be_bytes();
    data_encoding::BASE32_NOPAD.encode(&checksum).to_lowercase()
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Both identities share canonical text; external parsing/errors stay separate.
        fmt::Display::fmt(&crate::Account::from(*self), f)
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

#[derive(Debug, PartialEq, Eq)]
pub enum ICRC1TextReprError {
    DefaultSubaccountShouldBeOmitted,
    InvalidChecksum { expected: String },
    InvalidPrincipal(PrincipalError),
    InvalidSubaccount(String),
    LeadingZeroesInSubaccount,
    MissingChecksum,
}

impl Display for ICRC1TextReprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ICRC1TextReprError::DefaultSubaccountShouldBeOmitted => {
                write!(f, "default subaccount should be omitted")
            }
            ICRC1TextReprError::InvalidChecksum { expected } => {
                write!(f, "invalid checksum (expected: {expected})")
            }
            ICRC1TextReprError::InvalidPrincipal(e) => write!(f, "invalid principal: {e}"),
            ICRC1TextReprError::InvalidSubaccount(e) => write!(f, "invalid subaccount: {e}"),
            ICRC1TextReprError::LeadingZeroesInSubaccount => {
                write!(f, "subaccount should not have leading zeroes")
            }
            ICRC1TextReprError::MissingChecksum => write!(f, "missing checksum"),
        }
    }
}

impl Error for ICRC1TextReprError {}

impl FromStr for Account {
    type Err = ICRC1TextReprError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('.') {
            Some((principal_checksum, subaccount)) => {
                let (principal, checksum) = match principal_checksum.rsplit_once('-') {
                    // The checksum is 7 characters (crc32 encoded via base32) while principal
                    // groups are 5 characters
                    Some((_, checksum)) if checksum.len() != 7 => {
                        return Err(Self::Err::MissingChecksum);
                    }
                    Some(principal_and_checksum) => principal_and_checksum,
                    None => return Err(Self::Err::MissingChecksum),
                };
                if subaccount.starts_with('0') {
                    return Err(Self::Err::LeadingZeroesInSubaccount);
                }
                let owner = Principal::from_str(principal).map_err(Self::Err::InvalidPrincipal)?;
                let subaccount = hex::decode(format!("{subaccount:0>64}")).map_err(|e| {
                    Self::Err::InvalidSubaccount(format!("subaccount is not hex-encoded: {e}"))
                })?;
                let subaccount: Subaccount = subaccount.try_into().map_err(|_| {
                    Self::Err::InvalidSubaccount("subaccount is longer than 32 bytes".to_string())
                })?;
                if &subaccount == DEFAULT_SUBACCOUNT {
                    return Err(Self::Err::DefaultSubaccountShouldBeOmitted);
                }
                let expected_checksum =
                    full_account_checksum(owner.as_slice(), subaccount.as_slice());
                if checksum != expected_checksum {
                    return Err(Self::Err::InvalidChecksum {
                        expected: expected_checksum,
                    });
                }
                Ok(Self {
                    owner,
                    subaccount: Some(subaccount),
                })
            }
            None => Principal::from_str(s)
                .map_err(Self::Err::InvalidPrincipal)
                .map(Account::from),
        }
    }
}
