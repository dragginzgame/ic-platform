//! Decode-only selected response records.

use crate::candid::{self, Decode, Error, Limits, Reader, Schema};
use candidate_types::Principal;
use serde::{Deserialize, Serialize};

// One declaration emits fields, schema and reader dispatch. Optional policy is
// explicit at each field. Candid-compatible fallback is deliberately not part
// of this private strict projection declaration.
macro_rules! absent {
    (required) => {
        Err(Error::Type)
    };
    (strict_optional) => {
        Ok(None)
    };
}
macro_rules! projection {
    ($(#[$meta:meta])* arguments $name:ident { $($(#[$field_meta:meta])* $label:literal => $field:ident : $ty:ty),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
        pub struct $name { $($(#[$field_meta])* pub $field: $ty,)+ }
        impl $name {
            fn write(&self, writer: &mut ArgumentWriter) -> Result<(), EncodeError> {
                const LABELS: &[u32] = &[$($label),+];
                const { assert!(LABELS.len() <= 63); let mut i = 1; while i < LABELS.len() {
                    assert!(LABELS[i - 1] < LABELS[i]); i += 1;
                } }
                writer.bytes(b"DIDL")?;
                writer.uleb(LABELS.len() + 1)?;
                writer.byte(0x6c)?;
                writer.uleb(LABELS.len())?;
                for (index, label) in LABELS.iter().enumerate() {
                    writer.uleb(*label as usize)?;
                    // This selected graph has three option entries; their
                    // positive type references fit one signed LEB byte.
                    writer.byte((index + 1) as u8)?;
                }
                $(writer.bytes(&[0x6e, <$ty as Argument>::PRIMITIVE])?;)+
                writer.bytes(&[1, 0])?;
                $(self.$field.write(writer)?;)+
                Ok(())
            }
        }
    };
    (@schema) => { <() as Decode>::SCHEMA };
    (@schema $ty:ty) => { <$ty as Decode>::SCHEMA };
    (@read $reader:ident, $wire:ident, $name:ident, $variant:ident) => {
        { $reader.value::<()>($wire)?; Ok($name::$variant) }
    };
    (@read $reader:ident, $wire:ident, $name:ident, $variant:ident, $ty:ty) => {
        Ok($name::$variant($reader.value::<$ty>($wire)?))
    };
    ($(#[$meta:meta])* enum $name:ident { $($(#[$variant_meta:meta])* $label:literal => $variant:ident $(($ty:ty))?),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
        pub enum $name { $($(#[$variant_meta])* $variant $(($ty))?,)+ }
        impl Decode for $name {
            const SCHEMA: Schema = Schema::Variant(&[$(($label, &projection!(@schema $($ty)?))),+]);
            fn read(reader: &mut Reader<'_, '_>, id: i64) -> Result<Self, Error> {
                let (label, wire) = reader.variant(id)?;
                match label {
                    $($label => projection!(@read reader, wire, $name, $variant $(, $ty)?),)+
                    _ => Err(Error::Type),
                }
            }
        }
    };
    ($(#[$meta:meta])* $name:ident { $( $(#[$field_meta:meta])* $label:literal => $field:ident : $ty:ty [$policy:ident] ),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
        pub struct $name { $( $(#[$field_meta])* pub $field: $ty, )+ }
        impl Decode for $name {
            const SCHEMA: Schema = Schema::Record(&[$(($label, &<$ty as Decode>::SCHEMA)),+]);
            fn read(reader: &mut Reader<'_, '_>, id: i64) -> Result<Self, Error> {
                $(let mut $field = None;)+
                reader.record(id, |label, wire, reader| {
                    match label {
                        $($label => { $field = Some(reader.value::<$ty>(wire)?); },)+
                        _ => return Ok(false),
                    }
                    Ok(true)
                })?;
                Ok(Self { $($field: match $field { Some(value) => value, None => absent!($policy)?, }),+ })
            }
        }
    };
}

projection! {
    /// Raw Registry-copy progress; no freshness or atomic snapshot is implied.
    RegistryCopyStatus {
        /// Reported latest version.
        1_432_552_775 => latest: u64 [required],
        /// Reported copy version.
        2_096_228_120 => version: u64 [required],
        /// Source progress flag, without a derived availability claim.
        2_653_674_110 => caught_up: bool [required],
    }
}

/// Decode exactly one raw network-fee markup fraction.
///
/// All IEEE-754 bits are retained, including signed zero, infinities and NaNs.
/// The caller owns finite/nonnegative admission and must not use a fresh fee to
/// recalculate historical marketplace gross prices.
///
/// # Errors
/// Returns the shared typed framing, type, arity and resource refusals.
pub fn decode_get_network_fee_response(bytes: &[u8], limits: Limits) -> Result<f64, Error> {
    candid::decode(bytes, limits)
}

/// Decode exactly one Registry-copy progress record with required fields.
///
/// # Errors
/// Returns the shared typed framing, type, arity and resource refusals. Unknown
/// fields are validated and skipped within the supplied work and skip budgets.
pub fn decode_registry_copy_status_response(
    bytes: &[u8],
    limits: Limits,
) -> Result<RegistryCopyStatus, Error> {
    candid::decode(bytes, limits)
}

projection! {
    /// Selected Registry-copy subnet fields; decoding establishes no authenticity.
    EngineSubnet {
        /// Raw configured version text, including the empty standard-version marker.
        1_133_613_777 => replica_version: String [required],
        /// Optional configured bytes, retaining zero and the full unsigned range.
        1_588_007_470 => maximum_state_size: Option<u64> [strict_optional],
        /// Reported configured halt flag, not measured running state.
        3_065_459_808 => halted: bool [required],
        /// Registry-copy node order, independent of the engine-record list.
        4_257_184_827 => node_ids: Vec<Principal> [required],
    }
}
projection! {
    /// Selected public engine view, with raw source presence and ordering retained.
    EngineView {
        /// Opaque engine ID, without normalization or consumer admission.
        23_515 => id: String [required],
        /// Optional reported subnet identity.
        1_097_286_461 => subnet_id: Option<Principal> [strict_optional],
        /// Optional reported operator identity.
        1_645_921_689 => engine_operator_id: Option<Principal> [strict_optional],
        /// Optional Registry-copy subnet; absence is not an empty observation.
        2_584_155_869 => subnet: Option<EngineSubnet> [strict_optional],
        /// Optional signed deletion time, preserving zero and negative values.
        2_866_115_289 => deleted_at: Option<i64> [strict_optional],
        /// Engine-record node order, independent of Registry-copy membership.
        4_257_184_827 => node_ids: Vec<Principal> [required],
    }
}

/// Decode exactly one vector of selected engine views within explicit budgets.
///
/// Unknown fields are fully validated and discarded; both total and skipped work
/// are charged. Selected optional fields accept omission or wire `opt` values;
/// malformed present values never become `None`. No encoding implementation is
/// required for the response records. Empty output means only "not returned".
///
/// # Errors
/// Returns typed framing, type, resource or allocation refusals. The constructed
/// wire depth is limited to 16, even for skipped values. Opaque references and
/// future wire constructors are unsupported. Decoding authenticates nothing.
pub fn decode_get_engine_views_response(
    bytes: &[u8],
    limits: Limits,
) -> Result<Vec<EngineView>, Error> {
    candid::decode(bytes, limits)
}

#[cfg(test)]
mod tests;

#[cfg(feature = "cloud-engine-marketplace")]
mod arguments;
#[cfg(feature = "cloud-engine-marketplace")]
mod availability;
#[cfg(feature = "cloud-engine-marketplace")]
pub use arguments::{ListAvailableNodesPageArgs, encode_list_available_nodes_page_args};
#[cfg(feature = "cloud-engine-marketplace")]
mod marketplace;
#[cfg(feature = "cloud-engine-marketplace")]
pub use availability::{
    AvailableNode, AvailableNodesPage, AvailableNodesPageResult, EngineError,
    decode_list_available_nodes_page_response,
};
#[cfg(feature = "cloud-engine-marketplace")]
pub use marketplace::{
    MarketplaceEntry, NodeType, Price, PriceEntry, decode_list_marketplace_prices_response,
};
