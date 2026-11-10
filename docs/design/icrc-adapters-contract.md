# ICRC Account and archive adapter contract

Input for the authorized specification-only implementer and separate public-test
author. Do not inspect existing Account/archive implementations, their tests,
supplier implementations or history. The coordinator reviewed those separately.
This contract describes the existing public behavior, not a new protocol.

Scope is `icrc_ledger_types/account` and `icrc_ledger_types/archive` only. Retain
their public types and paths; do not edit codec mappings, ledger value/hash code,
other DTOs, dependencies or features. Existing owned Types Account and Principal
are the sole canonical identity owners. Do not read their source; use the public
API described here. No cryptographic implementation is authorized.

## Account

The public facade exports `icrc_ledger_types::Account`, a distinct type from
`crate::Account` (the canonical Types Account). It has public fields, in order:
`owner: crate::principal::Principal`, `subaccount: Option<[u8; 32]>`.
The private account module also exposes `Subaccount = [u8; 32]` and
`DEFAULT_SUBACCOUNT: &Subaccount` (32 zero bytes), used by sibling modules.
Retain Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
Hash, Display, FromStr and From<Principal>. Effective zero equality/order/hash
and Display can delegate to `crate::Account::from(*self)`: the facade already
supplies both field-preserving From conversions. Do not add another identity
abstraction or change those conversions. `effective_subaccount(&self)` returns
the present array by reference or the zero constant. From<Principal> means None.

Serde is the ordinary two-field struct Account (owner, subaccount), with the
optional subaccount represented as an array/sequence, NOT a byte buffer adapter.
Preserve Some(zero) versus None. Missing optional field is None; duplicate fields
and missing owner fail; unknown fields are ignored. Debug preserves both fields.
Candid mapping already exists elsewhere and must remain untouched.

Text formatting delegates to canonical Account: default account is owner text;
nonzero is owner, dash, seven lowercase unpadded Base32 CRC32 checksum characters,
period, minimal lowercase hexadecimal subaccount. The CRC32 covers owner bytes
then all 32 subaccount bytes. Use existing registry crc32fast/data-encoding; do
not implement checksum algorithms. Principal public APIs include FromStr,
as_slice(), try_from_slice(), anonymous(), management_canister(), and length-first
Ord. Public Principal/PrincipalError identities must remain unchanged.

External Account parsing intentionally differs from the bounded Types parser:
it has no total-text ceiling and retains these detailed errors. Preserve order:

1. Split at the FIRST period. Without a period, parse the whole input as Principal
   and make a None account, wrapping any PrincipalError.
2. Before the period, the LAST dash must delimit a checksum of exactly seven
   UTF-8 bytes, otherwise MissingChecksum.
3. A subaccount suffix starting with ASCII `0` fails LeadingZeroesInSubaccount.
4. Parse the owner, preserving PrincipalError.
5. Interpret the suffix as though left-padded with ASCII zeroes to at least 64
   Unicode scalar characters, then hex-decoded. Hex admission uses the padded
   UTF-8 byte sequence: odd byte length fails first; otherwise first non-ASCII
   hex byte fails with its padded byte offset. Both ASCII hex cases are valid.
   Preserve `hex::FromHexError` Display spelling inside the detailed error.
6. After valid hex admission, a decoded length other than 32 fails with detail
   `subaccount is longer than 32 bytes`. An empty suffix decodes all-zero and
   fails DefaultSubaccountShouldBeOmitted.
7. Recompute the checksum using packages, compare case-sensitively. Mismatch
   includes the expected seven-character string. Success retains Some(array).

Do not normalize whitespace or Unicode. Avoid temporary input-sized padding and
decoded vectors if possible while retaining the malformed-input error precedence
and offsets above. Bounded stack scratch is suitable for the successful case;
validation of oversized input may scan it without allocating a proportional copy.

The associated FromStr error remains `ICRC1TextReprError` in the private account
module, Debug/PartialEq/Eq/Display/std::error::Error, with no source() override:

| Variant | Display |
|---|---|
| DefaultSubaccountShouldBeOmitted | `default subaccount should be omitted` |
| InvalidChecksum { expected: String } | `invalid checksum (expected: {expected})` |
| InvalidPrincipal(PrincipalError) | `invalid principal: {error}` |
| InvalidSubaccount(String) | `invalid subaccount: {detail}` |
| LeadingZeroesInSubaccount | `subaccount should not have leading zeroes` |
| MissingChecksum | `missing checksum` |

Hex errors use detail `subaccount is not hex-encoded: {hex_error}`. The
coordinator will compare pre/post public observation transcripts, including
Unicode and oversized invalid text; passing only valid text is insufficient.

## Archive references

`QueryArchiveFn<Input, Output>` has public fields in order:
`canister_id: Principal`, `method: String`, `_marker: PhantomData<(Input, Output)>`.
Keep derived Debug, Serialize, Deserialize, PartialEq, Eq and their existing
generic bounds. Deserialize uses `#[serde(try_from =
"crate::candid_codec::references::Func")]`; Serialize remains the ordinary three
field struct including the unit marker. This intentional asymmetry must remain.
Clone has NO Input/Output bounds. Ord and PartialOrd require only Input: Eq,
Output: Eq, and compare canonical canister_id then method. Marker has no value.
`new(canister_id: Principal, method: impl Into<String>)` has no generic bounds.

Keep From<QueryArchiveFn<I,O>> for owned codec Func and TryFrom<Func> for
QueryArchiveFn<I,O>, Error=String. Both types already carry the same canonical
Principal. Move fields directly: byte reparsing and impossible error/panic paths
are unnecessary. Keep the fallible trait signature for API compatibility.
No method/canister authority restriction belongs here, including empty/Unicode
names and management/anonymous Principals.

Owned codec APIs available (do not inspect their implementation):
- Encode::TYPE is &'static Type; write(&self, &mut Writer) -> Result<(), Error>.
- `Type::Function { args: &'static [&'static Type], results: ...,
  annotations: &'static [u8] }`; the callback is one Input, one Output, query [1].
- `Func::write_parts(&Principal, &str, &mut Writer)` encodes payload without clones.
- Decode<'a>::read(&mut Reader<'a>, wire: i64) -> Result<Self, decoder::Error>.
- `Func::read(reader, wire, expected_signature)` validates complete signature,
  then returns owned Func { principal: Principal, method: String }.
Implement callback Encode and Decode when Input and Output implement Encode;
do not require either parameter to implement Decode.

Retain cfg(test)-only From/TryFrom bridges with `candid::types::reference::Func`.
Owned Func already implements From for the registry Func; outgoing conversion
uses candid::Principal::from_slice(owned.as_slice()). No production Candid edge.

Retain two records with ordinary Serde/Clone/Debug/PartialEq/Eq and field order:
- GetArchivesArgs { from: Option<Principal> }
- ICRC3ArchiveInfo { canister_id: Principal, start: Nat, end: Nat }
Nat is `crate::candid_codec::Nat`, preserving all sizes. Codec mappings elsewhere
remain in force. Provide useful field docs rather than blanket missing_docs lint
exceptions. Consumers retain network calls, budgets, authority and retry policy.

## Work separation and evidence

Implementer writes only the two modules, fresh adjacent tests.rs if needed, and
`docs/design/icrc-adapters-implementation.md`. Hold all product writes until the
coordinator captures old-owner public observations. No old source/tests/history
may be read. Test author separately writes an integration test using only these
public contracts and records cases, then freezes it for before/after comparison.
The coordinator owns manifest registration, Cargo commands, source guards and
documentation integration. Agents run formatting only, no Cargo concurrently.
Record admitted inputs/actions and limits, without legal-clearance claims.
