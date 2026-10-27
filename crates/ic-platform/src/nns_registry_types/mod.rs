//! NNS Registry subnet lookup; independent Rust declarations of the external protocol.
//!
//! Passive external Candid contracts; see docs/external-contracts.md for pinned
//! schemas and provenance. Decoding does not authenticate a response or authorize
//! a payment, retry, topology decision or resource allocation.

use candid::{CandidType, Principal};
use serde::Deserialize;

/// Registry subnet lookup request; callers explicitly select Some(target).
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct GetSubnetForCanisterRequest {
    /// External Candid `principal` field.
    pub principal: Option<Principal>,
}

/// GetSubnetForCanisterResponseOk.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct GetSubnetForCanisterResponseOk {
    /// External Candid `subnet_id` field.
    pub subnet_id: Option<Principal>,
}

/// External Candid GetSubnetForCanisterResponse.
pub type GetSubnetForCanisterResponse = Result<GetSubnetForCanisterResponseOk, String>;
