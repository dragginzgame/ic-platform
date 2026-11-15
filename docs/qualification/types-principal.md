# Local Types: Principal admission slice

The new unpublished `ic-platform-types` 0.21.0 package begins the local Types
cohort. The crates.io sparse index was checked on 2026-11-15: latest non-yanked
Types is 0.20.4, archive checksum
`a8df2753db752bd0c29fa87353b98613465a5f0c5269b6fb8dd0e45ea5c12a22`.
The archive's VCS identity is `343e3ae5a42fd8274330fa2171f5a9d53beab1dc`, path
`crates/ic-platform-types`. The local cached archive matches that checksum.
The source licence is Toko Studios Network Licence 1.0 and matches this
repository's installed LICENSE.md byte for byte.

## Narrow reuse decision

Retain only the published Principal module's byte/text admission, comparisons,
conversion traits and Serde adapters after reviewing their destination contract.
No old tests, Account implementation, self-authentication implementation,
manifests or workflow machinery are copied. CRC32 and Base32 remain delegated
to the same upstream suppliers; no cryptographic primitive is implemented.
The retained code has no unsafe operations and uses fixed buffers sized for the
63-byte text admission limit. Fresh owner-local tests qualify its boundary.

The required contract is at most 29 opaque significant bytes, zero-filled tail,
length-first ordering/hash identity, canonical checksummed Base32 text with
five-character groups, accepted ASCII case differences and bounded rejection
before decoding oversized input. Preserve the existing distinct malformed-text,
checksum and grouping errors. Human-readable Serde is text; binary Serde is
significant bytes. String/byte callbacks are accepted; numeric sequences refuse.
Identity validity grants no authority.

The actual new API is `Principal::write_text`, rendering into caller-provided
fixed storage without allocating a String. Existing text paths share that owner.
Moving the retained code alone is not counted as redesign or release readiness.

## Source identity and incomplete cohort

Protocol still selects registry Types 0.20.4. The local 0.21.0 package is tested
as its own root; there is no conversion or wrapper joining the two identities.
The version is the next pre-1.0 minor for the new, currently incomplete cohort.
Account/Subaccount now has a [separate implemented slice](types-account.md).
[Self-authentication and independent consumer qualification](types-consumer.md)
are now implemented. Production migration and full compatibility
qualification remain required before release. No API omission here is an approved
removal of those obligations. Both sources may occur in the workspace lock, but
that is not one production graph or a claim of interchangeable Rust identities.

## Focused validation — 2026-11-15

On Linux x86-64 with Rust 1.100.0, `make qualify-types` passed four independent
unit tests, strict Clippy, strict Rustdoc and compilation of the tests for
`wasm32-unknown-unknown`. The fixtures cover literal management/anonymous/canister
identities, all admitted byte lengths, zero-filled tails, length-first ordering,
distinct malformed-text refusals, borrowed rendering storage, JSON and binary
Serde tokens. Wasm execution and full published-contract equivalence are not
established by these checks.

The same target is wired into all three native CI jobs; hosted results for this
change remain pending. Local formatting, workflow lint, snapshot verification,
release-group inventory and all eleven existing Protocol graph selections pass.
An explicit Cargo tree observation of local Types on Wasm contains only CRC32,
Base32, Serde and error-derive suppliers and their build closure. There is no
Protocol, Runtime or second Types identity in that selected production graph.
