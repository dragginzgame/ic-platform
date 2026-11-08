# Platform changelog

This group covers ic-platform. The retired PocketIC [ledger](../pocket-ic/CHANGELOG.md) preserves its history.
Earlier coordinated history is retained once in [history](../history.md).
See also the [Types ledger](../types/CHANGELOG.md).

## [0.26.1] - Unreleased

### 🛠️ Tooling

- Adopt Shared 0.31.10: format only Cargo workspace members and use
  `make cloc CLOC_DEPTH=0` for crate rows or `CLOC_DEPTH=1` for module rows.
  Crate/module selectors narrow the report. Private tooling maintenance does
  not require a crate release.
- Cache authenticated pinned downloads before hosted tool setup and save after
  setup. Run one complete-history, redacted secret scan in repository CI,
  including documentation changes, independently of host and release matrices.
- Retire the unadopted Candid prototypes while preserving immutable reproduction
  sources, licenses and historical results. Separate current migration contracts
  from their earlier investigations.
- Share benchmark preparation, admission-test tools and fixture copying; batch
  release-lock observations and retain dependency-graph refusals in one pass.
- Organize installed qualification by scenario and declare fixture method names
  and IDs once, preserving their bytes, assertion order and lifecycle checks.

## [0.26.0] - 2026-11-08

### 💥 Breaking

- Retire the pure BLAKE3 feature and forwarding module after IcyDB and Wire
  removed their source callers. Existing hashing consumers select the same
  upstream implementation directly; their algorithm, features and bytes stay
  unchanged ([#150](https://github.com/dragginzgame/ic-platform/issues/150)).

### 🧹 Cleanup

- Share exact Candid reply-arity admission and protocol decoder generation while
  preserving diagnostics, budgets, optional-result behavior and external bytes.
- Consolidate test data decoding, native fixture setup and source-fingerprint
  normalization. Keep separate source guards, refusal tests and explicit cleanup.
- Render runtime qualification manifests in one pass and make repeated copy
  measurements opt-in. Ordinary qualification retains both copy paths at every
  payload size, with ten queries instead of sixty.

## [0.25.1] - 2026-11-07

### 🧪 Testing

- Adopt the published async Testkit client in native fixtures, with explicit
  finite request budgets, one caller-owned runtime and awaited teardown. Preserve
  installed protocol, executor and cryptographic integration coverage
  ([#148](https://github.com/dragginzgame/ic-platform/issues/148)).
- Reuse unchanged Linux release-policy fixtures through Shared's exact-input
  proofs and fresh per-group admissions. Keep every required release stage,
  version-dependent check and recovery obligation
  ([#149](https://github.com/dragginzgame/ic-platform/issues/149)).

### 🔧 Changed

- Adopt Shared Tooling 0.31.6 and its approved direct pure-hashing policy.
  BLAKE3 facade retirement remains gated on caller migration
  ([#150](https://github.com/dragginzgame/ic-platform/issues/150)).

## [0.25.0] - 2026-11-07

### 💥 Breaking

- Remove ordinary facade entrypoint macros and its `cdk` feature. Select
  `ic-platform-runtime/cdk` and import entrypoints from its canonical owner;
  explicit external-Candid adapters remain in Platform.

### 🔧 Maintenance

- Require the complete release gate for every group; retire the inactive
  selected-stage pilot and its native qualification workflow. Receipt, source,
  metadata and recovery admission remain mandatory.
- Preserve real preparation/rollback/custody checks for all three groups while
  running the complete metadata-fixture delivery/recovery cycle once, for Runtime.
- Pin the immutable Timers qualification source that also accepts Runtime 0.20,
  retaining the existing implementation and all installed composition checks.
- Resolve qualification locks offline inside metadata preparation, preserving
  foreign selections and necessary inactive registry owners across minor bumps.
  Exercise actual post-version graphs in the preparation fixture, and retain
  consumer failure/concurrent-edit checks with the current admission profiles.

### 🔧 Tooling

- Adopt Shared Tooling 0.31.5, including release-checker isolation and grouped
  dispatch, clearer refusal after active LOC scratch is removed, and the rule
  that output owners create their required directories on every fresh run.

## [0.24.0] - 2026-11-07

### 💥 Breaking

- Remove native offline IC public-key derivation, its `secp256k1` feature and
  the `ic-secp256k1` dependency. Consumers must remove the feature/imports;
  [Canic’s fetch-and-cache migration](https://github.com/dragginzgame/canic/issues/739)
  remains consumer-owned. The separate `ecdsa` capability and Agent are unchanged.
- Remove unconfigured `candid::decoder::{decode_one, decode_args}` shortcuts.
  Select the bounded helpers or an explicit decoder configuration. Caller
  migrations remain in [Canic #740](https://github.com/dragginzgame/canic/issues/740),
  [Blob #119](https://github.com/dragginzgame/ic-blob-storage/issues/119) and
  [IC Crates #25](https://github.com/dragginzgame/ic-crates/issues/25).

### 🔧 Tooling

- Stop replaying Shared's generic changelog-parser regression matrix in
  `check-release-commands`; retain Platform's group preparation, adapter and
  recovery coverage and normal release changelog admission.
- Retire the remaining unadopted crypto packaging patches; registry suppliers
  and their cryptographic implementations remain unchanged.

## [0.23.6] - 2026-11-07

### 🩹 Fixed

- Adopt Shared 0.31.1's Bash 3.2 checksum-parser correction for macOS tool setup.
- Use Shared's portable checksum helper throughout the optional frozen-batch
  qualifier, preserving archive checksum records without requiring GNU tools.
- Refuse runtime-lock checks and preparation when the release-group roster
  reader fails, including partial output, before changing either fixture lock.
- Refuse embedded IC0/executor source verification when its checksum-roster
  reader fails or emits no entries, including failure after partial output.

### ⚡ Performance

- Store Candid admission's type-child references in one shared buffer, preserving
  traversal order, wire admission, work quotas and the 16-level nesting ceiling.
  The matched external-Candid probe shrinks 372 total uncompressed bytes and
  402 code-section bytes. A 1,024-type header uses 44% fewer IC instructions;
  simple recursive refusals cost 161 additional instructions in the same probe.

### 🔧 Tooling

- Project adjacent release fields from one admitted group plan per command;
  retain fresh admission at subsequent phases and all rollback/recovery cases.
- Carry publication's admitted package selection into Clippy without changing
  compiler or lint flags; ordinary release qualification retains its full gate.
- Compare Wasm sizes on relevant main-branch pushes as well as pull requests
  and manual dispatches.
- Add opt-in matched System API and Candid admission instruction probes alongside
  the existing Wasm size profiles; ordinary installed regressions stay separate.

## [0.23.5] - 2026-11-06

### 🩹 Fixed

- Adopt Shared 0.31.0, including stricter snapshot/checksum verification,
  schema-2 release journals and guarded batch reconciliation and retirement.

### 🔧 Tooling

- Capture fresh grouped releases in a frozen candidate, prepare its own tools
  before version lookup and admit recovery offline. Explicit supplier-ordered
  cohorts use the shared batch lifecycle commands and renamed `RELEASE_GROUPS`,
  setup and publication settings; retired settings refuse. Publication remains
  opt-in.

## [0.23.4] - 2026-11-06

### 🔧 Tooling

- Adopt Shared 0.30.7 and enforce reviewed Rust CBOR source allowances and
  standalone fixture Cargo boundaries. Use Platform's typed codecs in runtime
  and fuzz callers instead of their direct generic CBOR dependencies.
- Supply the opt-in frozen release runner without activating it before
  Platform's actual release and publication hooks are qualified.

## [0.23.3] - 2026-11-06

### ✨ Features

- Expose standard ICP account-identifier derivation under `icp-ledger-types`,
  preserving upstream SHA-224/CRC32 bytes so consumers can remove local helpers
  ([#147](https://github.com/dragginzgame/ic-platform/issues/147)).

### 🔧 Maintenance

- Remove the facade's unused `serde_test` development dependency; Types and
  Runtime retain their own test dependencies.
- Select allocation-only encoding and byte-Serde utility features, preserving
  external bytes without forcing their standard-library features on consumers.

### 🧪 Testing

- Extend malformed-input fuzzing to certificate, hash-tree and typed IC packet
  CBOR, and share ingress compatibility fixtures between Rust and the browser.
- Compare representative Wasm capabilities against a committed baseline with
  identical build inputs, reporting total and code-section growth separately.

## [0.23.2] - 2026-11-05

### ➕ Added

- Expose the exact management `CreateCanisterArgs` on native and Wasm and add the
  narrow `candid::protocols::management::create_canister` codec, reusing existing
  settings/reply identities and preserving caller transport, cycles and recovery
  policy ([#146](https://github.com/dragginzgame/ic-platform/issues/146)).

### 🐛 Fixed

- Format rendered release manifests before staging so the normal commit hook
  preserves the exact release tree; qualify real hooks in grouped recovery tests.

### 🔧 Tooling

- Adopt Shared 0.30.5 with the qualified generic inherent-header correction and
  enforce named external Candid declarations in the existing boundary gate.
- Keep compact experimental conclusions in Git, retire obsolete trial tables
  and probes, and default new benchmark output to `target/evidence/`.
- Assemble native Profiler adapters through ordinary Rust module discovery,
  preserving the independent recorder, viewer assets and public aliases; include
  extracted patch tests and compiled-owner lint checks in native qualification
  ([#140](https://github.com/dragginzgame/ic-platform/issues/140)).

## [0.23.1] - 2026-11-05

The local release stopped before tag/push/publication when the commit hook
formatted a rendered feature array after staging. Its commit and journal are
retained; 0.23.2 delivers the reviewed changes through forward recovery.

## [0.23.0] - 2026-11-05

### ⚠️ Breaking

- Preserve Testkit's optional subnet-metrics leaf contract: `decode_subnet_metrics`
  now returns `Option<SubnetMetrics>`, accepting CBOR null as absence while
  retaining complete-input limits, malformed/trailing-data refusal and full u128
  cycle counts ([#145](https://github.com/dragginzgame/ic-platform/issues/145)).
  Callers must handle the optional result; certificate path lookup stays consumer-owned.

### 🔧 Tooling

- Use Shared's complete validation proofs instead of locally interpreting runner
  events and timings. Keep source/host/release custody and digest-bound recovery
  after metadata preparation; a stage PASS does not imply fresh execution
  ([#141](https://github.com/dragginzgame/ic-platform/issues/141)).

## [0.22.3] - 2026-11-05

### ➕ Added

- Add explicit `certification-cbor` APIs for byte-bounded standard certificate
  and canister-signature encoding/decoding. Keep caller-selected budgets,
  witness construction, store policy and verification with consumers; preserve
  upstream certificate/tree identities and standard CBOR bytes.
- Supply native typed IC packet codecs and explicit development fixtures so
  consumers can remove direct CBOR dependencies without dropping protocol or
  malformed-input checks ([#142](https://github.com/dragginzgame/ic-platform/issues/142),
  [#143](https://github.com/dragginzgame/ic-platform/issues/143)). Include the existing
  strict certificate/tree admission contract and preserve separate tagged/untagged
  packet bytes, upstream identities and caller-owned verification/effect policy.
- The same native fixture selection supplies Canic's query, controller-list and
  canister-range packet tests without generic CBOR exports
  ([#144](https://github.com/dragginzgame/ic-platform/issues/144)).

## [0.22.2] - 2026-11-05

### 🔧 Tooling

- Adopt Shared 0.30’s source-role LOC report, label generated proof/benchmark
  packages and use ordinary source files for the Canic benchmark driver.
- Keep CI-selection and release-recovery fixtures complete through Shared’s
  declared companion closure.
- Repair exported Shared documentation through committed adoption and check every
  exported Markdown payload in the normal documentation gate.
- Enforce bounded, dated recovery-evidence admission before release/publication;
  preserve the approved historical archive outside Git administrative state.

### 🧪 Testing

- Scope management schema comparisons to their native registry supplier and lint
  expectations to the actual native/Wasm visibility, preserving external DTO bytes.
- Move native Profiler inline tests into ordinary test modules and tighten owned
  lint scopes. Paired native-server qualification remains required under
  [#140](https://github.com/dragginzgame/ic-platform/issues/140).

## [0.22.1] - 2026-11-05

### 🐛 Fixes

- Restore upstream packed-map and sequence canister-signature envelope admission
  while preserving verification, delegated-key admission and authenticated time.

### ⚡ Performance

- Remove a redundant BLS key serialization/reparse from signature admission using
  the unchanged upstream checked point parser. Matched Canic release artifacts
  save about 3.3 KB of code each; retain all cryptographic and freshness checks.
  See the [matched Wasm report](../../docs/audits/platform-022-canic-wasm-2026-11-05.md).

## [0.22.0] - 2026-11-04

### ⚠️ Breaking

- Return BLS cryptography to upstream dependencies and delete the embedded curve,
  verifier, algorithm tests and source guards. Preserve key/signature bytes and
  identity-key rejection through small delegating key adapters. Signature and
  error types now have upstream identity; align consumers that exchange them.
  Defaults and RNG remain disabled, with allocated verification selected.

- Return certification and canister-signature formats/hashing/verification to
  registry dependencies. Certificate/tree types now share upstream identities
  with native Agent; remove recursive conversion adapters. Preserve owned issuer
  fields, DER framing, root/delegated identity-key refusal and authenticated time.
  Key admission/time extraction repeat CBOR decoding. Upstream packaging restores transitive
  Candid/printer, raw IC0, CBOR and a lazy root cache; no store API is exported.
  Remove embedded algorithm suites/source guards and the unused direct 4.x
  representation-independent-hash test dependency. No Wasm/performance win claimed.

### 🧪 Testing

- Refresh owner/Timers qualification locks for the registry certificate boundary
  and compatible Testkit 0.84.1; retain its published native Platform separately.

- Use ordinary module discovery for the shared malformed-Candid test/fuzz fixture.
  Adopt Shared 0.29.3's test-layout, changelog and snapshot-export rules; preserve
  imported source-layout exceptions and keep the root changelog as navigation.
- Format the shared malformed-Candid corpus directly while keeping the fuzz
  harness's copied module from breaking standalone formatting checks.
- Separate stable-identity and Candid packaging experiment tests from fixture
  implementations, preserving feature selection and generated-package coverage.

## [0.21.0] - 2026-11-03

### ⚠️ Breaking

- Transfer the maintained PocketIC client to `ic_testkit::pocket_ic`. Remove the
  client package, facade `pocket-ic` feature and module, and its release group.
  Testkit now owns one version for client and lifecycle helpers. Keep historical
  releases; migrate native test fixtures to Testkit 0.84.0. No Runtime,
  Types, Agent or crypto implementation changes
  ([#139 — Transfer PocketIC client ownership to Testkit](https://github.com/dragginzgame/ic-platform/issues/139),
  [Tooling #324 — Merge the PocketIC client into ic-testkit](https://github.com/dragginzgame/ic-tooling/issues/324)).
- Adopt Shared 0.29.1 ownership and contribution rules. Release selection now admits Platform,
  Runtime and Types only; retain full release validation pending fresh native
  qualification of this three-group mapping.

### 🧪 Testing

- Integrate #137's validation receipts with release preparation and recovery;
  reject changed source, execution settings or retained evidence. Add full and
  selected qualification on Linux and both macOS architectures. Enable qualified
  owner-source plans on Linux x86_64 and macOS 15 Intel/ARM64 after all three
  native qualifications pass. Facade, shared/unclassified changes and unknown
  hosts retain the full roster; fresh releases do not reuse qualification receipts.

## [0.20.0] - 2026-11-03

### ⚠️ Breaking

- Forward PocketIC 0.20's bounded snapshot transfer API and the new exhaustive
  `ClientError::ResponseTooLarge` variant. Consumers matching every error variant
  must handle the additional case. Types and Runtime identities are unchanged.

### 🧪 Testing

- [#137](https://github.com/dragginzgame/ic-platform/issues/137): add a guarded
  four-group validation pilot with complete stage rosters, source-bound receipts
  and failure/recovery tests. Linux full/selected mappings pass; keep the full
  release gate pending recovery integration and remaining native qualification.

## [0.19.0] - 2026-11-02

### ⚠️ Breaking

- Adopt the new canonical Types family and its Account subaccount byte encoding.
  Affected RPCs need matching schemas and fresh contract IDs; persisted data
  requires consumer-owned migration. Runtime retains the same canonical Principal.
- Forward PocketIC's fallible client API. Handle ClientError explicitly for
  construction and operations; uncertain submissions must not be replayed.

### 🧹 Maintenance

- Retain upstream ic-agent behind the explicit native facade; withdraw the
  unreleased owned Agent and its cryptographic-provider migration. Keep upstream
  type identities and consumer-owned trust, retry and admission policy.
- Remove unused direct ic_principal and thiserror dependencies from the facade;
  Types retains Principal implementation ownership and feature forwarding.

### 🩹 Fixed

- Prepare native-provider CI's manifest tools before reading the compiler floor
  ([#133](https://github.com/dragginzgame/ic-platform/issues/133)). Fail immediately
  if that lookup fails, before rendering an empty Cargo `rust-version`.

## [0.18.0] - 2026-11-02

### ⚠️ Breaking

- Remove the unused `stable-structures` feature, `stable_structures` module and
  `ic-stable-structures` dependency. Runtime's raw stable-memory system calls
  remain available; storage and allocation stay with their product owners.

- Require Rust 1.99.1 for the facade (#133). Keep
  isolated native/Wasm capability and archive proofs; qualify each compiler
  selection once instead of repeating the old 1.91/development matrix.
- The optional PocketIC facade now uses management DTO identity 0.11. Native
  clients can depend directly on the independently released `ic-platform-pocket-ic`;
  migration details are in the [client contract](../../docs/pocket-ic-client.md).

### 🧹 Maintenance

- Adopt Shared Tooling 0.27.27 at committed revision 651f5f7. Keep the full
  release gate for this combined compiler/ownership batch; the optional
  group-validation plan requires separate consumer mapping qualification.

- Keep the README's checked feature catalog, setup and release-group choices
  concise; move contributor and runtime qualification detail into a linked guide.

- Remove the direct test-only `assert_matches` dependency ([#123](https://github.com/dragginzgame/ic-platform/issues/123)).
  Standard assertions preserve match guards and failure values;
  source reconstruction retains the original upstream test fingerprints.
- Prepare historical registry owners used by singleton/composition rejection
  fixtures on fresh CI runners, and print fixture logs on failure. Principal
  mutation checks verify that the source changed and use the shared checksum
  verifier for vector diagnostics.
- Select documentation-only native CI only for an exact reviewed prose allowlist;
  retain all native hosts and full qualification for code or uncertain changes (#125).
- Bound new native profiler compiler scratch to 256 GiB/seven idle days under
  assembly custody. Retain executable copies and immutable receipts separately;
  preserve historical and operator-selected external caches (#132).

## [0.17.0] - 2026-11-01

### ⚠️ Breaking

- Delegate runtime capabilities to the independent Runtime 0.17 owner. Align
  Timers and Wire Endpoint once; later crypto/certification facade releases
  leave executor and generator identity unchanged. See the
  [Runtime ledger](../runtime/CHANGELOG.md) for ownership and qualification.

- `principal-self-authenticating` selects only owned Principal and upstream
  SHA-224. Select `principal-external` separately for registry conversions.
  The exact upstream derivation and resulting bytes remain unchanged;
  self-authentication-only production graphs drop `ic_principal`.

- Maintain the PocketIC 16.1 Rust client in Platform's native implementation
  package. Keep existing `ic_platform::pocket_ic` imports, method signatures,
  HTTP/JSON protocol and operation behavior; client, builder and client-owned DTOs
  now have Platform implementation identity. Consumers exchanging concrete
  registry `pocket-ic` types must align on the facade after adoption.
  The simulator server, execution engine and Testkit server pin are unchanged.

### 🧹 Cleanup

- Separate installed Platform runtime qualification from Timers composition.
  Keep both release gates and independent CI/advisory results; preserve the
  singleton guard and reuse the fixture with separately maintained locks.

- Keep client dependencies behind the existing native-only feature, with no
  Candid/crypto expansion in default, executor-only or Wasm PocketIC selections.
  Preserve Apache source provenance and qualify the existing Testkit contract.


### 🩹 Fixed

- Use physical workspace paths during grouped release preparation, including
  aliased macOS temporary directories; retain rollback and resume checks.

### ⬆️ Dependencies

- Adopt Host Process/FS/Artifacts 0.73.2 and core Wire 0.13.1 in the native
  Profiler Cargo/Bazel inputs. Keep Metrics 0.24.0 independently locked and
  preserve process observations, gzip bytes, frame admission and saved reports.

## [0.16.0] - 2026-11-01

### ⬆️ Dependencies

- **Breaking:** update the explicit Ed25519 surface to ed25519-dalek 3 and
  curve25519-dalek 5/signature 3, preserving fixed signing and strict-verification
  byte fixtures. Native Agent retains its separate older generation.
- Update the owned entrypoint generator to Syn 3 and the development-only
  representation-independent hash reference to 4.0.1.

- Maintain the existing BLS12-381 arithmetic inside Platform's private BLS
  module, preserving algorithms, original tests/vectors and MIT/Apache notices.
  Remove the registry curve from standalone BLS and canister-signature
  verification; these profiles now have zero DFINITY registry dependencies.
  Native Agent retains its independently selected wrapper/curve.

- **Breaking:** use the canonical owned Principal in canister-signature keys and
  certified subnet-range decoding. Remove registry Principal from standalone
  creation/verification profiles while preserving DER/CBOR bytes, length-first
  ordering and the original cryptographic bodies.

- **Breaking:** own certificate, hash-tree and RbTree identities and maintenance
  inside Platform. Standalone certification and canister-signature profiles no
  longer select registry certification. Preserve upstream algorithms, Serde
  bytes and tests; retain explicit native Agent certificate conversions while
  Agent/PocketIC remain externally maintained.

- **Breaking:** maintain the upstream BLS signature wrapper inside Platform,
  preserving public paths, crypto bodies and key/signature bytes while changing
  Rust type ownership. Remove its direct registry dependency and use std's
  lazy cache; retain upstream curve arithmetic. Native Agent still brings its
  registry wrapper transitively.
- **Breaking:** move the optional ECDSA surface to upstream K256 0.14/signature 3
  identities and enforce corrected bigint selection in downstream locks. Callers
  must adapt public-key serialization and explicit normalization APIs. This
  security upgrade retains a modest measured Wasm footprint increase with the
  qualified backend; Agent and native public derivation retain their older
  dependency generation.
  [Migration and compatibility](../../docs/migration.md#standard-secp256k1-ecdsa-maintenance-boundary).

### 🩹 Fixed

- Scope native entrypoint symbols to the consuming Cargo crate so multiple
  canister libraries can link in one host process. Retain portable method-name
  encoding and exact Wasm exports, codecs, guards and executor contexts.
  [#129](https://github.com/dragginzgame/ic-platform/issues/129).

- Apply the exact upstream BLS identity-public-key rejection and regression
  vector for [#127](https://github.com/dragginzgame/ic-platform/issues/127).
  Raw BLS and canister-signature verification reject forged identity-key
  signatures; source guards preserve the original fingerprints and admit only
  this reviewed correction. Native Agent's registry verifier is unchanged.

- Select RustCrypto's existing 32-bit backend for repository Wasm builds,
  recovering most of the ECDSA upgrade's size growth and reducing measured IC
  verification instructions. Canister consumers must carry the same flag in
  their build configuration; a published dependency cannot impose it.

### 🧪 Testing

- Qualify published Timers 0.35.1 with the current embedded runtime, restoring
  combined installed callback, cancellation and upgrade coverage. Preserve
  independent Types release custody when registry consumers reference local
  package identities.
- Retain BLS's allocation feature after measuring the upstream alternative:
  modest Wasm savings cost roughly 51% more verification instructions. Add
  reproducible size/instruction probes without maintaining cryptographic code.
  [Measurements and tradeoffs](../../docs/benchmarks/upstream-2026-10-31/README.md).

## [0.15.3] - 2026-10-31

### ⬆️ Dependencies

- Adopt published Host Process/FS/Artifacts 0.73.1, arithmetic-only Metrics 0.24
  and core Wire 0.13 in the native Profiler Cargo/Bazel inputs. Keep gzip bytes,
  counter quality, frame admission and saved report formats unchanged.

- Keep native Profiler gzip on Host Artifacts directly and remove the
  forwarding-only registry Platform dependency. Preserve codec bytes, levels,
  strict admission and existing API/recorder byte limits.
  [#122 — Native Profiler Host gzip](https://github.com/dragginzgame/ic-platform/pull/122).

### 🥾 Governance

- Adopt Shared Tooling 0.27.22 with a complete root member catalog, group alias
  rosters and cross-group checks for paths and full current versions. Label local
  groups and external dependency roles without adding a graph edge.
- Present release groups once in the catalog with short titles and separate
  changelog comments; remove the duplicate header map and repetitive commentary.
- Preserve normal whole-target Cargo cleanup and explicit evidence-retention
  boundaries introduced by Shared 0.27.20.

## [0.15.2] - 2026-10-31

### 🩹 Fixed

- Adopt symmetric crate groups with explicit package versions and group selection
  for release, resume and publication. Preserve numeric versions, dependency
  selections, transactional runtime metadata and recorded release evidence.

### 🥾 Governance

- Adopt Shared Tooling 0.27.19, including symmetric group ownership and delegated
  release/publication workflows, and move canonical group ledgers under releases/.
  Root Cargo has no shared version; root changelog is an index.

## [0.15.1] - 2026-10-31

### 🩹 Fixed

- Restore independent Platform/Types release groups in one workspace, starting
  from existing 0.15.0 identities. Preserve Types during runtime-only releases,
  qualify transactional runtime metadata custody and select publication by group.
  [#118 — Independent identity releases](https://github.com/dragginzgame/ic-platform/issues/118).

### ⬆️ Dependencies

- Adopt Shared Tooling release commit 0.27.17 (`39ba983`), including
  common metadata custody, Cargo group comments and canonical-ledger policy,
  through the snapshot exporter.
