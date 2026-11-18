//! Bounded borrowed Registry protobuf projections, without transport or authentication.
//! Unknown fields are framed and skipped. Selected singular duplicates refuse;
//! an absent inline value differs from a present empty value. Chunk fetching and
//! application error interpretation remain caller-owned.
#[cfg(test)]
mod tests;

use crate::registry_settings::{
    self,
    wire::{FieldValue, Reader},
};

/// Aggregate limits for one projection and its selected nested messages.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Complete encoded input ceiling.
    pub bytes: usize,
    /// All field occurrences, including skipped and selected nested fields.
    pub fields: usize,
    /// Requested vector storage bytes; borrowed strings/blobs allocate nothing.
    pub allocation: usize,
    /// Total selected repeated entries.
    pub items: usize,
}
/// Projection refusal without partial output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Input byte ceiling exceeded.
    ByteLimit,
    /// Aggregate field ceiling exceeded.
    FieldLimit,
    /// Malformed, overflowing or mistyped protobuf.
    Malformed,
    /// Unsupported protobuf wire type.
    UnsupportedWireType,
    /// A selected singular field was repeated.
    DuplicateField,
    /// Repeated-entry ceiling exceeded.
    ItemLimit,
    /// Vector storage allowance or allocation failed.
    AllocationLimit,
    /// Response requires unsupported follow-up chunk fetching.
    ChunkedValue,
    /// Encoded request exceeds its output ceiling.
    OutputLimit,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Registry protobuf refusal: {self:?}")
    }
}
impl std::error::Error for Error {}
impl From<registry_settings::Error> for Error {
    fn from(value: registry_settings::Error) -> Self {
        match value {
            registry_settings::Error::ByteLimit => Self::ByteLimit,
            registry_settings::Error::FieldLimit => Self::FieldLimit,
            registry_settings::Error::Malformed => Self::Malformed,
            registry_settings::Error::UnsupportedWireType => Self::UnsupportedWireType,
            registry_settings::Error::DuplicateField => Self::DuplicateField,
        }
    }
}
struct Budget {
    fields: usize,
    allocation: usize,
    items: usize,
}
trait Field<'a>: Default {
    const REPEATED: bool = false;
    fn read(&mut self, value: FieldValue<'a>, budget: &mut Budget) -> Result<(), Error>;
}
impl<'a> Field<'a> for &'a [u8] {
    fn read(&mut self, value: FieldValue<'a>, _: &mut Budget) -> Result<(), Error> {
        let FieldValue::Bytes(bytes) = value else {
            return Err(Error::Malformed);
        };
        *self = bytes;
        Ok(())
    }
}
impl<'a> Field<'a> for &'a str {
    fn read(&mut self, value: FieldValue<'a>, budget: &mut Budget) -> Result<(), Error> {
        let mut bytes = &[][..];
        bytes.read(value, budget)?;
        *self = std::str::from_utf8(bytes).map_err(|_| Error::Malformed)?;
        Ok(())
    }
}
impl<'a> Field<'a> for u64 {
    fn read(&mut self, value: FieldValue<'a>, _: &mut Budget) -> Result<(), Error> {
        let FieldValue::Integer(number) = value else {
            return Err(Error::Malformed);
        };
        *self = number;
        Ok(())
    }
}
impl<'a> Field<'a> for i32 {
    fn read(&mut self, value: FieldValue<'a>, budget: &mut Budget) -> Result<(), Error> {
        let mut number = 0u64;
        number.read(value, budget)?;
        let result = number as i32;
        if number != u64::from(result as u32) && number != result as i64 as u64 {
            return Err(Error::Malformed);
        }
        *self = result;
        Ok(())
    }
}
impl<'a> Field<'a> for bool {
    fn read(&mut self, value: FieldValue<'a>, budget: &mut Budget) -> Result<(), Error> {
        let mut number = 0u64;
        number.read(value, budget)?;
        *self = match number {
            0 => false,
            1 => true,
            _ => return Err(Error::Malformed),
        };
        Ok(())
    }
}
impl<'a, T: Field<'a>> Field<'a> for Option<T> {
    fn read(&mut self, value: FieldValue<'a>, budget: &mut Budget) -> Result<(), Error> {
        let mut result = T::default();
        result.read(value, budget)?;
        *self = Some(result);
        Ok(())
    }
}
impl<'a, T: Field<'a>> Field<'a> for Vec<T> {
    const REPEATED: bool = true;
    fn read(&mut self, value: FieldValue<'a>, budget: &mut Budget) -> Result<(), Error> {
        budget.items = budget.items.checked_sub(1).ok_or(Error::ItemLimit)?;
        budget.allocation = budget
            .allocation
            .checked_sub(std::mem::size_of::<T>())
            .ok_or(Error::AllocationLimit)?;
        let mut item = T::default();
        item.read(value, budget)?;
        self.try_reserve_exact(1)
            .map_err(|_| Error::AllocationLimit)?;
        self.push(item);
        Ok(())
    }
}
macro_rules! record {
    ($(#[$meta:meta])* $name:ident { $($(#[$doc:meta])* $number:literal => $field:ident : $ty:ty),+ $(,)? }) => {
        $(#[$meta])* #[derive(Debug,Default,PartialEq,Eq)]
        pub struct $name<'a> { $($(#[$doc])* pub $field:$ty,)+ }
        impl<'a> Field<'a> for $name<'a> {
            fn read(&mut self,value:FieldValue<'a>,budget:&mut Budget)->Result<(),Error>{
                let FieldValue::Bytes(bytes)=value else{return Err(Error::Malformed)};
                let mut reader=Reader::new(bytes); let mut seen=0u64;
                while let Some((number,value))=reader.next(&mut budget.fields)? {
                    match number {
                        $($number=>{
                            if !<$ty as Field>::REPEATED && seen & (1u64 << $number)!=0 {return Err(Error::DuplicateField)}
                            seen|=1u64 << $number; self.$field.read(value,budget)?;
                        },)+
                        _=>(),
                    }
                } Ok(())
            }
        }
    }
}

record! {
    /// Raw source application error; a present empty error remains present.
    RegistryError {
        /// Signed enum value, including unknown values.
        1 => code:i32,
        /// Valid UTF-8 reason, borrowed without normalization.
        2 => reason:&'a str,
        /// Opaque source key bytes.
        3 => key:&'a [u8],
    }
}

record! {
    /// Selected subnet configuration; node identities remain raw source bytes.
    SubnetRecord {
        /// Registered members in source order.
        3 => membership:Vec<&'a [u8]>,
        /// Raw replica version string.
        9 => replica_version_id:&'a str,
        /// Raw subnet enum, including unknown values.
        15 => subnet_type:i32,
        /// Raw halt setting.
        17 => is_halted:bool,
        /// Raw halt-at-CUP setting.
        28 => halt_at_cup_height:bool,
    }
}

record! {
    /// Selected node registration metadata.
    NodeRecord {
        /// Raw operator principal bytes.
        15 => node_operator_id:&'a [u8],
        /// Absent and empty chip identifiers are distinct.
        16 => chip_id:Option<&'a [u8]>,
        /// Absent and empty host OS versions are distinct.
        17 => hostos_version_id:Option<&'a str>,
    }
}

record! {
    /// Selected operator/provider/data-center join fields.
    NodeOperatorRecord {
        /// Raw operator principal bytes.
        1 => node_operator_principal_id:&'a [u8],
        /// Raw provider principal bytes.
        3 => node_provider_principal_id:&'a [u8],
        /// Raw data-center reference.
        4 => dc_id:&'a str,
    }
}

record! {
    /// Selected data-center source metadata, without identity or text policy.
    DataCenterRecord {
        /// Source identifier.
        1 => id:&'a str,
        /// Raw region.
        2 => region:&'a str,
        /// Raw owner.
        3 => owner:&'a str,
    }
}

fn decode<'a, T: Field<'a>>(bytes: &'a [u8], limits: Limits) -> Result<T, Error> {
    if bytes.len() > limits.bytes {
        return Err(Error::ByteLimit);
    }
    let mut result = T::default();
    let mut budget = Budget {
        fields: limits.fields,
        allocation: limits.allocation,
        items: limits.items,
    };
    result.read(FieldValue::Bytes(bytes), &mut budget)?;
    Ok(result)
}
macro_rules! decoder {
    ($name:ident,$ty:ident) => {
        /// Decode one bounded borrowed projection. No identity or authority is inferred.
        ///
        /// # Errors
        /// Refuses malformed input, duplicate selected fields and exhausted limits.
        pub fn $name(bytes: &[u8], limits: Limits) -> Result<$ty<'_>, Error> {
            decode(bytes, limits)
        }
    };
}

decoder!(decode_subnet_record, SubnetRecord);
decoder!(decode_node_record, NodeRecord);
decoder!(decode_node_operator_record, NodeOperatorRecord);
decoder!(decode_data_center_record, DataCenterRecord);

mod envelope;
pub use envelope::{
    GetValueResponse, decode_get_latest_version_response, decode_get_value_response,
    decode_subnet_list, encode_get_value_request,
};
