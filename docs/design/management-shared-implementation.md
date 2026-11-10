# Shared management declaration implementation record

## Scope and inputs

The assigned source scope is
`crates/ic-platform/src/management_canister_types/maintained/mod.rs`.
The implementation input for this component is
`docs/design/management-shared-contract.md`. Previously admitted repository
policies are `AGENTS.md`, `TOKO-TOOLING.md`, and
`docs/principles/rust-code-hygiene.md`. Earlier native-management contract facts
remain admitted, but no source from that component is needed here.

The coordinator reports extracting the contract's declaration facts from the
public API, excluding comments and implementation bodies.

The implementer has not read the replaced module, its codec or transform
implementation, existing tests, supplier implementations, source inventories,
Git history or diffs for this component. The coordinator owns baseline capture,
source retirement, licensing review and independent qualification. The same
implementer previously authored other specification-only replacements; this
record makes no claim that prior context or model training exposure was erased,
no claim of legal clearance, and no mathematically proven independence claim.

## Authored declarations

The new source contains 49 passive declarations, the `CanisterId` and `SnapshotId`
aliases, and a retained explicit `TransformFunc` reexport. Private `codec` and
`transform_func` module declarations remain intact. Field and variant order,
trait sets, defaults, Serde names and explicit byte-buffer adapters follow the
contract. Other byte vectors, including optional and nested vectors, retain
ordinary sequence Serde.

`Nat` fields retain their unbounded carrier. `HttpRequestArgs` and
`TransformContext` retain their restricted trait sets, and `TransformArgs`
retains Hash without ordering traits. No validator, limit, normalization,
network behavior, cryptographic operation, storage or timer policy is added.
The sole explicit conversion maps `EcdsaCurve::Secp256k1` and `Secp256r1` to
protocol cost codes 0 and 1, respectively. Transform behavior and codec bodies
remain outside this implementation's scope.

## Status and validation

The contract was reviewed for missing API details; none were identified.
The coordinator subsequently authorized a durable, unreferenced draft at
`crates/ic-platform/src/management_canister_types/replacement/mod.rs` during the
baseline hold. A process-name check found no active Cargo or rustc process
before that draft was written. No module declaration referenced the draft, and
the old `maintained/mod.rs` remained untouched throughout the hold.

The draft was written as a complete new file and formatted successfully with:

```text
rustfmt +1.99.1 --edition 2024 --config-path /dev/null --config skip_children=true crates/ic-platform/src/management_canister_types/replacement/mod.rs
```

Review was limited to the contract and this implementer's new source. A scoped
declaration count confirmed 49 structs/enums, two aliases and 12 explicitly
byte-adapted fields in the draft. No Cargo command, compilation, Clippy, tests or
runtime measurements have been run by this implementer. No duplicate private
test suite was added.

The coordinator released the hold after reporting four new independent public
tests, eight existing oracle tests and 11 public tests passed on the original
implementation. It reported freezing 24 serialized observation groups and
three deserialize-only Candid observations with an exact comparison assertion.
These are attributed coordinator baseline results, not validation performed by
this implementer or results for the replacement.

After release, a process-name check found no active Cargo or rustc process. The
draft was moved over `maintained/mod.rs` without reading the old file or a diff,
and the now-empty `replacement` directory was removed. The installed authored
file was formatted successfully with:

```text
rustfmt +1.99.1 --edition 2024 --config-path /dev/null --config skip_children=true crates/ic-platform/src/management_canister_types/maintained/mod.rs
```

The new source and this record are frozen for coordinator qualification. No
codec, transform adapter, parent export, dependency, test or license file was
modified by this implementer. Compilation, schema/value/Serde comparisons and
target qualification remain coordinator-owned.
