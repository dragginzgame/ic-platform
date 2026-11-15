# Changelog

## [0.1.0] - Unreleased

Initial Protocol version; publication remains disabled pending publisher selection.

### ➕ Added

- Introduce an unpublished Protocol package with a bounded Cloud Engine request
  encoder and selected response decoder. Preserve opaque IDs, exact arity,
  strict optional payload admission and canonical Principal identity. Decode-only
  records declare fields, schema and dispatch together.
- Add a separate qualification workspace exercising the public decoder,
  shared Principal type, Serde shape and portable consumer compilation.
- Adopt Toko 0.44.3 and Rust 1.100.0 for local development, including the shared
  `deps-list` and `deps-update` commands and focused encoder checks.
- Add CI definitions for Linux and both macOS architectures, with one repository
  secret-scan/workflow-lint owner and pinned tool/action selections. Hosted runs
  remain unqualified until the workflow is delivered and executed.
- Enforce the current Protocol and consumer dependency closures using Toko's
  canonical graph checker on native and Wasm selections, including one Types
  identity and dependency-free default/request profiles.

### 🧪 Testing

- Characterize published empty-vector/None admission independently from present
  incompatible payload refusal; do not claim full declared-schema compatibility.
- Execute separate old/new Wasm admission probes for signed numeric boundaries,
  Principal bytes, truncation, trailing data, arity and resource refusals without
  changing the fixed size-comparison workload.

- Add independent literal request frames and exact output-limit cases covering
  empty vectors, text preservation and multi-byte lengths/counts.
- Cover response numeric extremes, truncation, malformed selected/skipped values,
  cumulative budgets, reference values and the constructed-wire depth ceiling.
- Compare independently declared responses, request bytes and malformed frames
  against published Platform in native-only qualification; keep the oracle out
  of product and Wasm dependency graphs.
- Measure fixed-workload logical work, allocation and skip charges against
  Query's existing budgets; reject oversized zero-sized discarded vectors.
- Add isolated native heap profiling against published Platform, enforcing
  no total/peak allocation regression and no allocation growth for discarded text.
- Prove allocation cleanup on truncated/trailing responses and resource-budget
  refusals after partial construction, retaining the expected typed errors.
- Qualify nested JSON numeric boundaries, Principal text, optional presence and
  malformed/duplicate selected fields against published Platform independently
  of the Candid mapping.
- Add separate old/new Wasm probes with matching compiler settings and Node
  execution; report size differences with acceptance undecided pending a
  launch-derived budget and deferred contribution analysis.

### 🧹 Cleanup

- Validate generated static projection schemas at compilation instead of
  traversing them again for each response. Incoming schemas remain fully checked.
