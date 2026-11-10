# Account implementation record

On 2026-11-10 a separate implementer context authored Account from the reviewed
[contract](account-contract.md) and the reviewer's public
[observation transcript](account-api-observations.txt).

## Inputs and separation

The existing repository inputs read for this component, relative to
`/home/adam/projects/ic-platform`, were only:

- `docs/design/account-contract.md`
- `docs/design/account-api-observations.txt`

Previously read `AGENTS.md`, `TOKO-TOOLING.md`, and
`docs/principles/rust-code-hygiene.md` supplied policy. The previously supplied
`docs/design/principal-contract.md` supplied the permitted public Principal API
context; none of those files needed another read for Account.

The reviewer clarified that the error enum is `AccountError`, the text ceiling
is the associated `Account::MAX_LENGTH_IN_TEXT`, and the existing adapter
supports `#[serde(default, with = "serde_bytes")]` on `Option<[u8; 32]>`.
Previously confirmed public CRC32/Base32 APIs and existing `thiserror` support
were reused. No supplier implementation or documentation was opened.

The implementer did not read previous Account source or tests, other module
source, upstream implementations, Git history or diffs. It did not inspect the
current Principal implementation or its cryptographic child. Observation values
were supplied as black-box behavior, not source. No work was delegated.

The task paused before any Account file was written, then resumed after the
reviewer relayed the user's instruction that the other session had stopped.
Process-name inspection before editing found no Cargo/rustc process. The module
and new tests were written in full without reading previous contents. No files
outside the three listed below were changed.

## Authored files and behavior

- `crates/ic-platform-types/src/account/mod.rs`
- `crates/ic-platform-types/src/account/tests.rs`
- `docs/design/account-implementation.md`

Account keeps its public owner and optional subaccount fields. Comparison and
Hash use the owner followed by effective subaccount bytes; Serde retains absent
versus explicitly zero fields. The confirmed serde_bytes adapter supplies the
Option/byte-array representation and exact-length admission. Error variants,
messages and the wrapped Principal error source follow the contract.

Parsing checks the UTF-8 byte ceiling before splitting and applies the specified
checksum-presence, leading-zero, owner, hex-length, empty-component, hex-content
and checksum precedence. Hex is right-aligned by nibbles in 32 bytes. Formatting
uses the owner's public Display and bounded checksum/hex buffers, with no
proportional temporary heap buffer. CRC32 and Base32 delegate to existing
packages. No cryptographic algorithm, Candid behavior, feature or dependency
was added.

Eleven independently authored tests cover supplied text/serialization values,
all 64 significant hex widths with empty and maximum owners, case behavior,
the text ceiling and Unicode, malformed combinations and checksum mutations,
effective-zero equality/order/Hash, JSON/CBOR/postcard bytes, Serde struct fields
and byte callbacks, missing/unknown/duplicate fields, exact subaccount lengths,
and error formatting/source identity. The Hash test records standard trait
calls without implementing a hashing algorithm. No supplier tests or vectors
were inspected.

## Validation and limits

This command completed successfully with no output:

```text
rustfmt +1.99.1 --edition 2024 --config-path /dev/null --config skip_children=true crates/ic-platform-types/src/account/mod.rs crates/ic-platform-types/src/account/tests.rs
```

The explicit empty configuration and `skip_children` confined formatting to the
authored files. The implementer ran no Cargo command, Clippy, compilation or
tests. Parent-owned probes or integration checks are separate evidence and are
not claimed as implementer validation here. Parent integration owns removal of
old tests, qualification, licenses and provenance/guard updates.

This is an account of one separated implementation context's permitted inputs
and actions. It is not legal clearance, a claim about model training exposure,
or mathematical proof of independent authorship. Behavioral agreement alone
would not establish those conclusions.
