# Current handoff

The maintainer authorized replacing the full CDK with maintained
non-cryptographic adapters inside Platform. The candidate extends
[PR #35](https://github.com/dragginzgame/ic-platform/pull/35), starting at
`40f1d8a`, in isolated `feat/owned-candid-free-runtime`. Normal dirty dependency
scaffolding work and sibling checkouts remain untouched. Package versions stay
0.1.18 for qualification only. Breaking notes target **0.2.0**, requiring coordinated
Platform/generator publication; do not publish this candidate as 0.1.19.
No release, tag, publication or default-branch integration is performed here.

## Runtime and protocol contract

`cdk` selects owned explicit-codec entrypoints, system adapters, raw calls and
protected spawning. Its normal/build graph contains neither Candid nor full
`ic-cdk`/`ic-cdk-macros`. Every controlled application endpoint uses Wire;
Platform has no Wire dependency and installs no default codec or resource policy.
Query/update macros require explicit decoding even for unit requests, and explicit
encoding unless manually replying. Argument-bearing lifecycle hooks also require
a decoder; no-argument IC lifecycle hooks retain raw no-input behavior.
Implicit Candid endpoint/service generation is retired.

The sole executor remains registry `ic-cdk-executor =2.0.0`; raw system calls
retain `ic0 =1.2.0`. Imported call submission, callback/trap cleanup, protected
spawn semantics and the upstream multithreading refusal are preserved. No executor
implementation is copied. Call/response/cost/performance/error/stable-error types
now have Platform identities; Principal, RejectCode, external DTOs, certification
and cryptographic types retain their registry identities. See
[the migration contract](../migration.md#maintained-runtime-and-wire-entrypoint-adoption).

Raw call constructors begin with empty transport bytes. Every Wire call supplies
its request frame; required external no-argument Candid calls explicitly use
`with_args(&())`. Bounded calls still default to 300 seconds. Callers retain
cycles, deadlines, retries, budgets, authentication and application interpretation.
Select `candid` separately for management/system and standard ledger/NNS/SNS calls.
Existing Candid checkpoints remain an explicit legacy persisted-data exception,
with original bytes and restore behavior. No storage implementation, allocation,
retry policy, cryptographic primitive or trust root changes.

Imported maintained adapters retain the original Apache-2.0 license and provenance
under `crates/ic-platform/src/cdk/upstream/`; the combined library declares
MIT AND Apache-2.0. Reviewed unsafe callback/legacy byte-copy modules are narrowly
allowed under the workspace deny lint. The superseded CDK macro-gating patch and
obsolete Candid service fixture are retired. Pending crypto-scaffolding patches in
the normal checkout are not included or overwritten.

## Qualification evidence

Focused Linux checks pass at Rust 1.99 and MSRV 1.88: generator diagnostics/context
coverage, raw constructor and explicit external-unit tests, bounded Candid decoder
regressions, and the separate Platform-only native/Wasm consumer. This standalone
consumer declares no direct Candid/CDK dependencies. Evidence is in
`/tmp/platform-owned-runtime.kzhHt0/platform/target/boundary-consumer.P8ZOXK`.
Strict selected Clippy, warning-denied raw/external Rustdoc, formatting,
dependency pins/ownership, documentation links and runtime-lock checks pass.
Source fingerprints match the original CDK 0.20.3 callback/submission and protected
spawn regions; mutated cleanup predicates/diagnostics and licenses are refused.
Feature selection/closure/admission regressions and 34 focused actual native/Wasm
production graphs pass. The routine full matrix and local `make ci` are release-owned
and were not run for this task.

The actual committed Timers 0.29.2 source at
`41ebd6c3d29681e61be5b9b223333a6499930f87` was archived into
`/tmp/platform-runtime-timers-proof`. Both real runtime and size probes compile
for Wasm at Rust 1.91 with only a disposable Platform override and no source edits.
Their native/Wasm normal/build graphs are Candid-free and resolve the sole
qualified executor/ic0 identities. This capture selects Wire 0.4.6; the maintained
installed fixture selects Wire 0.4.7. Neither is a sibling migration or publication.

Installed Linux PocketIC **16.0.0** qualification passes for all four selections:
Wasm from Rust **1.91 and 1.99**, each pure Wire and explicit external Candid.
All application RPCs and installation/upgrade arguments use Wire in both modes.
The latter additionally calls management `raw_rand` with required Candid bytes
and checks original legacy checkpoint bytes across upgrade. Tests cover awaited
and composite calls, typed rejection/submission-pressure/insufficient-cycle
failures, guards, malformed/unit frames, all four Wire allowances, response-encode
and callback traps, protected/weak/migratory tasks, Timers await/trap/cancellation/
rearm, upgrade, cycles/refunds, stable bounds/rollback and certificate lookup.

Sources, frozen locks, metadata and four distinct Wasm artifacts are retained at
`/tmp/platform-owned-runtime.kzhHt0/platform/target/cdk-runtime.eNOQ6w`;
logs are `/tmp/platform-installed-{1.91.0,1.99.0}-{wire,external-candid}.log`.
Testkit owns the admitted server lifecycle and transport. These are installed
local simulator results, **not native macOS or live IC qualification**.
No measured Wasm-size reduction is claimed.

## Contributor tooling and adoption

Shared Tooling remains the reviewed **0.25.1** snapshot at
`d57c631206e3618755e45bebf2c88c673564ddc1`; no snapshot payload changes here.
Use numbered undated pending changelog headings. The pinned IC Build 0.18.0
release editor and two dependency-role catalog sections remain in place.

Frozen CI consumers need one unfiltered `cargo fetch --locked` during explicit
setup. Native plus Wasm target filters still missed `wasm-bindgen-futures 0.4.79`
for the native-only PocketIC client. A disposable public cache reproduces the
omission and successful unfiltered preparation at
`/tmp/platform-all-target-cache.vyrGZl`. The focused repair is in
[PR #34](https://github.com/dragginzgame/ic-platform/pull/34), linked to
[#33](https://github.com/dragginzgame/ic-platform/issues/33); PR #35 includes the
same prerequisite. Consumer validation remains frozen, with no per-consumer
fetch loops or lock-resolution changes. New three-host CI must qualify each head.

Consumer adoption requires the supplying next-minor registry release, actual
consumer generator/manifest/import migration, and its own wire/retry/storage
qualification. No compatibility re-export package or permanent duplicate owner is
introduced. Existing Testkit external Candid decoder/DTO responsibility remains
at Platform; Testkit owns transport/server policy. Parser, Agent and PocketIC stay
native opt-ins and are excluded from canister deployment graphs. Product artifact
admission remains consumer-owned; dependency-role labels do not enforce deployment
exclusions by themselves. Native macOS and live IC acceptance remain separate.
