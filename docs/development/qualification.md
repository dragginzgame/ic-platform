# Platform development and qualification

[Workspace and feature selection](../../README.md) · [Migration](../architecture/migration.md) · [Current handoff](../status/current.md)

Run the commands below from the repository root. Select checks by the changed
contract; aggregate and installed-runtime gates retain their explicit scope.

## Package placement

All five maintained product packages live under `crates/` and belong to the
root virtual workspace and its `Cargo.lock`. Platform, Types and Runtime retain
their three release groups; macros and the singleton guard belong to Runtime.
Use `cargo metadata --no-deps --locked --offline --format-version 1` to discover
members, rather than assuming a directory depth.

The independent [qualification workspace](../../qualification/README.md) holds
the matched published/current BLS Wasm and installed probe. The adopted wrapper
and its comparisons live in the facade package. The qualification workspace has
its own catalog and lockfile, is excluded from the root workspace, and belongs
to no product release group. Product packages never depend on it.

The manifests in `scripts/fixtures/` are templates for disposable projects
rendered under `target/`, including the Runtime canister/driver workspace. Their
source templates and selected Runtime fixture lock remain release inputs; their
generated directories are not product package locations. These follow
the generated-project exclusions in the
[workspace placement rules](https://github.com/dragginzgame/toko-tooling/blob/5806e6e4943f6be600cd53385498ce08a5f804c4/rules/rust-workspaces.md); historical release
and recovery evidence stays in its recorded custody.

The instrumented Profiler source provider and its IC submodule are retired from
this checkout. Historical consumer Git pins retain that source; no native
provider workspace or Python assembler remains maintained here.

Additional separate maintained qualification packages belong in that workspace;
map their required checks explicitly. Ordinary crate-local tests remain with
their product. Existing generated fixtures retain their template ownership.

## Runtime qualification and historical evidence

`bash scripts/ci/check-candid-declarations-consumer.sh` qualifies a standalone
native/Wasm consumer of the single-declaration record/variant macros. It copies
the maintained public integration case, selects only `candid-codec`, runs strict
Clippy on both targets and the native test, and refuses upstream Candid, Agent,
runtime and crypto in its normal/build graphs. The owning `check-consumer` gate
includes this leaf; ordinary development can run it directly without the broader
runtime/fleet matrix. Product mapping/oracle tests and compile-fail doctests retain
byte/schema and invalid-label evidence.

`make check-raw-rand` is the focused installed management-entropy proof; it needs
an explicitly prepared `POCKET_IC_BIN` or `IC_TESTKIT_POCKET_IC_URL`. It checks
native/Wasm normal/build graphs without general Candid, compares matched general
and narrow codec consumers with opt-level 3 / fat LTO / one codegen unit and
`ic-wasm shrink`, and calls management `raw_rand` through the existing Runtime.
It checks producer bytes against native reference Candid, structured rejection,
callback-trap cleanup, pending-call cancellation and a subsequent successful call.
Both runtime and raw_rand qualification capture source bytes and the complete
input roster before composing fixtures, then refuse source or snapshot drift.
Raw_rand's consumer workspace stays beside the captured workspace; its two
matched Wasms remain separate from the broader runtime qualification profiles.
Runtime qualification flags may appear in any order; duplicate flags,
conflicting operations and published/prepared-call combinations refuse before
fixture effects. `bash scripts/ci/test-runtime-selection.sh` checks this selection
contract without resolving Cargo or starting a server.

Use `bash scripts/ci/check-platform-raw-rand.sh --check-graph` for graph admission
alone, or `--build-only` for graphs and size measurements without a server. The
owner runtime gate includes this leaf; no Timers or sibling fleet suite is added.

The optional `cdk` surface uses maintained, non-cryptographic runtime adapters
and the owned generator. It does not depend on the full `ic-cdk` package.
All controlled endpoints use explicit Wire hooks; select `candid-codec` separately
for required external management/system interfaces and existing checkpoint bytes.
The 0.4 transport cut is historical; current execution ownership belongs to the
independently released Runtime group.
See [the migration contract](../architecture/migration.md) for ownership and adoption conditions.
The public GitHub repository exists; registry availability requires separate verification.

Installed-runtime qualification runs separately in `.github/workflows/runtime.yml`,
using Testkit's explicit server setup/admission and both compiler/protocol modes.
For a prepared registry cache and admitted server, run
`bash scripts/ci/check-platform-runtime.sh --published X.Y.Z`, selecting an actual
published Runtime version, to exercise its registry cohort without a local
Runtime override. Append `--check-graph`
for graph-only admission. Local CI cache preparation uses `--prepare`; ordinary
qualification never installs a server. Native macOS and live IC acceptance remain
separate from this Linux workflow.

Runtime qualification captures its input roster and bytes beneath the retained
fixture's `inputs/` directory before reading selections or rendering/copying
fixtures. The snapshot and current source roster/bytes are checked at completion;
only the selected maintained lock may change during explicit `--refresh-lock`.
Dependencies keep their canonical owner paths, so stop source writers during
qualification. Shared-cache builds retain executables/Wasms under an inherited
artifact lock before releasing the cache to another cooperative build.

Historical measurements and reproduction instructions are in
[the benchmark report](../benchmarks/0.4/README.md).
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
profiles are binary Principal, self-authenticating Principal, Runtime, signature verification
and strict certificate CBOR, using one `3-fat-1` recipe. `WASM_BENCH_PROFILES`
selects a smaller affected subset; opt-in `candid-args` covers the small unit/text
encoders. Both arms use the same current probe source, compiler, recipe and
current dependency lock. This strict mode refuses changed resolved external locks
or per-profile dependency features; it does not silently attribute supplier changes
to Platform implementation changes. All nested probe sources and recorded compiler,
target/backend and optimizer settings must match in either mode.

For a deployment comparison, append `--allow-dependency-changes` after the byte
allowance. Each arm then starts with its own revision's lockfile; Cargo resolves
the disposable probe offline without modifying either source lock. The comparison
records complete changed external lock entries and the reachable normal/build
external package identities/features for each probe in `after/dependency-changes.json`.
Those graphs describe resolved build inputs, not proof that every package survives
linking. Changed dependencies are part of the measured deployment, so these deltas
must not be described as implementation-only savings. Missing graph evidence or
different probe/build settings still refuse; the size threshold remains enforced.

The separate asynchronous Wasm size workflow uses this explicit deployment mode on relevant
PRs and main-branch pushes and retains total uncompressed and code-section
measurements before/after optimisation, absolute before/after values and deltas,
plus dependency reports. Its default allowance is 128 bytes per
measurement; dispatch can choose an explicit allowance. This is a regression trigger, not a consumer Wasm
budget, instruction benchmark or permission to weaken verification.
Pushes compare against the event's preceding main commit; PRs use their base
commit, and manual dispatch retains its explicitly selected baseline.
The [0.30 deployment comparison](../benchmarks/platform-030-deployment.md) records
the first dependency-aware run and its self-authentication size regression.

`node --test scripts/benchmarks/tests/*.test.mjs` checks comparison
admission and the common Wasm section reader. The shared ingress corpus at
`crates/ic-platform/tests/fixtures/ingress.tsv` is exercised by Rust unit tests
with `ic-protocol` and browser `ingress.test.mjs`. It deliberately records the
browser's extension-field acceptance and Rust's upstream text-to-byte coercion;
neither inspection API authenticates requests or replaces original signed bytes.
The adjacent `protocol.tsv` response corpus is checked with `ic-protocol-fixtures`
and the same browser tests. It preserves SDK self-described CBOR and Rust's
untagged framing separately, with identical opaque reply/certificate payloads.

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

Use `make cloc` for classified workspace totals, `make cloc CLOC_DEPTH=0` for
crate rows, or `make cloc CLOC_DEPTH=1` to include top-level modules. The script
equivalents use `-d 0` or `-d 1`; `bash vendor/toko-tooling/scripts/dev/cloc.sh --json` retains schema 2. It
separates library/application, tests, examples, benchmarks, build/tooling and
unclassified source, including inline test blocks. The report counts each source
path once and reports macro expansion limits; it is not a linked-Wasm metric.
Owned test files use `tests.rs`, `*_tests.rs` or `tests/`; the main workspace
has no singular `test/` or `test.rs` paths. Generated qualification and benchmark
packages declare `[package.metadata.toko-tooling.loc]` with the appropriate
`category`, even when their library target carries a consumer proof. Use
`--manifest /absolute/path/to/Cargo.toml` to classify an independent generated
workspace explicitly; the default report covers the five main members only. Select one member with
`bash vendor/toko-tooling/scripts/dev/cloc.sh -d 1 ic-platform-runtime`, or one branch with
`bash vendor/toko-tooling/scripts/dev/cloc.sh -d 1 ic-platform-runtime::executor`. Depth is relative
to that selection; unknown crate/module names refuse.
Terminal workspace names are green, crates blue and modules white. Redirected
output, `TERM=dumb` or a nonempty `NO_COLOR` stays plain. Sibling scan progress
updates one stderr line and clears it before the table; diagnostics are retained.

New Platform-owned qualification scratch defaults beneath `target/`, while
Shared snapshot helpers retain their supplier-owned scratch contracts.
`cargo clean` reclaims the whole default workspace target, including diagnostic
captures and the relocated old Runtime driver executables. The separately
approved historical archive under `recovery-evidence/` survives cleanup.
`make check-retained-evidence` enforces a 320 MiB aggregate, 256 MiB single-file
and 30-day bulk age limit through Shared’s read-only checker before release
preflight/publication. Its owner and dated review are in [AGENTS](../../AGENTS.md#recovery-evidence-custody).
The release runner separately enforces Shared’s Git-side defaults. Both refuse
without automatic cleanup; no archive, Git refs or recovery journals were deleted.


Platform 0.11 removed `wire-schema`. Principal and Account retain
their existing Serde representations; Wire owns the schema integration. See the
[migration contract](../architecture/migration.md#owned-principal-and-account-wire-metadata)
for Wire 0.10’s `platform-types` integration and consumer requirements.

Browser SDK development uses Node 24.21.0 and npm 12.2.0. Run
`make install-browser-tools` (or `make install-tools`) to prepare its pinned
TypeScript compilers and SDK fixtures. `make check-browser` checks strict
TypeScript, adapters and packed consumers. Node/npm setup identities live in
`scripts/fixtures/browser-compiler-floor/package.json`; the browser package
selects its development compiler independently.

`bash scripts/ci/check-advisories.sh online https://github.com/RustSec/advisory-db.git target/platform-advisory-attempt`
audits the workspace and owner-runtime locks independently without compiling feature matrices. Prepare
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
logs and raw audit statuses are retained. Separate `*.gate-status` files retain
the CI decision. Separate `*.index-status` files report
complete/incomplete coverage, or unavailable after preparation failure; index
failures reject even if cargo-audit exits zero.
Offline results do not establish advisory or registry freshness. Vulnerabilities
fail the check except the maintainer-approved workspace finding #50:
`RUSTSEC-2026-0387`. Its complete unfiltered log and raw failure remain retained;
cargo-audit rechecks the same captured input with only that advisory excluded
to determine the gate result. The exclusion never applies to the runtime lock.
Additional advisories, tool failures and incomplete index coverage still fail.
Revisit this CI disposition at the next Agent dependency review; it does not
remediate #50 or establish safety. Informational maintenance warnings stay visible.
Yanked versions retain cargo-audit's existing warning policy; missing
index coverage is a failing observation. No maintained lockfile or shared advisory
cache is updated. A separate PR/main CI job
runs the online check and retains its revision and findings in job logs.

Platform 0.18 retires the unused stable-structures feature, module and
upstream dependency; see the
[contract](../architecture/migration.md#stable-structures-substrate).

Prepare common tools explicitly with `make install-tools`; `make tools-check`
verifies them offline. Prepare locked dependency sources with
`cargo fetch --locked` before validation, including the Wasm target dependencies
used by frozen standalone consumer checks. Then run
`make prepare-consumer-dependencies` to fetch the registry executor, legacy
Platform owner and Wire used
only by standalone fixtures; the embedded runtime and schema transfer removed
them from the root lock. CI and admitted release preparation run this setup too.
Normal consumer checks remain offline and never install executables. Rust 1.100.0
is the minimum, development and release compiler. Prepare that exact toolchain
with `wasm32-unknown-unknown`; `make check-compiler` admits and records the actual
Cargo/rustc identities and requires the workspace floor and toolchain pin to agree.
The 1.100.0 support change is tracked in
[#157](https://github.com/dragginzgame/ic-platform/issues/157); frozen historical
reports retain their original compiler identities.

Product and qualification members inherit their owning workspace's common
Rust, Clippy and Rustdoc lints. Generated consumers receive that same policy and
compiler floor through `platform_fixture_policy`; Cargo does not inherit lints
from a dependency. `make check-primitives` includes actual native/Wasm admission
and disposable must-use, unsafe, unsafe-operation, Clippy correctness and broken
documentation refusals. The policy test verifies real member inheritance and
idempotent projection before checking those refusals. Safe-only product libraries
forbid unsafe code; Runtime and explicit fixture ABI exports retain local,
reasoned expectations.

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
The Make leaf uses `CARGO_BUILD_WARNINGS=deny`, with fresh/cached warning refusal
qualified by `test-platform-lint-policy.sh`; explicit compiler denial remains
appropriate for plain JSON diagnostic collectors and Rustdoc.
The former upstream-scaffolding experiment runner is retired with native offline
public-key derivation. Historical packaging proposals remain documented in the
[migration contract](../architecture/migration.md#scaffolding-only-packaging-qualification);
they are not selected dependencies or current qualification commands.

Consumer profiles are declared in
[`platform-consumer-profiles.tsv`](../../scripts/ci/platform-consumer-profiles.tsv):
capability, test source basename,
comma-separated asset globs (`-` for none), and Wasm command (`check`, `build` or
`skip`), and graph family. `skip` selects native-only tests with a development
Platform edge; portable library probes keep normal dependencies. Required features come from the corresponding Cargo
test declaration. The runner preserves native and Wasm tests on the supported
compiler and admits runtime graphs first.

`make check-consumer` qualifies the profiles in
`scripts/ci/platform-consumer-profiles.tsv`: owned identity, executor/Runtime,
System API, binary external endpoints/DTOs, certification, native Agent and
selected cryptographic boundaries. This is an aggregate release check. For a
focused source consumer, pass its profile name to
`bash scripts/ci/check-platform-consumer.sh PROFILE`. The script derives its
features from the corresponding Cargo test declaration and prepares a separate
consumer with warnings denied. Native-only Agent skips Wasm; the remaining
profiles select their declared Wasm check or build. The IC0 native test compiles
without executing a system call.

There is no textual-argument parser, DID exporter, dynamic Candid facade or
service-registration consumer. Owned binary endpoint tests qualify explicit
limits, argument arity and wire bytes. Management/ledger tests preserve external
schemas, account semantics, arbitrary-precision amounts and callback signatures.
Certification and signature tests preserve typed refusals, root/witness checks
and malformed-proof rejection. Applications retain authentication policy.
The feature gate verifies runtime/identity isolation and rejects candid_parser
in every production selection; the dependency policy also rejects it in tests.
`make install-hooks` installs the shared formatting hook; `make fmt-check`
checks Rust and manifest formatting. Authorized releases use the shared
`make release-patch`, `make release-minor` or `make release-major` workflow.
Fresh releases require an explicit group or supplier-ordered cohort:

`make release-patch RELEASE_GROUP=platform` releases the facade.
The native client now releases in Tooling’s Testkit group.
`make release-patch RELEASE_GROUP=runtime` releases Runtime, macros and exact guard. `make release-patch RELEASE_GROUP=types` releases the
canonical type supplier. No group is inferred from changed files or PACKAGE.
All members use literal package versions; root Cargo stores common metadata and
dependencies. The [root changelog](../../CHANGELOG.md) indexes the three active group ledgers
and preserved coordinated history.

Fresh group releases execute in Shared's frozen candidate and leave the normal
checkout's branch/versions unchanged until explicit reconciliation. To publish
as part of that same batch, select `RELEASE_PUBLISH_TARGET=publish` on the
initial release command. Multiple affected groups use one
`RELEASE_GROUPS=types,runtime,platform` roster, omitting unaffected groups;
do not append a normal-tree `publish` goal. Recover with the printed batch ID,
using `release-batch-status`/`release-batch-resume`, then reconcile deliberately.
See [the exact commands and effects](release-validation.md#commands-and-effects).
`make deps-list` reports every product and qualification package's version and
release group without display-only dependency aliases. The JSON policy retains
every product member; its dependency rosters cover only actual catalog aliases.
Cargo declarations retain owned paths and full current version requirements.
Generated standalone fixtures read their contributor-only selections from
`workspace.metadata.platform-fixtures.dependencies`, alongside actual member
dependencies, through the shared local reader. Duplicate selections are refused.

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

Root `[workspace.dependencies]` pins `toko-tooling` once at an exact registry
version. Every real member inherits a development selection and exempts
the source bundle in its cargo-machete metadata; no Rust import or helper package
is needed. Vendored sources and intentionally isolated fixture manifests retain
their own scope. Tooling stays outside normal/build production graphs.
Cargo.lock records the selected archive checksum and
[the snapshot](../../.toko-tooling.snapshot) records installed payload hashes.
Shared executable helpers, Make fragments, composite actions and operational
inputs live under `vendor/toko-tooling/`; product callers remain outside that
managed tree. Root baseline, license and contribution files retain their paths.
The snapshot explicitly selects both release and publication entrypoints. Hooks
use repository-local `core.hooksPath=vendor/toko-tooling/.githooks`.
After changing the root pin, run `cargo update -p toko-tooling`, then
`make toko-tooling-update` (or `make toko-tooling-install`) to install that
selection and prepare/check its tools. These Make commands do not edit Cargo.lock.
Status and check commands compare all member declarations, Cargo.lock and the
installation offline. Custom publication, release metadata and frozen setup
adapters admit tooling before effects. All product release gates and native-host
qualification requirements remain in force.

`ic-platform` owns the low-level boundary; its `call` module adds prepared one-shot
requests with owned or borrowed methods and bytes through its existing runtime.
The module uses the existing `runtime` feature and adds no dependencies or codecs.
See [the call contract](../../crates/ic-platform-runtime/src/call/README.md). `ic-platform-macros` is the production
proc-macro support package. Its Syn/Quote dependencies generate consumer
entrypoints. Identity, the owned Candid codec, Agent, certification, raw IC calls,
management/ledger DTOs, signature/BLS APIs, the executor and Serde have explicit
production feature selections. Upstream Candid is a development oracle or native
Agent transitive, never an owned codec dependency. Testkit is
test infrastructure used by the independent fixture. Its host driver selects
Platform's `certification-cbor` decoder instead of a direct generic CBOR codec.
Agent dependencies are restricted to non-Wasm targets.
The root dependency catalog contains actual product member requirements, including
their development dependencies. Fixture-only inputs live under
`workspace.metadata.platform-fixtures.dependencies`, read through the existing
fixture catalog helper, and stay outside product dependency declarations.
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
features without host leaves. The separate standalone runtime qualification and
release runtime-lock checks are unchanged. It enforces empty defaults
and exact qualified executor/system requirements. It rejects unreviewed feature
names until the matrix and isolation policy are updated, admits the owned
generator only with `candid-codec-endpoint`, rejects full CDK packages, and rejects additional executor/
system versions or non-registry sources in the transitive graph. It also
rejects the retired PocketIC feature and the `ic-testkit`/`ic-testkit-candid`
canister dependencies, and checks Candid/CDK-free defaults and Candid-free
executor-only graphs. Cargo reports canonical package names, so dependency
aliases do not bypass these exclusions. `check-boundary` independently checks
direct dependency ownership; role comments themselves are not executable policy.
`make check-features-exhaustive FEATURES=runtime,candid-codec,executor` audits every subset
of 1–12 explicitly selected features and actual defaults using the same admission
rules. Missing, unknown and oversized scopes are refused before graph queries;
the full feature power set is deliberately outside routine qualification. This manual
audit is outside routine CI/release validation. Routine coverage deliberately
omits other higher-order combinations. Wasm cache keys omit native-only leaves
only after their exact feature/dependency/target contracts pass validation.
Changed contracts fail closed. Defaults remain a separate observation. One batch
validator per target checks every original selection against its graph; cached
graphs are not copied per selection. Each `target/feature-graphs/attempt.*`
directory retains target-specific TSV mappings and the actual Cargo graphs.
This reduces cache-lock acquisitions; it does not eliminate contention with
other Cargo processes or claim a measured wall-clock speedup.
Certification-only builds are Candid/CDK-free. Native Agent and IC transport
select upstream Candid transitively; their features and dependencies are native
only. Management, ledger and other owned DTOs use the binary codec and canonical
Types Principal without production upstream Candid. Registry management and
signature composition packages are development-only independent oracles.
Canister-signature protocol composition uses external BLS, certified-tree and
SHA primitives, with no Candid, IC0, parser or executor in its production graph.
Consumers retain trust roots, authority, freshness and decoding budgets.

Runtime owns the executor and delegates System API calls to registry IC0 1.2.0.
Its exact guard prevents multiple Runtime executor owners. The facade forwards
execution to that owner; no textual Candid feature or native registration/export
API remains. Use `candid_codec::decoder` only for required binary external
interfaces, with explicit caller-selected limits and argument arity. See the
[package contract](../../crates/ic-platform/README.md) and
[migration contract](../architecture/migration.md).

Consuming applications own final artifact admission: qualify their exact
feature/target/lock selection and bind source and artifact digests to deployed
bytes. Platform's source graph results do not establish consumer deployment or
native macOS acceptance. Release gates and archive publication checks remain
separate from ordinary focused development validation.

### Maintained runtime regression

`make check-runtime` qualifies both raw calls and owned `ic_platform::call` requests in one
fixture, sharing the host-driver and compiler builds. It installs Wire canisters
through Testkit-managed
PocketIC on Rust 1.100.0. The pure `cdk` selection rejects Candid, parser,
full CDK and native host clients in its canister graph. A separate
`external-candid` selection tests an outbound management `raw_rand` call with
explicit Candid arguments and preserves the original persisted checkpoint bytes.
All RPCs and installation/upgrade arguments use Wire in both selections.

The fixture checks raw awaits, composite queries, guard/malformed-frame refusal,
all four explicit request allowances, response-encoding traps, callback cleanup,
weak/protected/migratory spawning, submission-pressure and insufficient-cycle
errors, direct executor cancellation, upgrade, cycle/refund counters, stable-byte
rollback and certificate lookup. It installs no alternative executor.

Run `make check-runtime-lock` for coordinated lock and graph checks, or
`bash scripts/ci/check-platform-runtime.sh --check-graph` without
compiling/starting a server. Independent ledger composition tests protect frozen
original outputs; its retired source fingerprints are no longer selected. Call transport
and executor behavior is covered by unit/public tests and installed callback-trap
qualification, including multiple outstanding calls cancelled by one recovery.
The sole executor and IC0 implementation live in `ic-platform-runtime`; the facade
delegates to that owner. The owner fixture
has no Timers dependency. `make check-runtime-all` runs standalone qualification,
including raw/prepared calls, both codecs, traps, cancellation and upgrades.
Scheduling owns Timers dispatch/rearm and shared-executor composition tests.
Platform release admission never selects a downstream scheduler. Singleton
refusal remains covered by the independent executor guard fixtures.

Prepare/admit PocketIC using Testkit, then set `POCKET_IC_BIN` or
`IC_TESTKIT_POCKET_IC_URL`. Owned native fixtures select published Testkit 0.102.0
and its re-export of upstream PocketIC's async client. Each executable owns one current-thread Tokio runtime
for the complete scenario, awaits network/lifecycle operations and explicitly
awaits instance teardown before releasing its managed server. Client requests
select upstream's 60-second request-polling limit, separate from the 60-second
startup deadline. This is not an absolute HTTP/body deadline. The benchmark and Agent fixtures use the same catalog selections; Tokio
is a native fixture dependency and never enters the canister graph.
The harness retains Testkit's bounded startup and upstream best-effort instance
deletion. Transport panics propagate; returned canister rejections retain upstream
identity. It never installs a server implicitly. Artifacts, sources and metadata
remain in `target/cdk-runtime.*`. Normal runs use the maintained frozen selection
in `scripts/fixtures/cdk-runtime/Cargo.lock`. Explicit `--refresh-lock` is a
reviewed dependency refresh that preserves existing foreign pins and resolves
necessary manifest changes, not normal validation. Release metadata rendering
runs that same operation offline inside its transaction; the native Testkit
supplier uses registry Platform 0.32.x only for management codecs, without Runtime
or crypto features. Identity admission records that infrastructure separately from
the selected local Platform implementation and checks PocketIC/Agent Principal
identity at their crossing. PocketIC 16.1.0 pins `thiserror = "=2.0.18"`, so native
qualification retains that version while the product workspace selects 2.0.21.
The independent
BLS benchmark lock also belongs to the release metadata transaction: local product
identities advance while its exact registry comparison baseline remains fixed.
Prepared graph checks still require the sole enabled canister executor.
`--build-only` compiles both
protocol selections without starting a server. Runtime qualification remains an
explicit target outside local full CI. Native macOS and live IC are separate
qualification from Linux PocketIC or Wasm compilation.

Standalone qualification retains all four raw/prepared-call ×
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
[retention rule](https://github.com/dragginzgame/toko-tooling/blob/5806e6e4943f6be600cd53385498ce08a5f804c4/rules/engineering-details.md#changelogs-and-artifact-preservation).
