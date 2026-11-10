//! Certified asset retrieval request and response declarations.

use crate::candid_codec::Nat;
use serde::Deserialize;

/// An asset key and the content encodings accepted by the caller.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct GetArgs {
    /// The requested asset key.
    pub key: String,
    /// The supplied accepted encodings.
    pub accept_encodings: Vec<String>,
}

/// The initial asset content and its reported metadata.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct GetResult {
    /// The returned content bytes.
    pub content: Vec<u8>,
    /// The reported content type.
    pub content_type: String,
    /// The reported content encoding.
    pub content_encoding: String,
    /// Optional opaque SHA-256 digest bytes.
    pub sha256: Option<Vec<u8>>,
    /// The reported total content length.
    pub total_length: Nat,
}

/// Parameters selecting an asset chunk.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct GetChunkArgs {
    /// The requested asset key.
    pub key: String,
    /// The requested content encoding.
    pub content_encoding: String,
    /// The requested chunk index.
    pub index: Nat,
    /// Optional opaque SHA-256 digest bytes supplied by the caller.
    pub sha256: Option<Vec<u8>>,
}

/// Content returned for one asset chunk.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct GetChunkResult {
    /// The returned content bytes.
    pub content: Vec<u8>,
}

// Numeric labels are checked against the independent pinned Candid schemas.
crate::candid_record!(GetArgs {
    5343647 => key: String,
    2245317897 => accept_encodings: Vec<String>,
});
crate::candid_record!(GetResult {
    427265337 => content: Vec<u8>,
    486188135 => sha256: Option<Vec<u8>>,
    1400566464 => content_type: String,
    3425830873 => content_encoding: String,
    3911979841 => total_length: Nat,
});
crate::candid_record!(GetChunkArgs {
    5343647 => key: String,
    486188135 => sha256: Option<Vec<u8>>,
    3189021458 => index: Nat,
    3425830873 => content_encoding: String,
});
crate::candid_record!(GetChunkResult {
    427265337 => content: Vec<u8>,
});
