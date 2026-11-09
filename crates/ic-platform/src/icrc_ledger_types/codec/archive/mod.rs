use super::super::archive::*;
use crate::{candid_codec::Nat, principal::Principal};
crate::candid_record!(GetArchivesArgs {1136829802 => from: Option<Principal>});
crate::candid_record!(ICRC3ArchiveInfo {5047259 => end: Nat, 1313628723 => canister_id: Principal, 2215343202 => start: Nat});
