//! One bounded management canister randomness request.

#[cfg(any(test, target_arch = "wasm32"))]
mod operation;
#[cfg(target_arch = "wasm32")]
pub use operation::request;

use crate::call::{Rejection, SubmitError};
use ic_platform_protocol::raw_rand as wire;

/// Failure of one request, preserving transport uncertainty and wire errors.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// The call could not be submitted.
    Submission(SubmitError),
    /// The call received a reject code and message.
    Rejected(Rejection),
    /// The IC stopped waiting without establishing the outcome.
    OutcomeUnknown(Rejection),
    /// The request or reply failed the canonical raw_rand wire profile.
    Frame(wire::Error),
}
