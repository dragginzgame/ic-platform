# Account compatibility contract

Input for a separate implementation context; do not read existing Account source,
old tests, history or supplier implementations. Public protocol reference:
[ICRC-1 textual encoding](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-1/TextualEncoding.md).
The reviewer ran published Types 0.19.0 through public APIs; the independent
[observation transcript](account-api-observations.txt) gives text, JSON, CBOR and
error results. Existing local Principal is the canonical owner, not a new type.

## Public surface and identity

Retain `pub type Subaccount = [u8; 32]`, `pub const DEFAULT_SUBACCOUNT:
&Subaccount` containing 32 zeros, and `Account { pub owner: Principal,
pub subaccount: Option<Subaccount> }`. Account is Clone, Copy, Debug, Serialize,
Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash, Display, FromStr and
From<Principal>. `MAX_LENGTH_IN_TEXT: usize = 136` and
`effective_subaccount(&self) -> &Subaccount` are public. From<Principal> sets
subaccount to None. Absent and present zero subaccounts compare/order/hash
identically. Compare owner first using Principal's length-first order, then the
32 effective subaccount bytes. Hash must feed owner followed by the effective
array through their standard Hash traits, preserving trait-call behavior.
Do not change or inspect the Principal implementation; use its public APIs.

## Text and admission

Default accounts format as owner alone. A nonzero subaccount formats as
owner, dash, seven lowercase unpadded Base32 checksum symbols, period, then
lowercase hexadecimal subaccount with leading zero nibbles omitted. The
checksum is big-endian CRC32(owner bytes concatenated with all 32 subaccount
bytes). Delegate CRC32/Base32 to existing packages; no algorithm implementation.
Odd hex lengths work and represent a right-aligned integer in 32 bytes.

Parsing preserves this precedence, including malformed combinations:
1. Above 136 UTF-8 bytes: TextTooLong, before any work proportional to input.
2. No period: parse the whole text as Principal, yielding a default account.
3. Before the first period, a final dash must separate a seven-byte checksum;
   otherwise MissingChecksum.
4. A hex component starting with ASCII `0`: LeadingZeroesInSubaccount.
5. Invalid owner: InvalidPrincipal with the original PrincipalError.
6. Hex longer than 64 bytes: InvalidSubaccount.
7. Empty hex: DefaultSubaccountShouldBeOmitted.
8. Non-hex content: InvalidSubaccount. Both ASCII hex cases are accepted.
9. Checksum mismatch: InvalidChecksum. Checksum comparison is case-sensitive;
   owner text accepts its existing ASCII case behavior.
10. On success preserve Some(subaccount), not None.

No whitespace trimming, Unicode hex, multiple-period extension, aliases or new
leniency. Formatting and parsing should use bounded stack scratch, not temporary
heap buffers proportional to input. This checksum is not cryptographic; use
upstream crc32fast and data-encoding directly. No crypto, Candid or new dependencies.

## Errors

Retain Clone, Debug, PartialEq, Eq, std::error::Error and Display:
- TextTooLong: `account text is too long`
- InvalidPrincipal(PrincipalError): `invalid principal: {error}`; expose the
  wrapped error through Error::source.
- MissingChecksum: `missing account checksum`
- InvalidChecksum: `invalid account checksum`
- InvalidSubaccount: `subaccount must contain at most 64 hex digits`
- LeadingZeroesInSubaccount: `subaccount should not have leading zeroes`
- DefaultSubaccountShouldBeOmitted: `default subaccount should be omitted`

## Serde

Account is a two-field struct named Account, owner then subaccount. Serialize
both fields even if subaccount is absent. The owner delegates to Principal;
subaccount is Option of bytes, using the existing serde_bytes adapter. Preserve
Some(all-zero) versus None. Missing subaccount deserializes as None, present
bytes/sequence must contain exactly 32 elements. Unknown fields are ignored;
duplicate fields and missing owner fail. JSON, CBOR and postcard bytes must
match the transcript/public data shape. Do not serialize Account as display text
or omit default fields. No Candid trait or external identity conversion here.

## Implementation and tests

Write only account/mod.rs, new account/tests.rs and a short implementation record
under docs/design/account-implementation.md. Do not read existing account tests
or source; overwrite the module from these facts. Parent removes old test files.
Use ordinary module layout and existing lint policy. Test empty/maximum owners,
all 64 significant hex widths, ASCII case, text ceilings/malformed Unicode,
checksum mutations/error precedence, logical zero equality/order/hash and
serialization distinctions/bounds. Use independently constructed fixtures and
the supplied black-box values; no upstream tests/vectors or new dev dependencies.
Run only formatting on authored files with skip_children; parent compiles/tests.
Record allowed inputs and limits; this procedure does not establish legal clearance.
