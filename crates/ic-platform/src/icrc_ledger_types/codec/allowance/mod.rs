use super::super::Account;
use super::super::allowance::*;
use crate::candid_codec::Nat;
crate::candid_record!(AllowanceArgs {2707029165 => account: Account, 3868658507 => spender: Account});
crate::candid_record!(Allowance {3230440920 => allowance: Nat, 3680359390 => expires_at: Option<u64>});
