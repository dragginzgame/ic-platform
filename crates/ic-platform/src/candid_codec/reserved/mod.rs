//! The Candid top type, retaining validated skipping rather than retaining a value.
use super::{
    Encode, Error, Type, Writer,
    decoder::{self, Decode, Reader},
};
/// A deliberately discarded Candid value. Skipping still enforces decoder budgets.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Reserved;
impl Encode for Reserved {
    const TYPE: &'static Type = &Type::Primitive(-16);
    fn write(&self, _: &mut Writer) -> Result<(), Error> {
        Ok(())
    }
}
impl<'a> Decode<'a> for Reserved {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, decoder::Error> {
        reader.skip(wire)?;
        Ok(Self)
    }
    fn missing() -> Result<Self, decoder::Error> {
        Ok(Self)
    }
}
impl serde::Serialize for Reserved {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}
impl<'de> serde::Deserialize<'de> for Reserved {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        serde::de::IgnoredAny::deserialize(deserializer).map(|_| Self)
    }
}
#[cfg(test)]
impl candid::CandidType for Reserved {
    fn _ty() -> candid::types::Type {
        candid::types::TypeInner::Reserved.into()
    }
    fn idl_serialize<S: candid::types::Serializer>(&self, serializer: S) -> Result<(), S::Error> {
        serializer.serialize_null(())
    }
}
#[cfg(test)]
mod tests;
