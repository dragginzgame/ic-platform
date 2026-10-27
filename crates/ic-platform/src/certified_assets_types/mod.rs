//! Native host certified-asset get/get_chunk contracts; decoding is not certification.
//!
//! Passive external Candid contracts; see docs/external-contracts.md for pinned
//! schemas and provenance. Decoding does not authenticate a response or authorize
//! a payment, retry, topology decision or resource allocation.

use candid::{CandidType, Nat};
use serde::Deserialize;

/// GetArgs.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct GetArgs {
    /// External Candid `key` field.
    pub key: String,
    /// External Candid `accept_encodings` field.
    pub accept_encodings: Vec<String>,
}

/// GetResult.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct GetResult {
    /// External Candid `content` field.
    pub content: Vec<u8>,
    /// External Candid `content_type` field.
    pub content_type: String,
    /// External Candid `content_encoding` field.
    pub content_encoding: String,
    /// External Candid `sha256` field.
    pub sha256: Option<Vec<u8>>,
    /// External Candid `total_length` field.
    pub total_length: Nat,
}

/// GetChunkArgs.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct GetChunkArgs {
    /// External Candid `key` field.
    pub key: String,
    /// External Candid `content_encoding` field.
    pub content_encoding: String,
    /// External Candid `index` field.
    pub index: Nat,
    /// External Candid `sha256` field.
    pub sha256: Option<Vec<u8>>,
}

/// GetChunkResult.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct GetChunkResult {
    /// External Candid `content` field.
    pub content: Vec<u8>,
}
