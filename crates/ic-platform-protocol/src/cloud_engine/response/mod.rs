//! Decode-only selected response records.

#[cfg(test)]
mod tests;

use crate::candid::{
    self, Decode, Error, Limits, Reader, Schema,
    projection::{absent, projection},
};
use candidate_types::Principal;
use serde::{Deserialize, Serialize};

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
