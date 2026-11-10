# External DTO compatibility contract

Coordinator-reviewed public API facts for isolated implementation/test authors.
The author must not read existing DTO bodies, comments, tests, codec mappings,
DID fixtures, supplier source, Git history or diffs. The coordinator extracted
only public declarations, traits, Serde names, feature gates and protocol facts;
no algorithm or expressive documentation is supplied.

## Scope

Replace the passive declarations in four facade modules listed below. Use
canonical crate::principal::Principal, owned crate::candid_codec::Nat and
Serde Deserialize. All listed types are public under their module. All listed
struct fields are public; enum named payload fields are public by Rust rules.
Keep declaration field/variant order and exact trait sets. Do not add Serialize,
Default, Copy, ordering or Hash where absent. Nat is unbounded, including above
u128. Byte vectors use ordinary sequence Serde, not serde_bytes. No validation,
normalization, cryptography, network, timer, storage or consumer policy.

Author only fresh unreferenced drafts at each module's replacement/mod.rs. Do
not read/change the compiled files; parent freezes old-owner observations first.
Parent will install each draft and append its existing owned codec mappings
unchanged. Do not reproduce codec macros in the draft. ICP retains private
account_identifier and callback modules and explicit public exports; do not read
or alter either. Use ordinary scope imports and freshly written concise docs.
Record exact inputs in docs/design/external-dtos-implementation.md. Scoped
rustfmt with skip_children is allowed; no Cargo before parent coordination.

## cmc_types

### NotifyTopUpArg (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
block_index: u64,
    canister_id: Principal,
~~~

### NotifyError (enum)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
Refunded {
        reason: String,
        block_index: Option<u64>,
    },
    Processing,
    TransactionTooOld(u64),
    InvalidTransaction(String),
    Other {
        error_code: u64,
        error_message: String,
    },
~~~

- Alias NotifyTopUpResult: Result<Nat, NotifyError>.

### IcpXdrConversionRate (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
timestamp_seconds: u64,
    xdr_permyriad_per_icp: u64,
~~~

### IcpXdrConversionRateResponse (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
data: IcpXdrConversionRate,
    hash_tree: Vec<u8>,
    certificate: Vec<u8>,
~~~

### NotifyMintCyclesArg (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
block_index: u64,
    to_subaccount: Option<Vec<u8>>,
    deposit_memo: Option<Vec<u8>>,
~~~

### NotifyMintCyclesSuccess (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
block_index: Nat,
    minted: Nat,
    balance: Nat,
~~~

- Alias NotifyMintCyclesResult: Result<NotifyMintCyclesSuccess, NotifyError>.

## icp_ledger_types

### Tokens (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
e8s: u64,
~~~

### TimeStamp (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
timestamp_nanos: u64,
~~~

### TransferArgs (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
memo: u64,
    amount: Tokens,
    fee: Tokens,
    from_subaccount: Option<Vec<u8>>,
    to: Vec<u8>,
    created_at_time: Option<TimeStamp>,
~~~

### TransferError (enum)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
BadFee {
        expected_fee: Tokens,
    },
    InsufficientFunds {
        balance: Tokens,
    },
    TxTooOld {
        allowed_window_nanos: u64,
    },
    TxCreatedInFuture,
    TxDuplicate {
        duplicate_of: u64,
    },
~~~

- Alias TransferResult: Result<u64, TransferError>.

### GetBlocksArgs (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
start: u64,
    length: u64,
~~~

### QueryArchiveError (enum)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
BadFirstBlockIndex {
        requested_index: u64,
        first_valid_index: u64,
    },
    Other {
        error_code: u64,
        error_message: String,
    },
~~~

- Alias QueryEncodedBlocksResult: Result<Vec<Vec<u8>>, QueryArchiveError>.

### ArchivedEncodedBlocksRange (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
callback: QueryEncodedBlocksFn,
    start: u64,
    length: u64,
~~~

### QueryEncodedBlocksResponse (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
certificate: Option<Vec<u8>>,
    blocks: Vec<Vec<u8>>,
    chain_length: u64,
    first_block_index: u64,
    archived_blocks: Vec<ArchivedEncodedBlocksRange>,
~~~

## cycles_ledger_types

### CreateCanisterArgs (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
from_subaccount: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    creation_args: Option<CmcCreateCanisterArgs>,
~~~

### CmcCreateCanisterArgs (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
settings: Option<CanisterSettings>,
    subnet_selection: Option<SubnetSelection>,
~~~

### EnvironmentVariable (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
name: String,
    value: String,
~~~

### LogVisibility (enum)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
#[serde(rename = "controllers")]
    Controllers,
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "allowed_viewers")]
    AllowedViewers(Vec<Principal>),
~~~

### CanisterSettings (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
controllers: Option<Vec<Principal>>,
    compute_allocation: Option<Nat>,
    memory_allocation: Option<Nat>,
    freezing_threshold: Option<Nat>,
    reserved_cycles_limit: Option<Nat>,
    wasm_memory_limit: Option<Nat>,
    wasm_memory_threshold: Option<Nat>,
    log_visibility: Option<LogVisibility>,
    environment_variables: Option<Vec<EnvironmentVariable>>,
~~~

### SubnetFilter (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
subnet_type: Option<String>,
~~~

### SubnetSelection (enum)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
Subnet {
        subnet: Principal,
    },
    Filter(SubnetFilter),
~~~

### CreateCanisterSuccess (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
block_id: Nat,
    canister_id: Principal,
~~~

### CreateCanisterError (enum)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
InsufficientFunds {
        balance: Nat,
    },
    TooOld,
    CreatedInFuture {
        ledger_time: u64,
    },
    TemporarilyUnavailable,
    Duplicate {
        duplicate_of: Nat,
        canister_id: Option<Principal>,
    },
    FailedToCreate {
        fee_block: Option<Nat>,
        refund_block: Option<Nat>,
        error: String,
    },
    GenericError {
        message: String,
        error_code: Nat,
    },
~~~

- Alias CreateCanisterResult: Result<CreateCanisterSuccess, CreateCanisterError>.

### WithdrawArgs (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
amount: Nat,
    from_subaccount: Option<Vec<u8>>,
    to: Principal,
    created_at_time: Option<u64>,
~~~

### RejectionCode (enum)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
NoError,
    CanisterError,
    SysTransient,
    DestinationInvalid,
    Unknown,
    SysFatal,
    CanisterReject,
~~~

### WithdrawError (enum)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
GenericError {
        message: String,
        error_code: Nat,
    },
    TemporarilyUnavailable,
    FailedToWithdraw {
        fee_block: Option<Nat>,
        rejection_code: RejectionCode,
        rejection_reason: String,
    },
    Duplicate {
        duplicate_of: Nat,
    },
    BadFee {
        expected_fee: Nat,
    },
    InvalidReceiver {
        receiver: Principal,
    },
    CreatedInFuture {
        ledger_time: u64,
    },
    TooOld,
    InsufficientFunds {
        balance: Nat,
    },
~~~

- Alias WithdrawResult: Result<Nat, WithdrawError>.

### BlockWithId (struct)

Traits/attributes: #[cfg(feature = "icrc-ledger-types")] #[derive(Clone, Debug, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
id: Nat,
    block: crate::icrc_ledger_types::Value,
~~~

### ArchivedBlocks (struct)

Traits/attributes: #[cfg(feature = "icrc-ledger-types")] #[derive(Clone, Debug, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
args: Vec<crate::icrc_ledger_types::GetBlocksRequest>,
    callback: crate::icrc_ledger_types::QueryArchiveFn<
        Vec<crate::icrc_ledger_types::GetBlocksRequest>,
        GetBlocksResult,
    >,
~~~

### GetBlocksResult (struct)

Traits/attributes: #[cfg(feature = "icrc-ledger-types")] #[derive(Clone, Debug, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
log_length: Nat,
    blocks: Vec<BlockWithId>,
    archived_blocks: Vec<ArchivedBlocks>,
~~~

## certified_assets_types

### GetArgs (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
key: String,
    accept_encodings: Vec<String>,
~~~

### GetResult (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
content: Vec<u8>,
    content_type: String,
    content_encoding: String,
    sha256: Option<Vec<u8>>,
    total_length: Nat,
~~~

### GetChunkArgs (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
key: String,
    content_encoding: String,
    index: Nat,
    sha256: Option<Vec<u8>>,
~~~

### GetChunkResult (struct)

Traits/attributes: #[derive(Clone, Debug, PartialEq, Eq, Deserialize)].

Ordered field/variant facts (Rust type notation):

~~~text
content: Vec<u8>,
~~~

## Public test and schema facts

A separate test author may read this contract only, not old/replacement DTOs,
old tests or DID text. Author a new native public integration target at
`crates/ic-platform/tests/external_dtos_consumer.rs` and an input record at
`docs/design/external-dtos-public-tests-implementation.md`. Parent owns manifest,
Cargo execution and frozen original observations. Cover each listed declaration,
every enum variant, absent/present options, u64 extrema, Nat above u128, empty
and nonempty arbitrary byte vectors/text, and exact callback schema behavior.
Do not call account_identifier, Value::hash or any cryptographic operation.

All declarations have owned Encode and lifetime-generic Decode supplied by
unchanged mappings. Public API: encode_one(&value, maximum_bytes),
decoder::decode_one::<T>(&bytes, Limits { bytes, header, types, work, allocation });
all Limits fields are usize. Nat::from(u64/u128) and decimal FromStr are available.
Nat Deserialize also accepts nonnegative JSON integers and decimal strings.
Principal::anonymous(), management_canister(), from_slice(&[u8]) (at most 29)
and to_text() are available. JSON Principal uses its text form. All 33 declaration
traits above intentionally omit Serialize. Use explicit JSON/CBOR inputs and
owned Candid bytes, not invented Serialize implementations. A deterministic
EXTERNAL_DTOS_OBSERVATIONS= JSON transcript may include input JSON text, input
CBOR bytes, decoded public field observations/Debug and Candid bytes. Parent
will freeze it after baseline execution. Keep observations bounded and grouped.

ICP QueryEncodedBlocksFn remains an existing public tuple wrapper around Func
{ principal: Principal, method: String }, with new(Principal, String), From<Func>,
Clone/Debug/Eq/PartialEq/Deserialize/Encode/Decode. It carries exactly one
GetBlocksArgs argument and one QueryEncodedBlocksResult result, query mode.
Func Serde accepts an owned byte buffer: marker 5, unsigned LEB128 UTF-8 method
length, method bytes, then remaining Principal raw bytes. Its tuple wrapper
requires deserialize_newtype_struct visitation. The earlier management-test
fixture correction is applicable; no implementation body is supplied here.

The three gated Cycles records use the existing ICRC Value, GetBlocksRequest
and QueryArchiveFn<Args,Result>. GetBlocksRequest has public start/length: Nat.
QueryArchiveFn has public canister_id: Principal and method: String fields, plus
new(Principal, String). Its Candid signature here takes one vector of ranges and
returns one Cycles GetBlocksResult in query mode. Value's public variants are
Blob(serde_bytes::ByteBuf), Text(String), Nat(Nat), Nat64(u64), Int(owned Int),
Array(Vec<Value>) and Map(BTreeMap<String, Value>); preserve Nat64 distinctly.
Provider records intentionally lack PartialEq/Eq; assert their public fields or
re-encoded bytes. Do not narrow them to the standard ICRC3Value schema.

### Independently authored small DID fixtures

The test author may also author four fresh files at
`crates/ic-platform/tests/fixtures/owned-{cmc,icp,cycles,assets}.did`. These are
minimal schema oracles, not service implementations. Do not read or copy the
old external-*.did files. Parent compares old/new schemas before retirement.
The original interface fact sources (reviewed locally by the coordinator) were:
IC f83a7646c20ce18100eb87b7865c12852bef8169 for CMC/ICP; Cycles Ledger
2faa00a86115d92cce599345b1f5ae04f0f7f2bb; SDK asset interface
271392a3b7746df539925a6333b4cd7ddc9f1972. Only the following facts and the
Rust declaration tables above are supplied to authors.

Translate the fields/variants above into Candid records/variants without changing
names (except the listed Serde renames). u64 means nat64, u8 means nat8,
String means text, Nat means nat, Principal means principal, Vec means vec,
Option means opt; Result<T,E> is variant { Ok: T; Err: E }. Unit variant payloads
are null. Numeric values are unbounded for nat/int. Keep all public type names
available in the DID when listed above; helper alias names may be chosen freely.
Candid field declaration order is not semantic; do not mimic old DID formatting.

Selected services, each with the exact arity below:

| Fixture | Method | Arguments | Results | Mode |
| --- | --- | --- | --- | --- |
| CMC | notify_top_up | NotifyTopUpArg | NotifyTopUpResult | update |
| CMC | notify_mint_cycles | NotifyMintCyclesArg | NotifyMintCyclesResult | update |
| CMC | get_icp_xdr_conversion_rate | none | IcpXdrConversionRateResponse | query |
| ICP | transfer | TransferArgs | TransferResult | update |
| ICP | query_encoded_blocks | GetBlocksArgs | QueryEncodedBlocksResponse | query |
| Cycles | create_canister | CreateCanisterArgs | CreateCanisterResult | update |
| Cycles | withdraw | WithdrawArgs | WithdrawResult | update |
| Cycles | icrc1_balance_of | Account | nat | query |
| Cycles | icrc1_transfer | ICRC transfer record below | variant Ok nat / Err transfer error below | update |
| Cycles | icrc1_fee | none | nat | query |
| Cycles | icrc3_get_tip_certificate | none | opt DataCertificate | query |
| Cycles | icrc3_get_blocks | vec GetBlocksRequest | GetBlocksResult | query |
| Assets | get | GetArgs | GetResult | query |
| Assets | get_chunk | GetChunkArgs | GetChunkResult | query |

ICP archive callback is func(GetBlocksArgs) -> (QueryEncodedBlocksResult) query.
Cycles archive callback is func(vec GetBlocksRequest) -> (GetBlocksResult) query.
Cycles Value protocol variants: Int int; Map vec record { text; Value }; Nat nat;
Nat64 nat64; Blob vec nat8; Text text; Array vec Value. Account fields are owner
principal and subaccount opt vec nat8. ICRC transfer record fields: from_subaccount
opt vec nat8; to Account; amount nat; fee opt nat; memo opt vec nat8;
created_at_time opt nat64. Its error variants: BadFee {expected_fee nat},
BadBurn {min_burn_amount nat}, InsufficientFunds {balance nat}, TooOld unit,
CreatedInFuture {ledger_time nat64}, Duplicate {duplicate_of nat},
TemporarilyUnavailable unit, GenericError {message text; error_code nat}.
DataCertificate has certificate and hash_tree fields, both vec nat8.

Do not add unselected service methods or initialization constructors. These
fixtures specify the supported method subset, never the entire remote service.
