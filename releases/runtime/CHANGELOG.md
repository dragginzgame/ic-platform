# Runtime changelog

This group owns ic-platform-runtime, ic-platform-macros and
ic-platform-runtime-guard. Earlier macro/guard history remains in
[history](../history.md) and [Platform](../platform/CHANGELOG.md).

## [0.21.2] - Unreleased

### 🔧 Changed

- Select Toko Tooling 0.39.12 for development and CI.
- Adopt Testkit 0.100.2 for installed Runtime qualification and prepare its
  required Wasm optimizer in hosted CI.

### 🩹 Fixed

- Keep the independent benchmark's local Runtime identity aligned during release
  metadata preparation, preserving its historical registry comparison baseline.

### 🧪 Testing

- Review current Testkit/Toko/native helpers and explicitly defer optional Wire
  fixture and metadata-editor upgrades to their next maintenance batches before
  the next release freeze; see the [Platform ledger](../platform/CHANGELOG.md).

## [0.21.1] - 2026-11-10

### 🧹 Cleanup

- Replace the imported entrypoint generator, dispatch and syntax helpers through
  a specification-only context. Preserve exact export names, guard/context order,
  explicit codecs, typed external adapter paths and handler inference. Retire
  the previous export directory/tests and source records. No dependency or public
  feature is added.
- Replace the remaining imported raw-call implementation through a specification-only
  context. Preserve public bytes/errors, cycle/wait policy and installed callback
  recovery, while keeping borrowed request data out of the callback environment
  and arbitrary Waker operations outside completion locks. Retire the old CDK
  directory and fingerprint scripts; retain public and installed comparisons.
- Replace imported System API, stable-memory and task-policy adapters through
  specification-only contexts, preserving public names, bytes, numeric ranges,
  traps and protected/weak/migratory behavior.
- Preserve protected-task cancellation before first poll and mark completion
  before destroying a finished inner future. Keep `pin-project-lite` for safe
  direct projection; its wrapper measures smaller than the qualified standard
  async alternative, with no new dependency or unsafe code.
- Replace the imported executor through a specification-only implementation
  context, with separate independent public lifecycle tests. Preserve the sole
  owner, public handles, protected/migratory routing, query isolation, coalesced
  wakes, cached wakers, generational refusal and drop outside executor borrows.
  Retire its source-fingerprint checker in favor of behavior comparisons.
- Reclaim protected tasks when the final method retention is dropped outside a
  context; the previous implementation left that state allocated. Cancellation
  still drops each future once, outside borrows and in stable membership order.

- Replace the rejection catalogue from protocol/API specifications in a separate
  implementation context. Independently written tests preserve all six numeric
  codes, diagnostics, Serde variants and unknown-code refusal. No dependency or
  unsafe code is added.
- Replace copied IC0 bindings with the registry `ic0` 1.2.0 package behind the
  existing explicit exports. Remove the embedded sources and their fingerprint
  machinery. Public signatures, stable-memory access, callback
  ABI and the sole executor remain unchanged; IC0 adds no production transitives.

### ⚡ Performance

- The matched installed raw Wire fixture shrinks by 46,580 uncompressed bytes
  (17.0%), including 28,291 code-section bytes (15.8%). The external-Candid fixture
  shrinks by 54,861 total bytes (7.5%) and 35,256 code bytes (6.9%). Compiler,
  fixture sources, lock and release settings match. These are fixture results,
  not a measured instruction-speed or application-wide size claim.

### 🧪 Testing

- Qualify the independent generator with fresh unit/public consumers, exact
  native symbols and all four installed profiles. A matched old/new generator
  comparison preserves every Wasm import/export; total size changes range from
  −872 to +51 bytes, with no general performance claim.
- Add independent public call contracts, 16 native call tests and an installed
  two-outstanding-call trap/cleanup scenario. Qualify all four raw/prepared and
  Wire/external-Candid profiles. Record the mixed size tradeoff: raw fixtures grow
  466–950 total Wasm bytes, while prepared fixtures shrink 493–1,001 bytes; no
  general speed or size claim is made for the call replacement.
- Qualify independent executor unit/public tests and all four installed raw /
  prepared × Wire / external-Candid lifecycle profiles, including callback traps,
  cancellation, upgrades and stable rollback. Keep downstream scheduler acceptance
  with Scheduling and native macOS evidence separate.

### 🛠️ Tooling

- Adopt Toko Tooling 0.39.8 and audit product/qualification package placement.
  All five maintained crates already use the product layout; disposable fixture
  templates retain their documented locations; the native provider is retired.
  Production boundaries and the three release groups remain unchanged.

## [0.21.0] - 2026-11-09

### ⚠️ Breaking

- Remove the macro package's `candid_query` and `candid_update` exports. Use
  `codec_query` / `codec_update` through Platform's `candid-codec-endpoint`
  adapter with explicit decode/reply limits, arity and empty-envelope policy.
  The existing generator retains guard ordering and the sole Runtime executor;
  no upstream Candid compatibility macro remains.

### 🧪 Testing

- Inherit published Toko Tooling 0.37.1 in Runtime, macros and the singleton guard.
  Release all three packages together with the exact guard requirement.

### 🛠️ Tooling

- Remove downstream Timers composition from Platform release admission and
  qualification-lock custody. Retain standalone Runtime and singleton-refusal
  coverage; Scheduling owns scheduler compatibility tests.

- Adopt published Toko Tooling 0.37.1, migrating to `TOKO-TOOLING.md`,
  `.toko-tooling.snapshot`, `toko-tooling-*` commands and Toko LOC metadata.
  CI and publication adapters admit the renamed snapshot before effects;
  existing product release requirements remain unchanged.

## [0.20.3] - 2026-11-08

### 🛠️ Tooling

- Inherit the exact published `toko-tooling` 0.36.0 development selection in
  Runtime, macros and the singleton guard, with source-bundle Machete exemptions.
  Normal/build dependencies, generated code and executor behavior are unchanged.

## [0.20.2] - 2026-11-08

### 🧹 Cleanup

- Share macro-test expansion rendering and ordered-statement assertions,
  retaining all generator and native-linking checks. Generator output and
  Runtime behavior are unchanged; the test source is included in the macros
  package archive.

## [0.20.1] - 2026-11-08

### 🧹 Cleanup

- Share mechanical option parsing between ordinary entrypoints and the external
  Candid adapter. Preserve emitted entrypoints, repeated guard ordering, explicit
  codecs and duplicate/type/retired-option diagnostics.

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
