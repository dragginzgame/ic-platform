//! ICRC values and their non-cryptographic conversions and formatting.

use super::account::Account;
use crate::candid_codec::{Int, Nat};
use crate::principal::Principal;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

mod hash;

/// A string-ordered map of general ledger values.
pub type Map = BTreeMap<String, Value>;

/// A string-ordered map of ICRC-3 values.
pub type ICRC3Map = BTreeMap<String, ICRC3Value>;

/// The fixed-size result of the value hashing adapter.
pub type Hash = [u8; 32];

/// An ICRC-3 value with arbitrary-precision integer carriers.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ICRC3Value {
    /// Opaque bytes.
    Blob(ByteBuf),
    /// Text without additional normalization.
    Text(String),
    /// An unbounded nonnegative integer.
    Nat(Nat),
    /// An unbounded signed integer.
    Int(Int),
    /// An ordered sequence of values.
    Array(Vec<ICRC3Value>),
    /// Values indexed by string-ordered keys.
    Map(ICRC3Map),
}

/// A ledger value retaining a distinct 64-bit natural-number variant.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    /// Opaque bytes.
    Blob(ByteBuf),
    /// Text without additional normalization.
    Text(String),
    /// An unbounded nonnegative integer.
    Nat(Nat),
    /// A distinct 64-bit nonnegative integer.
    Nat64(u64),
    /// An unbounded signed integer.
    Int(Int),
    /// An ordered sequence of values.
    Array(Vec<Value>),
    /// Values indexed by string-ordered keys.
    Map(Map),
}

impl From<Value> for ICRC3Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Blob(bytes) => Self::Blob(bytes),
            Value::Text(text) => Self::Text(text),
            Value::Nat(nat) => Self::Nat(nat),
            Value::Nat64(nat) => Self::Nat(nat.into()),
            Value::Int(int) => Self::Int(int),
            Value::Array(values) => Self::Array(values.into_iter().map(Self::from).collect()),
            Value::Map(values) => Self::Map(
                values
                    .into_iter()
                    .map(|(key, value)| (key, Self::from(value)))
                    .collect(),
            ),
        }
    }
}

impl From<ICRC3Value> for Value {
    fn from(value: ICRC3Value) -> Self {
        match value {
            ICRC3Value::Blob(bytes) => Self::Blob(bytes),
            ICRC3Value::Text(text) => Self::Text(text),
            ICRC3Value::Nat(nat) => Self::Nat(nat),
            ICRC3Value::Int(int) => Self::Int(int),
            ICRC3Value::Array(values) => Self::Array(values.into_iter().map(Self::from).collect()),
            ICRC3Value::Map(values) => Self::Map(
                values
                    .into_iter()
                    .map(|(key, value)| (key, Self::from(value)))
                    .collect(),
            ),
        }
    }
}

impl Value {
    /// Returns the exact spelling of this value's variant.
    #[must_use]
    pub fn variant_name(&self) -> String {
        match self {
            Self::Blob(_) => "Blob",
            Self::Text(_) => "Text",
            Self::Nat(_) => "Nat",
            Self::Nat64(_) => "Nat64",
            Self::Int(_) => "Int",
            Self::Array(_) => "Array",
            Self::Map(_) => "Map",
        }
        .to_string()
    }

    /// Creates a text value from its string representation.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "The public generic constructor accepts owned and borrowed ToString values"
    )]
    pub fn text(t: impl ToString) -> Self {
        Self::Text(t.to_string())
    }

    /// Creates a blob from owned bytes.
    pub fn blob(t: impl Into<Vec<u8>>) -> Self {
        Self::Blob(ByteBuf::from(t.into()))
    }

    /// Creates a map, retaining the last value for each converted string key.
    pub fn map<S, V>(v: V) -> Self
    where
        S: ToString,
        V: IntoIterator<Item = (S, Value)>,
    {
        Self::Map(
            v.into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
        )
    }

    /// Takes a blob, or returns the actual variant name.
    pub fn as_blob(self) -> Result<ByteBuf, String> {
        match self {
            Self::Blob(bytes) => Ok(bytes),
            value => Err(value.variant_name()),
        }
    }

    /// Takes text, or returns the actual variant name.
    pub fn as_text(self) -> Result<String, String> {
        match self {
            Self::Text(text) => Ok(text),
            value => Err(value.variant_name()),
        }
    }

    /// Takes a natural number, accepting Nat64 and nonnegative Int values too.
    pub fn as_nat(self) -> Result<Nat, String> {
        match self {
            Self::Nat(nat) => Ok(nat),
            Self::Nat64(nat) => Ok(nat.into()),
            Self::Int(int) => num_bigint::BigUint::try_from(int.0)
                .map(Nat)
                .map_err(|error| format!("Failed to convert Int to Nat: {error:?}")),
            value => Err(value.variant_name()),
        }
    }

    /// Takes an Int payload, or returns the actual variant name.
    pub fn as_int(self) -> Result<Int, String> {
        match self {
            Self::Int(int) => Ok(int),
            value => Err(value.variant_name()),
        }
    }

    /// Takes an array, or returns the actual variant name.
    pub fn as_array(self) -> Result<Vec<Value>, String> {
        match self {
            Self::Array(values) => Ok(values),
            value => Err(value.variant_name()),
        }
    }

    /// Takes a map, or returns the actual variant name.
    pub fn as_map(self) -> Result<Map, String> {
        match self {
            Self::Map(values) => Ok(values),
            value => Err(value.variant_name()),
        }
    }
}

impl TryFrom<Value> for ByteBuf {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value
            .as_blob()
            .map_err(|error| format!("Expecting variant Blob but found {error}"))
    }
}

impl TryFrom<Value> for Vec<u8> {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        ByteBuf::try_from(value).map(ByteBuf::into_vec)
    }
}

impl TryFrom<Value> for String {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value
            .as_text()
            .map_err(|error| format!("Expecting variant Text but found {error}"))
    }
}

impl TryFrom<Value> for Nat {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value
            .as_nat()
            .map_err(|error| format!("Expecting variant Nat but found {error}"))
    }
}

impl TryFrom<Value> for u64 {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Nat::try_from(value)?
            .0
            .to_u64()
            .ok_or_else(|| "Unable to convert nat {nat} to u64".to_string())
    }
}

impl TryFrom<Value> for Int {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value
            .as_int()
            .map_err(|error| format!("Expecting variant Int but found {error}"))
    }
}

impl TryFrom<Value> for Vec<Value> {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value
            .as_array()
            .map_err(|error| format!("Expecting variant Array but found {error}"))
    }
}

impl TryFrom<Value> for Map {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value
            .as_map()
            .map_err(|error| format!("Expecting variant Map but found {error}"))
    }
}

impl TryFrom<Value> for Account {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let mut values = value.as_array()?.into_iter();
        let owner = match (values.len(), values.next()) {
            (1 | 2, Some(owner)) => owner.as_blob()?,
            (length, _) => {
                return Err(format!(
                    "Account should be an array of either one or two elements but found an array of {length} elements"
                ));
            }
        };
        let owner = Principal::try_from_slice(owner.as_ref())
            .map_err(|error| format!("Unable to decode the owner of the account, error {error}"))?;
        let subaccount = match values.next() {
            Some(value) => {
                let bytes = value.as_blob()?;
                let subaccount = bytes.as_ref().try_into().map_err(|error| {
                    format!("Unable to decode the subaccount of the account, error {error}")
                })?;
                Some(subaccount)
            }
            None => None,
        };
        Ok(Self { owner, subaccount })
    }
}

impl From<Account> for Value {
    fn from(account: Account) -> Self {
        let mut values = vec![Self::from(account.owner)];
        if let Some(subaccount) = account.subaccount {
            values.push(Self::blob(subaccount.to_vec()));
        }
        Self::Array(values)
    }
}

impl TryFrom<Value> for Principal {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let bytes = value.as_blob()?;
        Self::try_from_slice(bytes.as_ref())
            .map_err(|error| format!("Unable to decode the principal, error {error}"))
    }
}

impl From<Principal> for Value {
    fn from(principal: Principal) -> Self {
        Self::blob(principal.as_slice().to_vec())
    }
}

fn display_blob(bytes: &[u8], f: &mut Formatter<'_>) -> fmt::Result {
    for byte in bytes {
        write!(f, "{byte:02x}")?;
    }
    Ok(())
}

fn display_array<T: Display>(values: &[T], f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str("Array(")?;
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            f.write_str(", ")?;
        }
        write!(f, "{value}")?;
    }
    f.write_str(")")
}

fn display_map<T: Display>(values: &BTreeMap<String, T>, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str("Map(")?;
    for (index, (key, value)) in values.iter().enumerate() {
        if index != 0 {
            f.write_str(", ")?;
        }
        write!(f, "{key}: {value}")?;
    }
    f.write_str(")")
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blob(bytes) => display_blob(bytes.as_ref(), f),
            Self::Text(text) => f.write_str(text),
            Self::Nat(nat) => write!(f, "{nat}"),
            Self::Nat64(nat) => write!(f, "{nat}"),
            Self::Int(int) => write!(f, "{int}"),
            Self::Array(values) => display_array(values, f),
            Self::Map(values) => display_map(values, f),
        }
    }
}

impl Display for ICRC3Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blob(bytes) => display_blob(bytes.as_ref(), f),
            Self::Text(text) => f.write_str(text),
            Self::Nat(nat) => write!(f, "{nat}"),
            Self::Int(int) => write!(f, "{int}"),
            Self::Array(values) => display_array(values, f),
            Self::Map(values) => display_map(values, f),
        }
    }
}
