//! Owned natural-number carrier; arithmetic and binary magnitude storage use num-bigint.
//! Serde accepts the established Candid Nat forms: nonnegative integers, decimal
//! strings, little-endian u32 limb sequences, and tagged Candid byte buffers.
use super::{
    Encode, Error, Type, Writer,
    decoder::{Decode, Reader},
};
use num_bigint::BigUint;
use serde::{
    Deserialize, Serialize,
    de::{self, SeqAccess, Visitor},
};
use std::{fmt, str::FromStr};

/// Unbounded natural number with the established external Nat Serde representation.
/// This is a Platform type; use `.0` for num-bigint arithmetic and explicit conversion
/// when interacting with a registry Candid Nat.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Nat(pub BigUint);

impl From<BigUint> for Nat {
    fn from(value: BigUint) -> Self {
        Self(value)
    }
}
impl From<Nat> for BigUint {
    fn from(value: Nat) -> Self {
        value.0
    }
}
macro_rules! unsigned {
    ($($ty:ty),*) => { $(
        impl From<$ty> for Nat {
            fn from(value: $ty) -> Self { Self(value.into()) }
        }
        impl PartialEq<$ty> for Nat {
            fn eq(&self, other: &$ty) -> bool { self.0 == BigUint::from(*other) }
        }
        impl PartialOrd<$ty> for Nat {
            fn partial_cmp(&self, other: &$ty) -> Option<std::cmp::Ordering> { self.0.partial_cmp(&BigUint::from(*other)) }
        }
    )* };
}
unsigned!(u8, u16, u32, u64, u128, usize);
impl FromStr for Nat {
    type Err = num_bigint::ParseBigIntError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self)
    }
}
impl fmt::Display for Nat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = self.0.to_str_radix(10);
        for (index, byte) in digits.bytes().enumerate() {
            if index != 0 && (digits.len() - index).is_multiple_of(3) {
                f.write_str("_")?;
            }
            write!(f, "{}", char::from(byte))?;
        }
        Ok(())
    }
}
impl<'de> Deserialize<'de> for Nat {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Natural;
        impl<'de> Visitor<'de> for Natural {
            type Value = Nat;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("Nat value")
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Nat, E> {
                Ok(value.into())
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Nat, E> {
                u64::try_from(value)
                    .map(Nat::from)
                    .map_err(|_| E::custom("negative natural number"))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Nat, E> {
                value.parse().map_err(E::custom)
            }
            fn visit_byte_buf<E: de::Error>(self, value: Vec<u8>) -> Result<Nat, E> {
                match value.split_first() {
                    Some((1, magnitude)) => Ok(Nat(BigUint::from_bytes_le(magnitude))),
                    _ => Err(E::custom("invalid natural number tag")),
                }
            }
            fn visit_seq<S: SeqAccess<'de>>(self, mut sequence: S) -> Result<Nat, S::Error> {
                let mut limbs = Vec::new();
                while let Some(limb) = sequence.next_element::<u32>()? {
                    limbs.push(limb);
                }
                Ok(Nat(BigUint::new(limbs)))
            }
        }
        deserializer.deserialize_any(Natural)
    }
}
impl Encode for Nat {
    const TYPE: &'static Type = BigUint::TYPE;
    fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        self.0.write(writer)
    }
}
impl<'a> Decode<'a> for Nat {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, super::decoder::Error> {
        BigUint::read(reader, wire).map(Self)
    }
}
#[cfg(test)]
impl candid::CandidType for Nat {
    fn _ty() -> candid::types::Type {
        candid::Nat::_ty()
    }
    fn idl_serialize<S: candid::types::Serializer>(&self, serializer: S) -> Result<(), S::Error> {
        serializer.serialize_nat(&self.to_string().parse::<candid::Nat>().unwrap())
    }
}
#[cfg(test)]
impl From<candid::Nat> for Nat {
    fn from(value: candid::Nat) -> Self {
        Self(value.0.to_string().parse().unwrap())
    }
}
#[cfg(test)]
impl From<Nat> for candid::Nat {
    fn from(value: Nat) -> Self {
        Self(value.0.to_string().parse().unwrap())
    }
}
#[cfg(test)]
mod tests;
