# Base Principal implementation record

On 2026-11-10 a separate implementer context authored the base Principal module
and tests from the [reviewed contract](principal-contract.md) and the reviewer's
[public API observations](principal-api-observations.txt). Self-authentication
and other cryptographic behavior were excluded from this component.

## Inputs and separation

The existing repository inputs permitted for this component, relative to
`/home/adam/projects/ic-platform`, were:

- `docs/design/principal-contract.md`, read for this component.
- `docs/design/principal-api-observations.txt`, read for this component.
- `AGENTS.md`, previously read policy and reused without another read.
- `TOKO-TOOLING.md`, previously read policy and reused without another read.
- `docs/principles/rust-code-hygiene.md`, previously read policy and reused
  without another read.

The observation transcript contained public values and behavior, not source.
The reviewer separately confirmed these public dependency API facts:

- `crc32fast::hash(&[u8]) -> u32` is available.
- `data_encoding::BASE32_NOPAD.encode_len(usize) -> usize` and
  `encode_mut(&[u8], &mut [u8])` are available, with an output slice whose length
  exactly matches `encode_len`.
- `decode_len(usize) -> Result<usize, DecodeError>` and
  `decode_mut(&[u8], &mut [u8]) -> Result<usize, DecodePartial>` are available;
  the latter requires exactly the capacity returned by `decode_len` and returns
  the number of decoded bytes.
- The existing `thiserror` dependency is available for the error derive.

During compatibility review, the reviewer supplied a further black-box
observation: published Types 0.19.0 accepts CBOR text `"aaaaa-aa"` as the
management Principal, accepts CBOR bytes, and refuses an empty CBOR array with
the visitor expectation `bytes or string`. The implementer amended the visitor
and tests using that observation. Only its own three authored files were read
for this amendment; no prior implementation or tests were consulted.

No previous Principal or Account source or tests, upstream implementation,
hashing implementation, Git history or diffs were read. The implementer did not
read other repository Rust source or inspect linked protocol pages directly.
No source inspection, behavioral result or authorship conclusion from another
component was used as evidence for this implementation. No work was delegated.

An operational process-name check before editing found no Cargo/rustc process.
The existing target files were overwritten without first reading their contents.
The private self-authentication child was neither read nor written; only its
existing feature-gated module declaration was retained as required by the
contract.

## Authored files and behavior

The implementer authored only:

- `crates/ic-platform-types/src/principal/mod.rs`
- `crates/ic-platform-types/src/principal/tests.rs`
- `docs/design/principal-implementation.md`

The base identity holds a length and 29 fixed bytes. Constructors zero the tail;
standard derives preserve the specified field-shaped Debug and Hash observations
and length-first equality/ordering behavior. Fallible and const constructors,
conversions, text methods and Serde retain the specified public surface. The
error derive preserves the five empty tuple variants, the identity-bearing
grouping variant and the observed Display strings.

Text processing uses bounded stack buffers. Input length is checked before
copying or decoding; CRC32 and Base32 delegate to the existing suppliers. The
parser distinguishes invalid encoding, decoded length, checksum and grouping
failures in the contract's order. Display and human-readable serialization use
bounded canonical text; `to_text` returns its required owned String. Binary
serialization includes only significant bytes. Deserializer entrypoint selection
remains based on human readability, while one visitor accepts both string and
byte callbacks to preserve the observed CBOR text compatibility. Owned callbacks
use Serde's forwarding behavior. Numeric sequences remain refused. No Candid or
cryptographic implementation was added.

Fourteen new test functions cover const APIs, every significant byte length,
zero tails, length-first ordering and trailing zeros, conversions and bound
failures, Debug/Hash observations, text case and grammar, checksum/grouping and
error precedence, Unicode and ASCII refusal, significant-byte Serde, cross-format
string/byte callback admission, numeric-sequence refusal, and exact error
formatting and Serde variant shapes. Valid
text/byte cases compare against the public development Principal oracle using
independently constructed fixtures. The Hash test records trait calls only;
it implements no hashing algorithm. No unsafe allocator instrumentation or
cryptographic tests were added.

Licenses, manifests, other modules, source/provenance guards and integration
changes remain with the reviewer. The reviewer must remove the old
`principal/tests/mod.rs` before building the new ordinary `tests.rs` module.

## Validation and limitations

The following formatting command completed successfully with no output:

```text
rustfmt +1.99.1 --edition 2024 --config-path /dev/null --config skip_children=true crates/ic-platform-types/src/principal/mod.rs crates/ic-platform-types/src/principal/tests.rs
```

The empty configuration and `skip_children` confined formatting to the two
authored files. The same formatting command was rerun successfully after the
CBOR text compatibility amendment. The implementer ran no Cargo command, Clippy, compilation,
tests, allocation instrumentation or performance measurements. The integrating
reviewer owns those checks and their results.

This record describes the inputs and actions of one separated implementation
context. It does not establish legal clearance, claim absence of model training
exposure, or provide mathematical proof of independent authorship. Compatibility
or oracle checks alone would not establish those conclusions.
