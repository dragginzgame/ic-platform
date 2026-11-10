# ICRC value helper implementation record

## Scope and admitted inputs

The sole component contract read was `docs/design/icrc-value-contract.md`.
The coordinator reviewed existing source to supply public API and behavior
facts and owns independent baseline capture. No cryptographic implementation
was supplied. Previously admitted policy inputs are `AGENTS.md`,
`TOKO-TOOLING.md`, and `docs/principles/rust-code-hygiene.md`.

The implementer did not read the existing value implementation, existing tests,
source guards, codec mappings, supplier source, Git history or diffs, or any
hashing implementation or vectors. Review was limited to the contract and the
newly authored draft. Previous specification-only components remain in this
implementer's context; this record does not claim erased context or training
exposure, legal clearance, or mathematically proven independence.

## Authored draft

The complete unreferenced draft was authored at
`crates/ic-platform/src/icrc_ledger_types/value/replacement/mod.rs`. The compiled
value module was not read or edited by this implementer. The draft declares private
`mod hash;` without implementing, reading or copying that child's contents.
The coordinator retains the original hashing methods, integer helpers and
vectors unchanged, with their notices and source guards.

The draft contains the two value enums and three aliases, preserving enum
order, exact derived traits, external Serde tagging, ByteBuf behavior and
canonical integer carriers. Conversions move nested values recursively;
Nat64 widens to Nat only when converting to ICRC3Value or through the specified
natural-number accessors. Constructors, consuming accessors and TryFrom
implementations preserve specified admissions and diagnostic strings, including
the literal braces in the u64 overflow diagnostic. Account conversion preserves
absent versus explicit-zero subaccounts and the existing empty-array panic.
Principal and Account conversions perform no derivation.

Two requested allocation changes are explicit in the implementation:
`TryFrom<Value> for Vec<u8>` uses `ByteBuf::into_vec`, and ICRC3Value formatting
borrows its own children directly. Small private formatting helpers share blob,
array and map presentation across both enums. Formatting retains lowercase
hexadecimal blobs, literal text and keys, string-ordered maps, separators and
error propagation. No full-tree clone or conversion is used for Display.

No dependency, public API, validation policy, normalization rule or cryptographic
behavior was added. No bug fix was mixed into the empty-account-array behavior.
No performance, memory-footprint or Wasm-size gain is claimed without measurement.

## Validation and handoff

A process-name check found no active Cargo or rustc process before drafting.
The authored draft was formatted successfully with:

```text
rustfmt +1.99.1 --edition 2024 --config-path /dev/null --config skip_children=true crates/ic-platform/src/icrc_ledger_types/value/replacement/mod.rs
```

The implementer reviewed the conversion admission paths, error text and
formatting against the contract. No Cargo command, compilation, Clippy, tests or
runtime measurement was performed here. No duplicate private test suite was
added. The draft was frozen and handed to the coordinator.

The coordinator subsequently reported installing the replacement after a
baseline of six independent tests and a separate blob observation passed. It
froze 15 value observation rows and eight exact errors. Baseline supplements
recorded that Nat64 widening changes Display grouping and that the existing
Int deserializer rejects its serialized-sequence representation; neither fact
required an implementation edit.

The coordinator reported retaining the two original hash methods, integer
helpers and four vectors unchanged in the private hash child, verifying their
byte regions and moving the source guards to that retained child. It also
reported passing native library and new-test Clippy. These are attributed
coordinator actions and results, not checks performed by this implementer.

After-installation tests remain pending at this record update. Target checks,
independent observation comparison and allocation qualification remain
coordinator responsibilities. This update changed only this record, without
source reads, Cargo commands or implementation edits.

The coordinator subsequently completed fresh-build qualification and source-guard
adaptation; see [the qualification report](icrc-value-qualification.md). That
parent-added evidence includes the corrected Cargo timestamp reuse, the passing
owned-buffer transfer assertion and unchanged hash regions. It does not expand
the implementation author's admitted inputs.
