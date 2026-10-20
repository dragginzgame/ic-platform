# Current handoff

## Pending 0.4.2 substrate boundary

PR #32 is refreshed onto published 0.4.1 without rewriting its history. The
optional stable-structures boundary retains upstream 0.7.2 substrate identities;
its explicit VectorMemory export supplies Testkit PR #97's target-neutral backing.
Storage and Memory retain governance. Publication and ordinary Testkit adoption
remain pending. Linux Rust 1.88 native tests and Wasm checking, current strict
selected Clippy, 76 graph-admission fixtures, and six default/executor/substrate
native/Wasm graphs pass. Identity coverage includes the original VectorMemory;
the Platform-only consumer checks its shared clone bytes and growth.

A disposable Testkit PR #97 consumer replaces its direct substrate edge and
imports with this Platform feature. Its six instrumentation tests, strict Clippy,
Rust 1.91 native/Wasm and direct-boundary admission pass. Its default graph remains
dependency-free; the selected memory graph has one upstream 0.7.2 substrate and no
Candid/PocketIC/CDK/Agent. This is unpublished-path qualification, not ordinary
registry adoption. Native macOS and fresh required hosted CI remain separate.

## Pending 0.4.1 Account formatting

Account Display now encodes subaccount hexadecimal text into one fixed 64-byte
buffer rather than repeatedly invoking integer formatting. No public identity,
checksum, cryptographic body, persisted bytes or feature selection changes.
The [follow-up measurements](../benchmarks/0.4.1/README.md) put nonzero-subaccount
formatting at 234.7/289.6 ns for 10/29-byte owners, versus 377.7/491.5 ns for the
external implementation in the same run. Prior owned measurements were
540.7/613.0 ns in a separate run; these are native timings, not Wasm/cycle claims.

Rust 1.88 passes 14 library, three ledger differential and two generated parser
tests plus ledger-feature Wasm checking. Strict selected Clippy and formatting
pass. Evidence: `/tmp/platform041-{tests,wasm,clippy}.log`,
`target/type-benchmark.6cMHAN`, and the retained benchmark trials. The 0.4.1
changelog includes the preceding qualification work; package versions remain
0.4.0 for the maintainer's release tooling.

GitHub [CI run 39392859266](https://github.com/dragginzgame/ic-platform/actions/runs/39392859266)
passed Linux, macOS Intel and macOS ARM for released `26a07c5`. It does not qualify
the pending changes. The new installed-runtime workflow is still an untracked
local file and cannot be dispatched on GitHub before the maintainer pushes it.
Fresh-runner Testkit provisioning and native macOS qualification of this patch
therefore remain outstanding. No commit, push or publication was performed.

## 0.4 post-release qualification

HEAD is the maintainer's 0.4.0 release (`26a07c5`). The follow-up adds a separate
Linux installed-runtime workflow using Testkit-owned setup/check, and an exact
registry mode for the installed fixture. Published mode rejects local facade or
generator replacements, retains its disposable resolved lock, and applies the
existing single-executor/ic0 graph admission. Local `--prepare` is the explicit
cache setup step; ordinary qualification does not provision a server.

Published 0.4.0 passes all four installed PocketIC 16.0.0 combinations (Rust 1.91
and 1.99.1; Wire-only and external Candid), including traps, timers, upgrades,
cycles, stable rollback and certificates. Evidence: `target/cdk-runtime.UaFe6A`
and `/tmp/platform-registry-installed-unrestricted.log`. The first sandboxed run
could not bind loopback; the admitted rerun passed. This is Linux local-simulator
evidence, not a completed GitHub workflow run, native macOS or live IC acceptance.

The [benchmark evidence](../benchmarks/0.4/README.md) now retains previous trials,
reproduction tooling and new Account/type-layout measurements. Account parsing
improves, but that initial run found a nonzero-subaccount formatting regression;
the pending fix above addresses it without cryptographic changes. Generated Unicode/boundary/round-trip parser
tests pass Rust 1.88. Coverage-guided fuzzing is a separate explicit tooling command.
The incoming rustls-webpki lock update is preserved; no release or publication
was performed.

Isolated sibling snapshots at `/tmp/platform-adoption-audit` passed Auth's 11
canister-signature tests (`24bbad5`, clean source), IcyDB's 17 Account stored-byte
and ordering tests (`07a5964` plus pending work), and Canic's four call-adapter
tests (`8c2dda4` plus pending work). A disposable test added only to the Wire
snapshot (`fe7ab95`, incoming lock changes) compares 120 Principal and 360 Account
encodings against registry external types and decodes the owned values, preserving
absent/explicit-zero fields. It passes against registry Platform 0.4.0.
Snapshot HEAD/status records and logs are retained alongside the copies. This
qualifies the selected tests, not all sibling endpoints or historical database
upgrades. No sibling checkout was edited.

Canic's frozen Wasm normal/build graph still includes Platform 0.2.2 through
`ic-auth-protocol-types 0.10.7` and `ic-timers 0.29.4`, alongside 0.4.0.
Executor 2.0.0 and ic0 1.2.0 remain single identities. This is not a complete
consumer cutover; dependency adoption belongs to those consumers. IcyDB Core
and Auth default graphs select Platform 0.4.0 without Candid. Retained graphs and
the Canic reverse-dependency trace are in the snapshot evidence directory.

The coverage-guided parser campaign passes 1,441,163 executions in 61 seconds
with AddressSanitizer and seed 42. Evidence: `target/parser-fuzz.WPlyho` and
`/tmp/platform-parser-fuzz-asan.log`. The first campaign completed 1,543,528
executions but LeakSanitizer failed at shutdown under sandbox process inspection;
the successful rerun set `ASAN_OPTIONS=detect_leaks=0`. Leak detection is not
qualified, and the initial empty crash artifact records that tooling failure.
Rust 1.88 generated parser tests and CDK/ledger Wasm checking, strict selected
Clippy, registry-admission refusal tests, source fingerprints, pins/boundaries,
formatting, shell/YAML parsing and documentation links pass. Full CI remains
release-owned; the new GitHub workflow itself has not yet run.

## Released 0.4 transport-only calls

The maintainer requested further justified breaking cuts in the same migration.
Raw Call/Response no longer implement Candid request/response methods, and the
combined call Error and string-only CandidDecodeFailed are retired. Use explicit
Candid encoders/decoders around raw bytes; encoding traps, decoding/arity limits,
diagnostics and retry decisions remain caller-owned. Existing oneway and unknown
reject-code error types are now explicitly nameable through the facade. The
borrowed `encode_args_ref` helper preserves the previous borrowed-argument use.
See [the transport migration](../migration.md#transport-and-codec-separation-04).

Canic's request builder already encodes explicitly; its response helpers and
obsolete combined-error mapping are the targeted migration points. No sibling
files were edited. Checkpoint implementation/bytes, system calls, executor identity,
callback/trap cleanup, timeouts and cycles remain unchanged. The maintainer released this work as 0.4.0 at `26a07c5`.

Linux validation passes: Rust 1.88 all-feature tests/Wasm, standalone CDK and Candid
native/MSRV/Wasm consumers, strict Clippy, warning-denied Rustdoc, source/license
and mutation guards, formatting and documentation checks. The new installed
fixture explicitly encodes management arguments and uses its original upstream
decoding policy. All four PocketIC 16.0.0 runs pass (Rust 1.91 and 1.99.1, Wire-only
and external-Candid), including original checkpoint bytes and upgrade/rollback.
Evidence: `target/cdk-runtime.oau1gn` and `/tmp/platform04-transport-*.log`.
Actual consumer migration, native macOS and live IC qualification remain separate.
No new feature, dependency, publication, measured size claim or crypto change.

## Released 0.4 runtime codes and signature packaging

The maintainer authorized both changes after the 0.3 audit. Runtime RejectCode
is owned, preserving codes 1–6, Display/Serde spelling and raw unknown-code
refusal. Agent/PocketIC enums remain external. Its Strum iteration and external
ErrorCode conversion are intentionally not retained. Signature creation/verifier
source is packaged inside Platform with unchanged upstream bodies and 29 tests;
only documented imports and the unused store declaration differ. Exact source
and license fingerprints plus mutation refusals guard the imported modules.
BLS, certificate and hash algorithms are unchanged. See the
[0.4 migration](../migration.md#runtime-codes-and-signature-packaging-04).

These public identity changes shipped in the maintainer’s 0.4.0 release. No commits, publication or sibling edits were made.
The prior handoff correction is preserved. The root lock removes the retired
packages and adds only the imported test helper; the runtime fixture lock removes
its obsolete runtime error-catalogue edge without changing other selections.

Linux qualification passes: Rust 1.88 all-feature tests and Wasm checking,
standalone CDK/creation/verifier native/MSRV/Wasm consumers, strict all-feature
Clippy and warning-denied Rustdoc. Eighteen native/Wasm production graphs confirm
signature-only selections exclude Candid/ic0 and both registry signature packages;
CDK-only excludes ic-error-types/Strum. Graph admission includes 71 regression
cases. Feature closures/selections, runtime source/mutation/lock checks, dependency
ownership/pins, formatting and documentation checks pass. Signature fingerprints
also retain the original tests and licenses. This is not a measured Wasm saving.

Four installed PocketIC 16.0.0 runs pass: Rust 1.91 and 1.99.1 Wasm, each Wire-only
and external-Candid mode, covering callback/trap cleanup, timer cancellation,
upgrade, cycles, stable rollback and certificates. Artifacts are retained at
`target/cdk-runtime.0tbMOx`; logs at `/tmp/platform04-installed-*.log`. Other focused
logs are `/tmp/platform04-{msrv,consumers,all-msrv,all-wasm,final-clippy,doc}.log`;
graphs are in `target/platform04-graphs`. Native macOS, live IC and actual Auth/Canic
migration or installed signature-policy qualification remain separate work.

## Released 0.3 owned account

The historical 0.3.0 release was `390644f`, following implementation commit
`80a81e8`. The maintainer reports it pushed; this pass does not independently
establish crates.io publication. Qualification below records the implementation
checks, not completed sibling adoption.

The unconditional `Account`, `Subaccount` and `AccountError` now accompany owned
Principal. Effective-zero equality, ordering and hashing match upstream; Serde
and conversions preserve absent versus explicit zero fields. Text parsing admits
at most 136 bytes, decoding subaccounts into a fixed array. Formatting preserves
the ledger CRC32/Base32/hex spelling with no intermediate strings. No cryptographic
implementation, dependency, feature or storage integration is added.

The existing external ledger DTO is unchanged, also named
`account::ExternalAccount` under `icrc-ledger-types`. Explicit conversions preserve
owner bytes and optional subaccount fields. Owned Account has no Candid trait.
Apache source/license provenance is retained under `src/account`. See the
[account migration](../migration.md#owned-account-migration-03).

Linux qualification passes: Rust 1.88 unit tests, Platform-only standalone native
and Wasm consumers on Rust 1.88 and the development toolchain, strict selected
Clippy, formatting, documentation links and six native/Wasm production graphs.
Differential coverage checks 23,040 account cases against upstream text and Candid
bytes, plus marker-free owned/borrowed Serde and effective-zero identity tests.
Logs: `/tmp/platform-account-{consumers,clippy}.log`; graphs: `target/account-graphs`.
These are source/codec checks, not sibling Wire integration, persisted-data,
native macOS or installed-canister qualification. No sibling files, release
versions, commits or publication changed; prior Principal work remains intact.

The follow-up account sanity review found no production defect. Expanded tests
cover all 64 trimmed hex lengths, 53,504 text mutations against upstream admission,
4,096 ordering comparisons, upstream hash parity, explicit-zero Serde tokens and
malformed array lengths. Rust 1.88's 14 unit/3 ledger integration tests, standalone
native/MSRV/Wasm consumer and strict selected Clippy pass. Evidence is retained
in `/tmp/platform-account-review-{tests,consumer,clippy}.log`.

## Released 0.3 owned Principal

The maintainer authorized an owned Principal inside Platform, retaining Apache
provenance and unchanged upstream cryptography. Source now exposes owned root
`Principal`/`PrincipalError`, with explicit registry
`principal::{ExternalPrincipal, ExternalPrincipalError}` for external DTO/client
boundaries. Candid's contract still exports the registry type. The owned type
has no Candid trait or marker handling and no generic range-step implementation.
Parsing admits at most 63 text bytes before work; parsing/Display use stack
buffers. Original bytes, padding and length-first ordering are preserved.
Self-authentication delegates to registry ic_principal; this is not dependency
removal. See [the migration](../migration.md#owned-principal-migration-03).

Workspace versions are now 0.3.0. The implementation pass preserved the incoming
webpki-root-certs lock update and added direct dependency edges and the Serde test
helper; the maintainer subsequently committed and released that work. It did not
qualify sibling migration.

Qualification passes on Linux: all-feature native tests and strict Clippy;
Rust 1.88 Principal/runtime/Candid tests; standalone Principal, Candid and CDK
native/MSRV/Wasm consumers; sixteen native/Wasm production graphs; warning-denied
Rustdoc; formatting, pins, boundary, documentation and runtime provenance checks.
Principal regressions compare 7,680 byte/text values with the upstream owner,
check both owned/borrowed Serde bytes and preserve self-authentication vectors.
A disposable Rust 1.88 allocation probe confirms zero allocation/reallocation
calls for parsing, oversized rejection and Display into a prepared destination;
`to_text` performs one result allocation. This is not a speed or Wasm-size claim.
Probe evidence: `/tmp/platform-principal-measure/run.log`.

Installed Testkit/PocketIC 16.0.0 qualification passes for Rust 1.91 and 1.99 Wasm,
each Wire-only and external-Candid mode: RPCs, raw awaits, traps, timer cancellation,
upgrade, cycles, stable rollback and certificates. External mode additionally
preserves management-call and checkpoint bytes. Sources/artifacts are retained in
`target/cdk-runtime.Jy8nvl`; logs are
`/tmp/platform-principal-installed-{1.91.0,1.99.0}-{wire,external-candid}.log`.

The maintained runtime fixture's older Testkit/Timers require registry Platform
0.1.18. Its previously rewritten lock omitted that compatible version; it is now
resolved explicitly beside the local candidate. The graph still has one registry
executor 2.0.0 and ic0 1.2.0. Release-lock regression permits only the necessary
local version-qualified dependency-reference updates, preserving registry nodes.
This fixture is compatibility evidence, not actual next-minor sibling adoption.
Native macOS, live IC and consumer migration qualification remain outstanding.

## Prior delivery evidence

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
now have Platform identities. The 0.4 candidate also owns runtime RejectCode and
signature-key identity; ExternalPrincipal, external DTOs, certification and BLS
retain registry identities. See
[the migration contract](../migration.md#maintained-runtime-and-wire-entrypoint-adoption).

Raw call constructors begin with empty transport bytes. Every Wire call supplies
its request frame; required external no-argument Candid calls explicitly use
`candid::contract::encode_args(())` and supply the resulting raw bytes. Bounded calls still default to 300 seconds. Callers retain
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
