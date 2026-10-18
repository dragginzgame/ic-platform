# Changelog

## [0.1.13]

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
