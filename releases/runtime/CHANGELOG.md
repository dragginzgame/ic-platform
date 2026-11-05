# Runtime changelog

This group owns ic-platform-runtime, ic-platform-macros and
ic-platform-runtime-guard. Earlier macro/guard history remains in
[history](../history.md) and [Platform](../platform/CHANGELOG.md).

## [0.19.2] - 2026-11-05

### 🔧 Maintenance

- Replace obsolete/broad lint allowances with scoped expectations while retaining
  the original embedded-source fingerprints and callback/ABI behavior.
- Derive the stable-memory error’s mechanical display/source implementation using
  the existing dependency, preserving its variant and message.

### 🧪 Testing

- Keep the Timers composition dependency in the generated workspace catalog and
  canister dependency table when source-role metadata is present. Preserve the
  frozen supplier selection and all installed runtime checks.

## [0.19.1] - 2026-11-04

### 🧪 Testing

- Move owned signature-analysis, external-Candid adapter, call-policy and System
  API text tests into adjacent `tests.rs` modules. Preserve test bodies and
  behavior, with unchanged protected runtime regions and source fingerprints.
- Separate reusable prepared-call and installed-runtime driver/canister tests from fixture
  implementations. Generated consumers copy their test modules alongside the
  source, preserving raw/Wire/external-Candid feature coverage.

## [0.19.0] - 2026-11-02

### ⚠️ Breaking

- Adopt Types 0.18 as the canonical Principal owner shared with Platform.
  Rebuild consumers against this Runtime generation; its coordinated macros and
  exact singleton guard advance together. Execution, IC0 and call bodies remain
  unchanged. Raw IC0/executor builds still avoid application Types.

## [0.18.1] - 2026-11-02

### ➕ Added

- Expose the existing safe stable-memory grow/read/write primitives through
  `ic0`, alongside size, without enabling the executor, guard or Types
  ([#135](https://github.com/dragginzgame/ic-platform/issues/135)). Preserve
  upstream argument order, traps and growth refusal; Storage retains governance.

## [0.18.0] - 2026-11-02

### ⚠️ Breaking

- Require Rust 1.99.1 across Runtime, macros and the exact singleton guard (#133).
  Preserve raw IC0/executor isolation and all owner/composition protocol and
  lifecycle profiles; consolidate identical minimum/development compiler runs.

## [0.17.1] - 2026-11-01

### ⚠️ Breaking

- Move IC0, executor, CDK adapters and prepared calls into the sole canonical
  runtime owner. Timers and Wire Endpoint must adopt this owner directly.
  Preserve callback/trap/upgrade behavior, guarded source and canonical Types.
- Advance macros/guard together to distinguish this owner from the previously
  embedded executor. The exact guard refuses mixed owners; facade versions that
  delegate to the same runtime can coexist without duplicate executors.

### 🧹 Cleanup

- Keep Candid, crypto/certification and native clients outside this release group.
  Direct runtime macros default to the runtime path; facade entrypoints retain
  their existing paths and delegate to the same implementation.
