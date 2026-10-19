# Current handoff

Issue [#27](https://github.com/dragginzgame/ic-platform/issues/27)'s existing
[PR #28](https://github.com/dragginzgame/ic-platform/pull/28) implementation is now
applied locally on released 0.2.1 (`de1c469`), with pending 0.2.2 notes. It adds
eight original PocketIC root/REST identities, including effective-principal
routing, pending ingress and the management 0.8 chunked-install mode. It changes
no client behavior, server ownership or retries. The pre-existing Tokio lockfile
update is preserved unchanged and is not part of this fix.

Focused Linux validation passes: upstream identity coverage on Rust 1.88, all
three Platform-only consumer tests and Wasm checks on development Rust and Rust
1.88, eight native/Wasm dependency graphs, admission refusal tests, formatting,
pins, boundaries and documentation links. Evidence is retained at
`target/boundary-consumer.6tXBv4` and `target/issue27-graphs`.
This is local source qualification, not a supplying publication, native macOS or
installed consumer qualification. Keep #27 open through delivery; Testkit must
require the supplying release before Backup/Canic/Blob can adopt its forwarded
surface through ordinary registry resolution. Existing PR #28 remains the delivery
vehicle; no replacement PR, release or sibling edit was made in this pass.


The checkout at `c7da516` declares **0.2.0** for Platform and its generator and
contains the maintained non-cryptographic runtime introduced through
[PR #35](https://github.com/dragginzgame/ic-platform/pull/35). The earlier 0.1.18
candidate identity below describes historical qualification, not the current
package version. This documentation pass does not establish registry publication
or current GitHub CI status. Sibling checkouts remain read-only.

The protocol contract explicitly includes public, third-party callers: all owned
application endpoints use Wire. Candid is only an adapter for required IC service
interfaces, plus legacy persisted checkpoints. The [maintenance and recovery
contract](../migration.md#runtime-maintenance-and-recovery) defines remaining
upstream responsibilities, intentional runtime-change admission and recovery
acceptance. A clean-cache recovery drill and actual Canic consumer qualification
are not established by the existing fixture evidence. No new dependency fork,
cryptographic change, release or publication is part of this pass.

This contract pass passes formatting, documentation links, runtime source and
mutation checks, lock/graph regressions and all five generator tests on Rust 1.88.
It changes no runtime behavior and does not rerun installed or macOS qualification.

The prior runtime integration passed Rust 1.88 generator/runtime tests and Candid-free
Wasm compilation, 24 actual native/Wasm production graphs, 66 admission cases,
feature selection/closure regressions, formatting, runtime provenance/lock checks,
dependency pins/ownership and documentation links. The three remaining scaffolding
source/license checks pass at `target/upstream-scaffolding.dayzwB`. The original
pending work is backed up at `/tmp/platform-runtime-local-integration.msFBme`.
That integration reconciled pending tooling/docs and removed its superseded CDK
proposal branch. Temporary paths below are historical evidence locations, not
durable recovery artifacts.

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
obsolete Candid service fixture are retired. Pending crypto-scaffolding patches
remain local uncommitted work, separate from PR #35.

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
fetch loops or lock-resolution changes. Those PR-head observations are historical; this pass does not recheck GitHub
status. Native results must be tied to the actual supplying revision.

Consumer adoption requires the supplying next-minor registry release, actual
consumer generator/manifest/import migration, and its own wire/retry/storage
qualification. No compatibility re-export package or permanent duplicate owner is
introduced. Existing Testkit external Candid decoder/DTO responsibility remains
at Platform; Testkit owns transport/server policy. Parser, Agent and PocketIC stay
native opt-ins and are excluded from canister deployment graphs. Product artifact
admission remains consumer-owned; dependency-role labels do not enforce deployment
exclusions by themselves. Native macOS and live IC acceptance remain separate.

## Preserved pending scaffolding work

The prior scaffolding qualification started at `392032a` (released 0.1.18).
That pass left production dependencies, versions and locks unchanged. Its pending
notes now join 0.2.0. Production adoption awaits the pending
source-distribution decision: maintained source variants and explicit identity
migration, or continued registry identities with the unsupported trims deferred.
No source-admission policy was weakened and no crypto replacement was selected.

New packaging patches retain the signature store under a default-enabled feature,
disable it only on the verifier edge, and gate secp256k1 scaffolding behind a
default-enabled `full` feature. The CDK macro patch is now superseded by the
maintained runtime.
`make check-upstream-scaffolding` now checks the three remaining crypto-scaffolding
proposals. Its earlier run verified source/license preservation and prepared
separate vector builds at `target/upstream-scaffolding.wtHSSX`; a
deliberate hash-body mutation is refused (`/tmp/platform-scaffolding-refusal.log`).
Source comparison preserves every original secp source line after removing only
feature attributes; no cryptographic function body, constant or vector changed.

Disposable qualification at `/tmp/platform-scaffolding.mpkcl3` passes Rust 1.88
creation tests with the store disabled (17), verifier tests (12), and default
creation tests with its store retained (24). Separate original/reduced/full secp
builds match 64 derivations plus four malformed-key outputs; the reduced native
and Wasm graphs compile without rand, rand_chacha, getrandom, PEM or simple_asn1.
Inherited upstream deprecation/unused-import warnings remain in this proposal;
strict-lint qualification is not claimed. Standalone Platform creation, verifier
and CDK consumers pass native tests and Wasm checks/builds on Rust 1.99 and 1.88.

The superseded optional-macros CDK variant passed installed Linux PocketIC 16.0.0
tests with Rust 1.99/1.91 Wasm producers, including callbacks/traps, timer cancellation,
upgrade and rollback. Evidence is retained under the disposable Platform copy at
`platform/target/cdk-runtime.ImAZc8`; its refreshed test lock selects Testkit
0.54.1 and Timers 0.29.2. Executor =2.0.0 and ic0 =1.2.0 remain single registry
identities. This is source-copy qualification, not a supplying release, macOS
qualification or live-network qualification.

Thirteen frozen normal/build consumer graphs were inspected read-only: Auth default,
portable verification, chain-key verification and signature preparation; Timers;
Host certificate and interface/certificate; Query certificate-host; Testkit native
and Wasm; Canic Core native/Wasm and Host native. No further justified manifest-only
cut was found; signature preparation retains Candid through upstream packaging.
Canic Host inherits CDK through shared Core: removing that edge needs a Core
feature/API split rather than an import edit. Canic Core and Testkit Wasm graphs
exclude host/parser/testing clients (excluding the selected root itself).
Evidence: `/tmp/platform-consumer-scaffolding-graphs`.
The preserved local graph changes reject unintended RNG/entropy implementations
while permitting rand_core traits and selected native host requirements. Prior routine graph qualification
passes 145 selections per target with 222 queries at
`target/feature-graphs/attempt.pJssgw`, including the portable crypto triple.
Source checks, synthetic refusals, pins/boundaries, formatting and doc links pass.
Those proofs did not migrate siblings or publish source variants.

Maintainer constraint for dependency trimming: retain DFINITY-authored
cryptographic implementations and upstream identities. Do not replace public-key
derivation, signature verification, key encoding/hashing or the BLS backend with
Platform-owned implementations. Earlier replacement proposals are withdrawn;
existing crypto prototypes remain historical evidence, not adoption candidates.
Further trimming targets unused modules, duplicate generators and unnecessary
feature/dependency activation while preserving the selected cryptographic code.
