//! Explicit wire mappings; declarations and imported algorithms retain their owners.
mod account;
mod allowance;
mod archive;
mod blocks;
mod icrc10;
mod icrc21_errors;
mod icrc21_requests;
mod icrc21_responses;
mod metadata;
mod transfer;
mod value;

impl crate::candid_codec::Encode for super::Memo {
    const TYPE: &'static crate::candid_codec::Type =
        <serde_bytes::ByteBuf as crate::candid_codec::Encode>::TYPE;
    fn write(
        &self,
        writer: &mut crate::candid_codec::Writer,
    ) -> Result<(), crate::candid_codec::Error> {
        self.0.write(writer)
    }
}
impl<'a> crate::candid_codec::decoder::Decode<'a> for super::Memo {
    fn read(
        reader: &mut crate::candid_codec::decoder::Reader<'a>,
        wire: i64,
    ) -> Result<Self, crate::candid_codec::decoder::Error> {
        serde_bytes::ByteBuf::read(reader, wire).map(Self)
    }
}
