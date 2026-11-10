# Native management declaration implementation record

## Scope and inputs

This replacement covers only the native management declaration modules
`crates/ic-platform/src/management_canister_types/native/mod.rs` and
`crates/ic-platform/src/management_canister_types/native/client/mod.rs`.
The latter replaces the former flat `native/client.rs` path under the
repository's ordinary directory-module policy; `mod client` and public paths
are unchanged.

The implementation input was `docs/design/management-native-contract.md`.
Previously admitted repository policy inputs were `AGENTS.md`,
`TOKO-TOOLING.md`, and `docs/principles/rust-code-hygiene.md`. The coordinator
supplied the scope, source-exclusion rules, baseline hold, and qualification
responsibilities. No additional implementation source was needed.

The implementer did not read the replaced modules, existing tests, maintained
shared management declarations, codec implementation, supplier implementation,
Git history or diffs, or provenance/source-inventory reports for this component.
Review was limited to the contract and the implementer's newly authored drafts.
The same implementer previously authored other specification-only replacements;
this record does not imply that earlier context or model training exposure was
erased, and does not claim legal clearance or mathematically proven independence.

## Authored declarations

Fresh complete drafts were written under `/tmp/management-native-draft/` while
the coordinator's product-write hold remained active. The two modules contain
18 passive public types and the `StoredChunksResult` alias, with fresh concise
field and variant documentation. Field order, variant order, trait sets, optional
fields, Serde names and numeric representations follow the contract.

The five specified `serde_bytes` fields remain explicit: metadata-read certified
data, the selected Wasm chunk hash, snapshot-read chunk bytes, install arguments,
and log content. Upload certified data, upload chunks, Wasm module bytes and
snapshot identifiers retain ordinary sequence serialization. Float fields remain
`f32`/`f64`; the `V128` carrier remains unbounded `Nat`. No validation, numeric
normalization, limits, network behavior, storage behavior or cryptographic
operation was added. Only the provisional-create request derives `Default`.

The native module retains its `client`, `codec` and conditional `tests`
declarations and the four explicit client type reexports. Shared declarations,
codec implementations, parent exports, target gates, dependencies and licenses
are outside this authored scope and remain coordinator-owned.

## Validation and integration status

Both draft files were formatted successfully with:

```text
rustfmt +1.99.1 --edition 2024 --config-path /dev/null --config skip_children=true /tmp/management-native-draft/mod.rs /tmp/management-native-draft/client.rs
```

The implementer reviewed the new declarations against the specified surface.
No Cargo command, Clippy, compilation, test or runtime measurement was performed
by this implementer. No duplicate private test suite was added; independent
public schema/value/Serde qualification belongs to the coordinator and its
separate test author.

The coordinator released the hold after reporting that the original
implementation passed five new public tests and three native oracle tests,
with 18 grouped observations frozen. Those are coordinator-reported baseline
results, not checks performed by this implementer or validation of the new
implementation.

After release, a process-name check found no active Cargo or rustc process.
The drafts were installed by complete-file copying without reading either old
module. The client draft was installed at `native/client/mod.rs`; the assigned
obsolete `native/client.rs` was removed without inspection. No other source
file was changed. Formatting of both installed authored files succeeded:

```text
rustfmt +1.99.1 --edition 2024 --config-path /dev/null --config skip_children=true crates/ic-platform/src/management_canister_types/native/mod.rs crates/ic-platform/src/management_canister_types/native/client/mod.rs
```

The authored source is frozen for coordinator qualification. Compilation,
schema/Serde comparisons, preservation guards and target checks remain with
the coordinator.
