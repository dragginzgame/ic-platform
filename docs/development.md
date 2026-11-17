# Development

The current recorded published product versions are Types 0.21.2, Platform 0.34.15,
Protocol 0.3.5, and Runtime/macros/guard 0.23.17. Qualification packages remain
private and outside release groups. Publication covers implemented scopes;
production migrations and broader design qualification remain separate.
See the [latest release evidence](releases.md#runtime-test-layout-release).

Qualification selects Testkit 0.109.0 and explicitly names two Platform
roles: `candidate-platform` tests local source, while `published-platform`
checks the registry verifier. The qualification manifest and lock select their
versions; equal version numbers do not merge their Cargo source identities. Legacy decoder
oracles have been removed; qualification uses independent schemas and literal
fixtures. The separately pinned Auth fixture
uses its recorded Testkit cohort with the local Platform authentication and
construction APIs; all five installed cases pass without legacy Platform.

`make qualify-platform-crypto` checks the local Platform primitive
candidate and executes independent native/Wasm signature vectors.
`make qualify-auth-crypto` checks pinned actual Auth Ed25519/root-delegation code;
`make qualify-auth-wire` checks its generator with published Wire 0.24.2 and
Types 0.21.1. Both archive `AUTH_SOURCE` (default `../ic-auth`) without modifying
it. The Wire check requires that consumer's installed TypeScript tool and fetches
a pinned historical oracle when absent. `make qualify-query-settings` freezes
and tests current `QUERY_SOURCE` (default `../ic-query`) read-only, recording dirty
source separately from HEAD. These consumer commands are focused qualification,
not sibling adoption or publication. See [crypto evidence](qualification/auth-crypto.md)
and [Wire evidence](qualification/auth-wire.md).

Prefer existing names for retained concepts. Check crates.io when introducing
each package: select its next SemVer version if it already exists, otherwise
0.1.0. Breaking pre-1.0 APIs require the next minor, while compatible changes use
the next patch. A published dependency such as Types retains its supplier's
version until a local replacement is actually introduced.

GitHub source delivery is a version boundary for every product group, including
Protocol and Types. When crate contents change after a delivered snapshot,
advance the version before the next delivery, even with `publish = false`.
Compatible changes advance the patch; breaking pre-1.0 contracts advance the
minor. Unchanged groups retain their versions. Runtime's macros and guard
advance together with Runtime. Keep both workspace locks and group notes aligned.

## Canister upgrades

The supported Runtime upgrade workflow is **stop → wait for Stopped → upgrade →
start**. `Stopping` means the IC is still draining call contexts; it is not
permission to install. The old handler must finish before its Wasm heap is
replaced. Runtime does not persist suspended futures, cancel remote effects,
or add a pre-upgrade guard. This follows the
[IC upgrade contract](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/).

Preview the operator workflow with an explicitly selected network, identity,
canister principal and already-built, uncompressed Wasm artifact:

```sh
node scripts/upgrade-canister.mjs \
  --network "$NETWORK" --identity "$IDENTITY" \
  --canister "$CANISTER_ID" --wasm "$WASM_FILE"
```

The preview prints the target, SHA-256 and stages without invoking icp. Add
`--execute` to carry out that exact workflow. Run from the consuming deployment
project if its network configuration is required. The operator supplies icp and
controller credentials; this repository has no deployment target or identity
configured. No live deployment was performed to validate the wrapper.

Optional `--argument-file "$HEX_FILE"` supplies raw upgrade argument bytes as
hexadecimal (for example `4449444c0000` for Candid's empty argument sequence).
Omitting it supplies zero raw bytes. The wrapper does not infer argument schemas
or add textual DID tooling. Use the bytes required by the consuming canister.
`--timeout-seconds 300` bounds each icp command; allowed values are 1–3600.

The tool checks status, requests stop if needed, prints periodic drain progress,
and explicitly requires `Stopped` before installing with `--mode upgrade`.
It refuses unrecognized status output and changed local artifacts. After install,
it verifies the reported module hash against the chosen Wasm and requires that
the canister is still stopped. Only then does it start and confirm `Running`.
An initially stopped target is upgraded and started by the same workflow.
An application-specific health check remains the operator's responsibility.

| Operator observation | Meaning and action |
| --- | --- |
| Waiting for stop / Stopping | Outstanding call contexts are draining. Do not install code yet. An unbounded call can delay this indefinitely. |
| Stop timeout or error | No install follows. The on-chain stop may still be progressing; inspect status and dependent services before deciding to continue or start the old canister. |
| Install or hash-verification failure | No automatic start follows. Inspect status/module hash and resolve the failure while the canister remains stopped. |
| Start failure | The upgrade may already have committed. Inspect status and module hash; do not blindly repeat installation. |
| Running confirmed | The lifecycle workflow completed; perform the application's health check. |

For inspection and deliberate recovery, use the same explicit identity/network:

```sh
icp canister status "$CANISTER_ID" --identity "$IDENTITY" --network "$NETWORK" --json
# Only after assessing the current module and state:
icp canister start "$CANISTER_ID" --identity "$IDENTITY" --network "$NETWORK"
```

A local timeout does not cancel an on-chain operation. The wrapper does not
silently retry install/start or force installation while stopping. Coordinate a
single deployment owner: a separate controller could otherwise start or change
the canister between checks. Keep the selected artifact immutable during use.

Use ICP CLI exclusively for canister operations. The wrapper uses its JSON status
output, requiring the selected principal and a full management status; public-only
fallback output fails closed. The command interface was checked against installed
ICP CLI 1.6.0 help and [its status implementation](https://github.com/dfinity/icp-cli/blob/v1.6.0/crates/icp-cli/src/commands/canister/status.rs).
Select a configured network name; URL/root-key selection is outside this wrapper.
`make check-upgrade-tooling` checks command sequencing and failure handling with
a fake executable; `make qualify-runtime-installed` separately exercises real
IC stop/drain/upgrade/start semantics through PocketIC. CI runs both.

## Setup and dependencies

Use Rust 1.100.0 with rustfmt, Clippy and `wasm32-unknown-unknown`, as selected in
`rust-toolchain.toml`. Toko Tooling 0.45.0 is selected exactly by Cargo and installed via
its authenticated exporter. Root company files and `vendor/toko-tooling/` are
managed; consumer commands live in the root Makefile.

```sh
make install-tools
make tools-check
make toko-tooling-check
make deps-list
```

Tool setup can download executables and populate Toko Tooling's per-user store. Checks
are offline and do not install tools. Linux x86-64 and macOS ARM64/x86-64 are the
intended development hosts. The initial focused suite passed on all three in
[hosted run 43717278830](https://github.com/dragginzgame/ic-platform/actions/runs/43717278830)
for source `08c72219ad6f261067c00eece9c8c71852e6b561`.

Setup also installs and admits ICP CLI 1.7.0 from Toko Tooling's pinned tool catalog.
`make qualify-native-signed` and `make qualify-release` check that tool before
running; Make selects this checkout's `.tools/ic/bin/icp` through `ICP_CLI`.
An ambient PATH executable or sibling checkout is not needed. Regular native CI
runs the signed/anonymous installed journey and the native/Wasm Blob codec
targets (`qualify-caffeine qualify-management-ledger`) on all three hosts.

Release qualification prepares a fresh isolated RustSec database online with
Toko Tooling's `prepare-rustsec-db.sh`, then audits both root and qualification lockfiles
using `--no-fetch`. Failure to acquire or audit the database refuses qualification;
there is no shared-cache fallback. The receipt binds the verified revision,
preparation log, exact reports and lockfile hashes. Cargo-audit can emit null Git
metadata with `--no-fetch`; the verified preparation sidecar supplies that identity.
Publication requires unchanged evidence and an observation no older than 24 hours.
Expired evidence requires fresh qualification. `make check-release-evidence`
checks the receipt refusal cases and local tool/CI wiring.

The initial [CI workflow](../.github/workflows/ci.yml) selects Ubuntu 24.04
x86-64, macOS 15 ARM64 and macOS 15 Intel. These are the intended native
qualification hosts, covered by the initial run above. Setup requires Bash,
GNU Make, Git, Perl/core modules, curl, tar and SHA-256 utilities, plus Rustup and
native compiler/linker prerequisites supplied by the hosted images. Toko Tooling installs
the selected jq/yq and other host tools before its toolchain-file preparer reads
`rust-toolchain.toml`; the complete tool setup/check follows. Node is pinned in
`.node-version` for standalone Wasm execution, with no npm packages required.

`deps-list` lists workspace packages, versions and release groups, not transitive
dependencies. A dash means no release group has been configured. Use Cargo's
`tree --locked` with the relevant target/features to inspect a dependency graph.

```sh
# Networked, deliberate lockfile update; optional package/version selection:
make deps-update DEPENDENCY_PACKAGE=toko-tooling DEPENDENCY_PRECISE=0.45.0
```

Without selectors, `deps-update` updates all dependencies within the declared
requirements. It covers the root and `qualification/Cargo.toml`, including the
`runtime-installed` member. Both listing and updates use that same selection.
For a deliberate Toko Tooling upgrade, review the new release, edit both exact workspace pins,
update the lock, then run `make toko-tooling-preview` and `make toko-tooling-update`.
Do not edit installed bytes or their receipt.

## CI graph and synthetic-key checks

The native `engine-view-consumer` fixture intentionally uses Candid to construct
independent reference schemas. Its explicit native-only graph allowance includes
that oracle's dependencies. Marketplace therefore has two bigint identities on
native (candidate plus oracle), but one on Wasm. Product and Wasm graph allowances
remain unchanged; all paths still require one canonical local Types/Protocol
identity. The graph-command regression checks all four CI target triples and
refuses Agent, Runtime and unknown additions to the oracle allowance.

The two secp256k1 PEM findings introduced in `e186c72` are public test material:
the scalar is exactly 32 bytes of 0x07. `.gitleaksignore` records only the two
exact historical fingerprints, alongside earlier Ed25519 fixture fingerprints.
No operational key or file/rule-wide exemption is introduced. Full-history
secret scanning remains enabled in both CI workflows.

Local repair validation passed full-history scanning, all graph checks for
Wasm/Linux/macOS ARM/macOS Intel target selections, 56 flow/wiring tests, actual
marketplace qualification, formatting and workflow lint. Logs are retained under
`target/ci-repair/`. macOS graph inspection from Linux is not native macOS
execution; fresh hosted results require delivery of these local changes.

## Focused implementation checks

`make qualify-runtime-call` checks inert requests and native executor ownership
regressions, Clippy, Rustdoc and Wasm compilation. It selects local Types;
IC0 execution is checked separately below. See the
[Runtime slice](qualification/runtime-call.md).

`make qualify-runtime-consumer check-runtime-consumer-graph` validates the
independent public local Types/Runtime crossing and executes its standalone Wasm
corpus. This checks request/observation APIs, not IC submission or an executor.

`make qualify-runtime-ic0` builds the opt-in wasm32 cost-observation boundary and
executes an independent import host checking exact lengths, initialized output
storage, u128 handling, overflow and consuming one-way submission/refusal. The [IC0 record](qualification/runtime-ic0-boundary.md)
documents the approved unsafe scope and outstanding installed qualification.
The same target executes the two-import raw-argument harness: size admission
before allocation/copy, exact bytes, explicit oversize errors and guarded
admission with caller-selected decoding. See the
[argument boundary](qualification/runtime-arguments.md).
It also runs the manual update adapter's context/guard/decode admission corpus
and explicit raw reply/duplicate-refusal checks;
see the [update boundary](qualification/runtime-update.md).

`make qualify-runtime-awaited` executes the experimental awaited transport's
first-poll, callback and cleanup corpus in a separate Wasm artifact, including
out-of-order task completion and multiple outstanding calls across cleanup. See the
[evidence and executor boundary](qualification/runtime-awaited.md). This host
does not simulate or prove replica rollback.

`make qualify-runtime-guard` tests guard version admission and five valid/two
conflicting Cargo graphs offline. On a fresh cache, first run
`node qualification/runtime-guard.mjs --prepare` after fetching both workspace
locks; this fetches the exact published 0.22.5 guard used as a conflict oracle.
CI performs this preparation explicitly. See the
[guard qualification boundary](qualification/runtime-guard.md).

`make qualify-runtime-input-ui` checks owned/static update inputs and explicit
decoder signatures using maintained compile-pass/fail Wasm fixtures. The driver
uses temporary manifests and checks structured compiler error codes; it does not
generate or replace Rust source.
Input and export fixtures share `qualification/runtime-ui.mjs`, selected with
`input` or `exports`. Rust error-code checks require a primary span in the
maintained fixture; export-name conflicts still reach real Wasm code generation.
`qualify-runtime-ic0` and `qualify-runtime-awaited` share `lint-runtime-ic0`, so
one combined Make invocation runs the common Clippy selection once.

`make qualify-runtime-exports` tests the macro crate, builds a dedicated Wasm
export consumer and invokes its actual IC exports in a host. It also builds
maintained invalid declarations to verify compiler/codegen refusals and checks
the one-Runtime/one-guard/one-macro/one-Types graph. Export and installed-rollback
fixtures use the approved generated export-attribute exception; older
non-export qualification targets retain crate-root unsafe forbids.

`make qualify-runtime-timer` is the focused installed timer command: it checks
the fixture with Clippy, builds its Wasm and exercises completion, initial traps,
callback cleanup, failed-cleanup uncertainty, disarming and stopped upgrades.
These cases also run under `qualify-runtime-installed`; do not select both for
one validation batch. The local candidate is Runtime/macros/guard 0.23.12, still
unpublished. See the [timer boundary](qualification/runtime-timers.md).

`make qualify-types` tests local Principal and Account/Subaccount identity, text and Serde contracts,
runs strict Clippy and Rustdoc, and compiles its tests for Wasm. It selects the
local manifest explicitly; Protocol and Runtime now select that same Types source.
Wasm compilation does not establish execution;
production consumer migration remains pending. See the
[Types qualification record](qualification/types-principal.md).

`make qualify-types-consumer` compares the independent consumer with published
Types natively and executes default/self-authentication Wasm artifacts. Use
`make check-types-graphs` to enforce their normal/build dependency closures.
The optional product feature is `principal-self-authenticating`; defaults stay
hash-free. See [consumer evidence](qualification/types-consumer.md).

```sh
make test-engine-request
make lint-engine-request
make doc-engine-request
make check-engine-request-wasm
make fmt-check
```

These select the Cloud Engine request encoder, not a full future Platform gate.
`make fmt` uses the installed formatting tools for Rust and Cargo manifests.
Run `make install-hooks` once per clone after preparing tools. This activates the
canonical Toko Tooling pre-commit hook through repository-local Git configuration; the
installer refuses to replace existing hooks. The hook and `make fmt-check` use
the same two-workspace formatting selection. Git configuration is local and
must be set up again in a fresh checkout. Local Linux activation and the Toko Tooling
consumer fixture passed automatic formatting/dependency sorting, partial-stage
refusal, formatter-failure isolation and preservation of unrelated working edits.
macOS hook behavior remains pending native evidence. Literal-frame unit tests stay with the product;
the separate public consumer proof lives under `qualification/`.

## Initial contract and implementation boundary

`cloud_engine::encode_get_engine_views_args(&[String], max_bytes)` encodes exactly
one Candid text-vector argument. It preserves empty/Unicode/NUL/whitespace text
and order, checks the complete frame size before output allocation, and reports
typed output-limit/allocation failures. Request-count limits belong to the caller.
The request-only production dependency graph is empty; the source-only Toko Tooling dev marker is not
compiled into the product. The default feature set is empty.

The wire contract comes from the selected external method documented in
[the requirements](requirements.md) and the Candid binary format. The old
Platform implementation was inspected at the design's pinned source; no source
or tests were copied. This is not a clean-room claim. The new bounded encoder
and literal tests are authored here. Request tests do not establish response
decoding, identity crossing, a general codec, or complete Query migration.
The response-specific implementation and evidence are described below.

## Selected response decoding

Enable `cloud-engine-response` for `EngineView`, `EngineSubnet` and
`decode_get_engine_views_response`. This feature selects local Types 0.21.0
and Serde 1.0.229. A private declarative macro emits Rust records, schema and
reader dispatch once; no response Encode implementation is required. Required
and strict-optional policies are explicit. A public/general declaration API and
Candid-compatible optional fallback are not implemented by this increment.

```sh
make test-engine-response
make lint-engine-response
make doc-engine-response
make qualify-engine-response
make measure-engine-response
```

The qualification command checks both tooling selections, runs the independent public
consumer and published-Platform comparisons natively, applies Clippy, and
compiles the portable consumer tests for Wasm. It does not
execute Wasm or prove installed-canister/macOS behavior. `qualification/` owns
its manifest, lockfile and target directory, has no release group and is excluded
from product membership. Product code has no reverse qualification dependency.
`make deps-list`, `make deps-update` and formatting cover both workspaces,
including the installed-runtime driver.

The selected decoder requires exactly one vector result. Unknown fields are
validated and skipped without retaining their values. It accepts the standard
constructed types for skipping, including transparent service/function references;
opaque references and future constructors refuse. Strict option fields require
wire `opt` when present and never use fallback to hide a malformed payload. An
absent field or an encoded None remains absent; no nested value is manufactured.
Compatibility is currently value-based, matching published Platform 0.33.7:
an empty `vec bool` can decode as an empty result, and a selected subnet declared
`opt bool` with None remains absent. The same optional type with a boolean
payload refuses with Type; no failed payload is converted to None. Literal
comparison tests preserve this distinction. Full declared-type compatibility
even without values is not implemented or claimed.

Wire grammar was reviewed against the [Candid binary format](https://github.com/dfinity/candid/blob/master/spec/Candid.md#binary-format)
on 2026-11-15 (document version 0.1.8; fetched Git blob
`6a95576e230252ce6b5bd0e2280455c799588257`). This is a specification source,
not a copied implementation or a differential oracle.

Limits are mandatory: input bytes, header end offset, type count, cumulative
work, logical allocation, per-vector count and cumulative skipped work. Every
byte read and value traversal costs work; skipped values consume both budgets.
Header references/type traversal and selected vector admission also cost work.
Allocation accounts for the retained type-table containers and projected vectors
and strings before reservation; borrowed service names and discarded text are
not copied. Units are implementation-specific, not a process heap bound or a
promise of numerical equivalence with old decoder work counters. Both simple
constructed type paths and runtime nesting retain the 16-level ceiling.

Acceptance uses independent literal frames, including malformed skipped UTF-8,
strict optional failure, exact/one-below budgets and large zero-sized vectors.
The qualification consumer additionally proves a direct public Principal type
crossing and the JSON Serde shape. Native-only tests use independently authored upstream Candid
schemas, literal expected values, fixed refusal classes and 189 retained mutation
admission expectations. No legacy decoder is selected. Work counters remain
implementation-specific. Local and registry Types select the versions in their workspace catalogs; source
identity remains distinct and comparisons cross that boundary only through bytes,
text or JSON. These focused checks do not establish full Query adoption.

Protocol 0.2.4 adds Registry envelope/record and placement codecs with independent
literal/schema checks. `make qualify-query-hard-cut` tests the actual migrated
Query consumer; `make qualify-native-read` exercises native signature and HTTP
refusal tests. See [contracts, evidence and remaining gates](qualification/query-hard-cut.md).

`measure-engine-response` reports the smallest admitted logical work, allocation
and skip allowances for four fixed workloads. See the [criteria and measured
results](qualification/engine-view.md). These are deterministic resource-counter
observations, not timing, instruction or allocator measurements. The static
projection schema is now validated by a compile-time assertion; incoming type
table validation remains metered on every decode.

`make profile-engine-response` runs the isolated native heap comparison in the
release profile, then Clippy for that qualification package. It uses the
qualification lock's dhat development dependency; prepare new locked sources
with `cargo fetch --locked --manifest-path qualification/Cargo.toml` when needed.
The five fixed workloads satisfy zero regression in total/peak requested bytes
and zero allocation growth for discarded text. See the [heap criteria and
results](qualification/engine-view-heap.md) for scope, exact inputs and limits.
Profiler dependencies remain outside the product and portable consumer graphs.
The same serial heap test also requires complete allocation cleanup on six
partially constructed response failures: truncation, trailing data, exhausted
work and exhausted logical allocation. It checks the expected typed refusal as
well as zero live blocks/bytes; it does not inject host allocator failure.

`make qualify-engine-response` also exercises the [literal JSON compatibility
corpus](qualification/engine-view-serde.md): numeric boundaries, nested presence,
canonical Principal text, malformed selected fields and duplicate-field refusal.
The literal tests compile for Wasm; the published comparison executes on native
only. Wasm compilation is not evidence of Wasm execution.

`make measure-engine-wasm` builds and executes separate old/new decoder probes
using Node and reports complete module and code-section bytes with hashes. It
reports a 1,693-byte code-size increase, with size acceptance **undecided**; see
the [recipe and evidence](qualification/engine-view-wasm.md). The initial
agent-selected zero-growth threshold had no established launch-budget basis.
Contribution analysis is deferred; this measurement is not release admission.
Its private package requires exactly one of `candidate` or `published`.

`make qualify-engine-wasm` separately builds and executes an admission corpus in
Node for both decoder selections, then runs Clippy. It checks signed timestamp
extremes, Principal bytes, every truncated prefix of those frames, trailing data,
arity and five resource limits. It uses a separate binary so the fixed size
workload is unchanged. See [executed Wasm evidence](qualification/engine-view-wasm.md#executed-admission-corpus--2026-11-15).

## Continuous integration

The workflow runs on main pushes, pull requests (including documentation-only
changes) and manual dispatch. A workflow/ref concurrency group cancels superseded
runs. Actions are pinned to reviewed release commit SHAs, permissions are limited
to reading repository contents, and checkout does not persist credentials. No
publication, release, deployment or write-token step is present.

Each native host prepares locked sources before running offline Make checks:
formatting/snapshot admission, request/response tests and lints/docs, portable
consumer qualification, heap profiling and executed Wasm admission. The fixed
size comparison remains an explicit informational target outside required CI;
its acceptance is undecided. No all-features build combines the two probe owners.

One independent Linux repository job owns the full-history Gitleaks scan and
Actionlint/ShellCheck validation. It is outside the native matrix and runs even
for docs changes. Reproduce with `make install-ci-tools`, then
`make lint-workflows check-secrets`. Installation uses Toko Tooling's pinned CI catalog;
checks install nothing. The selected catalog provides these CI tools on Linux
x86-64 only; native macOS jobs do not invoke them. Gitleaks scans committed Git
history, not uncommitted files or ignored build output.

Local Linux validation passed workflow lint, the existing committed-history
scan, tool admission, formatting and snapshot checks. The initial hosted run
linked above also passed repository checks and all three native jobs, including
tool preparation, dependency boundaries, decoder tests/docs/lints, public-consumer
and heap qualification, and standalone Wasm execution. This proves the focused
suite at that source, not release readiness, installed IC behavior or Query adoption.
The native assertions retain their separately recorded local evidence; a valid
workflow definition is not a successful hosted run.

## Dependency boundary checks

`make check-protocol-graphs` uses the installed Toko Tooling graph checker with locked,
offline Cargo observations on `wasm32-unknown-unknown`. Override `GRAPH_TARGET`
with an explicit target triple for native checks; CI also runs it with the host
reported by `rustc -vV` on every matrix runner.

Eleven selections are checked independently: Registry settings, default Protocol, request-only,
response, the public consumer with and without Registry settings, and the candidate Wasm
probe with and without Registry settings, plus marketplace Protocol and its public
consumer and marketplace Wasm probe. Default and request-only builds admit only the Protocol root. The
five EngineView/combined selections use the reviewed package-name allowlist in Makefile and require exactly
one rendered Types identity; both consumers also require one Protocol identity.
Dependencies outside that list, including Runtime, published Platform, registry
Candid, crypto suppliers and heap/oracle tooling, refuse with an introducing
dependency path. A new supplier requires an explicit policy review, not just
updating the lockfile.

The three marketplace selections additionally admit num-bigint, num-integer,
num-traits and build-only autocfg, and require one bigint supplier. This separate
allowlist preserves the bigint-free EngineView and Registry selections.

Development dependencies are deliberately excluded: published Platform remains
the native test oracle and dhat remains a separate profiler. This is not a
whole-lock duplicate ban. The shared checker distinguishes versions and source
qualifiers shown by Cargo; rendered identity counts do not authenticate source
bytes. Locked builds and the public Principal crossing still supply separate
evidence. This policy applies to the current Protocol slice, not future Platform
crypto or Runtime packages.

The initial five selections passed on Linux x86-64 and Wasm in local checks. An
intentionally restricted allowlist that excluded Types failed at the response
selection with a Types dependency path, confirming the target propagates policy
failures. Workflow lint, formatting and the 51-file Toko Tooling snapshot check passed.
The same checks passed on both macOS architectures in the hosted run linked above.

`make qualify-registry-settings` runs the selected protobuf projection's tests,
Clippy, Rustdoc and Wasm compilation, followed by independent public consumer
and Serde tests, consumer Clippy and consumer Wasm compilation. Its separate product graph selection permits only
Protocol and Serde's closure. The new selection passed locally; hosted evidence
above predates this addition. See [Registry qualification](qualification/registry-settings.md)
for exact scope and outstanding consumer gates.

`make qualify-registry-runtime` runs the Registry heap corpus in a separate test
process and executes its standalone Wasm admission binary. Fixture construction
is outside heap measurement. The Wasm runner's `registry` selection reports only
admission and artifact identity, without changing the EngineView size comparison
or its undecided acceptance threshold. Both checks are wired into every native
CI job; local Linux results do not establish hosted/macOS success.

`make qualify-marketplace` tests the opt-in price-row projection and its natural
and variant readers, checks Clippy/Rustdoc, compares public JSON and wire values
with the published native oracle, and compiles the consumer for Wasm. The
[marketplace evidence](qualification/marketplace-prices.md) records the new
supplier closure, logical natural-number charges and remaining gates.
The same target now covers available-page responses and request encoding,
including a separate native heap test for output admission before allocation.

`make qualify-marketplace-wasm` executes the standalone price/page/request and
fee/status corpus using the existing runner's `marketplace` selection. It reports
admission and artifact identity without a size threshold; see the
[executed Wasm evidence](qualification/marketplace-wasm.md).

## Installed Runtime qualification

`qualification/runtime-installed` is a native driver package in the standard
qualification workspace. The candidate pins `thiserror = "=2.0.18"` to align
with PocketIC. It consumes
the Wasm built from `qualification/runtime-rollback`, without linking candidate
Rust identities into the native adapter. Prepare its locked sources with
`cargo fetch --locked --manifest-path qualification/Cargo.toml`.

Run `make install-runtime-server` for explicit qualification-owned server provisioning,
`make runtime-server-check` for offline admission, and
`make qualify-runtime-installed` for the focused immediate/awaited update rollback probe.
The latter checks host/fixture Clippy, actual installed execution and the Wasm
dependency graph. See [the evidence and limits](qualification/runtime-installed.md).
Server setup supports the existing Linux and macOS CI hosts; only local Linux
execution is established at this point.

## Release-group ownership

The [group policy](../ci/cargo-release-groups.json) records implemented product
groups. Each primary package's literal `package.version` owns that group's
version; there is no workspace version or shadow version file. The root
[changelog](../CHANGELOG.md) is an index, and group notes live under
`releases/<group>/CHANGELOG.md`, following the adopted Toko Tooling convention.

Product versions are owned by their manifests; the current published cohort is
recorded at the top of this document. Runtime, macros and guard remain coordinated.
Private qualification packages stay outside product release groups.
`make deps-list` uses the installed Toko Tooling reader for membership.

### Qualification and publication flow — Platform #26

The execution change for [Platform #26](https://github.com/dragginzgame/ic-platform/issues/26)
preserves the full obligation catalogs and every existing target's package,
feature, target and pinned consumer selection. It consolidates dispatch, rather
than inferring that an unchanged product has no reverse-consumer obligations.
No private package or product version is introduced for this maintenance.

| Phase / entrypoint | Owner and guarantee | Inputs and trigger |
| --- | --- | --- |
| Development: the focused `make qualify-…` targets below | Platform's selected product/consumer contracts; leaf commands explicitly select packages, features and native/Wasm targets | Changed source, manifests/locks, fixture source and tool pins; run affected contracts during development |
| Tooling wiring: `make check-release-evidence` | Local audit, collector, publication-admission and Make/CI integration refusals | Collector/admission scripts, audit policy, catalogs, Makefile, group policy and workflow wiring; affected tooling changes and repository CI; excluded from routine release qualification |
| Repository CI: `.github/workflows/ci.yml`, `types-release.yml` | Supported Linux/macOS coverage and product/fixture guarantees | Each workflow's committed source and host/tool selections; CI remains asynchronous and is not a blocking local publication matrix |
| Release: `make qualify-release` | Complete local source-bound catalog, fresh isolated RustSec audits of both locks | Clean exact commit, both catalogs, compiler/host, root and qualification locks and every fixture's pinned inputs; once per final source |
| Publication: `make publish RELEASE_GROUP=… PACKAGE=… RELEASE_RECEIPT=…` | Shared adapter owns selection/dispatch/custody; Platform admits one package's source, registry destination and extracted archive | Exact receipt, package owner/version, clean source, local/remote annotated tag, fresh registry absence, packaged dependency graph and VCS provenance; fresh for each package/upload attempt |
| Optional preflight: `make publish-check` with the same selections | Same admission plus Cargo dry run; no upload | Only when a separate preflight is useful; it is not a prerequisite of `publish` |
| Recovery / readback | Shared attempt evidence plus independent registry reconciliation | Inspect intent/dispatch/status and remote index/archive/VCS evidence after uncertain upload; never automatically rerun an upload |

`ci/release-profiles.json` supplies repository, Protocol, Runtime and Platform
goals; `ci/types-release-profiles.json` supplies Types goals and their scopes.
The collector first invokes the installed shared Make execution guard, refusing
dry-run and ignored-error modes. It expands both catalogs once, retains
first-occurrence order, and invokes one
serial Make graph. Make deduplicates common targets and prerequisites. In
particular Blob supplier qualification, Types/Runtime crossing and graph checks
now run once even when multiple owners require them. `test-native` is the common
native unit prerequisite of both read and signed qualification; both entrypoints
still run it when invoked independently. No tests are replaced with cache hits. The obsolete Types-only admission runner
is retired; its entrypoint delegates to the general admission and its refusal
target depends on the same local flow fixtures for affected tooling work and CI. The former assertion that only
Types may be publishable is replaced by exact package/group admission.

Run `make check-release-evidence` when changing collector/admission scripts,
audit policy, release catalogs, Make wiring, group policy or workflow integration.
Both CI workflows retain that coverage (Types through `check-types-publication`).
Neither release catalog selects those regression fixtures: `qualify-release`
still performs fresh real audits of both locks and validates their evidence;
`publish` still performs fresh receipt/source/group/tag/registry/archive admission.
Changing either catalog invalidates existing receipts. No audit or admission
operation has been removed or replaced by CI success.

The complete product gate is deliberately retained for every release group.
The current catalogs do not declare complete file/feature/reverse-consumer input
closures: actual Auth, Blob and Query fixtures cross owners. A selected publisher
group is therefore not evidence that another catalog's checks are inapplicable.
There is no new local diff classifier, cache, runner or metadata framework.
Narrowing that gate would require a qualified input/impact policy using shared
mechanics. This is an explicit retained-work disposition, not a claim of scoped
product qualification. The flow was qualified on Toko Tooling 0.44.11; its subsequent 0.45.0 adoption
is recorded below.

Schema-2 receipts bind the complete ordered target list and one retained
`qualification.log`, both catalog hashes, exact commit, compiler/host and the
existing fresh audit evidence. The collector writes a receipt only after success
and final clean-source admission. `timings.json` records audit, qualification and
total elapsed time; timings are observations, never authority. Old schema-1
receipts remain historical evidence but cannot admit publication under the new
flow. Changed source/catalog/compiler/host, missing/tampered logs or invalid audit
evidence require a fresh full run; there is no partial-success resume.

Workspace and extracted-archive checks remain distinct: Cargo normalizes package
manifests and resolves registry dependencies. Publication retains all-feature
native/Wasm archive Clippy, native tests and independently fetched source
provenance. Cargo's own package/publish compilation is retained as its integrity
check; no `--no-verify` shortcut is introduced. If an operator elects to run both
preflight and upload, admission repeats because no shared archive-result reuse
contract is adopted here. Normal delivery runs `publish` once, not a mandatory
`publish-check` followed by `publish`.

For a multi-package Runtime delivery, use one final-source receipt and publish
its guard and macros before Runtime. Other product groups retain their own tags
and versions. Keep one package per attempt and independently verify each published
archive against its admitted bytes and tagged VCS source. A failed dispatch
marker without completion is uncertain: preserve the lock/evidence, inspect the
registry and reconcile before selecting any fresh attempt.

### Platform #26 local qualification evidence

On 17 November 2026 the before/after real product catalogs both passed on Linux
with the pending Platform 0.34.15 source. The harness used pinned disposable
`QUERY_SOURCE=target/deploy-query-source` and `BLOB_SOURCE=target/blob-types-source`;
no sibling checkout changed. The grouped baseline used the old dispatch shape;
the final graph used shared targets and the replacement Types refusal entrypoint.

| Qualification phase | Observed elapsed seconds |
| --- | ---: |
| Before: Types | 22.64 |
| Before: repository | 8.25 |
| Before: Protocol | 74.75 |
| Before: Runtime | 50.74 |
| Before: Platform | 86.85 |
| Before: total grouped qualification | 243.24 |
| After: complete single Make graph | 197.68 |

These are sequential warm-cache observations, not a controlled cold-cache
benchmark or an end-to-end publication speed claim. Later cache warming,
changed refusal-fixture coverage and system variation prevent attributing the
whole difference to dispatch consolidation. Fresh audit preparation, tagging,
archive admission, upload and registry readback were outside this comparison.
No live upload was needed. The real after-log confirms one Blob supplier run,
one native unit run and one flow/refusal invocation. Timings and full logs:
`target/platform26-comparison/timings.json` and its sibling log files.

All 54 focused flow/audit/wiring tests pass, including the two final shared
execution-guard cases added after the full graph's 52-case fixture invocation.
The retained `make check-types-publication` entrypoint passes all 54 as well.
Tests execute real Make and consumer admission processes in disposable roots;
registry, package and provenance effects are simulated. They establish dispatch,
refusal and fresh-run recovery behavior, not live upload or macOS proof.
Evidence: `target/platform26-flow-final4.log`,
`target/platform26-types-entrypoint.log`, and `target/platform26-format.log`.
The comparison ran against a dirty development snapshot and deliberately issued
no release receipt. Final clean-source qualification and archive admission remain
required before deployment. Historical release receipts were not modified.

## Platform certificate boundary

`make qualify-platform-certificate` checks empty defaults, independent literal
certificate/refusal tests, compile-fail immutable-access checks, native/Wasm
Clippy, Rustdoc and normal/build dependency boundaries. The opt-in
`certificate-decode` feature currently supplies decoding only; see the
[scope and provenance](qualification/platform-certificate.md).

`make qualify-platform-authentication` includes those checks and exercises the
opt-in direct/delegated verifier, its immutable evidence, refusal cases and selected
BLS dependency graph. Callers supply a prepared trusted root, effective canister,
separate inclusive signing/delegation windows, combined traversal bounds, embedded
certificate byte allowance and selected range count. Exactly one delegation is
supported, with complete legacy or selected sharded range admission.
Wasm checks compile the tests; they do not prove installed execution or completion
of the full authentication gate.

`make qualify-platform-observed` runs the native public-API consumer against
fresh direct/delegated certificates from installed PocketIC canisters. Prepare
its verified server with `make install-runtime-server`. It writes observations
under `target/platform-certificates/`; frozen reviewed frames are replayed by
`qualify-platform-authentication` without the server. This exercises the native
verifier against replica output, not verifier execution in Wasm or mainnet adoption.

For selected-subnet observations, `authenticate_subnet_time` uses the same feature
and qualification targets. Supply the requested subnet, explicit signing/delegation
windows, and an optional trusted root-subnet identity. Direct-root observations
refuse without that independent trust binding. Query owns network admission,
response retrieval, sampling and presentation; see
[issue #1 acceptance](qualification/platform-certificate.md#selected-subnet-signing-time--issue-1).

Agent callers that already received an upstream certificate use
`authenticate_subnet_time_from_certificate(&certificate, ...)`. It rechecks the
same authority/time contract and retains the immutable object borrow; original
encoded bytes are unavailable and reported as None. Do not re-encode the object
into the encoded entry point. Consumer migration must align the candidate's exact
thiserror 2.0.18 pin; see the
[qualified response boundary](qualification/platform-certificate.md#agent-response-compatibility-and-supplier-completion--issue-1).

`make qualify-platform-wasm` runs the frozen PocketIC and mainnet direct/delegated certificate corpora
through native and standalone Wasm authentication, with native/Wasm Clippy.
It needs Node and the installed Wasm target, but no PocketIC server. It checks
signature, subnet, freshness and resource refusals; it is not an IC cost model.

`make qualify-canister-signature-decode` checks the unverified canister-signature
envelope boundary: literal/refusal tests, compile-fail immutability, native/Wasm
Clippy, Rustdoc and the dependency graph. It does not authenticate signatures.

`make qualify-canister-signature-authentication` checks key admission and complete
signature composition: message/seed witness, certified-data binding, certificate
authority, separate time policies and the signature-specific subnet-type rule.
It includes synthetic direct/delegated refusal tests, compile-fail evidence,
native/Wasm Clippy, Rustdoc and normal/build dependency checks. Observed canister
signature execution is covered separately below; Auth integration remains pending.

`make qualify-canister-signature-observed` uses the admitted PocketIC server to
capture direct/delegated signature envelopes and authenticate them through the
public API. Prepare the server with `make install-runtime-server`. Fresh output
goes to `target/platform-signatures/` without changing frozen fixtures.

`make qualify-canister-signature-wasm` replays frozen envelopes and refusal cases
on native and standalone Wasm, checks native/Wasm Clippy and the portable graph,
and reports the Wasm digest. It requires Node and the Wasm target, but no server
or network. These targets qualify controlled replica signatures, not Auth
adoption or mainnet signature interoperability; see the
[evidence](qualification/canister-signature.md#observed-public-consumer-replay--2026-11-16).

`make qualify-auth-consumer` runs an isolated issuer-policy adapter on native
and Wasm, including focused Clippy and graph checks. It uses frozen signatures
over Auth's exact domain-framed claims digest; fresh capture is included in
`make qualify-canister-signature-observed`. No Auth or Canic checkout is built or
modified. See the [component contract and limits](qualification/canister-signature.md#isolated-auth-issuer-policy-consumer--2026-11-16).

Testkit's removed lifecycle helpers are replaced by the private
[qualification server owner](qualification/testkit-server.md). Installed commands
run through `node qualification/server.mjs run COMMAND ...`; direct fixture runs
require an explicitly owned `POCKET_IC_SERVER_URL`. `make install-runtime-server`
prepares assets and `make runtime-server-check` verifies them without downloading.

## Toko Tooling 0.45.0 adoption

Both workspace catalogs and locks now select published Toko Tooling 0.45.0.
The archive checksum is
`3789a925c1165317eff84cc80436195b66c0478b0c2b441bc2f1a755caf6a23a`.
The supported preview/update path selects the managed baseline and grouped Cargo
projection helpers; host/Rust/IC tool pins are unchanged. Its consistent-flow
policy matches the qualified Platform flow and the subsequent move of regression
fixtures to affected tooling work/CI. The conservative complete product gate
remains explicit until narrower reverse-consumer scope is qualified.

This is tooling adoption, not new product or native-host qualification. Existing
unrelated lock changes remain preserved. Review/install/check logs are retained
under `target/tooling0450/`. Snapshot verification (86 files), installed host/Rust/IC
tool admission, all 55 local flow/audit/wiring tests, formatting, workflow lint
and read-only group planning for Types, Protocol, Runtime and Platform passed.
No public product or downstream consumer release is claimed by this adoption.

## Historical Toko Tooling 0.44.11 adoption

At that adoption, both workspace catalogs/locks and the installed snapshot selected
published Toko Tooling 0.44.11. Its supported installer verified the registry source, installed managed
files and admitted the existing host/Rust tools. This is a local tooling update,
not a product release or a reason to repeat the complete release catalog.

The old fleet-wide thiserror hold is superseded. The current product catalog
uses a compatible thiserror requirement; its lock selects 2.0.21. Qualification
may still resolve 2.0.18 because published Types 0.21.1 and PocketIC constrain
that graph. Published Types 0.21.2 removes its exact pin without changing
its error API or encoded identities. Already-published Types 0.21.1 retains
its immutable exact requirement; the current qualification lock still selects
thiserror 2.0.18. A compatible requirement does not force a lockfile upgrade. Root packaging of other
crates against that old registry cohort is not evidence for the new selection;
qualify their archive dependencies when preparing their next delivery.


## Historical Testkit 0.109 qualification adoption

The qualification catalog and materialized Auth/Blob manifests select published
Testkit 0.109.0. Its archive SHA-256 is
`6405078e5332b34ab898d4c32c41f39b358655bb3d4aae508d3ff17f781cf66c`,
verified against the registry index. The new held-controller clock-step/error
surface requires no source changes in these callers; they do not exhaustively
match `ControlError`. Existing held-reply behavior is qualified through Blob.
PocketIC client/server remain 16.1.0, and Testkit defaults stay disabled in the
qualification catalog. Historical fixture provenance remains unchanged.
Testkit remains a qualification dependency. The complete adoption is included in
the published Platform 0.34.14 source; see the [release evidence](releases.md#exact-offline-signing-and-secp256k1-query-release)
for the additional locked supplier selections admitted with this release.

Focused Linux validation passed affected-package Clippy, actual installed Auth
and Blob signed/callback/enrollment journeys, signed transport with both identity
algorithms and post-expiry recovery, and Runtime raw system operations. Root
qualification and both materialized consumer locks resolve exactly Testkit
0.109.0. Logs: `target/testkit109-consumers.log`, `target/testkit109-signed.log`
and `target/testkit109-runtime.log`. The complete local release catalog subsequently
passed. New macOS evidence and consumer production migration remain separate.


## Testkit 0.110 and Host qualification adoption

Registry freshness review selected Testkit 0.110.0, Host Process 0.81.0,
Host Artifacts 0.80.3 and Host FS 0.80.3. Query's Host Text 0.77.2 remains
current. The qualification catalog selects Testkit and Host Process; the
materialized Auth, Blob and Storage manifests adopt their applicable suppliers.
Auth also selects IC Build 0.87.4, whose Host Process 0.81 dependency removes
its older transitive Process selection. Pinned source archives and sibling
checkouts remain unchanged. General dependency refresh remains separate.

The Testkit archive SHA-256 is
`16709c74c5c169ea786224ea8e10a8b45446219000c18c441e8aa73bd1409252`;
Host Process is
`1b81431d49e906feee6812a4fa8539feea57803f9937b926472ae947fa9bf6c5`.
Both downloaded archives matched the registry index. Existing synchronous
process supervision requires no API changes; the new launch/completion API is
not used. PocketIC client/server remain 16.1.0, qualification Testkit defaults
remain disabled, and product dependency graphs are unchanged.

Focused Linux validation passed server cleanup on success, command failure and
invalid readiness; actual Auth installed tests; Blob signed, callback and
enrollment journeys; signed/anonymous transport and post-expiry recovery;
Runtime raw system operations; and Storage's selected Wasm compilation/Clippy.
The 56 release-tooling regression tests, upgrade tooling, formatting and workflow
lint also passed. Evidence is in `target/testkit110/qualification.log`,
`target/testkit110/consumers.log` and `target/testkit110/auth-final.log`.
Materialized locks confirm the selected suppliers. Storage's evidence remains
compile-only; this does not establish production migration, new macOS execution
or registry publication. This source delivery includes Toko Tooling 0.45.0 and the CI native-oracle
graph/historical synthetic-key fixes. No product crate implementation changed;
product versions and registry publications remain unchanged.


## Stable structures export — Platform #30

Platform 0.34.16 adds `stable-structures`, re-exporting the unmodified
upstream 0.7.2 crate as `ic_platform::stable_structures`. See the
[consumer contract and provenance](requirements.md#storage-stable-collection-types).
`make qualify-stable-structures` is in native CI and the Platform release catalog.
It checks the requested public names and upstream identity, host memory reopen,
Wasm Clippy, documentation, singleton upstream resolution and dependency-free
defaults. Allocation and migration remain consumer policy.

Local Linux qualification passed the focused target for native and Wasm graphs,
all Platform feature tests (75 tests plus 12 doctests), all 56 release-tooling
tests and workflow lint. An extracted Cargo archive outside the workspace passed
the public integration test and Wasm compilation with only `stable-structures`
enabled. Logs are under `target/stable-structures/`. Publication is verified in the [release record](releases.md#stable-structures-export-release);
Storage installed upgrade/rollback acceptance remains separate.
