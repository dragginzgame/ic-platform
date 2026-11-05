//! Selected upstream ICRC implementation; see ../NOTICE.md.
#![expect(
    missing_docs,
    reason = "Selected upstream ICRC declarations contain undocumented public fields and variants."
)]

use candid::{CandidType, Deserialize, Nat};
use serde::Serialize;

#[derive(Debug, CandidType, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorInfo {
    pub description: String,
}

#[derive(Debug, CandidType, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Icrc21Error {
    UnsupportedCanisterCall(ErrorInfo),
    ConsentMessageUnavailable(ErrorInfo),
    InsufficientPayment(ErrorInfo),
    GenericError {
        error_code: Nat,
        description: String,
    },
}
