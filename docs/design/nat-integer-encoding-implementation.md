# Natural-number encoding implementation record

On 2026-11-10 a separate implementer context authored the integer payload
encoder and tests from the [reviewed contract](nat-integer-encoding-contract.md).
This work covers unsigned LEB128 bytes only. It does not inspect or alter any
hashing implementation, domain selection, record ordering or cryptography.

## Inputs and separation

The existing repository inputs permitted for this component, relative to
`/home/adam/projects/ic-platform`, were:

- `docs/design/nat-integer-encoding-contract.md`, read for this component.
- `AGENTS.md`, previously read policy and reused without another read.
- `TOKO-TOOLING.md`, previously read policy and reused without another read.
- `docs/principles/rust-code-hygiene.md`, previously read policy and reused
  without another read.

Public dependency API documentation was also permitted, but no dependency
source or documentation was opened for this component. The integrating reviewer
supplied these public API facts without implementation snippets:

- `BigUint::iter_u32_digits(&self)` returns `U32Digits<'_>` with borrowed,
  least-significant-first traversal.
- `BigUint::bits(&self)` returns `u64`.
- `BigUint::parse_bytes(&[u8], u32)` returns `Option<BigUint>`; this API was
  confirmed but not needed by the implementation.
- The development oracle `candid::Nat` supports `FromStr` and public
  `encode(&mut writer)`.

The implementer did not read the existing helper or tests, any hashing source,
other repository Rust source, upstream implementation, Git history or diffs.
It did not inspect the linked Candid specification directly; the reviewed
contract supplied the required protocol facts. No observations from the earlier
rejection catalogue task were used as evidence about this component. No work
was delegated to another agent.

An operational process-name check before editing found no Cargo/rustc process.
The existing target files were overwritten with complete new contents without
reading their prior contents.

## Authored files and behavior

The implementer authored only:

- `crates/ic-platform/src/icrc_ledger_types/hash_encoding/mod.rs`
- `crates/ic-platform/src/icrc_ledger_types/hash_encoding/tests.rs`
- `docs/design/nat-integer-encoding-implementation.md`

The encoder borrows 32-bit digits, carries at most 38 pending bits in a `u64`,
and batches payload bytes in a 64-byte stack buffer. The significant bit count
terminates the shortest encoding independently of digit padding. Zero emits
one zero byte. `Write::write_all` supplies short-write, interruption and
zero-progress behavior; failures propagate without flushing the writer.
The production encoder introduces no unsafe code, allocation, full-magnitude
clone, dependency or public API.

The independently authored tests derive small fixtures from seven-bit groups,
exercise power boundaries through 8192 bits and a scattered large magnitude,
and compare with the Candid oracle through decimal strings. They also check
length and continuation/minimality rules, appending through an unsized writer,
short and interrupted writes, zero-progress refusal, and a correct partial
prefix before an IO failure. Test-only allocations construct values and collect
expected and actual payloads; they are not part of the encoder's scratch space.

Licenses, manifests, provenance and integration guards remain with the reviewer.
The implementer did not read or modify them.

## Validation and limitations

The following formatting command completed successfully with no output:

```text
rustfmt +1.99.1 --edition 2024 --config-path /dev/null --config skip_children=true crates/ic-platform/src/icrc_ledger_types/hash_encoding/mod.rs crates/ic-platform/src/icrc_ledger_types/hash_encoding/tests.rs
```

The empty explicit configuration and `skip_children` confined formatting to the
two authored Rust files. No Cargo command, Clippy, compilation, tests or
performance measurement was run by the implementer. The integrating reviewer
owns those checks and their results. The bounded scratch design is not a speed
claim.

This record describes the inputs and actions in one separated implementation
context. It does not establish legal clearance, make a claim about model
training exposure or provide mathematical proof of independent authorship.
Behavioral agreement with an oracle would not establish those conclusions.
