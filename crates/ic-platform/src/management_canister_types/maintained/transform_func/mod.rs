use super::{HttpRequestResult, TransformArgs};
use crate::{
    candid_codec::{
        Encode, Error, Type, Writer,
        decoder::{self, Decode, Reader},
        references::Func,
    },
    principal::Principal,
};

/// Typed HTTP transform callback. The payload remains untrusted until consumer admission.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct TransformFunc(pub Func);
impl TransformFunc {
    /// Construct a query reference without authorizing its destination or method.
    pub fn new(principal: Principal, method: String) -> Self {
        Self(Func { principal, method })
    }
}
impl From<Func> for TransformFunc {
    fn from(value: Func) -> Self {
        Self(value)
    }
}
impl Encode for TransformFunc {
    const TYPE: &'static Type = &Type::Function {
        args: &[TransformArgs::TYPE],
        results: &[HttpRequestResult::TYPE],
        annotations: &[1],
    };
    fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        self.0.write(writer)
    }
}
impl<'a> Decode<'a> for TransformFunc {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, decoder::Error> {
        Func::read(reader, wire, Self::TYPE).map(Self)
    }
}
#[cfg(test)]
impl From<candid::types::reference::Func> for TransformFunc {
    fn from(value: candid::types::reference::Func) -> Self {
        Self(value.into())
    }
}
