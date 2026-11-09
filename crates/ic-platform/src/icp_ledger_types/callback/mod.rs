use super::{GetBlocksArgs, QueryEncodedBlocksResult};
use crate::{
    candid_codec::{
        Encode, Error, Type, Writer,
        decoder::{self, Decode, Reader},
        references::Func,
    },
    principal::Principal,
};

/// Typed query archive callback. The payload remains untrusted until consumer admission.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct QueryEncodedBlocksFn(pub Func);
impl QueryEncodedBlocksFn {
    /// Construct a query reference without authorizing its destination or method.
    pub fn new(principal: Principal, method: String) -> Self {
        Self(Func { principal, method })
    }
}
impl From<Func> for QueryEncodedBlocksFn {
    fn from(value: Func) -> Self {
        Self(value)
    }
}
impl Encode for QueryEncodedBlocksFn {
    const TYPE: &'static Type = &Type::Function {
        args: &[GetBlocksArgs::TYPE],
        results: &[QueryEncodedBlocksResult::TYPE],
        annotations: &[1],
    };
    fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        self.0.write(writer)
    }
}
impl<'a> Decode<'a> for QueryEncodedBlocksFn {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, decoder::Error> {
        Func::read(reader, wire, Self::TYPE).map(Self)
    }
}
#[cfg(test)]
impl From<candid::types::reference::Func> for QueryEncodedBlocksFn {
    fn from(value: candid::types::reference::Func) -> Self {
        Self(value.into())
    }
}
