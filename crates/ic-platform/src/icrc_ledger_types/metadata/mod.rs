//! Typed ledger metadata values with unbounded integer representations.

use crate::candid_codec::{Int, Nat};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

/// A ledger metadata value in its external representation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataValue {
    /// An unbounded nonnegative integer.
    Nat(Nat),
    /// An unbounded signed integer.
    Int(Int),
    /// UTF-8 text preserved verbatim.
    Text(String),
    /// Opaque bytes.
    Blob(ByteBuf),
}

impl From<i64> for MetadataValue {
    fn from(value: i64) -> Self {
        Self::Int(Int::from(value))
    }
}

impl From<i128> for MetadataValue {
    fn from(value: i128) -> Self {
        Self::Int(Int::from(value))
    }
}

impl From<u64> for MetadataValue {
    fn from(value: u64) -> Self {
        Self::Nat(Nat::from(value))
    }
}

impl From<u128> for MetadataValue {
    fn from(value: u128) -> Self {
        Self::Nat(Nat::from(value))
    }
}

impl From<Nat> for MetadataValue {
    fn from(value: Nat) -> Self {
        Self::Nat(value)
    }
}

impl From<String> for MetadataValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for MetadataValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<Vec<u8>> for MetadataValue {
    fn from(value: Vec<u8>) -> Self {
        Self::Blob(ByteBuf::from(value))
    }
}

impl From<&[u8]> for MetadataValue {
    fn from(value: &[u8]) -> Self {
        Self::Blob(ByteBuf::from(value))
    }
}
