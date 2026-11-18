//! Registry canister placement: explicit target encoding and bounded result decoding.
use crate::{
    candid::{
        self, Decode, Error, Limits, Reader, Schema,
        projection::{absent, projection},
    },
    cloud_engine::EncodeError,
};
use candidate_types::Principal;
use serde::{Deserialize, Serialize};

projection! {
    /// A source lookup result, without a canister-existence or liveness claim.
    Placement {
        /// Missing subnet remains distinct from Registry rejection or malformed input.
        1_097_286_461 => subnet_id: Option<Principal> [strict_optional],
    }
}

projection! {
    /// Application-level Registry result; wire errors are returned separately.
    enum PlacementResult {
        /// Successful lookup, possibly without a subnet.
        17_724 => Ok(Placement),
        /// Raw Registry rejection text.
        3_456_837 => Err(String),
    }
}

/// Decode exactly one result with shared framing, traversal and allocation budgets.
///
/// # Errors
/// Refuses malformed/incompatible data, trailing bytes and exhausted bounds.
pub fn decode_get_subnet_for_canister_response(
    bytes: &[u8],
    limits: Limits,
) -> Result<PlacementResult, Error> {
    candid::decode(bytes, limits)
}

/// Encode one record containing the exact selected canister principal as Some.
///
/// # Errors
/// Refuses the complete output bound before allocation, or allocation failure.
pub fn encode_get_subnet_for_canister_args(
    principal: Principal,
    max_bytes: usize,
) -> Result<Vec<u8>, EncodeError> {
    // record { principal: opt principal }; field id is idl_hash("principal").
    const HEADER: &[u8] = b"DIDL\x02\x6c\x01\xae\x9d\xb1\x90\x01\x01\x6e\x68\x01\x00\x01\x01";
    let size = HEADER.len() + 1 + principal.as_slice().len();
    if size > max_bytes {
        return Err(EncodeError::OutputLimit { max_bytes });
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(size)
        .map_err(|_| EncodeError::Allocation)?;
    output.extend_from_slice(HEADER);
    output.push(principal.as_slice().len() as u8);
    output.extend_from_slice(principal.as_slice());
    Ok(output)
}
