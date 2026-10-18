//! Explicit native IC Agent transport, identity and signed-request APIs.
//!
//! These are upstream identities and operations, with PEM identity support. This
//! feature does not select a runtime, endpoint, trust root, response limit, query
//! verification mode, deadline, retry or reconciliation policy for consumers.
//! Consumers retain those choices and upstream internal HTTP retry behavior.
//! Use `crate::certification` for certificate/hash-tree identities and operations.
//! [`lookup_value`] only reads a certificate tree; it does not authenticate it.
//! [`signed_update_inspect`] compares envelope content; it does not verify its
//! signature or authorize replay. This module and Agent are absent on Wasm.
//! Delegated identity constructors retain upstream chain validation; unchecked
//! construction does not authenticate a chain or enforce expiry/target policy.
//! Call response variants grant no permission to poll, retry or resubmit.

pub use ic_agent::agent::agent_error::{HttpErrorPayload, TransportError};
pub use ic_agent::agent::signed::SignedUpdate;
pub use ic_agent::agent::{
    AgentBuilder, CallResponse, Envelope, EnvelopeContent, RejectCode, RejectResponse,
    ReplyResponse, RequestStatusResponse, signed_update_inspect,
};
pub use ic_agent::identity::{
    AnonymousIdentity, BasicIdentity, DelegatedIdentity, Delegation, Prime256v1Identity,
    Secp256k1Identity, SignedDelegation,
};
pub use ic_agent::{
    Agent, AgentError, Identity, RequestId, TransportCallResponse, lookup_value, to_request_id,
};
