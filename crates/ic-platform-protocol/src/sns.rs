//! Selected SNS Governance neuron observations, without collection or entitlement policy.
//! Decode success authenticates nothing. IDs, permission codes and amounts remain raw.
pub use crate::candid::encode::EncodeError;
use crate::candid::{
    self, Decode, Error, Limits, Reader, Schema,
    encode::{Encode, Writer},
    projection::{absent, projection},
};
use candidate_types::Principal;
use serde::{Deserialize, Serialize};

projection! {
    /// Raw neuron ID record; callers validate length and requested-ID equality.
    both NeuronId {
        /// Exact ID bytes; no 32-byte or ownership assumption.
        23515 => id: Vec<u8> [required],
    }
}

projection! {
    /// One get_neuron argument; absence is explicit.
    encoding GetNeuronArgs {
        /// Requested optional ID.
        4246264679 => neuron_id: Option<NeuronId>,
    }
}

projection! {
    /// One list_neurons argument; pagination remains caller-owned.
    encoding ListNeuronsArgs {
        /// Optional selected principal, without an ownership inference.
        1213075014 => of_principal: Option<Principal>,
        /// Requested page limit; response limits remain explicit.
        1963007803 => limit: u32,
        /// Optional raw page cursor.
        4027886598 => start_page_at: Option<NeuronId>,
    }
}

projection! {
    /// Raw permission entry, including unknown signed codes.
    NeuronPermission {
        /// Optional principal; absence does not create a default identity.
        302796462 => principal: Option<Principal> [strict_optional],
        /// Raw signed codes, retaining unknown values and order.
        2192336682 => permission_type: Vec<i32> [required],
    }
}

projection! {
    /// Distinct delay versus absolute dissolve time.
    enum DissolveState {
        /// Raw delay in seconds.
        1620537965 => DissolveDelaySeconds(u64),
        /// Raw absolute timestamp in seconds.
        4238420994 => WhenDissolvedTimestampSeconds(u64),
    }
}

projection! {
    /// Selected raw neuron evidence; this is not a full neuron export.
    Neuron {
        /// Optional returned ID.
        23515 => id: Option<NeuronId> [strict_optional],
        /// Optional raw staked_maturity_e8s_equivalent.
        41099906 => staked_maturity_e8s_equivalent: Option<u64> [strict_optional],
        /// Raw ordered permissions.
        248806532 => permissions: Vec<NeuronPermission> [required],
        /// Raw maturity_e8s_equivalent; no derived entitlement.
        1029637143 => maturity_e8s_equivalent: u64 [required],
        /// Raw cached_neuron_stake_e8s; no derived entitlement.
        1257408332 => cached_neuron_stake_e8s: u64 [required],
        /// Raw created_timestamp_seconds; no derived entitlement.
        1392680831 => created_timestamp_seconds: u64 [required],
        /// Optional raw source_nns_neuron_id.
        1794496375 => source_nns_neuron_id: Option<u64> [strict_optional],
        /// Optional source auto-stake flag.
        1895351622 => auto_stake_maturity: Option<bool> [strict_optional],
        /// Raw aging_since_timestamp_seconds; no derived entitlement.
        2399567118 => aging_since_timestamp_seconds: u64 [required],
        /// Optional distinct dissolve state.
        3084775299 => dissolve_state: Option<DissolveState> [strict_optional],
        /// Raw voting_power_percentage_multiplier; no derived entitlement.
        3244889588 => voting_power_percentage_multiplier: u64 [required],
        /// Optional raw vesting_period_seconds.
        3380002190 => vesting_period_seconds: Option<u64> [strict_optional],
        /// Raw neuron_fees_e8s; no derived entitlement.
        3439871066 => neuron_fees_e8s: u64 [required],
    }
}

projection! {
    /// Governance refusal, distinct from malformed input or missing result.
    GovernanceError {
        /// Exact bounded diagnostic text.
        1389388560 => error_message: String [required],
        /// Raw signed error code, including unknown values.
        3790638545 => error_type: i32 [required],
    }
}

projection! {
    /// Governance response result.
    enum GetNeuronResult {
        /// Governance refused the observation.
        106380200 => Error(GovernanceError),
        /// Selected neuron observation.
        2767950995 => Neuron(Neuron),
    }
}

projection! {
    /// Optional result; missing result remains None.
    GetNeuronResponse {
        /// Absent or explicitly returned result.
        142895325 => result: Option<GetNeuronResult> [strict_optional],
    }
}

projection! {
    /// One bounded page; ordering and pagination consistency remain external.
    ListNeuronsResponse {
        /// Raw returned neurons in source order.
        3923068064 => neurons: Vec<Neuron> [required],
    }
}

/// Decode exactly one GetNeuronResponse with explicit input, collection, work and storage limits.
/// # Errors
/// Refuses malformed, incompatible, truncated, trailing or over-budget input.
pub fn decode_get_neuron(bytes: &[u8], limits: Limits) -> Result<GetNeuronResponse, Error> {
    candid::decode(bytes, limits)
}

/// Decode exactly one ListNeuronsResponse with explicit input, collection, work and storage limits.
/// # Errors
/// Refuses malformed, incompatible, truncated, trailing or over-budget input.
pub fn decode_list_neurons(bytes: &[u8], limits: Limits) -> Result<ListNeuronsResponse, Error> {
    candid::decode(bytes, limits)
}
