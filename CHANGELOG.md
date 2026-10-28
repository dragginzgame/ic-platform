# Changelog

## [0.7.3] - 2026-10-28

### 🔧 Changed

- Adopt IC Wire 0.7.19 for host metadata and runtime qualification, including
  its matching derive crate. Preserve the existing Platform surface, framed
  protocol bytes and caller-owned admission policies.

## [0.7.2] - 2026-10-28

### ➕ Added

- Add `ic-call` for prepared one-shot calls with owned dynamic method names and
  borrowed or owned raw arguments. Reuse Platform's existing call implementation,
  cost query and error identities with explicit waits and separate codecs
  ([#75](https://github.com/dragginzgame/ic-platform/issues/75)).
- Qualify browser declarations with TypeScript 6.0.3 and the producer compiler,
  documenting the consumer floor separately from application tooling peers
  ([#76](https://github.com/dragginzgame/ic-platform/issues/76)).

### 🩹 Fixed

- Publish native preparation summaries in the consuming checkout, including
  cached/forced preparation, while keeping candidate receipts separate
  ([#77](https://github.com/dragginzgame/ic-platform/pull/77)).
- Reject truncated and noncanonical short-form canister-signature DER before
  the unchanged upstream parser, preserving valid keys and cryptographic code
  ([#72](https://github.com/dragginzgame/ic-platform/issues/72)).
- Share Host/Origin admission across the native provider's profiling APIs,
  trace/snapshot controls and viewer startup on both listeners; preserve local
  nonbrowser clients ([#74](https://github.com/dragginzgame/ic-platform/issues/74)).

## [0.7.1] - 2026-10-27

### ➕ Added

- Own the opt-in instrumented PocketIC source provider for IC Profiler, keeping
  native adapters and IC build inputs below the facade while the consumer retains
  its recorder/viewer. Exclude Motoko/lifeline from the native server profile
  ([#65 — Instrumented PocketIC boundary for IC Profiler](https://github.com/dragginzgame/ic-platform/issues/65)).

- Expose optional native bounded gzip encoding/decoding through IC Tooling's
  existing codec for IC Profiler, retaining consumer-owned levels and limits
  without direct compression-backend dependencies in the consumer.

### 🩹 Fixed

- Publish native preparation summaries under the consuming checkout, including
  cached preparation, so split provider/consumer layouts cannot leave a stale
  receipt behind. Keep candidate probes separate from the selected assembly.

- Admit gzip's independent native-only feature in dependency-graph validation,
  while retaining Principal prerequisites for other native features and strict
  target/dependency checks ([#71](https://github.com/dragginzgame/ic-platform/pull/71)).

### 🔧 Changed

- Adopt Shared Tooling 0.26.26 and its visible formatting passes while preserving
  grouped manifests, all standalone Rust fixtures and edition 2024. Qualify
  check/write behavior, failure stopping and isolated staged-hook formatting.
- Share the Principal parser across input types and serialize human-readable
  Principals directly from their stack buffer. Simplify Account formatting and
  derive existing typed errors without changing messages or source chains.
- Normalize Principal self-authentication inputs at the owned wrapper to reduce
  generic code duplication while retaining the unchanged upstream implementation.
- Keep System API UTF-8 validation and panic reasons, omitting rejected-byte
  debug dumps from invalid-text panics to reduce retained runtime formatting.
- Validate feature contracts in one pass, preserving isolation diagnostics and
  adding negative coverage for removed dependencies and changed selections.

### ➕ Added

- Add a manual Rust 1.91 Wasm size sweep covering Principal/Account, raw calls,
  external management Candid and unchanged signature verification. Record raw
  and optimized sizes across five compiler recipes; application savings and
  instruction costs require separate consumer qualification.
- Add isolated-change attribution and binary/self-authentication controls so
  text-probe savings are not mistaken for costs of the raw Principal/Account types.
- Add opt-in Candid decoder/sequence comparison probes and a regression for
  enormous admitted size hints. Retain the existing implementations after
  measuring the prototypes' size and native-runtime tradeoffs.

## [0.7.0] - 2026-10-27

### ⚠️ Breaking

- Retire Candid JavaScript/TypeScript compiler exports and the compiler-only
  `IDLMergedProg` helper. Owned bindings use Wire; required external Candid
  parsing, type checking, textual arguments and service comparison remain
  available. This narrows Platform's API without removing the upstream parser
  or its transitive generator dependencies
  ([#38 — Retire Candid frontend compiler exports](https://github.com/dragginzgame/ic-platform/issues/38)).

## [0.6.21] - 2026-10-27

### ➕ Added

- Publish the separate browser package `@tokostudios/ic-platform` 0.1.1 with
  the signing-request and expiry exports needed by Auth; SDK Core remains 6.1.1
  ([#59](https://github.com/dragginzgame/ic-platform/issues/59)).
- Expose pinned management lifecycle/settings/ECDSA and complete change-origin
  records needed by Canic's external adapters, preserving upstream identities.
- Expose ICRC transfer arguments/memos and a distinct ICRC-21 consent namespace.
  Consumers retain authorization, limits, original retry bytes and consent policy.

## [0.6.20] - 2026-10-27

### ➕ Added

- Supply optional CMC, legacy ICP Ledger and Cycles Ledger contracts without
  service engines. Preserve rate certificates, archive callback signatures and
  complete creation/withdrawal refund and duplicate evidence
  ([#61](https://github.com/dragginzgame/ic-platform/issues/61),
  [#62](https://github.com/dragginzgame/ic-platform/issues/62)).
- Supply canonical NNS Registry subnet-lookup and native certified-asset read
  DTOs, including the optional Registry target and asset content type. Consumer
  adoption requires publication and owner qualification
  [#63](https://github.com/dragginzgame/ic-platform/issues/63),
  [#64](https://github.com/dragginzgame/ic-platform/issues/64).
- Prepare explicit browser SDK signing-request and expiry exports for Auth's
  guarded-identity tests. Existing SDK identities and signing behavior are
  preserved; adoption requires a separate compatible npm release
  ([#59](https://github.com/dragginzgame/ic-platform/issues/59)).

## [0.6.19] - 2026-10-26

### 📝 Summary

- Adopt Shared Tooling 0.26.20, including the compact baseline and corrected
  historical-draft release checks; use its SDK boundary guard in packed browser
  consumers while retaining Platform as the SDK dependency owner.
- Raise Wire runtime/metadata minimums to 0.7.16 and align the independent runtime
  fixture lock. Preserve existing payload bytes, admission policies and Rust 1.91.

- Prepare both locked audit inputs in an isolated Cargo cache and reject
  incomplete yanked-package observations even when cargo-audit exits zero.
  Retain raw audit findings and statuses separately from index coverage
  ([#56 — Prepare registry index coverage before yanked-package audit checks](https://github.com/dragginzgame/ic-platform/issues/56)).

## [0.6.18] - 2026-10-26

- Add optional upstream BLAKE3 `hash`, `Hasher` and `Hash` exports with `std` only,
  independent of IC codecs, identities and runtime features.
- Admit source-qualified package identities and effective BLAKE3 features in
  locked native/Wasm graphs; qualify standalone consumers without a direct
  BLAKE3 dependency. Keep existing hash formats and cryptographic code unchanged.
- Bound the feature-selection regression's exhaustive model to four synthetic
  features; retain the real singles/pairs/critical matrix without enumerating
  millions of combinations during routine validation.
- Include the BLAKE3 graph helpers in standalone consumer admission fixtures;
  verify duplicate identities and extra features fail before compilation.
- Supply the independent browser package as `@tokostudios/ic-platform` 0.1.0
  over SDK Core 6.1.1, with verified registry archive identity and standalone
  consumer installation. Blob/Canic adoption remains in their own issues
  ([#57 — Supply and publish the browser SDK Core boundary](https://github.com/dragginzgame/ic-platform/issues/57)).
- Keep compatibility decisions in existing migration/inventory documents and
  affected implementation issues; retire the central acknowledgement queue
  ([#58 — Coordinate the current IC integration contract](https://github.com/dragginzgame/ic-platform/issues/58)).

## [0.6.17] - 2026-10-26

- Adopt Shared Tooling 0.26.18, including fresh normal releases with explicit
  recovery, tool-admission diagnostics and common external IC dependency guards.
- Add the initial browser SDK Core boundary with canonical Principal admission,
  checked identity restoration, explicit Agent policy inputs and raw query errors.
- Qualify packed npm consumers, Caffeine/Auth type compatibility and actual
  browser/module-worker integration; document separate publication and adoption.
- Copy Node Buffer-backed trust/query views without aliasing, refuse invalid raw
  byte shapes, and qualify fresh source builds before npm publication (#57).
- Adapt host metadata fixtures to the selected Wire generator's explicit
  diagnostics/acknowledgements interface.
- Align the unpublished browser package with Blob 0.48's SDK Core 6.1.1 graph,
  preserve effective-target query diagnostics, and qualify Caffeine composition
  with compatible SDK-6 Auth types; document application override requirements.
- Reject duplicate, aliased and shadow SDK graphs; verify copied and packed MIT
  licenses, and cover browser/worker qualification on supported macOS runners.

## [0.6.16] - 2026-10-25

- Simplify consumer qualification with one profile table, shared registry identity
  admission, and checked-in runtime fixture manifests.
- Share the reviewed feature inventory/native-only policy and extra formatting
  source list; retain independent dependency-graph refusal tests.
- Prepare only common host/Rust tools in installed-runtime CI; standard CI keeps
  its pinned Node/npm/TypeScript Wire frontend proof.
- Separate entrypoint validation/planning from token emission while preserving
  codecs, diagnostics, guard order, export names and executor contexts.
- Preserve structured Principal deserializer errors without redundant wrapping;
  remove unreachable experimental checker branches without changing crypto code.

## [0.6.15] - 2026-10-24

- Adopt published Wire 0.7.2 for native Principal/Account metadata and runtime
  qualification; host schema producers must upgrade their Wire selection together.
- Use Deserialize-only fixture decoders, retain frame bytes and consumer-owned
  budgets, and document one-pass decoding/normalization and error migration.
- Qualify Principal/Account Wire trait identity and Blob Contracts all-feature
  adoption; add exact-registry metadata verification for publication follow-up (#55).

## [0.6.14] - 2026-10-24

- Keep standalone consumer library imports and service declarations warning-free;
  deny warnings in every generated consumer profile.
- Finish consumer qualification without rereading a concurrently edited script
  tail; preserve Cargo failures and regression-test live source edits.
- Add authenticated signing-certificate time evidence using unchanged signature
  verification helpers, with typed verification/time-format failures (#51).
- Preserve the original verifier and cryptographic bodies; qualify Auth policy
  compatibility and standalone native/MSRV/Wasm consumers.
- Prepare pinned Node/npm before installed-runtime CI tool setup (#52).

## [0.6.13] - 2026-10-24

- Gate Principal/Account generated Wire clients in CI with explicitly prepared,
  pinned Node/npm/TypeScript tooling on each supported native host.
- Qualify malformed Account request/response refusals in Rust and TypeScript.
- Reuse equivalent Wasm graphs for host-only metadata selections while guarding
  both dependency declarations and preserving Principal implications.
- Refresh published metadata adoption guidance and the current handoff.

## [0.6.12] - 2026-10-24

- Add native-only Wire metadata for owned Account under `wire-schema`, preserving
  existing bytes and absent versus explicit-zero subaccount representation.
- Qualify standalone Account Rust/TypeScript request and response frames.

## [0.6.11] - 2026-10-23

- Keep release-editor setup and offline checks working with inherited CDPATH
  and relative script invocation, including checkout paths containing spaces.

## [0.6.10] - 2026-10-23

- Adopt Shared Tooling 0.26.8 at `0b07c2a`, including Cargo setup lock-race
  handling and concurrent source-cache initialization.
- Select Wire 0.6.7 for Principal metadata and the independent runtime fixture;
  preserve existing protocol bytes, caller budgets and external Candid paths.

## [0.6.9] - 2026-10-23

- Align feature-selection regression expectations with the native-only
  `wire-schema` feature so release validation admits the updated matrix.
- Raise the minimum supported Rust version to 1.91 for all features.
- Add optional native-only `wire-schema` metadata for owned Principal, preserving
  byte-buffer encoding and the 29-byte limit in generated TypeScript (#48).
- Record the authorized owned-endpoint Wire migration and consumer-first
  retirement contract; retain Candid for required external IC interfaces.

## [0.6.8] - 2026-10-22

- Audit both maintained lockfiles against an isolated, recorded RustSec revision
  in separate CI; retain maintenance warnings without treating them as vulnerabilities.
- Extend bounded Candid malformed-input corpus and fuzz coverage; characterize
  the inherited short-DER panic without changing cryptographic implementations.

## [0.6.7] - 2026-10-21

- Record an individual export usage audit, separating observed sibling use,
  generated-code/type contracts and unproven consumer demand without removing APIs.
- Document IC dependency/API ownership and dated sibling usage; enforce feature
  and direct-dependency inventory consistency in CI without scanning siblings.
- Correct README default-feature and stable-structures coverage.

## [0.6.6] - 2026-10-21

- Adopt Shared Tooling 0.26.4, including numbered `Unreleased` heading support,
  release recovery safeguards and improved validation failure evidence.
- Run Clippy before runtime fixture tests in the release gate and before
  publication packaging; optional IC tool setup now requires explicit selection.

## [0.6.5] - 2026-10-21

- Expose exact upstream `MemoryId`, `MemoryManager` and `VirtualMemory` identities
  under the existing optional `stable-structures` feature for IC Persistence.
  Memory retains allocation, leases and checked reopening; no implementation,
  dependency selection or persisted format changes
  ([#45 — Expose exact stable-memory manager identities for IC Persistence](https://github.com/dragginzgame/ic-platform/issues/45)).

## [0.6.4] - 2026-10-21

- Use the declared development compiler in CI and installed-runtime qualification; retain independent minimum-Rust gates and explicit Wasm target preparation.

## [0.6.3] - 2026-10-21

### Fixed

- Make standalone consumer graph admission compatible with macOS Bash 3.2,
  preserving CDK/runtime isolation, explicit external Candid selection and
  failure propagation ([#43 — Make standalone consumer graph admission portable to native macOS Bash](https://github.com/dragginzgame/ic-platform/issues/43)).
  No library API, dependency or wire-format changes.

## [0.6.2] - 2026-10-21

### Fixed

- Use short feature-graph cache filenames with retained closure mappings, avoiding
  filesystem filename limits as the supported feature inventory grows.
- Qualify standard-library paths in `cdk::println!` so consumer-local `std` and
  `format!` names cannot capture its native or Wasm expansion. Printing behavior
  and both existing invocation forms are unchanged.

### Added

- Add optional `stable-structures` exports retaining upstream 0.7.2 collection,
  Memory, backing, iterator and codec identities for Storage/Testkit. Complete
  PR #32's codec-export feedback; no storage implementation or policy moves.
- Add caller-bounded Candid sequence deserialization and strict reserved-marker
  admission from PR #26. Callers retain element ceilings, diagnostics and decoder
  quotas; strict payload refusal requires a zero skipping quota. No wire changes.

### Improved

- Qualify explicit external Candid entrypoints and service registration together
  in a Platform-only consumer with fixed wire bytes, bounded empty-request
  decoding, native/MSRV checks and linked Wasm builds. Document the migration.
- Run full all-target/all-feature Clippy before test targets in the fail-fast
  release gate, catching lint failures before longer validation work.

## [0.6.1] - 2026-10-21

### Fixed

- Scope secp256k1 consumer-test imports to the test that uses them, keeping
  standalone library builds warning-free without changing coverage.
- Update the runtime qualification fixture's TLS lock to rustls-webpki 0.103.16
  and webpki-root-certs 1.0.10, addressing RUSTSEC-2026-0382 and
  RUSTSEC-2026-0383. Production library dependency selections are unchanged.

## [0.6.0] - 2026-10-21

### Breaking

- Gate Principal, Account, Subaccount and their errors/conversions behind
  `principal`. Runtime, Candid, signature creation and external client/DTO/parser
  features include it. Type-only consumers must select it explicitly. Empty and
  executor/IC0-only production graphs no longer select application types.

### Improved

- Keep unused upstream delegation/CBOR helpers in signature reference tests,
  preserving their bodies and vectors. Production signature creation no longer
  selects representation-independent hashing or CBOR decoding; verification
  retains its required CBOR parser. No cryptography or wire bytes change.

## [0.5.1] - 2026-10-20

### Documentation

- Record an isolated Agent request-copy benchmark and retry qualification.
  The candidate saves one payload-sized allocation; production Agent remains
  unchanged pending a supported distribution decision.

### Improved

- Read message bytes directly into vector spare capacity, avoiding zero
  initialization immediately before the IC overwrites the buffer. Installed
  fixture read costs fall about 30% at 1 KiB and 37% at 2 KiB; these are not
  whole-RPC speedups. Preserve exact bytes, caller admission and trap behavior.
- Remove the unused, unexported lazy root-key cache and Platform's direct
  `lazy_static` dependency. Root-key constants, parsing and cryptography remain
  unchanged; no consumer migration is needed.
- Stop enabling Syn's unused `extra-traits` feature in the entrypoint generator.
  Retain full syntax parsing and existing generated endpoint behavior.

## [0.5.0] - 2026-10-20

### Breaking

- Reject `composite` on non-query entrypoints and `manual_reply` on lifecycle
  hooks instead of silently ignoring those options.

- Reject the retired no-op `hidden` entrypoint option.
- Replace panic-prone `Call::change_timeout` with explicit
  `with_wait_policy(WaitPolicy)` selection of bounded or unbounded waiting.
  Preserve constructor defaults, cycle payments, raw payloads and retry policy.

- Require `candid-service` for `candid_method`, `export_service` and their printer.
  Plain `candid` retains external DTO derives, arbitrary-precision numbers and
  decoding, without selecting service-generation printer dependencies.

- Retire unused `cdk::storage::{stable_save, stable_restore}` and their private
  stable-memory writer/copy machinery. Raw stable-memory passthroughs remain.
  Existing persisted formats remain consumer-owned; no data is rewritten.
- Remove the unreachable `StableMemoryError::OutOfBounds` variant. Raw reads and
  writes still trap on bounds violations; failed growth returns `OutOfMemory`.

### Improved

- Support raw Rust handler identifiers such as `r#match` without a macro panic.
  Default endpoint names omit Rust's `r#` escape; explicit names stay verbatim.

- Encode native entrypoint export symbols without lossy punctuation replacement,
  allowing distinct names such as `native-name` and `native_name` to coexist in
  host builds. Wasm export names remain unchanged.

- Generate hygienic entrypoint locals and positional argument bindings, fixing
  valid handler/codec names and argument patterns that previously collided with
  generated variables. Qualify generated standard-library types so consumer
  aliases cannot change guard/reply typing.

- Parse entrypoint options directly with Syn, removing the generator's Darling
  dependency and redundant argument-type clones. Preserve ordered guards and
  explicit codecs; reject duplicate options and string-encoded boolean flags.

- Separate `runtime` from `cdk` entrypoint generation. Raw-call libraries can use
  the same `cdk::*` APIs and executor without selecting `ic-platform-macros`.

- Copy caller, canister and subnet Principal identities through fixed stack
  buffers, removing their temporary Vec allocation without changing valid bytes.

## [0.4.1] - 2026-10-20

### Fixed

- Format nonzero Account subaccounts through a fixed hex buffer, avoiding repeated
  integer formatting while preserving canonical text, checksums and external bytes.

### Qualification

- Add a separate installed-runtime workflow with Testkit-owned server provisioning,
  and exact-registry runtime qualification with source/version admission tests.
- Add generated parser properties and an opt-in coverage-guided Principal/Account
  fuzz harness; keep fuzz dependencies confined to disposable tooling fixtures.
- Retain reproducible benchmarks, Account timing and type-layout evidence, and
  update the release handoff and consumer qualification records.

## [0.4.0] - 2026-10-20

### Breaking

- Own the runtime RejectCode identity while preserving numeric codes, Serde names,
  Display spelling and unknown-code refusal. Retire its Strum iteration and
  external ErrorCode conversion; Agent/PocketIC DTO identities remain unchanged.
- Package the original signature creation/verifier implementations inside Platform.
  CanisterSigPublicKey now has a Platform identity. Preserve cryptographic bodies
  and vectors; remove the unused store, Candid import and registry package edges.
  Signature-only production graphs no longer require Candid or raw IC calls.
- Make raw calls transport-only: retire Candid argument/response convenience
  methods and their combined Error/CandidDecodeFailed model. Encode/decode required
  external messages through the explicit Candid boundary, retaining typed errors
  and caller-selected policies. Expose the existing oneway/unknown-code errors.

## [0.3.0] - 2026-10-20

### Changed

- Select Rust 1.99.1 for development and matching CI checks, retaining existing
  package MSRVs.

### Added

- Own Candid-free Account and Subaccount identities with bounded text parsing,
  original effective-zero equality and explicit external ledger DTO conversions.
  Preserve optional subaccount representation and ledger text checksums.

### Breaking

- Own the unconditional Principal and parse-error types. Convert explicitly to
  the retained external Principal for Agent, PocketIC, Candid DTOs and crypto APIs.
  Preserve original identity bytes and ordering; delegate self-authentication to
  the original cryptographic implementation.
- Bound principal text admission and remove intermediate formatting allocations.
  Treat owned/borrowed binary Serde bytes identically; do not inherit Candid marker
  handling or generic range stepping. See the [Principal migration](docs/migration.md#owned-principal-migration-03).

## [0.2.2] - 2026-10-19

### Added

- Complete PocketIC placement, pending-message, gateway, HTTP fixture and chunked
  install-mode exports used through Testkit by Backup, Canic and Blob. Preserve
  original upstream identities and native-only selection
  ([#27 — Complete PocketIC exports used by Testkit consumers in Backup and Canic](https://github.com/dragginzgame/ic-platform/issues/27)).

## [0.2.1] - 2026-10-19

### Changed

- Specify Wire for all owned application endpoints, including public endpoints;
  reserve Candid for required IC interfaces and legacy checkpoint compatibility.
- Document runtime maintenance, reviewed source-fingerprint changes and clean-cache
  recovery acceptance without claiming dependency independence or a completed drill.
- Correct stale 0.1.18 candidate guidance for the released 0.2.0 runtime boundary.

## [0.2.0] - 2026-10-19

### Breaking

- Replace the full CDK dependency with maintained non-cryptographic adapters
  inside Platform. Keep the registry executor =2.0.0 and ic0 =1.2.0 identities.
  Call/response, cost/error and stable-error types now have Platform identities.
- Make `cdk` Candid-free. Require explicit Wire endpoint codecs, including unit
  RPC decoding; remove default Candid endpoints and service export generation.
  Raw calls start with empty bytes; supply Wire frames or explicit external
  Candid arguments. Select `candid` separately for management/system calls and
  existing persisted checkpoint helpers. No stored bytes or crypto are changed.
- Preserve Apache-2.0 provenance for imported runtime code; the combined consumer
  package declares MIT AND Apache-2.0. No executor implementation is copied.

### Testing

- Qualify standalone native/MSRV/Wasm consumers, real Timers probe compilation,
  graph isolation and imported callback/spawn source fidelity. Installed Wire
  fixtures cover raw awaits, submission/cycle failures, protected/weak/migratory
  spawning, traps, timer cancellation, upgrade and external Candid/checkpoints.

### Changed

- Reject unintended RNG and entropy dependencies in portable crypto, runtime and
  Wasm feature graphs. Cover the combined BLS, ECDSA and Ed25519 selection.
- Add source-preservation checks for dependency scaffolding proposals, retaining
  upstream cryptographic bodies and licenses. Qualify optional signature-store
  and secp256k1 scaffolding without selecting production replacements.

## [0.1.18] - 2026-10-19

### Changed

- Refresh the runtime qualification fixture to Testkit 0.54.1 and Timers 0.29.1,
  with compatible Host, Metrics and build-tool lock updates. Qualify combined
  callback, cancellation and upgrade behavior on installed Linux PocketIC.
- Refresh crypto-common to 0.1.7 with its upstream generic-array =0.14.7
  compatibility pin; verify focused Candid and crypto tests on Rust 1.88.
- Adopt Shared Tooling 0.25.1 and the pinned IC Build release editor. Standard
  release commands prepare the editor; standalone checks remain offline.

### Fixed

- Use the supported numbered, undated pending heading so release preflight can
  admit the 0.1.18 notes.

## [0.1.17] - 2026-10-19

### Added

- Add optional native/Wasm `ecdsa` exports for K256 signatures, keys and prehash
  traits, preserving upstream identities without RNG or runtime integration.
  Qualify strict consumer rejection and explicit normalization with standalone
  tests and a source-copy Auth migration.

## [0.1.16] - 2026-10-19

### Added

- Add optional native/Wasm `ed25519` exports with original Dalek identities,
  no default or RNG features, and standalone RFC 8032 verification coverage.
  Consumers retain strict verification, key custody and admission policy.

### Changed

- Replace routine exhaustive feature enumeration with defaults, singles, all
  pairs, all features and critical integration groups. Retain the full power set
  under `make check-features-exhaustive` as a separate manual audit; routine
  validation no longer claims every higher-order combination.
- Keep feature queries and resolved standalone consumer builds offline
  with frozen lockfiles, avoiding repeated network work during release validation.
  Report graph query counts and progress; both modes use the same admission rules.
  Cargo cache locks held by other processes can still block offline commands.

## [0.1.15] - 2026-10-18

### Added

- Add an optional native-only `pocket-ic` maintenance boundary with explicit
  client, error and fixture DTO exports. Testkit retains server lifecycle and
  compatibility policy. Keep PocketIC out of default, executor and Wasm graphs.

## [0.1.14] - 2026-10-18

### Added

- Prepare caller-bounded Candid sequences and strict reserved-marker admission
  under the existing `candid` feature, preserving external wire bytes and
  caller-selected quotas. Includes standalone Platform-only consumer coverage;
  [PR #26](https://github.com/dragginzgame/ic-platform/pull/26) addresses
  [#24](https://github.com/dragginzgame/ic-platform/issues/24) and awaits integration.

### Testing

- Reduce feature graph queries with guarded native-only leaf equivalence on
  Wasm from 9,218 to 5,186, retaining all 16,386 selection checks. Validate cached
  graphs in one process per target and remove redundant graph copies
  ([#23](https://github.com/dragginzgame/ic-platform/issues/23),
  [PR #25](https://github.com/dragginzgame/ic-platform/pull/25)). Query reduction
  does not establish a measured wall-clock speedup.

## [0.1.13] - 2026-10-18

### Added

- Add optional native `secp256k1` public-key derivation exports pinned to
  `ic-secp256k1 =0.3.0`, preserving upstream identities and excluding its RNG
  dependencies from Wasm. Cover standalone MSRV usage and original-owner bytes.

### Testing

- Add an exact-version registry-only standalone consumer mode, rejecting local
  facade/generator overrides, to qualify published adoption separately from source.

### Fixed

- Enforce host parser, Agent and Testkit exclusions in the combined CDK/Timers
  canister graph, alongside qualified runtime identities. Add twelve rejection
  cases and a graph-only qualification mode without compiler or server startup.

## [0.1.12] - 2026-10-18

### Added

- Complete explicit CDK checkpoint and cycle-burn exports for
  [Timers #19](https://github.com/dragginzgame/ic-platform/issues/19), stable-memory
  passthroughs for [Wire #20](https://github.com/dragginzgame/ic-platform/issues/20),
  certificate publication for [Auth #21](https://github.com/dragginzgame/ic-platform/issues/21),
  and typed call errors, costs, counters and system helpers for
  [Canic #22](https://github.com/dragginzgame/ic-platform/issues/22).

### Fixed

- Coordinate runtime qualification lock identities with release preparation,
  without resolving or changing external dependencies; reject stale locks.

### Testing

- Extend standalone MSRV/Wasm and installed PocketIC checks for upstream type
  identity, checkpoint bytes, stable rollback, certificates and system contexts.

## [0.1.11] - 2026-10-18

### Added

- Expose the original CDK cycle availability, acceptance and refund helpers plus
  instruction/context counters, completing Blob's remaining helper imports
  ([#1 — Runtime boundary](https://github.com/dragginzgame/ic-platform/issues/1#issuecomment-6250276514)).

### Testing

- Add an explicit PocketIC regression target for CDK callbacks, trap cleanup,
  timer cancellation and upgrade reinitialization. The Platform-only consumer
  gate now links Wasm on both the development compiler and Rust 1.88.
- Reuse equivalent feature graphs within each validation run, reducing Cargo
  graph queries while retaining every feature-selection admission check.
- Freeze runtime qualification dependencies behind an explicit lock refresh,
  isolate concurrent build artifacts, and require the exact guard rejection.
  Add installed cycle-refund, callback rollback and counter checks.

## [0.1.10] - 2026-10-18

### Added

- Add optional Platform-owned CDK entrypoint/service macros and explicit upstream
  runtime calls, helpers and spawning. Consumers can generate Platform Candid paths
  without upstream changes; adopt the paired generator/facade release and update
  consumer generators before removing direct declarations
  ([#1 — Support Query’s CDK call and entrypoint boundary](https://github.com/dragginzgame/ic-platform/issues/1)).
- Expose canister-signature public-key formats, root-key constants and hashing
  through an independent optional feature, preserving upstream DER/domain bytes
  ([#16 — Expose canister-signature key and hashing APIs](https://github.com/dragginzgame/ic-platform/issues/16)).
- Complete the certification surface with upstream mutable tree and hash/trait
  identities needed by Auth's signature store, without adding store policy
  ([#17 — Complete certification exports for Auth signature-store trees](https://github.com/dragginzgame/ic-platform/issues/17)).
- Add independent canister-signature verification and BLS features, preserving
  upstream results, typed key/signature errors and allocation support. BLS alone
  stays Candid-free; the complete canister verifier still brings transitive Candid
  ([#18 — Expose IC signature verification and isolated BLS capabilities](https://github.com/dragginzgame/ic-platform/issues/18)).

### Changed

- Update migration guidance for published Testkit 0.54's canonical decoder
  identities; older Testkit selections retain their narrow native exception
  until their driver upgrade is qualified
  ([#2 — Coordinate published Testkit limits identity](https://github.com/dragginzgame/ic-platform/issues/2)).

## [0.1.9] - 2026-10-18

### Added

- Export upstream `DeleteCanisterSnapshotArgs` through the existing management
  feature, preserving Canic's current snapshot-delete request bytes
  ([#13 — Complete snapshot/status management DTOs](https://github.com/dragginzgame/ic-platform/issues/13)).
- Expose upstream `contract::{DecoderConfig, decode_args_with_config}` for
  generated Candid contracts, retaining existing configuration and diagnostics.
  Include a qualified upstream CDK Candid-path patch proposal; published CDK
  macro support remains a separate adoption prerequisite
  ([#1 — Support Query’s CDK call and entrypoint boundary](https://github.com/dragginzgame/ic-platform/issues/1)).
- Expose the existing raw replicated-execution flag, controller check, reply
  append and reject operations through the independent `ic0` boundary. Preserve
  upstream integer/signature identities and caller-owned byte/authentication policy
  ([#1 — Support Query’s CDK call and entrypoint boundary](https://github.com/dragginzgame/ic-platform/issues/1)).

### Fixed

- Check supported Wasm consumers and the all-feature library at Rust 1.88 in
  maintained CI/release validation, with explicit MSRV target preparation
  ([#15 — Qualify the Rust 1.88 floor on Wasm](https://github.com/dragginzgame/ic-platform/issues/15)).
- Reject unreviewed features, unsupported CDK frontend packages and additional
  executor/system versions or sources in feature-graph validation, including
  graphs that select the executor
  ([#14 — Close feature-graph admission gaps](https://github.com/dragginzgame/ic-platform/issues/14)).

## [0.1.8] - 2026-10-18

### Added

- Expose Backup's existing management snapshot and status DTOs through the
  optional management boundary, preserving upstream 0.11.0 identities and Candid
  bytes. Snapshot execution, limits, restore safety and recovery remain consumer-owned
  ([#13 — Expose Backup snapshot and status management DTOs](https://github.com/dragginzgame/ic-platform/issues/13)).

## [0.1.7] - 2026-10-18

### Added

- Complete the explicit Candid boundary with structural subtyping, upstream
  default decoders, and native dynamic values/type-value serializers for refusal
  fixtures. Existing limits, diagnostics and wire bytes remain caller-owned
  ([#5 — Support Canic Candid service generation and decoder contracts](https://github.com/dragginzgame/ic-platform/issues/5)).
- Expose native Agent anonymous/delegated identities and signed-update response
  enums without introducing polling, retry or trust policy
  ([#9 — Complete the Agent boundary for consumer adoption](https://github.com/dragginzgame/ic-platform/issues/9)).

### Changed

- Organize the dependency catalog into production and tests/tooling sections,
  preserving them consistently through formatting, hooks and release checks.
- Clarify the existing raw CDK macro subset that avoids generated Candid paths;
  typed entrypoints and the full runtime boundary remain unsupported
  ([#1 — Support Query’s CDK call and entrypoint boundary](https://github.com/dragginzgame/ic-platform/issues/1)).

## [0.1.6] - 2026-10-18

### Added

- Add optional ICRC ledger types used by Query and Canic, preserving upstream
  account, metadata, allowance and block/archive/certificate identities and wire
  formats without enabling storage, executor or host transport features.
- Expose the existing raw IC debug-print operation for Testkit benchmark markers,
  preserving caller-formatted bytes without adding a logger or runtime dependency.
  [#11 — Expose debug_print for Testkit benchmark marker emission](https://github.com/dragginzgame/ic-platform/issues/11).
- Expose management-canister history DTOs used by Blob continuity recovery,
  preserving upstream identities and Candid formats while consumers retain
  bounded calls, history admission and recovery policy.
  [#12 — Expose management canister history DTOs for Blob continuity recovery](https://github.com/dragginzgame/ic-platform/issues/12).

## [0.1.5] - 2026-10-18

### Added

- Add optional management-canister HTTP outcall DTOs from exactly 0.11.0,
  preserving upstream types and external Candid formats without selecting an
  executor or host transport. Callers retain bounds, pricing and retry policy.
- Add optional native `agent` support for explicit upstream Agent builders,
  PEM identities, transport errors, signed updates and request-status APIs.
  Keep transport out of Wasm and runtime-only builds; consumers retain endpoint,
  trust-root, response-bound, retry and reconciliation policies.
  [#9 — Publish Agent boundary needed to finish Query dependency adoption](https://github.com/dragginzgame/ic-platform/issues/9).
- Expose subtree lookup results and labelled hashing through certification for
  Agent consumers without introducing another hash-tree owner.
  [#4 — Complete certification exports for Query subtree lookup and witness fixtures](https://github.com/dragginzgame/ic-platform/issues/4).
- Expose upstream staged decoding/configuration and Candid service generation,
  reflection and encoding macros without changing existing decoder policies.
  [#3 — Preserve staged Candid decoder failures](https://github.com/dragginzgame/ic-platform/issues/3),
  [#5 — Support Canic Candid service generation, reflection and existing decoder contracts](https://github.com/dragginzgame/ic-platform/issues/5).
- Add an independent `ic0` feature for Timers' qualified raw system calls,
  retaining upstream callback signatures and caller-owned timer/recovery policy.
  [#8 — Expose Timers’ qualified raw IC system-call boundary](https://github.com/dragginzgame/ic-platform/issues/8).

### Fixed

- Constrain the shared executor to qualified 2.0.0 and raw system ABI to 1.2.0
  in downstream dependency resolution.
  [#7 — Preserve Timers’ exact qualified executor selection](https://github.com/dragginzgame/ic-platform/issues/7).
- Filter standalone-consumer metadata to its native target so fresh CI caches
  do not require unrelated foreign-target sources while offline.
  [#10 — Scope standalone consumer metadata to the native target in fresh CI caches](https://github.com/dragginzgame/ic-platform/issues/10).

## [0.1.4] - 2026-10-18

### Added

- Add optional native `candid-parser` support for explicit upstream parsing,
  checking, service equality, textual arguments and browser binding generation.
  Keep parser dependencies out of Wasm and runtime-only builds; callers retain
  admission limits, import resolution and diagnostic policy.

### Fixed

- Enforce empty default features and check the actual default dependency graph.

## [0.1.3] - 2026-10-18

### Added

- Export `ic_platform::Principal` unconditionally, preserving the Candid
  contract's upstream type identity. Identity-only consumers need no feature or
  direct `ic_principal` dependency; default builds remain Candid/CDK-free.
- Add an independent optional `certification` surface for upstream certificate
  and hash-tree types and operations used by Host, Query and Canic. Preserve
  upstream type identity and Serde support; bounded decoding and verification
  policy remain consumer-owned. Qualify a standalone certification consumer.

## [0.1.2] - 2026-10-18

### Added

- Add `make publish` for explicit crates.io publication of the current committed
  package version, and `make publish-check` for an all-feature Cargo dry run.
  Retain command output and source identity for each attempt.

### Changed

- Adopt Shared Tooling 0.24.0 with production dependency-role labels, matching
  formatter/release ordering checks and the shared formatting hook payload.
- Check host-parser and Testkit exclusions transitively across every supported
  feature selection on native and Wasm targets. Deployment artifact admission
  remains the responsibility of consuming applications.

## [0.1.1] - 2026-10-18

### Added

- Support Testkit/Query generic argument helpers and external arbitrary-precision
  and reserved values through the explicit Candid boundary.
- Extend standalone consumer qualification to Rust 1.88, recursive DTOs,
  independent wire vectors and typed decoder causes, with Wasm compilation.

### Changed

- Document consumer migration and decoder retirement requirements, including
  the remaining CDK runtime and host-parser adoption boundaries.
- Check native/Wasm executor dependency graphs and parser isolation while
  preserving empty default features and Candid-free executor-only builds.

## [0.1.0]

### Added

- Establish the controlled IC platform boundary with no default capabilities.
- Adopt Testkit's caller-bounded Candid decoding, preserving exact-result and
  argument-list policies, compact diagnostics and typed causes.
- Expose DTO derives and encoding through an explicit external-contract surface.
- Expose the shared CDK executor independently of Candid. No full CDK runtime,
  host parser or consumer migration is included in this initial local candidate.
