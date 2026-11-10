# Remaining passive ICRC DTO compatibility contract

Specification-only input for independently implementing the remaining passive
ICRC declarations/helpers. Do not read the current or historical target modules,
their tests, supplier implementations or copied documentation. The coordinator
reviewed the existing public interface separately. Admitted policy is unchanged.

Write only these `crates/ic-platform/src/icrc_ledger_types/` modules:
`metadata/mod.rs`, `transfer/mod.rs`, `allowance/mod.rs`, `blocks/mod.rs`, and
`icrc21/{requests,responses,errors}/mod.rs`, plus a design implementation record.
Hold product writes until the coordinator captures public baseline observations;
drafts can live under `/tmp/icrc-dtos-draft`. No Cargo commands. No dependencies,
feature changes, new wrappers, type aliases replacing existing structs/enums,
crypto, codec mappings or value/hash changes. Account/archive were independently
replaced already; use their public names without reading their implementations.

## Common identity and serialization

`Nat` and `Int` come from `crate::candid_codec`, backed by current num-bigint.
`Nat.0` is BigUint; num_traits::ToPrimitive::to_u64 is available. Preserve
unbounded integer magnitudes until an explicitly requested narrowing method.
`ByteBuf` is serde_bytes::ByteBuf. It has From<Vec<u8>>, From<&[u8]>, into_vec,
as_slice and standard collection traits. Do not add new public conversions.
`Account` is the existing `super::account::Account`; Subaccount is its `[u8;32]`
alias. `ICRC3Value` and generic `QueryArchiveFn` are available via `super`.

All fields below are public in their listed order. Enum variants and their named
fields retain the listed order for Serde indices and diagnostics. All records and
enums use ordinary derived Serialize/Deserialize with exactly their Rust names;
no field/variant renaming, flattening, tag change, skip policy or custom serializer.
Explicit default attributes and transparent memo are noted below. Retain the
listed trait sets, standard derive behavior, and meaningful rustdoc for every
public field/variant. Remove blanket missing_docs allowances by supplying docs.
Own short explanatory prose rather than importing protocol documentation.

The default common trait set is Clone, Debug, PartialEq, Eq, Serialize,
Deserialize. Do not broaden traits or enforce application policy.

## Metadata (`metadata`)

MetadataValue enum, common traits, variants in order:
Nat(Nat), Int(Int), Text(String), Blob(ByteBuf).
Existing From conversions only:
- i64 and i128 -> Int, preserving signed extrema;
- u64 and u128 -> Nat, preserving unsigned extrema;
- Nat -> Nat (move without cloning magnitude);
- String -> Text (move); &str -> Text (owned copy);
- Vec<u8> -> Blob (move); &[u8] -> Blob (owned copy).
No From<Int> or From<ByteBuf> is exposed. A small local conversion macro may
remove repeated implementation boilerplate if clearer than handwritten repeats;
do not invent another abstraction or widen accepted input types.

## Transfer (`transfer`)

Preserve module aliases NumTokens = Nat and BlockIndex = Nat.
TransferArg, common traits, fields in order:
from_subaccount: Option<Subaccount> (serde default), to: Account,
fee: Option<NumTokens> (serde default), created_at_time: Option<u64> (serde
default), memo: Option<Memo> (serde default), amount: NumTokens.
Absent optional fields default None, but serialization always includes them.
Subaccounts remain arrays/sequences, not a serde_bytes adapter.

Memo is pub tuple struct Memo(pub ByteBuf), serde transparent; common traits plus
Hash, PartialOrd, Ord, Default. From<u64> encodes EXACTLY eight bytes in big-endian
order, including leading zeros. From<Vec<u8>>/From<ByteBuf> move the buffer;
From<Memo> for ByteBuf returns that buffer without cloning. No size policy here.

TransferError, common traits plus Display (NOT std::error::Error), variants:
BadFee { expected_fee: NumTokens }, BadBurn { min_burn_amount: NumTokens },
InsufficientFunds { balance: NumTokens }, TooOld,
CreatedInFuture { ledger_time: u64 }, TemporarilyUnavailable,
Duplicate { duplicate_of: BlockIndex },
GenericError { error_code: Nat, message: String }.
Display strings are public behavior and must remain exact:

| Variant | Display |
|---|---|
| BadFee | `transfer fee should be {expected_fee}` |
| BadBurn | `the minimum number of tokens to be burned is {min_burn_amount}` |
| InsufficientFunds | `the debit account doesn't have enough funds to complete the transaction, current balance: {balance}` |
| TooOld | `transaction's created_at_time is too far in the past` |
| CreatedInFuture | `transaction's created_at_time is in future, current ledger time is {ledger_time}` |
| TemporarilyUnavailable | `the ledger is temporarily unavailable` |
| Duplicate | `transaction is a duplicate of another transaction in block {duplicate_of}` |
| GenericError | `{error_code} {message}` |

## Allowance (`allowance`)

Common traits:
- AllowanceArgs { account: Account, spender: Account }
- Allowance { allowance: Nat, expires_at: Option<u64> (serde default) }
No expiry interpretation, amount constraint or caller authorization is added.

## Blocks (`blocks`)

Common traits:
- BlockWithId { id: Nat, block: ICRC3Value }
- ArchivedBlocks { args: Vec<GetBlocksRequest>,
  callback: QueryArchiveFn<Vec<GetBlocksRequest>, GetBlocksResult> }
- GetBlocksResult { log_length: Nat, blocks: Vec<BlockWithId>,
  archived_blocks: Vec<ArchivedBlocks> }
- GetBlocksRequest { start: Nat, length: Nat }, also PartialOrd and Ord.

GetBlocksRequest::as_start_and_length(&self) -> Result<(u64,u64),String> narrows
start first and length second independently. Zero and u64::MAX are valid.
It does not add start+length or check overflow of that sum. An oversized start
fails before oversized length. Errors preserve Nat Display and exact text:
`transaction index {start} is too large, max allowed: 18446744073709551615` and
`requested length {length} is too large, max allowed: 18446744073709551615`.
Retain borrowed magnitudes; do not clone arbitrary integers to narrow them.

These two structs have ONLY Debug, Serialize, Deserialize:
- ICRC3DataCertificate { certificate: ByteBuf, hash_tree: ByteBuf }
- SupportedBlockType { block_type: String, url: String }
They are untrusted responses; validation/verification stays consumer-owned.

## Consent (`icrc21`)

All common traits unless noted, passive DTOs only. No language/time validation,
UI rendering, consent grant or payment/authorization decision belongs here.

requests module:
- ConsentMessageMetadata { language: String, utc_offset_minutes: Option<i16> }
- DisplayMessageType enum: GenericDisplay, FieldsDisplay
- ConsentMessageSpec { metadata: ConsentMessageMetadata,
  device_spec: Option<DisplayMessageType> }
- ConsentMessageRequest { method: String, arg: Vec<u8>,
  user_preferences: ConsentMessageSpec }. arg uses ordinary Vec sequence Serde.

responses module (metadata type imported from super::requests):
- Value enum: TokenAmount { decimals: u8, amount: u64, symbol: String },
  TimestampSeconds { amount: u64 }, DurationSeconds { amount: u64 },
  Text { content: String }.
- FieldsDisplay { intent: String, fields: Vec<(String,Value)> }, also Default.
- ConsentMessage enum: GenericDisplayMessage(String),
  FieldsDisplayMessage(FieldsDisplay).
- ConsentInfo { consent_message: ConsentMessage, metadata: ConsentMessageMetadata }.

errors module:
- ErrorInfo { description: String }
- Icrc21Error enum: UnsupportedCanisterCall(ErrorInfo),
  ConsentMessageUnavailable(ErrorInfo), InsufficientPayment(ErrorInfo),
  GenericError { error_code: Nat, description: String }.

## Independent public tests

A separate author creates `crates/ic-platform/tests/icrc_dtos_consumer.rs` from
this contract, without old/new source/tests, and a record under docs/design.
Public names are exported from ic_platform::icrc_ledger_types; consent names from
its icrc21 module. NumTokens/BlockIndex remain private-module aliases, so tests
need not name them. Account via same export; Principal via ic_platform::Principal.
Codec public APIs: candid_codec::{Nat,Int,Encode,encode_one},
decoder::{Decode,Limits,decode_one}; encode_one(&value,byte_limit)->Result<Vec<u8>,Error>;
decode_one::<T>(&bytes,Limits)->Result<T,decoder::Error>.
Limits fields bytes/header/types/work/allocation are usize. Existing serde_json,
serde_cbor and serde_bytes are available to integration tests.

Prioritize full integer ranges, memo buffer moves/big-endian bytes, exact error
Display and narrowing precedence; explicit defaults, Serde byte-vs-sequence
shapes, field order/variant order, nested consent and archive callback types.
Create a finite deterministic before/after transcript of representative JSON,
CBOR bytes and owned Candid bytes using only public APIs. Print one JSON line
prefixed `ICRC_DTO_OBSERVATIONS=` (labels and values/bytes) for the coordinator to
capture; do not make runtime environment variables select expectations.
The coordinator will freeze the original outputs as a permanent fixture.
No private tests are necessary just to repeat these public checks.

Coordinator owns all Cargo commands, baseline capture, existing schema/byte tests,
notice/provenance integration and qualification. Author contexts run only scoped
rustfmt with skip_children; record admitted inputs/actions and limits without
claiming legal clearance or excluded model-training exposure.
