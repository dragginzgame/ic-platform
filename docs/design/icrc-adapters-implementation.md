# ICRC Account and archive implementation record

On 2026-11-10 this specification-only implementation context authored the two
adapters from [icrc-adapters-contract.md](icrc-adapters-contract.md). Existing
ICRC Account/archive source, tests, supplier implementations and history were
not consulted.

## Inputs and separation

The only repository file newly read for this component was
`docs/design/icrc-adapters-contract.md`, relative to
`/home/adam/projects/ic-platform`. Previously admitted `AGENTS.md`,
`TOKO-TOOLING.md` and `docs/principles/rust-code-hygiene.md` supplied policy.
No current Types source/test files or other product modules were read.
This context had authored canonical Types replacements in earlier tasks; that
context history is not claimed to be erased. For this component the canonical
owner was used through the public APIs specified in the new contract, without
consulting those implementations or copying their semantic bodies.

The coordinator confirmed owned codec import paths and signatures, public hex
error constructors, and the cfg(test) registry-to-owned Func conversion. Review
also clarified that this feature does not select the facade's optional thiserror
dependency; the Account error therefore uses direct Display and Error impls with
no source override. No dependency/feature change was made.

During the baseline hold, drafts existed only under
`/tmp/icrc-adapters-draft/`. After the explicit release signal and a process-name
check finding no Cargo/rustc process, the drafts were copied over the assigned
product files without reading their previous contents. The coordinator captured
and compared old-owner observations independently; those artifacts and the
separate public test source were not read by this implementer. No work was
delegated by this context.

## Authored files and behavior

Only these durable files were written:

- `crates/ic-platform/src/icrc_ledger_types/account/mod.rs`
- `crates/ic-platform/src/icrc_ledger_types/account/tests.rs`
- `crates/ic-platform/src/icrc_ledger_types/archive/mod.rs`
- `crates/ic-platform/src/icrc_ledger_types/archive/tests.rs`
- `docs/design/icrc-adapters-implementation.md`

The distinct ICRC Account retains ordinary sequence-shaped Serde. Equality,
ordering, Hash and Display delegate through the existing field-preserving
canonical Account conversion. Parsing retains the external adapter's unbounded
text admission and detailed errors. It counts at most 64 Unicode scalars for
conceptual padding, checks padded byte parity, validates hex in byte order with
padded error offsets, then checks decoded length. Valid data uses a fixed 32-byte
array; oversized input needs no proportional padding or decoded allocation.
Checksum work delegates to existing CRC32/Base32 packages with bounded scratch.

Archive references move canonical Principal and String fields directly through
owned Func conversions while retaining TryFrom's String error type. Clone has
no marker bounds; ordering uses the specified Eq bounds. Callback codec bounds,
query signature, Serde's intentional serialization/deserialization asymmetry,
development-only registry bridges and archive record fields follow the contract.
No ledger value/hash body, other DTO, codec mapping or cryptographic algorithm
was read or modified.

Four small private tests cover Unicode padding/error offsets, oversized hex
error precedence and absent error sources, no-bound cloning/direct String moves,
and cfg(test) registry bridges. The separate public author owns the broader
before/after compatibility matrix.

## Validation and limitations

Rustfmt 1.99.1 succeeded on all four authored Rust files using
`--edition 2024 --config-path /dev/null --config skip_children=true`, first on
the drafts and then on the installed product files. This implementer ran no
Cargo command, Clippy, compilation, tests or performance measurements. The
coordinator's native/Wasm, public-observation, frozen-wire and source-guard
results are separate evidence, not implementer-run passes here. No speed claim
is made from the bounded-buffer design alone.

This record describes admitted inputs and actions, not legal clearance, absence
of model training exposure, or mathematical proof of independent authorship.
Behavioral agreement alone cannot establish those conclusions.
