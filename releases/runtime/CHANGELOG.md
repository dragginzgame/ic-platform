# Runtime changelog

This group owns ic-platform-runtime, ic-platform-macros and
ic-platform-runtime-guard. Earlier macro/guard history remains in
[history](../history.md) and [Platform](../platform/CHANGELOG.md).

## [0.18.1] - Unreleased

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
