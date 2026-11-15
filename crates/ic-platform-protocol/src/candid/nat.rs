//! Natural-number representation; arithmetic and magnitude conversion stay upstream.

use num_bigint::BigUint;
use serde::{
    Deserialize, Serialize,
    de::{self, SeqAccess, Visitor},
};
use std::{fmt, str::FromStr};

/// Arbitrary natural with upstream magnitude storage and exact limb-array Serde.
///
/// Deserialization also accepts nonnegative integers, decimal strings and Candid
/// tagged byte buffers (tag 1 followed by a little-endian magnitude). Serde calls
/// use their deserializer's admission policy, not the Candid decoder's limits.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Nat(pub BigUint);

impl FromStr for Nat {
    type Err = num_bigint::ParseBigIntError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.parse().map(Self)
    }
}

impl From<u64> for Nat {
    fn from(value: u64) -> Self {
        Self(value.into())
    }
}

impl<'de> Deserialize<'de> for Nat {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Natural;
        impl<'de> Visitor<'de> for Natural {
            type Value = Nat;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a nonnegative integer, decimal string or natural-number limbs")
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Nat, E> {
                Ok(Nat::from(value))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Nat, E> {
                u64::try_from(value).map(Nat::from).map_err(E::custom)
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Nat, E> {
                value.parse().map_err(E::custom)
            }
            fn visit_byte_buf<E: de::Error>(self, bytes: Vec<u8>) -> Result<Nat, E> {
                match bytes.split_first() {
                    Some((1, magnitude)) => Ok(Nat(BigUint::from_bytes_le(magnitude))),
                    _ => Err(E::custom("expected Candid natural tag 1")),
                }
            }
            fn visit_seq<S: SeqAccess<'de>>(self, mut sequence: S) -> Result<Nat, S::Error> {
                let mut words = Vec::new();
                while let Some(word) = sequence.next_element::<u32>()? {
                    words.push(word);
                }
                Ok(Nat(BigUint::new(words)))
            }
        }
        deserializer.deserialize_any(Natural)
    }
}
