# Principal compatibility contract

Input for a separated implementation context. Do not inspect previous Principal
source, tests, patches or upstream implementation. Protocol facts come from the
[IC interface specification](https://docs.internetcomputer.org/references/ic-interface-spec/#principals).
The reviewer also observed the existing public API through a standalone executable;
its output is [recorded here](principal-api-observations.txt). These observations
contain values and public behavior, not implementation text.

## Protocol facts

A Principal holds zero through 29 opaque bytes. The management identifier has
zero bytes; anonymous has the single byte 4. Text is the unpadded RFC 4648 Base32
of the four-byte big-endian CRC32 checksum followed by the significant bytes.
Groups are separated every five characters, with no leading/trailing separator.
Output is lowercase; admission accepts ASCII case differences. The longest
canonical form is 63 ASCII characters. Use the existing external `crc32fast`
and `data-encoding` packages; do not implement either algorithm.

## Existing public Rust surface

Implement `Principal` and `PrincipalError` in the existing Types principal module,
with no new public API, feature, crate or dependency. Principal is a copyable
fixed-capacity value. Preserve these associated APIs (including constness):

```text
const MAX_LENGTH_IN_BYTES: usize = 29
const MAX_LENGTH_IN_TEXT: usize = 63
const fn management_canister() -> Self
const fn anonymous() -> Self
const fn from_slice(slice: &[u8]) -> Self
const fn try_from_slice(slice: &[u8]) -> Result<Self, PrincipalError>
fn from_text<S: AsRef<str>>(text: S) -> Result<Self, PrincipalError>
fn to_text(&self) -> String
const fn as_slice(&self) -> &[u8]
const fn len(&self) -> u8
const fn as_fixed_bytes(&self) -> &[u8; 29]
```

`from_slice` panics above 29 bytes; the fallible constructor returns
`BytesTooLong()`. Significant bytes round-trip exactly. Fixed bytes are zero
after the significant length. Preserve Clone, Copy, Debug, PartialEq, Eq,
PartialOrd, Ord, Hash, Display, FromStr, AsRef<[u8]>, TryFrom<&str>,
TryFrom<Vec<u8>>, TryFrom<&Vec<u8>> and TryFrom<&[u8]>.

Equality distinguishes lengths, including trailing zero bytes. Ordering compares
length first, then significant bytes lexicographically. Debug and Hash behavior
are observable in the supplied transcript and should remain compatible.

Error variants are `BytesTooLong()`, `InvalidBase32()`, `TextTooShort()`,
`TextTooLong()`, `CheckSequenceNotMatch()`, and `AbnormalGrouped(Principal)`.
Preserve Clone, Debug, Eq, PartialEq, serde Serialize/Deserialize, std::error::Error
and the Display strings recorded in the transcript. Note that the five empty
tuple variants serialize differently from unit variants; preserve their shape.

## Text admission

Refuse inputs longer than 63 **bytes** before allocating or decoding them.
Within that bound, invalid Base32 symbols, impossible digit counts, padding and
nonzero trailing bits are `InvalidBase32()`. Dash positions and ASCII case do
not change the underlying Base32 payload. Fewer than four decoded bytes is
`TextTooShort()`; more than 29 payload bytes is `TextTooLong()`; a wrong checksum
is `CheckSequenceNotMatch()`. A valid checksum/payload with noncanonical dash
grouping returns `AbnormalGrouped` containing the parsed identity. It is not a
success case. The transcript supplies representative precedence observations.
Do not trim whitespace or accept Candid markers.

Formatting, parsing and Serde must avoid heap scratch proportional to input
size. Bounded stack buffers are appropriate; `to_text` necessarily returns one
owned String. Error handling must not panic on malformed Unicode or arbitrary
input within the bound. Keep the existing scoped Clippy expectation for a `len`
method without adding a new `is_empty` API: a zero-length Principal identifies
the management canister.

## Serde and self-authentication

Human-readable serializers receive canonical text. Binary serializers receive
only significant bytes, without Candid's marker or fixed-capacity padding.
Deserialize through strings for human-readable formats and bytes for binary
formats; byte slices and owned byte buffers are admitted with the same 29-byte
ceiling. A generic numeric sequence is not an extra binary identity format.
The published 0.19.0 black-box decoder also accepts CBOR text as a Principal,
despite selecting the binary deserializer entrypoint. Preserve both string and
byte/owned-byte visitor callbacks in either readability mode. Expected-type
diagnostics are `bytes or string`; do not narrow this accepted behavior.

Retain the private `self_authenticating` child module gated by the existing
`principal-self-authenticating` feature. **Do not read or implement that child**:
the reviewer will replace it with delegation to the external Principal package.
The new base Principal must not depend on its private fields or hashing.

## Independent tests

Write new tests from this contract and public supplier API observations, without
reading previous tests. Test every significant length, zero tails, length-first
ordering, conversions/const constructors, bound refusals, Unicode/ASCII grammar,
case/grouping/checksum behavior, exact error Serde/Display, and significant-byte
Serde. Public `ic_principal` is available as a development oracle for valid
text/bytes (its permissiveness outside Platform's bound is not the contract).
Existing dev dependencies: serde_test, serde_json, serde_cbor, postcard.
No copied supplier vectors or cryptographic tests. Add no unsafe allocator hooks.

Record allowed inputs, authored files and actual validation separately. Protocol
and black-box compatibility tests are evidence of behavior, not legal clearance.
