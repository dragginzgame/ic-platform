//! Owned signed integer carrier. Magnitude arithmetic stays in num-bigint.
use super::{
    Encode, Error, Nat, Type, Writer,
    decoder::{self, Decode, Reader},
};
use num_bigint::{BigInt, BigUint, Sign};
use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
};
use std::{fmt, str::FromStr};

/// Unbounded signed integer with the established external Int Serde representation.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Int(pub BigInt);
impl From<BigInt> for Int {
    fn from(value: BigInt) -> Self {
        Self(value)
    }
}
impl From<Int> for BigInt {
    fn from(value: Int) -> Self {
        value.0
    }
}
impl From<Nat> for Int {
    fn from(value: Nat) -> Self {
        Self(value.0.into())
    }
}
macro_rules! integer {
    ($($ty:ty),*)=>{$(impl From<$ty> for Int {fn from(value:$ty)->Self{Self(value.into())}})*};
}
integer!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl FromStr for Int {
    type Err = num_bigint::ParseBigIntError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self)
    }
}
impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = self.0.to_str_radix(10);
        let start = usize::from(digits.starts_with('-'));
        for (index, byte) in digits.bytes().enumerate() {
            if index > start && (digits.len() - index).is_multiple_of(3) {
                f.write_str("_")?;
            }
            write!(f, "{}", char::from(byte))?;
        }
        Ok(())
    }
}
impl<'de> Deserialize<'de> for Int {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Integer;
        impl Visitor<'_> for Integer {
            type Value = Int;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("Int value")
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Int, E> {
                Ok(value.into())
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Int, E> {
                Ok(value.into())
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Int, E> {
                value.parse().map_err(E::custom)
            }
            fn visit_byte_buf<E: de::Error>(self, value: Vec<u8>) -> Result<Int, E> {
                match value.split_first() {
                    Some((0, bytes)) => Ok(Int(BigInt::from_signed_bytes_le(bytes))),
                    Some((1, bytes)) => Ok(Int(BigInt::from_biguint(
                        Sign::Plus,
                        BigUint::from_bytes_le(bytes),
                    ))),
                    _ => Err(E::custom("invalid integer tag")),
                }
            }
        }
        deserializer.deserialize_any(Integer)
    }
}
impl Encode for Int {
    const TYPE: &'static Type = BigInt::TYPE;
    fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        self.0.write(writer)
    }
}
impl<'a> Decode<'a> for Int {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, decoder::Error> {
        BigInt::read(reader, wire).map(Self)
    }
}
#[cfg(test)]
impl candid::CandidType for Int {
    fn _ty() -> candid::types::Type {
        candid::Int::_ty()
    }
    fn idl_serialize<S: candid::types::Serializer>(&self, serializer: S) -> Result<(), S::Error> {
        serializer.serialize_int(&self.to_string().parse::<candid::Int>().unwrap())
    }
}
#[cfg(test)]
impl From<candid::Int> for Int {
    fn from(value: candid::Int) -> Self {
        Self(value.0.to_string().parse().unwrap())
    }
}
#[cfg(test)]
impl From<Int> for candid::Int {
    fn from(value: Int) -> Self {
        Self(value.0.to_string().parse().unwrap())
    }
}
#[cfg(test)]
mod tests;
