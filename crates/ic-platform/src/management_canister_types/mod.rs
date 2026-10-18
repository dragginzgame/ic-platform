//! External management-canister HTTP outcall types, preserving upstream identities.
//!
//! The exact 0.11.0 dependency retains required Candid formats, including pricing
//! and replication fields. Consumers own bounds, cycles, transforms, calls and
//! retry policy. This feature brings transitive Candid without selecting the
//! Platform codec, host parser, Agent or executor features.

pub use ic_management_canister_types::{
    HttpHeader, HttpMethod, HttpRequestArgs, HttpRequestResult, TransformArgs, TransformContext,
    TransformFunc,
};
