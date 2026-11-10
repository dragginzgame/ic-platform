//! Passive failures returned by consent-message services.

use crate::candid_codec::Nat;
use serde::{Deserialize, Serialize};

/// A service-supplied failure description.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// The description preserved verbatim.
    pub description: String,
}

/// A consent-message service refusal in its external representation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Icrc21Error {
    /// The service does not support describing the requested call.
    UnsupportedCanisterCall(ErrorInfo),
    /// No consent message is available for the request.
    ConsentMessageUnavailable(ErrorInfo),
    /// The service reports insufficient payment.
    InsufficientPayment(ErrorInfo),
    /// A service-defined error code and description.
    GenericError {
        /// The unbounded error code.
        error_code: Nat,
        /// The description preserved verbatim.
        description: String,
    },
}
