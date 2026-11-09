use super::{Decode, Error, Id, Reader, Result};
use std::collections::BTreeMap;
impl<'a, T: Decode<'a>, const N: usize> Decode<'a> for [T; N] {
    fn read(reader: &mut Reader<'a>, wire: Id) -> Result<Self> {
        Vec::<T>::read(reader, wire)?
            .try_into()
            .map_err(|_| Error::Type)
    }
}
impl<'a, K: Decode<'a> + Ord, V: Decode<'a>> Decode<'a> for BTreeMap<K, V> {
    fn read(reader: &mut Reader<'a>, wire: Id) -> Result<Self> {
        let entries = Vec::<(K, V)>::read(reader, wire)?;
        reader.allocate(
            entries
                .len()
                .checked_mul(size_of::<(K, V)>())
                .ok_or(Error::Limit)?,
        )?;
        reader.charge(entries.len())?;
        // Match Serde's BTreeMap admission: arbitrary order, last duplicate wins.
        Ok(entries.into_iter().collect())
    }
}
#[cfg(feature = "icrc-ledger-types")]
impl<'a> Decode<'a> for serde_bytes::ByteBuf {
    fn read(reader: &mut Reader<'a>, wire: Id) -> Result<Self> {
        Vec::<u8>::read(reader, wire).map(Self::from)
    }
}
