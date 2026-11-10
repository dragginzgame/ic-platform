//! Cycles ledger creation, withdrawal and block-response declarations.

use crate::candid_codec::Nat;
use crate::principal::Principal;
use serde::Deserialize;

/// Parameters for creating a canister using ledger cycles.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CreateCanisterArgs {
    /// The optional source subaccount bytes.
    pub from_subaccount: Option<Vec<u8>>,
    /// The optional creation-time value.
    pub created_at_time: Option<u64>,
    /// The supplied cycle amount.
    pub amount: Nat,
    /// Optional cycles-minting service creation parameters.
    pub creation_args: Option<CmcCreateCanisterArgs>,
}

/// Creation settings and subnet selection supplied to the cycles-minting service.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CmcCreateCanisterArgs {
    /// Optional canister settings.
    pub settings: Option<CanisterSettings>,
    /// Optional subnet selection.
    pub subnet_selection: Option<SubnetSelection>,
}

/// A named environment value supplied during canister creation.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EnvironmentVariable {
    /// The variable name.
    pub name: String,
    /// The variable value.
    pub value: String,
}

/// A requested canister-log visibility setting.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum LogVisibility {
    /// Access for the canister's controllers.
    #[serde(rename = "controllers")]
    Controllers,
    /// Public access.
    #[serde(rename = "public")]
    Public,
    /// Access for the listed principals.
    #[serde(rename = "allowed_viewers")]
    AllowedViewers(Vec<Principal>),
}

/// Optional canister settings carried by a cycles ledger creation request.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CanisterSettings {
    /// The optional controller list.
    pub controllers: Option<Vec<Principal>>,
    /// The optional compute allocation.
    pub compute_allocation: Option<Nat>,
    /// The optional memory allocation.
    pub memory_allocation: Option<Nat>,
    /// The optional freezing threshold.
    pub freezing_threshold: Option<Nat>,
    /// The optional reserved-cycle limit.
    pub reserved_cycles_limit: Option<Nat>,
    /// The optional Wasm-memory limit.
    pub wasm_memory_limit: Option<Nat>,
    /// The optional Wasm-memory threshold.
    pub wasm_memory_threshold: Option<Nat>,
    /// The optional log visibility.
    pub log_visibility: Option<LogVisibility>,
    /// The optional environment-variable list.
    pub environment_variables: Option<Vec<EnvironmentVariable>>,
}

/// A subnet filter supplied during canister creation.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct SubnetFilter {
    /// The optional subnet-type name.
    pub subnet_type: Option<String>,
}

/// A subnet choice or filter supplied during canister creation.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum SubnetSelection {
    /// An explicitly named subnet.
    Subnet {
        /// The requested subnet principal.
        subnet: Principal,
    },
    /// A filter used for subnet selection.
    Filter(SubnetFilter),
}

/// The canister and ledger block reported after successful creation.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CreateCanisterSuccess {
    /// The reported ledger block identifier.
    pub block_id: Nat,
    /// The created canister's principal.
    pub canister_id: Principal,
}

/// An error reported for a cycles ledger canister-creation request.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum CreateCanisterError {
    /// The available funds were insufficient.
    InsufficientFunds {
        /// The reported balance.
        balance: Nat,
    },
    /// The request was reported as too old.
    TooOld,
    /// The request timestamp was reported as being in the future.
    CreatedInFuture {
        /// The reported ledger time.
        ledger_time: u64,
    },
    /// The service was temporarily unavailable.
    TemporarilyUnavailable,
    /// The request was reported as a duplicate.
    Duplicate {
        /// The original request's block identifier.
        duplicate_of: Nat,
        /// The optionally reported canister principal.
        canister_id: Option<Principal>,
    },
    /// Canister creation failed.
    FailedToCreate {
        /// The optional fee block identifier.
        fee_block: Option<Nat>,
        /// The optional refund block identifier.
        refund_block: Option<Nat>,
        /// The reported diagnostic text.
        error: String,
    },
    /// Another service error.
    GenericError {
        /// The reported diagnostic text.
        message: String,
        /// The reported error code.
        error_code: Nat,
    },
}

/// The outcome of a cycles ledger canister-creation request.
pub type CreateCanisterResult = Result<CreateCanisterSuccess, CreateCanisterError>;

/// Parameters supplied for a cycles withdrawal.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct WithdrawArgs {
    /// The cycle amount to withdraw.
    pub amount: Nat,
    /// The optional source subaccount bytes.
    pub from_subaccount: Option<Vec<u8>>,
    /// The receiving canister principal.
    pub to: Principal,
    /// The optional creation-time value.
    pub created_at_time: Option<u64>,
}

/// A rejection category reported by the cycles ledger protocol.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum RejectionCode {
    /// No error was reported.
    NoError,
    /// A canister error was reported.
    CanisterError,
    /// A transient system error was reported.
    SysTransient,
    /// An invalid destination was reported.
    DestinationInvalid,
    /// The category was reported as unknown.
    Unknown,
    /// A fatal system error was reported.
    SysFatal,
    /// A canister rejection was reported.
    CanisterReject,
}

/// An error reported for a cycles withdrawal.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum WithdrawError {
    /// A service error with a code and diagnostic text.
    GenericError {
        /// The reported diagnostic text.
        message: String,
        /// The reported error code.
        error_code: Nat,
    },
    /// The service was temporarily unavailable.
    TemporarilyUnavailable,
    /// Delivery of the withdrawal failed.
    FailedToWithdraw {
        /// The optional fee block identifier.
        fee_block: Option<Nat>,
        /// The reported rejection category.
        rejection_code: RejectionCode,
        /// The reported rejection reason.
        rejection_reason: String,
    },
    /// The withdrawal was reported as a duplicate.
    Duplicate {
        /// The original withdrawal's block identifier.
        duplicate_of: Nat,
    },
    /// The supplied fee was not accepted.
    BadFee {
        /// The reported expected fee.
        expected_fee: Nat,
    },
    /// The receiving principal was not accepted.
    InvalidReceiver {
        /// The reported receiver principal.
        receiver: Principal,
    },
    /// The request timestamp was reported as being in the future.
    CreatedInFuture {
        /// The reported ledger time.
        ledger_time: u64,
    },
    /// The request was reported as too old.
    TooOld,
    /// The available funds were insufficient.
    InsufficientFunds {
        /// The reported balance.
        balance: Nat,
    },
}

/// The block identifier or error reported for a cycles withdrawal.
pub type WithdrawResult = Result<Nat, WithdrawError>;

/// An ICRC block paired with its ledger identifier.
#[cfg(feature = "icrc-ledger-types")]
#[derive(Clone, Debug, Deserialize)]
pub struct BlockWithId {
    /// The reported block identifier.
    pub id: Nat,
    /// The returned block value.
    pub block: crate::icrc_ledger_types::Value,
}

/// Archived block ranges and the function that serves them.
#[cfg(feature = "icrc-ledger-types")]
#[derive(Clone, Debug, Deserialize)]
pub struct ArchivedBlocks {
    /// The ranges supplied for the archive query.
    pub args: Vec<crate::icrc_ledger_types::GetBlocksRequest>,
    /// The archive query function reference.
    pub callback: crate::icrc_ledger_types::QueryArchiveFn<
        Vec<crate::icrc_ledger_types::GetBlocksRequest>,
        GetBlocksResult,
    >,
}

/// A cycles ledger block response with archive references.
#[cfg(feature = "icrc-ledger-types")]
#[derive(Clone, Debug, Deserialize)]
pub struct GetBlocksResult {
    /// The reported log length.
    pub log_length: Nat,
    /// The returned blocks and their identifiers.
    pub blocks: Vec<BlockWithId>,
    /// The returned archive query ranges.
    pub archived_blocks: Vec<ArchivedBlocks>,
}

// Numeric labels are checked against the independent pinned Candid schema.
crate::candid_record!(CreateCanisterArgs {
    1835347746 => from_subaccount: Option<Vec<u8>>,
    3258775938 => created_at_time: Option<u64>,
    3573748184 => amount: Nat,
    4132524221 => creation_args: Option<CmcCreateCanisterArgs>,
});
crate::candid_record!(CmcCreateCanisterArgs {
    1337893162 => subnet_selection: Option<SubnetSelection>,
    2336062691 => settings: Option<CanisterSettings>,
});
crate::candid_record!(EnvironmentVariable {
    834174833 => value: String,
    1224700491 => name: String,
});
crate::candid_record!(CanisterSettings {
    238856128 => freezing_threshold: Option<Nat>,
    241629288 => wasm_memory_threshold: Option<Nat>,
    356749323 => environment_variables: Option<Vec<EnvironmentVariable>>,
    570880087 => controllers: Option<Vec<Principal>>,
    1095112320 => reserved_cycles_limit: Option<Nat>,
    2033331437 => log_visibility: Option<LogVisibility>,
    3380736376 => wasm_memory_limit: Option<Nat>,
    3844961758 => memory_allocation: Option<Nat>,
    4174053672 => compute_allocation: Option<Nat>,
});
crate::candid_record!(SubnetFilter {
    3816945308 => subnet_type: Option<String>,
});
crate::candid_record!(CreateCanisterSuccess {
    430475981 => block_id: Nat,
    1313628723 => canister_id: Principal,
});
crate::candid_record!(WithdrawArgs {
    25979 => to: Principal,
    1835347746 => from_subaccount: Option<Vec<u8>>,
    3258775938 => created_at_time: Option<u64>,
    3573748184 => amount: Nat,
});
crate::candid_variant!(LogVisibility {
    570880087 => Controllers,
    1727234217 => Public,
    2235716810 => AllowedViewers(Vec<Principal>),
});
crate::candid_variant!(SubnetSelection {
    3116995544 => Filter(SubnetFilter),
    3466300157 => Subnet{ 2584155869 => subnet: Principal },
});
crate::candid_variant!(CreateCanisterError {
    260448849 => GenericError{ 2584819143 => message: String, 3601615940 => error_code: Nat },
    658180290 => TemporarilyUnavailable,
    1122632043 => Duplicate{ 326934155 => duplicate_of: Nat, 1313628723 => canister_id: Option<Principal> },
    2608432112 => CreatedInFuture{ 2709806499 => ledger_time: u64 },
    3210455508 => FailedToCreate{ 1932118984 => error: String, 2817723142 => refund_block: Option<Nat>, 3052565332 => fee_block: Option<Nat> },
    3373249171 => TooOld,
    4206284395 => InsufficientFunds{ 596483356 => balance: Nat },
});
crate::candid_variant!(RejectionCode {
    327055271 => NoError,
    424852481 => CanisterError,
    2231315613 => SysTransient,
    2600479273 => DestinationInvalid,
    2932624234 => Unknown,
    3163375575 => SysFatal,
    3224277766 => CanisterReject,
});
crate::candid_variant!(WithdrawError {
    189253090 => FailedToWithdraw{ 2800423587 => rejection_code: RejectionCode, 3052565332 => fee_block: Option<Nat>, 3241622202 => rejection_reason: String },
    260448849 => GenericError{ 2584819143 => message: String, 3601615940 => error_code: Nat },
    658180290 => TemporarilyUnavailable,
    1122632043 => Duplicate{ 326934155 => duplicate_of: Nat },
    2142953889 => BadFee{ 3725446591 => expected_fee: Nat },
    2446930726 => InvalidReceiver{ 3808249135 => receiver: Principal },
    2608432112 => CreatedInFuture{ 2709806499 => ledger_time: u64 },
    3373249171 => TooOld,
    4206284395 => InsufficientFunds{ 596483356 => balance: Nat },
});

#[cfg(feature = "icrc-ledger-types")]
crate::candid_record!(BlockWithId {23515 => id: Nat, 3036443981 => block: crate::icrc_ledger_types::Value});
#[cfg(feature = "icrc-ledger-types")]
crate::candid_record!(ArchivedBlocks {1081380189 => args: Vec<crate::icrc_ledger_types::GetBlocksRequest>, 2131139013 => callback: crate::icrc_ledger_types::QueryArchiveFn<Vec<crate::icrc_ledger_types::GetBlocksRequest>, GetBlocksResult>});
#[cfg(feature = "icrc-ledger-types")]
crate::candid_record!(GetBlocksResult {2799807105 => log_length: Nat, 2817142406 => blocks: Vec<BlockWithId>, 4171053571 => archived_blocks: Vec<ArchivedBlocks>});
