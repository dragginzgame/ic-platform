# Protocol changelog

Release group: `protocol`. Covered package: `ic-platform-protocol`.
Return to the [group index](../../CHANGELOG.md).

## [0.1.0] - Unreleased

Initial Protocol version; publication remains disabled pending publisher selection.

### ➕ Added

- Decode available-node pages and all nine application errors using the shared
  variant declaration, preserving unready/empty distinctions, canonical cursors
  and bounded skipped IP fields. Add exact bounded page-argument encoding from
  one field declaration, with published-byte comparisons and preallocation heap
  admission checks.

- Add opt-in marketplace price decoding with six node classes, arbitrary naturals,
  signed timestamps and strict optional scopes. Reuse the record declaration and
  bounded reader; keep the bigint supplier outside other feature closures.

- Decode raw Cloud Engine network fees and Registry-copy status through the
  existing bounded response reader, with literal and published-oracle tests.
  Available-node responses are supplied by the marketplace feature.

- Add an opt-in Registry settings protobuf projection with raw presence semantics,
  shared byte/field bounds and duplicate selected-field refusal, plus focused
  Make/CI checks and literal malformed-wire fixtures. Consumer adoption is pending.

- Introduce an unpublished Protocol package with a bounded Cloud Engine request
  encoder and selected response decoder. Preserve opaque IDs, exact arity,
  strict optional payload admission and canonical Principal identity. Decode-only
  records declare fields, schema and dispatch together.
- Add a separate qualification workspace exercising the public decoder,
  shared Principal type, Serde shape and portable consumer compilation.
- Adopt Toko 0.44.3 and Rust 1.100.0 for local development, including the shared
  `deps-list` and `deps-update` commands and focused encoder checks.
- Add CI definitions for Linux and both macOS architectures, with one repository
  secret-scan/workflow-lint owner and pinned tool/action selections. The initial
  focused suite passed on all three hosted native runners at `08c7221`.
- Enforce the current Protocol and consumer dependency closures using Toko's
  canonical graph checker on native and Wasm selections, including one Types
  identity and dependency-free default/request profiles.

### 🧪 Testing

- Execute marketplace, availability, page requests and fee/status fixtures on
  standalone Wasm, including 2^140 naturals and 32-bit row-count overflow.
  Add the artifact admission target and dependency selection to CI.

- Qualify Registry settings through the public consumer with literal JSON
  presence, exact integer, omission and duplicate/malformed-value fixtures.
- Test composition with the published Registry envelope decoder, retaining
  mutation evidence, application-error priority and distinct envelope/payload
  refusals. Record the independent per-stage budget boundary.
- Execute Registry Wasm admission, including 32-bit length overflow, and measure
  zero heap allocation across seven successful/refused native decode workloads.
  Wire both checks into the focused Make interface and native CI matrix.

- Validate the pinned Query EngineView caller in a disposable snapshot: its seven
  existing tests pass before and after adaptation, along with topology Clippy,
  CLI compilation and a single Types identity. Actual consumer adoption remains
  separate. The maintainer subsequently authorized applying the six-file adapter
  to the actual Query working tree; its focused checks passed without committing
  or pushing that sibling.

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

- Establish the Protocol group policy and move its existing pending notes into
  this ledger. Other product groups gain their own ledgers when implemented;
  qualification packages retain independent private versions.

- Validate generated static projection schemas at compilation instead of
  traversing them again for each response. Incoming schemas remain fully checked.
