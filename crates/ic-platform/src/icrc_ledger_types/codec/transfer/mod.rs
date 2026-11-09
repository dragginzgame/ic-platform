use super::super::account::{Account, Subaccount};
use super::super::transfer::*;
use crate::candid_codec::Nat;
crate::candid_record!(TransferArg {25979 => to: Account, 5094982 => fee: Option<NumTokens>, 1213809850 => memo: Option<Memo>, 1835347746 => from_subaccount: Option<Subaccount>, 3258775938 => created_at_time: Option<u64>, 3573748184 => amount: NumTokens});
crate::candid_variant!(TransferError {260448849 => GenericError{2584819143 => message: String, 3601615940 => error_code: Nat}, 658180290 => TemporarilyUnavailable, 1093787796 => BadBurn{2765068699 => min_burn_amount: NumTokens}, 1122632043 => Duplicate{326934155 => duplicate_of: BlockIndex}, 2142953889 => BadFee{3725446591 => expected_fee: NumTokens}, 2608432112 => CreatedInFuture{2709806499 => ledger_time: u64}, 3373249171 => TooOld, 4206284395 => InsufficientFunds{596483356 => balance: NumTokens}});
