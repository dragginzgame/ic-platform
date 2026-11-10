//! Ledger protocol composition using the external SHA-256 implementation.

use sha2_11::{Digest, Sha256};

use super::{Hash, ICRC3Value, Value};
use crate::candid_codec::{Int, Nat};

impl Value {
    /// Hashes the value using Platform's established ledger representation.
    ///
    /// Nonnegative `Int` values retain unsigned LEB128 framing. This historical
    /// behavior differs from signed framing at some positive integer boundaries.
    ///
    /// # Panics
    ///
    /// Panics when an `Int` is outside the signed 128-bit range.
    pub fn hash(&self) -> Hash {
        match self {
            Self::Blob(bytes) => hash_bytes(bytes.as_ref()),
            Self::Text(text) => hash_bytes(text.as_bytes()),
            Self::Nat(value) => hash_nat(value),
            Self::Nat64(value) => hash_unsigned(u128::from(*value)),
            Self::Int(value) => hash_int(value),
            Self::Array(values) => hash_array(values.iter().map(Self::hash)),
            Self::Map(values) => hash_map(
                values
                    .iter()
                    .map(|(key, value)| (hash_bytes(key.as_bytes()), value.hash())),
            ),
        }
    }
}

impl ICRC3Value {
    /// Hashes this value with the same representation and integer limits as [`Value::hash`].
    ///
    /// # Panics
    ///
    /// Panics when an `Int` is outside the signed 128-bit range.
    pub fn hash(self) -> Hash {
        match self {
            Self::Blob(bytes) => hash_bytes(bytes.as_ref()),
            Self::Text(text) => hash_bytes(text.as_bytes()),
            Self::Nat(value) => hash_nat(&value),
            Self::Int(value) => hash_int(&value),
            Self::Array(values) => hash_array(values.into_iter().map(Self::hash)),
            Self::Map(values) => hash_map(
                values
                    .into_iter()
                    .map(|(key, value)| (hash_bytes(key.as_bytes()), value.hash())),
            ),
        }
    }
}

fn hash_bytes(bytes: &[u8]) -> Hash {
    Sha256::digest(bytes).into()
}

fn hash_nat(value: &Nat) -> Hash {
    let mut bytes = Vec::new();
    value
        .encode(&mut bytes)
        .expect("writing a natural number to a vector cannot fail");
    hash_bytes(&bytes)
}

fn hash_unsigned(mut value: u128) -> Hash {
    let mut bytes = [0_u8; 19];
    let mut length = 0;
    loop {
        let payload = value.to_le_bytes()[0] & 0x7f;
        value >>= 7;
        bytes[length] = payload | if value == 0 { 0 } else { 0x80 };
        length += 1;
        if value == 0 {
            return hash_bytes(&bytes[..length]);
        }
    }
}

fn hash_int(value: &Int) -> Hash {
    let mut value = i128::try_from(&value.0).unwrap_or_else(|_| {
        panic!(
            "BUG: blocks cannot contain integers that do not fit into the 128-bit representation"
        )
    });
    if value >= 0 {
        return hash_unsigned(value.unsigned_abs());
    }

    let mut bytes = [0_u8; 19];
    let mut length = 0;
    loop {
        let payload = value.to_le_bytes()[0] & 0x7f;
        value >>= 7;
        let complete = value == -1 && payload & 0x40 != 0;
        bytes[length] = payload | if complete { 0 } else { 0x80 };
        length += 1;
        if complete {
            return hash_bytes(&bytes[..length]);
        }
    }
}

fn hash_array(values: impl Iterator<Item = Hash>) -> Hash {
    let mut digest = Sha256::new();
    for value in values {
        digest.update(value);
    }
    digest.finalize().into()
}

fn hash_map(entries: impl Iterator<Item = (Hash, Hash)>) -> Hash {
    let mut entries: Vec<_> = entries.collect();
    entries.sort_unstable();
    let mut digest = Sha256::new();
    for (key, value) in entries {
        digest.update(key);
        digest.update(value);
    }
    digest.finalize().into()
}
