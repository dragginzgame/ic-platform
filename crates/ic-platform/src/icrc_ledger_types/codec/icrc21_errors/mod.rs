use super::super::icrc21::*;
use crate::candid_codec::Nat;
crate::candid_record!(ErrorInfo {1595738364 => description: String});
crate::candid_variant!(Icrc21Error {260448849 => GenericError{1595738364 => description: String, 3601615940 => error_code: Nat}, 752613667 => InsufficientPayment(ErrorInfo), 1019593370 => UnsupportedCanisterCall(ErrorInfo), 4060111587 => ConsentMessageUnavailable(ErrorInfo)});
