# Platform development and qualification

[Workspace and feature selection](../README.md) · [Migration](migration.md) · [Current handoff](status/current.md)

Run the commands below from the repository root. Select checks by the changed
contract; aggregate and installed-runtime gates retain their explicit scope.

## Runtime qualification and historical evidence

The optional `cdk` surface uses maintained, non-cryptographic runtime adapters
and the owned generator. It does not depend on the full `ic-cdk` package.
All controlled endpoints use explicit Wire hooks; select `candid` separately
for required external management/system interfaces and existing checkpoint bytes.
These breaking runtime and transport changes are released in the 0.4 line.
See [the migration contract](../docs/migration.md) for ownership and adoption conditions.
The public GitHub repository exists; registry availability requires separate verification.

Installed-runtime qualification runs separately in `.github/workflows/runtime.yml`,
using Testkit's explicit server setup/admission and both compiler/protocol modes.
For a prepared registry cache and admitted server, run
`bash scripts/ci/check-platform-runtime.sh --published 0.4.0` to exercise the exact
published facade and generator without a local override. Append `--check-graph`
for graph-only admission. Local CI cache preparation uses `--prepare`; ordinary
qualification never installs a server. Native macOS and live IC acceptance remain
separate from this Linux workflow.

Historical measurements and reproduction instructions are in
[the benchmark report](../docs/benchmarks/0.4/README.md).
`bash scripts/ci/fuzz-parsers.sh` runs a separate 60-second coverage-guided
Principal/Account differential campaign and caller-budgeted Candid decoding
through all three public decoder policies, plus strict certificate/hash-tree and
typed IC packet CBOR decoding. Successful strict certificates are compared with
the typed upstream-backed decoder; strict trees round-trip through Platform's
typed encoder. Independent strict/upstream differential tests remain inside the
codec unit tests; byte-budget refusals are checked independently.
Deterministic recursive-type, overlong-header, extra-argument, CBOR mutation and
truncation cases also run in consumer tests. Prepare nightly-2026-11-05, a C++
compiler and registry sources explicitly first; the command installs nothing.
It retains its corpus, full log, crash artifacts and disposable lock under `target/`.
The sanitizer needs host process inspection; a sandbox refusal is not a passing
campaign and must not be hidden by disabling the sanitizer.
This tooling-only libFuzzer selection does not enter consumer dependency graphs.

### Wasm regression comparison

Run `bash scripts/benchmarks/check-wasm-size-regression.sh BASE_REF [MAX_GROWTH_BYTES]`
with the compiler, registry sources and `ic-wasm` already prepared. The default
profiles are binary Principal, Runtime, external Candid, signature verification
and strict certificate CBOR, using one `3-fat-1` recipe. `WASM_BENCH_PROFILES`
selects a smaller affected subset. Both arms use the same current probe source,
compiler, recipe and current dependency lock; this isolates implementation changes,
not dependency-upgrade effects. Incompatible resolved external locks refuse comparison.
The separate asynchronous Wasm size workflow runs the default selection on relevant
PRs and main-branch pushes and retains total uncompressed and code-section
measurements before/after optimisation. Its default allowance is 128 bytes per
measurement; dispatch can choose an explicit allowance. This is a regression trigger, not a consumer Wasm
budget, instruction benchmark or permission to weaken verification.
Pushes compare against the event's preceding main commit; PRs use their base
commit, and manual dispatch retains its explicitly selected baseline.

`node --test scripts/benchmarks/tests/compare-wasm-sizes.test.mjs` checks comparison
admission. The shared ingress corpus at
`crates/ic-platform/tests/fixtures/ingress.tsv` is exercised by Rust unit tests
with `ic-protocol` and browser `ingress.test.mjs`. It deliberately records the
browser's extension-field acceptance and Rust's upstream text-to-byte coercion;
neither inspection API authenticates requests or replaces original signed bytes.

Manual release-mode benchmarks are ignored by ordinary tests:
`cargo test -p ic-platform-runtime --features executor --release --lib benchmark_protected -- --ignored --nocapture`
and `cargo test -p ic-platform --features candid --release --lib benchmark_shared_type -- --ignored --nocapture`.
These measure native bookkeeping/admission, not installed Wasm instructions.

The installed Runtime fixture also has an opt-in cancellation measurement:
`PLATFORM_EXECUTOR_BENCHMARK=1 POCKET_IC_BIN=/prepared/pocket-ic RETAINED_DRIVER RETAINED_WASM`.
Use the driver and Wasm from a matching Wire owner profile; this mode replaces
the ordinary installed corpus for that invocation. It measures only cancellation
of 0, 1, 32, 1,024 and 8,192 protected tasks in forward/reverse order, with task
creation outside the counter and destructor-order/count assertions. Compare
identical fixture sources, compiler, lock and build settings between arms.

Routine installed checks exercise both buffer-copy implementations once at each
of five payload sizes (ten queries). Collect the former six-round instruction
medians explicitly with `PLATFORM_BUFFER_COPY_BENCHMARK=1` and a matching retained
Wire driver/Wasm, using the same invocation pattern above. This mode replaces
the remaining installed corpus for that invocation; correctness checks still
compare both implementations' complete returned bytes at every size.

`WASM_BENCH_PROFILES='runtime-system candid-admission'` additionally builds
opt-in System API and Candid admission probes with the same size runner/recipe.
Use `PLATFORM_CODEC_BENCHMARK=system` or `candid` with a matching retained Runtime
driver and the corresponding raw probe Wasm. The driver replaces its ordinary
installed corpus for that invocation. System measurements create an explicit NNS
subnet for Testkit's root-key byte comparison and decode the returned certificate;
ordinary runtime qualification keeps its original topology. Candid measurements
compare typed refusal text with the native reader, including work exhaustion and
recursive-value refusal. Request admission and reply framing stay outside the
counters. Neither probe changes production exports or consumer resource policy.

Consumers own schemas, authentication, byte/work budgets, deadlines, retries and
recovery. IC Wire owns controlled application protocols. Candid remains required
where external protocols specify it. Moving a dependency behind this boundary
does not itself remove upstream code or shrink a canister.

## Development

Use `make cloc` (`CLOC_SUBCRATES=1` for package rows) for Shared 0.30’s
source classification, or `bash scripts/dev/cloc.sh --json` for schema 2. It
separates library/application, tests, examples, benchmarks, build/tooling and
unclassified source, including inline test blocks. The report counts each source
path once and reports macro expansion limits; it is not a linked-Wasm metric.
Owned test files use `tests.rs`, `*_tests.rs` or `tests/`; the main workspace
has no singular `test/` or `test.rs` paths. Generated qualification and benchmark
packages declare `[package.metadata.shared-tooling.loc]` with the appropriate
`category`, even when their library target carries a consumer proof. Use
`--manifest /absolute/path/to/Cargo.toml` to classify an independent generated
workspace explicitly; the default report covers the five main members only.

New Platform-owned qualification scratch defaults beneath `target/`, while
Shared snapshot helpers retain their supplier-owned scratch contracts.
`cargo clean` reclaims the whole default workspace target, including diagnostic
captures and the relocated old Runtime driver executables. The separately
approved historical archive under `recovery-evidence/` survives cleanup.
`make check-retained-evidence` enforces a 320 MiB aggregate, 256 MiB single-file
and 30-day bulk age limit through Shared’s read-only checker before release
preflight/publication. Its owner and dated review are in [AGENTS](../AGENTS.md#recovery-evidence-custody).
The release runner separately enforces Shared’s Git-side defaults. Both refuse
without automatic cleanup; no archive, Git refs or recovery journals were deleted.


Platform 0.11 removed `wire-schema`. Principal and Account retain
their existing Serde representations; Wire owns the schema integration. See the
[migration contract](../docs/migration.md#owned-principal-and-account-wire-metadata)
for Wire 0.10’s `platform-types` integration and consumer requirements.

Browser SDK development uses Node 24.21.0 and npm 12.2.0. Run
`make install-browser-tools` (or `make install-tools`) to prepare its pinned
TypeScript compilers and SDK fixtures. `make check-browser` checks strict
TypeScript, adapters and packed consumers. Node/npm setup identities live in
`scripts/fixtures/browser-compiler-floor/package.json`; the browser package
selects its development compiler independently.

`bash scripts/ci/check-advisories.sh online https://github.com/RustSec/advisory-db.git target/platform-advisory-attempt`
audits the workspace, owner-runtime and Timers-composition locks independently without compiling feature matrices. Prepare
`cargo-audit` explicitly first (CI selects 0.22.2) and prepare common host tools;
the destination must be new. Online mode performs three locked dependency fetches
in a fresh `CARGO_HOME` under the evidence directory, including the runtime's
existing `--prepare` operation. This downloads registry data and crate sources
without compiling or sharing the user's package-cache lock.
Failed preparation marks that graph's audit unavailable and fails the aggregate;
successful producer audits still run and retain their results.
For offline inspection, replace `online URL` with `local DATABASE_REPOSITORY`
and select an already prepared Cargo cache through `CARGO_HOME` when needed.
The isolated database revision, tool version, copied locks, registry preparation,
logs and raw audit statuses are retained. Separate `*.index-status` files report
complete/incomplete coverage, or unavailable after preparation failure; index
failures reject even if cargo-audit exits zero.
Offline results do not establish advisory or registry freshness. Known vulnerabilities
fail the check; informational maintenance warnings remain visible and are not
exemptions. Yanked versions retain cargo-audit's existing warning policy; missing
index coverage is a failing observation. No maintained lockfile or shared advisory
cache is updated. A separate PR/main CI job
runs the online check and retains its revision and findings in job logs.

Platform 0.18 retires the unused stable-structures feature, module and
upstream dependency; see the
[contract](../docs/migration.md#stable-structures-substrate).

Prepare common tools explicitly with `make install-tools`; `make tools-check`
verifies them offline. Prepare locked dependency sources with
`cargo fetch --locked` before validation, including the Wasm target dependencies
used by frozen standalone consumer checks. Then run
`make prepare-consumer-dependencies` to fetch the registry executor, legacy
Platform owner and Wire used
only by standalone fixtures; the embedded runtime and schema transfer removed
them from the root lock. CI and admitted release preparation run this setup too.
Normal consumer checks remain offline and never install executables. Rust 1.99.1
is the minimum, development and release compiler. Prepare that exact toolchain
with `wasm32-unknown-unknown`; `make check-compiler` admits and records the actual
Cargo/rustc identities and requires the workspace floor and toolchain pin to agree.
The previous 1.91 support promise is retired by [#133](https://github.com/dragginzgame/ic-platform/issues/133).

Native CI uses Shared's conservative documentation classifier with the exact
prose allowlist in `ci/docs-only-paths.txt`. Those files already participate in
`check-doc-links`; eligible changes retain snapshot and link checks under the
existing job names. Code, packaged Markdown, policy/workflow changes and uncertain
observations retain the full Linux/macOS matrix. Advisory, browser, installed
runtime and release gates remain independent. `bash scripts/ci/test-platform-ci-selection.sh`
checks the actual workflow wiring and fallback cases.

Installed-runtime CI prepares only the common tools, in order with
`make install-host-tools` and `make install-rust-tools`. It does not need the
release editor or Node/npm/TypeScript. Normal `make install-tools` still prepares
the complete contributor set, and standard CI retains the Wire frontend proof.

Tool setup includes the pinned IC Build 0.73.0 release editor with its `cli`
feature, which requires Rust 1.91 or later to build. `make install-release-editor`
prepares it separately;
`make release-editor-check` admits it offline. Standard release commands prepare
tools during admitted preflight; standalone metadata checks never install them.
Pending changelog notes use `## [X.Y.Z]`; `## [X.Y.Z] - Unreleased` is also
accepted and dated during release preparation.
Platform does not require shared IC executables for normal setup. Optional
`make install-ic-tools` and `make ic-tools-check` calls must explicitly select
the required tools with `IC_TOOLS`, for example `IC_TOOLS=ic-wasm`.

Select the affected package and features for development, for example:

```sh
cargo clippy --locked --offline -p ic-platform-types --lib -- -D warnings
cargo test --locked --offline -p ic-platform-types --lib
```

`make check`, `make test`, `make clippy`, `make wasm-check` and `make msrv`
are aggregate checks, not focused selections. Use them for explicitly requested
broad validation or release qualification. `make ci` is the full release gate;
its fail-fast ordering runs Clippy before tests. Documentation-only work uses
`make check-doc-links` and diff/consistency review.
For a selected package with all its features/targets, the existing lint leaf
accepts `make clippy PLATFORM_CLIPPY_PACKAGES=ic-platform-runtime`; use direct
Cargo arguments above when only a narrower feature selection is affected.
The former upstream-scaffolding experiment runner is retired with native offline
public-key derivation. Historical packaging proposals remain documented in the
[migration contract](migration.md#scaffolding-only-packaging-qualification);
they are not selected dependencies or current qualification commands.

Consumer profiles are declared in
[`platform-consumer-profiles.tsv`](../scripts/ci/platform-consumer-profiles.tsv):
capability, test source basename,
comma-separated asset globs (`-` for none), and Wasm command (`check`, `build` or
`skip`), and graph family. Required features come from the corresponding Cargo
test declaration. The runner preserves native and Wasm tests on the supported
compiler and admits runtime graphs first.

`make check-consumer` tests separate Candid-only, certification-only, parser-only,
dynamic, management/Candid, ledger/Candid, ic0-only, Agent/certification three signature/BLS consumers and a CDK canister
with only Platform declared, on Rust 1.99.1. The CDK standalone
consumer proves raw entrypoints/calls without Candid or full CDK dependencies.
Generated consumer packages deny warnings in ordinary library, test and Wasm
builds, without changing lint flags for upstream dependencies.
The `external-candid` profile separately qualifies required external Candid
entrypoints, explicit service registration and fixed wire bytes without a direct
Candid dependency; the supported compiler links its Wasm library.
Candid and certification
consumers also compile for Wasm, as do the management, ledger and raw-system
consumers, all three signature/BLS selections and the CDK canister. `make msrv`
remains an alias for the native all-feature tests and all-feature Wasm check;
CI runs those proofs once through `test` and `wasm-check`.
The ic0-only caller compiles without executing a system call on native hosts.
Management qualification covers fixed HTTP/history wire bytes and change variants.
Ledger qualification covers account semantics, fixed account/block wire bytes,
representation-independent hashes, arbitrary-precision amounts and archive callbacks.
It covers DTOs, argument helpers and wire vectors, plus hash-tree constructors,
lookup outcomes, witness hashes and certificate/delegation fields. The parser
consumer runs natively and covers external checking, constructor types, service drift
and typed textual-argument encoding. The Agent consumer checks offline signing,
request identity, effective-ID routing, typed errors and shared certificate types.
Signature qualification preserves original-owner DER/hash/BLS/certificate vectors,
typed errors and malformed-proof refusal. Certification also covers Auth's nested
mutable tree updates and root-preserving witnesses. BLS keys use deterministic
caller-supplied fixture bytes; no entropy source or signing policy is installed.
Parser and Agent dependencies and APIs are absent on
Wasm, including with all features enabled.
`make check-features` verifies default, executor and parser-isolation graphs.
`make install-hooks` installs the shared formatting hook; `make fmt-check`
checks Rust and manifest formatting. Authorized releases use the shared
`make release-patch`, `make release-minor` or `make release-major` workflow.
Fresh releases require an explicit group or supplier-ordered cohort:

`make release-patch RELEASE_GROUP=platform` releases the facade.
The native client now releases in Tooling’s Testkit group.
`make release-patch RELEASE_GROUP=runtime` releases Runtime, macros and exact guard. `make release-patch RELEASE_GROUP=types` releases the
canonical type supplier. No group is inferred from changed files or PACKAGE.
All members use literal package versions; root Cargo stores common metadata and
dependencies. The [root changelog](../CHANGELOG.md) indexes the three active group ledgers
and preserved coordinated history.

Fresh group releases execute in Shared's frozen candidate and leave the normal
checkout's branch/versions unchanged until explicit reconciliation. To publish
as part of that same batch, select `RELEASE_PUBLISH_TARGET=publish` on the
initial release command. Multiple affected groups use one
`RELEASE_GROUPS=types,runtime,platform` roster, omitting unaffected groups;
do not append a normal-tree `publish` goal. Recover with the printed batch ID,
using `release-batch-status`/`release-batch-resume`, then reconcile deliberately.
See [the exact commands and effects](platform-release-validation.md#commands-and-effects).
Root dependency blocks name each group once, with a separate changelog comment.
Their path entries catalog every member's full version and retain requirement
operators. The unused facade entry adds no dependency edge. Dependency and release
checks enforce complete catalog/alias coverage,
owned paths and current full versions across all three groups.

Standalone `make publish-check RELEASE_GROUP=platform` packages/verifies the
selected normal-checkout group without
uploading; `make publish RELEASE_GROUP=platform` uploads clean committed packages
in Cargo dependency order. Select types explicitly for its separate publication.
`PACKAGE='name ...'` may narrow the selected group after partial-publication
reconciliation; cross-group batches refuse. When several groups change, publish the
supplier group first, then its compatible consumer group. All retain full gates,
transactional runtime metadata and exact source/group-bound resume. Publication
does not bump versions, commit, tag or push.
Both commands use the lockfile and retain evidence under `target/publication/`.
Publication Clippy uses exactly the admitted package selection, retaining the
pinned compiler, all-target/all-feature checks and denied warnings. It does not
replace or narrow the separate full release-qualification gate.
They require the pinned host tools (`make install-host-tools`). Cargo verification
is not the full release gate or native macOS qualification. No upload retries are
automatic: after an interrupted dispatch, reconcile every dispatched exact registry
package version before invoking publication again. Partial publication is not rolled back. A successful upload command alone does not
prove registry visibility or consumer adoption.

For focused release-recovery changes, run
`bash scripts/ci/test-platform-group-preparation.sh runtime` (or `platform`/`types`).
Omitting the group exercises all three. These fixtures use disposable local Git
remotes and substitute the full release gate; they do not release this checkout.

Plain `cargo clean` removes the whole workspace `target/`, including frozen
candidates, nested build caches and evidence. Do not run it while a frozen batch
needs its candidate; use the explicit shared retirement flow after reconciliation
and evidence preservation. Keep evidence that must survive cleanup elsewhere.
The separately managed `.tools/` store is outside Cargo cleanup.

Linux x86-64 and macOS 15 on Intel and Apple Silicon are supported hosts.
CI configures all three; native execution qualifies each separately. Wasm compile
checks establish target compatibility, not installed executor/trap behavior.

## Dependency roles and deployment admission

The reviewed Shared Tooling revision is recorded in [the snapshot](../.shared-tooling.snapshot).
`ic-platform` owns the low-level boundary; its `call` module adds prepared one-shot
requests with owned or borrowed methods and bytes through its existing runtime.
The module uses the existing `runtime` feature and adds no dependencies or codecs.
See [the call contract](../crates/ic-platform-runtime/src/call/README.md). `ic-platform-macros` is the production
proc-macro support package. Its Syn/Quote dependencies generate consumer
entrypoints. Principal, Candid, the host parser, Agent, certification, raw IC calls,
management/ledger DTOs, signature/BLS APIs, the executor and Serde are production
dependencies, including optional and derive use. Testkit and Timers are
tests/tooling entries used by the independent fixture. Its host driver selects
Platform's `certification-cbor` decoder instead of a direct generic CBOR codec.
Optional parser
and Agent dependencies are restricted to non-Wasm targets.
The root catalog has `Production dependencies` and `Tests and tooling only`
sections. Fixture-only selections stay in the second section and outside the
consumer library's production graph.
Production subsections distinguish local group catalogs, external runtime and
native-only adapters, host proc-macro dependencies, and mixed production/test use.
Members declare no build-dependencies tables; their normal macro dependencies run
on the compiler host. Actual member tables and features determine dependency edges.
`fmt`, `fmt-check`, the shared hook and release metadata checks all preserve
sections with grouped ordering. Shared 0.27.7 owns the formatting passes;
Platform selects its existing standalone Rust source roster and edition 2024
through `FORMAT_RUST_ARGS`. These selections run uncached and show per-pass
timings. `make check-formatting` qualifies the actual Make policy and staged hook
in an isolated fixture, including standalone sources and failure stopping.
Keep a blank line after each section heading so sorting preserves its position.
The common Rust tool set now contains only cargo-sort and cargo-sort-derives;
this repository has no Candid extraction command requiring a selected extractor.

`make check-features`, included in CI/release validation, checks normal/build
graphs offline (`--frozen`, preserving the lockfile). Prepare missing registry
inputs once with unfiltered `cargo fetch --locked`. Target filters can omit
foreign-target dependencies of native-only clients needed by frozen consumer checks.
Both matrix modes report unique query counts and progress every 100 queries.
Standalone consumer compilation
also stays offline after its existing offline metadata resolution. Offline mode
prevents repeated network work but does not bypass Cargo cache locks held by
another process. Publication still uses the network normally.

The feature gate checks normal/build
transitive graphs for the native host and `wasm32-unknown-unknown`, with no
features, actual defaults, every individual feature, every feature pair, all
features together and explicit critical groups. Those groups cover the runtime
with its prerequisites, runtime with management/ledger/certificate DTOs, combined
signature capabilities, runtime with all native-only leaves, and all canister
features without host leaves. The separate CDK/Timers runtime qualification and
release runtime-lock checks are unchanged. It enforces empty defaults
and exact qualified executor/system requirements. It rejects unreviewed feature
names until the matrix and isolation policy are updated, admits the owned
generator only with `cdk`, rejects full CDK packages, and rejects additional executor/
system versions or non-registry sources in the transitive graph. It also
rejects the retired PocketIC feature and the `ic-testkit`/`ic-testkit-candid`
canister dependencies, and checks Candid/CDK-free defaults and Candid-free
executor-only graphs. Cargo reports canonical package names, so dependency
aliases do not bypass these exclusions. `check-boundary` independently checks
direct dependency ownership; role comments themselves are not executable policy.
`make check-features-exhaustive FEATURES=cdk,candid,executor` audits every subset
of 1–12 explicitly selected features and actual defaults using the same admission
rules. Missing, unknown and oversized scopes are refused before graph queries;
the whole 31-feature power set would exceed two billion selections. This manual
audit is outside routine CI/release validation. Routine coverage deliberately
omits other higher-order combinations. Wasm cache keys omit native-only leaves
only after their exact feature/dependency/target contracts pass validation.
Changed contracts fail closed. Defaults remain a separate observation. One batch
validator per target checks every original selection against its graph; cached
graphs are not copied per selection. Each `target/feature-graphs/attempt.*`
directory retains target-specific TSV mappings and the actual Cargo graphs.
This reduces cache-lock acquisitions; it does not eliminate contention with
other Cargo processes or claim a measured wall-clock speedup.
Certification-only builds are Candid/CDK-free. Native Agent itself requires
transitive Candid and certification; it does not select those Platform features.
Management DTOs also require transitive Candid and are admitted only when selected.
Their `=0.11.0` requirement preserves the maintainer-selected external type contract;
enable `candid` separately when using Platform's encoding and bounded decoding.
Ledger DTOs similarly bring transitive Candid without selecting Platform's codec,
executor, parser or Agent. Upstream defaults/storage integration are disabled;
the graph gate rejects `ic-stable-structures` and unselected ledger dependencies.
Canister-signature creation and verification select upstream certification and
external Principal conversions. Their registry packages bring Candid/printer,
CBOR, representation-independent hashing and raw IC0 transitively; only
verification brings BLS. CanisterSigPublicKey retains the owned Principal field
through a delegating adapter. DER framing and both root/delegated identity-key
refusals remain integration guards. Signing-time extraction follows upstream
verification and requires an additional CBOR decode. The upstream signature store
is not exported. Standalone BLS still excludes creation, Candid, certification,
CDK and host tooling; defaults/RNG are disabled and alloc is explicit.
Graph admission checks the qualified registry signature/BLS/certification suppliers.
Agent and certification expose the same registry certificate identity, removing
tree-copying conversions. Consumers own their own qualification.
`candid_parser` and `ic-agent` are admitted only with explicit native host
selection; no runtime feature enables them. Host retains strict bounded CBOR decoding;
trust roots, authentication and application interpretation remain consumer-owned.

Platform embeds the executor and IC0 implementations from upstream 2.0.0/1.2.0.
Their registry packages are absent from raw Runtime graphs. The selected upstream
canister-signature dependency may bring raw IC0, but never another executor. The inherited Cargo singleton guard lives in
an internal helper selected only by `executor`, requiring one Platform runtime
owner across Timers and applications. Native consumers without runtime can coexist with
an unrelated registry executor; task/method handles
now have Platform identity. See the [breaking migration](../docs/migration.md#embedded-ic0-and-executor).
Changes require combined callback/trap/cancellation/upgrade qualification.
Use `candid::decoder` when preserving upstream configuration/defaults or observing
construction, value, finalization and drop separately. The `candid` feature also
supports explicit service registration/export and reflection; see the packaged
[consumer contract](../crates/ic-platform/README.md) for generated paths and the
CDK adoption requirements and older Testkit direct-dependency exception.

This library has no deployment entrypoint. Missing deployment enforcement belongs
at each consuming application: run exclusions against its exact final feature,
target and lockfile selection, retain successful builds, and bind source/lockfile
identity and artifact digest to deployed bytes. This repository does not provide
that artifact admission or infer new exclusions automatically from role comments.
`make publish` does not run the graph gate; registry publication is separate from
the release gate and from deployment qualification. Native macOS must run its own
checks; Linux/Wasm graph results do not qualify macOS or installed canisters.

The facade combines MIT and Apache-2.0 sources; its imported decoder retains
IC Testkit's copyright notice. The maintained generator is Apache-2.0.

### Maintained runtime regression

`make check-runtime` qualifies both raw calls and owned `ic_platform::call` requests in one
fixture, sharing the host-driver and compiler builds. It installs Wire canisters
through Testkit-managed
PocketIC on Rust 1.99.1. The pure `cdk` selection rejects Candid, parser,
full CDK and native host clients in its canister graph. A separate
`external-candid` selection tests an outbound management `raw_rand` call with
explicit Candid arguments and preserves the original persisted checkpoint bytes.
All RPCs and installation/upgrade arguments use Wire in both selections.

The fixture checks raw awaits, composite queries, guard/malformed-frame refusal,
all four explicit request allowances, response-encoding traps, callback cleanup,
weak/protected/migratory spawning, submission-pressure and insufficient-cycle
errors, direct executor cancellation, upgrade, cycle/refund counters, stable-byte
rollback and certificate lookup. It installs no alternative executor.

Run `make check-runtime-lock` for coordinated lock, graph and imported-source
checks, or `bash scripts/ci/check-platform-runtime.sh --check-graph` without
compiling/starting a server. Source fingerprints protect the imported call,
callback/trap cleanup and protected-spawn regions; mutation cases test refusal.
The sole executor and IC0 implementation live in `ic-platform-runtime`; the facade
delegates to that owner. The owner fixture
has no Timers dependency. `make check-runtime-composition` reuses the fixture with
Timers 0.37.0, adding dispatch, callback trap/cleanup, cancellation, rearm and
post-upgrade dispatch. `make check-runtime-all` runs both paths even if one fails.
PR CI reports independent jobs; release verification requires both paths to pass.
The prepared release version must also pass composition graph admission, preventing
a version bump from silently invalidating the tested scheduler selection.
No scheduler version mismatch waives that release requirement. This split does
not change runtime package identity or executor singleton admission.
After preparing both profiles' caches, run
`bash scripts/ci/test-runtime-independence.sh --prepare` once to fetch the
historical Timers input. Then `make check-runtime-independence` exercises
the real resolver with incompatible Timers 0.35.1 in a disposable copy: owner
graphs must pass and composition must fail the singleton constraint.

Prepare/admit PocketIC using Testkit, then set `POCKET_IC_BIN` or
`IC_TESTKIT_POCKET_IC_URL`. Owned native fixtures select published Testkit 0.92
and its async-only client. Each executable owns one current-thread Tokio runtime
for the complete scenario, awaits network/lifecycle operations and explicitly
awaits instance teardown before releasing its managed server. Client requests
have an explicit 60-second finite budget, separate from the 60-second startup
budget. The benchmark and Agent fixtures use the same catalog selections; Tokio
is a native fixture dependency and never enters the canister graph.
The harness retains Testkit's bounded startup and
teardown. It never installs a server implicitly. Artifacts, sources and metadata
remain in `target/cdk-runtime.*`. Normal runs use the maintained frozen selection
in `scripts/fixtures/cdk-runtime/Cargo.lock` (owner) or
`scripts/fixtures/timers-runtime/Cargo.lock` (composition). Prefix the runtime
script's mode with `--timers` to select composition; explicit `--refresh-lock` is a
reviewed dependency refresh that preserves existing foreign pins and resolves
necessary manifest changes, not normal validation. Release metadata rendering
runs that same operation offline inside its transaction; the native Testkit
supplier may retain an inactive registry Runtime after a minor owner bump.
Prepared graph checks still require the sole enabled canister executor.
`--build-only` compiles both
protocol selections without starting a server. Runtime qualification remains an
explicit target outside local full CI. Native macOS and live IC are separate
qualification from Linux PocketIC or Wasm compilation.

Owner and Timers composition each retain all four raw/prepared-call ×
Wire/external-Candid profiles on the declared compiler. Driver tests, exact Wasm
artifacts, graph isolation and lifecycle tests remain separate obligations.
No measured Wasm-size reduction is claimed.

## Candid and CBOR source admission and experiment retention

`make check-boundary` includes the lightweight Shared Candid source checker with
`--cbor` and Platform policy refusal cases. `ci/candid-boundary.json` admits exact
named external declarations and file/crate CBOR uses, each with reason and review
condition. Coverage includes crates, scripts and native provider/overlay Rust
sources. Platform's typed codecs, Types' existing serializer tests and one
protected imported native ingress fixture retain explicit allowances. There is
no directory or module exemption. New direct uses and stale allowances fail.
The checker is lexical, not macro expansion/type resolution. Upstream generators
remain separately covered by their source and external schema contracts.

Shared's Cargo inventory additionally rejects direct generic CBOR dependencies
outside Platform, except Types' development-only serializer tests. Independent
consumer, runtime, fuzz, benchmark, stable-identity and native HTTP workspaces run
that check on their materialized manifests as well as the root workspace.
This is direct-dependency admission, not a claim that their transitive graphs are
CBOR-free. The separately pinned native IC workspace remains external
infrastructure with paired native qualification; root checks do not qualify it.

Keep one compact conclusion per experimental topic, with at most two small data
files and 100 KiB combined; the repository budget is 5 MiB. New manual benchmark
output defaults to `target/evidence/`. Historical tables/probes retired during
cleanup remain in Git history; a summary or hash does not prove missing raw
artifacts are available. Active release/recovery custody and maintained test
vectors keep their separate contracts. See the Shared
[retention rule](../rules/engineering-details.md#changelogs-and-artifact-preservation).
