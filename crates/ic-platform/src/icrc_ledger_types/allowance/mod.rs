//! Passive allowance request and response records.

use super::account::Account;
use crate::candid_codec::Nat;
use serde::{Deserialize, Serialize};

/// Identifies the account and spender whose allowance is requested.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowanceArgs {
    /// The account granting an allowance.
    pub account: Account,
    /// The spender associated with that allowance.
    pub spender: Account,
}

/// An allowance amount and optional expiration reported by a ledger.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Allowance {
    /// The unbounded amount reported as available to the spender.
    pub allowance: Nat,
    /// The optional expiration timestamp, without local time interpretation.
    #[serde(default)]
    pub expires_at: Option<u64>,
}
