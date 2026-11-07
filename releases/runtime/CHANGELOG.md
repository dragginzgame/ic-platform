# Runtime changelog

This group owns ic-platform-runtime, ic-platform-macros and
ic-platform-runtime-guard. Earlier macro/guard history remains in
[history](../history.md) and [Platform](../platform/CHANGELOG.md).

## [0.20.0] - 2026-11-07

### 💥 Breaking

- Keep one ordinary entrypoint macro family, exported through Runtime’s `cdk`
  module and defaulting to that owner. Remove the prefixed `runtime_*` macro
  exports and the facade’s ordinary front door; explicit crate-path overrides,
  export names, codecs, guard ordering and executor behaviour are preserved.

## [0.19.5] - 2026-11-07

### ⚡ Performance

- Locate interior protected-task entries once and bulk-shift their Vec tail,
  preserving stable cancellation/destructor order and existing wake routing.
  The matched 8,192-task FIFO fixture uses 69% fewer installed Wasm instructions;
  the Runtime probe shrinks by 347 bytes in total uncompressed and code-section
  size. Reverse cancellation retains the existing tail fast path.
- Avoid redundant zero initialization before complete IC0 copies of reject/method
  text, root keys and certificates. Share the existing message-copy initialization
  proof in one private helper; preserve UTF-8 errors, absence and trap semantics.
  A matched 1,111-byte certificate read saves 1,227 IC instructions. The Runtime
  probe shrinks a further 28 total uncompressed/code bytes.

### 🔧 Maintenance

- Remove the unused private null-context entry helper and its dead-code
  expectation, retaining null method IDs and migratory wake routing.

## [0.19.4] - 2026-11-06

### ➕ Added

- Expose the existing safe `ic0::stable64_read_uninit` helper for complete
  initialization of caller-provided buffers, preserving IC trap semantics and
  the dependency-free IC0 boundary. Storage can avoid zeroing buffers before IO.

## [0.19.3] - 2026-11-06

### ⚡ Performance

- Avoid scanning protected-task membership when removing its final entry,
  preserving cancellation order and callback/trap cleanup. Retain a focused
  native benchmark for forward and reverse removal patterns.

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
