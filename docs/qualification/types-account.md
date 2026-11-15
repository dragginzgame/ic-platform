# Local Types: Account and Subaccount

This slice retains the reviewed Account admission and identity contract from
registry Types 0.20.4, with the archive, VCS and matching licence provenance
recorded in [Principal qualification](types-principal.md). Only that small module
is reused; no published tests, manifests or ledger hashing code are copied.

The destination owns an opaque 32-byte Subaccount and an Account containing the
local Principal. Equality, ordering and hashing use the effective zero subaccount
when absent. Serde must preserve absence versus explicit zero and represent a
present subaccount as bytes. Text omits the effective zero value, otherwise uses
the owner, CRC32/Base32 checksum and minimal hexadecimal subaccount. Preserve
the 136-byte admission bound, refusal precedence, case rules and checksum
framing (owner bytes followed by all 32 subaccount bytes).

CRC32 and Base32 remain upstream dependencies. The sole additional production
supplier is serde_bytes, already admitted by the published Types contract.
No unsafe code, cryptographic primitive, authorization or ledger storage policy
is introduced. Fresh literal fixtures and Serde tokens provide independent proof;
round trips alone are insufficient.

The new API renders Account text into caller-owned fixed storage, sharing that
path with Display and the local Principal renderer. Source relocation alone is
not qualification or redesign. Protocol remains on registry Types; real consumer
migration remains open. [Self-authentication and independent consumer proof](types-consumer.md)
are now implemented separately.

## Validation — 2026-11-15

`make qualify-types` passes on Linux x86-64 with Rust 1.100.0: eight identity
tests (four Account tests), strict Clippy, strict Rustdoc and Wasm test
compilation. Formatting and diff whitespace checks also pass. Literal checksums
were calculated independently with Node's zlib CRC32 over owner bytes followed
by the full subaccount, then encoded as unpadded Base32. Fixtures cover odd hex
lengths, leading zeroes, case rules, invalid Unicode, wrong checksums, maximum
text length, subaccount length refusal and binary Serde tokens. No published
test corpus was copied. Hosted/macOS execution and Wasm execution of this slice
remain pending; compilation is not execution evidence.
