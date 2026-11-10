# ICRC value helper compatibility contract

Subsequent maintainer authorization fixes the two behavior defects originally
preserved by this source-only contract: empty Account arrays return an error,
and Int accepts its serialized sign/magnitude sequence. The original author
inputs below are retained as history; current behavior and evidence are in
[the follow-up](icrc-value-qualification.md#follow-up-bug-fixes).

Reviewed public API/behavior facts for isolated authors. Do not read original or
replacement implementation bodies, old tests, source guards, supplier source,
history or diffs. Parent read existing source to review this contract and owns
black-box baseline capture. No cryptographic implementation is provided here.

## Scope and private boundary

Author an unreferenced draft at
`crates/ic-platform/src/icrc_ledger_types/value/replacement/mod.rs` only, plus
`docs/design/icrc-value-implementation.md`. Parent installs it after baseline
capture. Keep `mod hash;`: parent supplies the retained imported hash methods,
private integer encoding and upstream vectors, unchanged, in that private module.
Do not implement or inspect them. Crypto source notices/guards remain.

Use canonical crate::principal::Principal, crate::candid_codec::{Int,Nat},
serde::{Serialize,Deserialize}, serde_bytes::ByteBuf, std::collections::BTreeMap,
and super::account::Account. Nat wraps num_bigint::BigUint; Int wraps BigInt,
both in public .0 fields. num_traits::ToPrimitive is available. No dependencies,
validation policy, public methods or trait implementations may be added.

## Public declarations

Aliases in the private value module: Map = BTreeMap<String,Value>,
ICRC3Map = BTreeMap<String,ICRC3Value>, Hash = [u8;32]. Only Value and ICRC3Value
are re-exported at ic_platform::icrc_ledger_types; tests must not widen exports.

Both enums derive Serialize, Deserialize, Clone, Debug, PartialEq, Eq,
PartialOrd, Ord, with no extra Serde attributes. Ordered variants:

- ICRC3Value: Blob(ByteBuf), Text(String), Nat(Nat), Int(Int),
  Array(Vec<ICRC3Value>), Map(ICRC3Map).
- Value: Blob(ByteBuf), Text(String), Nat(Nat), Nat64(u64), Int(Int),
  Array(Vec<Value>), Map(Map).

The declaration order determines Ord and binary Serde variant order. Preserve
external tagging, blob byte-buffer behavior, canonical carriers and numeric
ranges. Nat64 remains distinct from Nat; no normalization on construction.

## Conversion and constructor behavior

- From<Value> for ICRC3Value moves recursively. Nat64 becomes Nat of the same
  nonnegative magnitude; all other variants retain their matching kinds/values.
- From<ICRC3Value> for Value moves recursively, without inventing Nat64.
- Value::variant_name(&self) -> String returns exactly the variant spelling.
- Value::text(t: impl ToString) -> Self produces Text with ToString output.
- Value::blob(t: impl Into<Vec<u8>>) -> Self produces Blob with those bytes.
- Value::map<S,V>(v: V) -> Self, where S: ToString and
  V: IntoIterator<Item=(S,Value)>, produces a BTreeMap using converted keys.
  Duplicate converted keys keep the last value; ordering follows String Ord.

Consuming accessors all return Result<payload,String>: as_blob, as_text,
as_nat, as_int, as_array, as_map. Except as_nat, accept only the named variant.
Mismatch error is exactly the actual variant name. as_nat accepts Nat unchanged,
Nat64 widened, and nonnegative Int converted via BigUint::try_from(BigInt).
On negative Int, its error is `Failed to convert Int to Nat: {error:?}` using
that external conversion error's Debug representation. It does not accept Text.

TryFrom<Value> has Error=String for ByteBuf, Vec<u8>, String, Nat, u64, Int,
Vec<Value>, Map, Account and Principal. ByteBuf/String/Nat/Int/Vec<Value>/Map
use their consuming accessor and prefix any error with
`Expecting variant NAME but found DETAIL`, with NAME Blob/Text/Nat/Int/Array/Map.
Thus a negative Int converted to Nat includes the complete accessor error as
DETAIL. Vec<u8> has the same admission and error as ByteBuf. It may transfer the
owned blob allocation rather than copy it. u64 first follows Nat TryFrom, then
checked magnitude narrowing; overflow text is literally
`Unable to convert nat {nat} to u64` (braces retained).

Account is icrc_ledger_types::Account { owner: Principal,
subaccount: Option<[u8;32]> }, not a new identity abstraction. Consuming Account
conversion first requires Array using as_array's bare mismatch error. More than
two elements produces exactly `Account should be an array of either one or two
elements but found an array of N elements`, with decimal N. Empty arrays
currently panic; preserve that behavior in this source-only replacement (the
parent records it separately, rather than silently mixing a behavior fix).
The first element must be Blob (bare actual variant name on mismatch), admitted
by Principal::try_from_slice. Principal admission failure text is
`Unable to decode the owner of the account, error {error}`.
An optional second Blob must contain exactly 32 bytes; conversion failure text
is `Unable to decode the subaccount of the account, error {error}` using the
standard slice-to-array conversion error's Display. Preserve None versus Some
all-zero bytes. From<Account> for Value produces an Array containing the owner
blob and, only when present, a subaccount blob. It performs no derivation.

Principal TryFrom<Value> requires Blob with the bare as_blob error. Checked
length admission errors become `Unable to decode the principal, error {error}`.
From<Principal> for Value produces a Blob of the raw principal bytes.

## Display behavior and safe improvements

Both values display identically for corresponding variants:

- Blob: lowercase hexadecimal, two digits per byte, no prefix; empty is empty.
- Text: literal text, without quotes or escaping.
- Nat, Nat64 and Int: forward their existing decimal Display.
- Array: `Array(` followed by child displays separated by `, ` and `)`.
- Map: `Map(` followed by String-ordered `key: VALUE` entries separated by
  `, ` and `)`. Keys are literal, unquoted/unescaped. Empty is `Map()`.

Preserve formatter/error propagation. Width/precision/alternate requests do not
add custom padding, truncation or changes. ICRC3Value formatting may borrow its
children directly instead of cloning/converting the complete tree. ByteBuf
conversion may move its Vec allocation. Do not claim speed/Wasm gains without
measurement. Keep helpers small; no new public abstraction or alternate value
identity. Do not change either hash method: parent retains ICRC3Value::hash(self)
and Value::hash(&self) separately.

## Independent public tests

A separate test author writes tests/icrc_value_consumer.rs (under the facade)
and docs/design/icrc-value-public-tests-implementation.md from this contract
only. Fresh fixtures should exercise every variant, recursive arrays/maps,
duplicate constructor keys, negative and oversized integers, all consuming
accessor/TryFrom failures, exact errors, Principal and Account conversions,
None versus explicit zero, trait ordering, Display and JSON/CBOR/Candid bytes.
Capture empty-account panic without requiring an implementation-specific message.
Never call or rewrite hashes, derivation, cryptographic primitives or vectors.

Emit one finite ICRC_VALUE_OBSERVATIONS= JSON transcript. Parent executes it
against the original, freezes the fixture and later compares the replacement.
Public serde_json/serde_cbor and codec APIs are available. Codec APIs remain
encode_one(&value, maximum_bytes), decoder::decode_one::<T>(&bytes, Limits {
bytes, header, types, work, allocation }); all limits are usize. Both values
implement Encode/Decode through unchanged mappings. Principal supports
from_slice, as_slice, anonymous, management_canister and try_from_slice.
Nat/Int support decimal FromStr and integer From conversions. Avoid assuming
an upstream Rust identity or adding a production Serialize bridge.

Parent separately qualifies the two intended allocation changes after source
replacement; original code is not required to meet those new performance checks.
No existing expressive tests or implementation text is admitted to either author.

## Original-owner observations supplement

Original execution confirms that converting Value::Nat64(u64::MAX) into
ICRC3Value changes its displayed spelling from `18446744073709551615` to the
owned Nat carrier's `18_446_744_073_709_551_615`. Nat and Int forward their
carriers' grouped decimal Display; Nat64 forwards u64's ordinary decimal. Do not
assume the Nat64-to-Nat conversion also preserves Display text.

The original owned Int carrier serializes as a BigInt limb sequence but its
Deserialize rejects that sequence (`invalid type: sequence, expected Int value`).
This pre-existing carrier asymmetry is outside the Value helper replacement.
Freeze the serialized JSON/CBOR and decoded value or exact failure; do not add
normalization or change carrier code to make a new test roundtrip. Candid
roundtrips remain required. No carrier implementation body is supplied here.
