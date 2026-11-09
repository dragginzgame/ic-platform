//! Explicit typed-reference payloads. Consumers retain destination/method authority.
use super::{
    Encode, Error, Type, Writer,
    decoder::{self, Reader},
};
use crate::principal::Principal;
use serde::de::{self, Visitor};
use std::fmt;

/// Function-reference payload; its enclosing Rust mapping supplies the signature.
/// Decoding a reference never authorizes following it or submitting a call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Func {
    /// Canonical owned Principal; external reference bytes remain unchanged.
    pub principal: Principal,
    /// Untrusted method name, without normalization.
    pub method: String,
}
impl Func {
    /// Encode the ordinary (non-opaque) function payload, without a type header.
    pub fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        Self::write_parts(&self.principal, &self.method, writer)
    }
    pub(crate) fn write_parts(
        principal: &Principal,
        method: &str,
        writer: &mut Writer,
    ) -> Result<(), Error> {
        writer.bytes(&[1])?;
        principal.write(writer)?;
        method.write(writer)
    }
    /// Decode a payload only after validating the complete expected function signature.
    pub fn read(
        reader: &mut Reader<'_>,
        wire: i64,
        signature: &'static Type,
    ) -> Result<Self, decoder::Error> {
        reader.function(wire, signature)
    }
}
impl<'de> serde::Deserialize<'de> for Func {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Function;
        impl Visitor<'_> for Function {
            type Value = Func;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("Func value")
            }
            fn visit_byte_buf<E: de::Error>(self, bytes: Vec<u8>) -> Result<Func, E> {
                let Some((&5, payload)) = bytes.split_first() else {
                    return Err(E::custom("not a function reference"));
                };
                let mut length = 0u64;
                for (index, byte) in payload.iter().copied().take(10).enumerate() {
                    if index == 9 && byte & 127 > 1 {
                        return Err(E::custom("invalid method length"));
                    }
                    length |= u64::from(byte & 127) << (7 * index);
                    if byte & 128 == 0 {
                        let end = usize::try_from(length)
                            .ok()
                            .and_then(|length| length.checked_add(index + 1))
                            .ok_or_else(|| E::custom("invalid method length"))?;
                        let method = payload
                            .get(index + 1..end)
                            .ok_or_else(|| E::custom("truncated method"))?;
                        let principal = payload
                            .get(end..)
                            .ok_or_else(|| E::custom("truncated principal"))?;
                        return Ok(Func {
                            method: std::str::from_utf8(method).map_err(E::custom)?.to_owned(),
                            principal: Principal::try_from_slice(principal).map_err(E::custom)?,
                        });
                    }
                }
                Err(E::custom("invalid method length"))
            }
        }
        deserializer.deserialize_any(Function)
    }
}
#[cfg(test)]
impl From<candid::types::reference::Func> for Func {
    fn from(value: candid::types::reference::Func) -> Self {
        Self {
            principal: Principal::from_slice(value.principal.as_slice()),
            method: value.method,
        }
    }
}
#[cfg(test)]
impl From<Func> for candid::types::reference::Func {
    fn from(value: Func) -> Self {
        Self {
            principal: candid::Principal::from_slice(value.principal.as_slice()),
            method: value.method,
        }
    }
}

#[cfg(test)]
mod tests;
