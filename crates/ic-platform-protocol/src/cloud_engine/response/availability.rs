//! Available-node evidence without pagination or interpretation policy.

use super::marketplace::NodeType;
use crate::candid::projection::{absent, projection};
use crate::candid::{self, Decode, Error, Limits, Reader, Schema};
use candidate_types::Principal;
use serde::{Deserialize, Serialize};

projection! {
    /// Selected candidate node; eligibility is not a reservation or liveness claim.
    AvailableNode {
        /// Raw data-center scope; absent differs from present empty text.
        22_399 => dc: Option<String> [strict_optional],
        /// Source node class, without inferred hardware specifications.
        287_472_983 => node_type: NodeType [required],
        /// Canonical candidate identity.
        385_029_432 => node_id: Principal [required],
        /// Source provider identity.
        590_999_857 => provider: Principal [required],
        /// Source operator identity.
        1_148_394_404 => operator: Principal [required],
        /// Source chip-registration flag, not verified attestation.
        3_027_975_590 => sev_enabled: bool [required],
    }
}
projection! {
    /// One source page; equal versions do not establish an atomic inventory.
    AvailableNodesPage {
        /// Raw continuation cursor; caller owns traversal and cycle checks.
        1_224_901_875 => next: Option<Principal> [strict_optional],
        /// Source-reported Registry-copy version.
        1_679_652_694 => registry_version: u64 [required],
        /// Source node order, without uniqueness or admission policy.
        2_680_076_337 => nodes: Vec<AvailableNode> [required],
    }
}
projection! {
    /// Declared application errors; source text remains uninterpreted.
    enum EngineError {
        /// Source internal error detail.
        36_184_925 => Internal(String),
        /// Source controller error detail.
        266_990_956 => ControllerError(String),
        /// Source invalid-transition detail.
        1_759_258_444 => InvalidTransition(String),
        /// Source slug-conflict detail.
        1_843_796_157 => SlugConflict(String),
        /// Requested item not found.
        1_851_232_335 => NotFound,
        /// Source authorization refusal.
        2_471_582_292 => Unauthorized,
        /// Requested item already exists.
        2_488_679_284 => AlreadyExists,
        /// Requested item not vetted.
        3_243_640_289 => NotVetted,
        /// Source invalid-request detail.
        3_929_426_986 => BadRequest(String),
    }
}
projection! {
    /// Lowercase wire/Serde result with strict present-page admission.
    enum AvailableNodesPageResult {
        /// None means Registry copy unready; an empty page remains Some.
        #[serde(rename = "ok")]
        24_860 => Ok(Option<AvailableNodesPage>),
        /// Raw application failure, distinct from malformed wire input.
        #[serde(rename = "err")]
        5_048_165 => Err(EngineError),
    }
}

/// Decode exactly one available-node page result with shared resource budgets.
///
/// Unknown application/node-class variants refuse. A malformed present page
/// never becomes unready. Unselected fields, including IP text, are validated
/// and skipped under total and skipped-work limits without retaining their text.
/// No page traversal, retry, node admission or authentication is performed.
///
/// # Errors
/// Returns shared framing, type, arity, allocation and resource refusals.
pub fn decode_list_available_nodes_page_response(
    bytes: &[u8],
    limits: Limits,
) -> Result<AvailableNodesPageResult, Error> {
    candid::decode(bytes, limits)
}
