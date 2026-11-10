# Natural-number payload encoding contract

This is a protocol/API specification for an implementation context that has not
read the previous integer helper or tests. It supplies no implementation source.

## Required bytes and API

The [Candid specification, memory encoding](https://github.com/dfinity/candid/blob/master/spec/Candid.md#memory)
defines a `nat` payload as unsigned LEB128. Emit the shortest representation:
low seven-bit groups first, bit seven set on every byte except the last. Zero
is one zero byte. There is no DIDL header, type table, size prefix or sign byte
in this helper. The value has no fixed integer-width ceiling.

Platform exposes an existing `crate::candid_codec::Nat` tuple struct whose
public field `0` is `num_bigint::BigUint` 0.5.1. This helper supplies the inherent
crate-private method:

```text
Nat::encode<W: std::io::Write + ?Sized>(&self, writer: &mut W) -> std::io::Result<()>
```

Preserve that signature. The module is
`crates/ic-platform/src/icrc_ledger_types/hash_encoding/mod.rs`. It is selected
with the existing `icrc-ledger-types` feature. Do not add a crate, feature,
dependency or public API. Do not read or modify any hashing implementation.
This is an integer byte encoder only: hashes, domains, record ordering and
cryptographic operations remain outside it and unchanged.

Use ordinary `Write` semantics: append the encoding, handle short writes and
interrupted writes, propagate failures, and return `WriteZero` if a writer makes
no progress. A failed write may leave a prefix of the correct encoding. Do not
flush or replace the writer. No guarantee is made about individual write-call
sizes. The helper must never truncate a large magnitude to a machine integer.

## Implementation goals

Implement directly from these facts. Avoid unsafe code and whole-magnitude
clones or temporary allocations proportional to the magnitude. The BigUint
supplier's public borrowed digit iterator can be used, as can bounded stack
scratch space. Do not introduce a second generic Candid codec or change Nat's
representation. Optimize only while preserving the exact bytes and IO behavior.

## Independent tests

Write new tests from this specification, without inspecting previous tests:

- Check zero and boundaries around powers of 2 at 7-, 32- and 64-bit transitions,
  plus large magnitudes (at least 8,192 bits).
- Check expected payload length and that all bytes except the last carry the
  continuation flag, with no redundant final zero group.
- Compare with the public dev-only `candid::Nat::encode` oracle by constructing
  the oracle number from a decimal string. Do not inspect that implementation
  or copy its test vectors.
- Test appending after existing bytes, a writer that accepts only a short prefix
  per call, interrupted writes, write-zero refusal and error propagation after
  a correct partial prefix. An unsized writer must compile.

Existing dependencies include `num-bigint`, `num-traits`, `leb128`, and the
development-only Candid oracle. Their public APIs may be used; their implementation
source is not input to the replacement. Independently derived fixture values
and tests are required. The reviewer owns integration checks and any subsequent
performance comparison; an allocation goal alone is not a measured speed claim.
