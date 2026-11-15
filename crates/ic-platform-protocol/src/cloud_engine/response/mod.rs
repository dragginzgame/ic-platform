//! Decode-only selected response records.

use crate::candid::{self, Decode, Error, Limits, Reader, Schema};
use ic_platform_types::Principal;
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
