//! Container adapters preserve the existing vector, blob and map wire shapes.
use super::{Encode, Error, Type, Writer};
use std::collections::BTreeMap;
impl<T: Encode> Encode for [T] {
    const TYPE: &'static Type = &Type::Vec(T::TYPE);
    fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        writer.unsigned(self.len() as u128)?;
        T::write_slice(self, writer)
    }
}
impl<T: Encode, const N: usize> Encode for [T; N] {
    const TYPE: &'static Type = <[T]>::TYPE;
    fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        self.as_slice().write(writer)
    }
}
impl<K: Encode, V: Encode> Encode for BTreeMap<K, V> {
    const TYPE: &'static Type = <Vec<(K, V)>>::TYPE;
    fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        writer.unsigned(self.len() as u128)?;
        for (key, value) in self {
            key.write(writer)?;
            value.write(writer)?;
        }
        Ok(())
    }
}
#[cfg(feature = "icrc-ledger-types")]
impl Encode for serde_bytes::ByteBuf {
    const TYPE: &'static Type = <Vec<u8>>::TYPE;
    fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        self.as_ref().write(writer)
    }
}
