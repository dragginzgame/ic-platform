# Development

The product workspace contains unpublished Protocol 0.2.0 and the initial Types
0.21.0 candidate, plus the Runtime 0.23.7 local development increment
(following registry 0.22.5).
Types follows the published 0.20.4 baseline; its identity
slice is not yet a replacement for the complete published identity contract.
The private consumer fixture also starts at 0.1.0. Neither package name appeared
in the crates.io sparse index when checked on 2026-11-15. Publication and release
adapters are not enabled; name availability does not reserve a registry name.

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
`rust-toolchain.toml`. Toko 0.44.5 is selected exactly by Cargo and installed via
its authenticated exporter. Root company files and `vendor/toko-tooling/` are
managed; consumer commands live in the root Makefile.

```sh
make install-tools
make tools-check
make toko-tooling-check
make deps-list
```

Tool setup can download executables and populate Toko's per-user store. Checks
are offline and do not install tools. Linux x86-64 and macOS ARM64/x86-64 are the
intended development hosts. The initial focused suite passed on all three in
[hosted run 43717278830](https://github.com/dragginzgame/ic-platform/actions/runs/43717278830)
for source `08c72219ad6f261067c00eece9c8c71852e6b561`.

The initial [CI workflow](../.github/workflows/ci.yml) selects Ubuntu 24.04
x86-64, macOS 15 ARM64 and macOS 15 Intel. These are the intended native
qualification hosts, covered by the initial run above. Setup requires Bash,
GNU Make, Git, Perl/core modules, curl, tar and SHA-256 utilities, plus Rustup and
native compiler/linker prerequisites supplied by the hosted images. Toko installs
the selected jq/yq and other host tools before its toolchain-file preparer reads
`rust-toolchain.toml`; the complete tool setup/check follows. Node is pinned in
`.node-version` for standalone Wasm execution, with no npm packages required.

`deps-list` lists workspace packages, versions and release groups, not transitive
dependencies. A dash means no release group has been configured. Use Cargo's
`tree --locked` with the relevant target/features to inspect a dependency graph.

```sh
# Networked, deliberate lockfile update; optional package/version selection:
make deps-update DEPENDENCY_PACKAGE=toko-tooling DEPENDENCY_PRECISE=0.44.5
```

Without selectors, `deps-update` updates all dependencies within the declared
requirements. It covers the root and `qualification/Cargo.toml`, including the
`runtime-installed` member. Both listing and updates use that same selection.
For a deliberate Toko upgrade, review the new release, edit the exact root pin,
update the lock, then run `make toko-tooling-preview` and `make toko-tooling-update`.
Do not edit installed bytes or their receipt.

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
canonical Toko pre-commit hook through repository-local Git configuration; the
installer refuses to replace existing hooks. The hook and `make fmt-check` use
the same two-workspace formatting selection. Git configuration is local and
must be set up again in a fresh checkout. Local Linux activation and the Toko
consumer fixture passed automatic formatting/dependency sorting, partial-stage
refusal, formatter-failure isolation and preservation of unrelated working edits.
macOS hook behavior remains pending native evidence. Literal-frame unit tests stay with the product;
the separate public consumer proof lives under `qualification/`.

## Initial contract and implementation boundary

`cloud_engine::encode_get_engine_views_args(&[String], max_bytes)` encodes exactly
one Candid text-vector argument. It preserves empty/Unicode/NUL/whitespace text
and order, checks the complete frame size before output allocation, and reports
typed output-limit/allocation failures. Request-count limits belong to the caller.
The request-only production dependency graph is empty; the source-only Toko dev marker is not
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
crossing and the JSON Serde shape. Native-only comparison tests use published
Platform 0.33.7's codec with independently authored record declarations and its
existing selected response decoder. They compare 30 combinations of absent/
present bindings, signed timestamps, unsigned sizes, Unicode/NUL text and node
ordering, plus request bytes, truncated/malformed frames and 189 byte mutations.
Refusal classes are compared for the fixed malformed corpus; mutation tests
compare admission and decoded values. Work counters are implementation-specific
and are not equated by these tests.

The oracle is a native-only development dependency of the private consumer;
it does not enter the product or Wasm consumer graph. The candidate uses local
Types 0.21.0; the oracle retains registry Types 0.20.4. Comparisons cross that
intentional boundary through bytes or JSON, never assumed Rust type equality. No direct registry Candid
dependency or copied source/test corpus is introduced. These checks are bounded
differential evidence, not full Query adoption. Broader optional-coercion,
protocol-family and real-consumer qualification remains required before cutover.

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
`make lint-workflows check-secrets`. Installation uses Toko's pinned CI catalog;
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

`make check-protocol-graphs` uses the installed Toko graph checker with locked,
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
failures. Workflow lint, formatting and the 51-file Toko snapshot check passed.
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

Run `make install-runtime-server` for explicit Testkit-owned server provisioning,
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
`releases/<group>/CHANGELOG.md`, following the adopted Toko convention.

Protocol currently owns `ic-platform-protocol` and its pending 0.2.0
notes. Its earlier GitHub snapshots reused 0.1.0; the changelog records that
historical versioning gap without assigning retroactive versions. Repository setup and qualification notes supporting Protocol remain
in that ledger; they do not create releases for private test packages.

Types owns the incomplete `ic-platform-types` 0.21.0 candidate and its separate
pending changelog. Runtime and its exact guard share version 0.23.7; advance both
and the exact root guard dependency together. The macros package now shares this
version and has an exact Runtime dependency selection as well.
Platform now owns `ic-platform` 0.34.0 and its certificate-decoding slice. Browser
remains a future group; register it only with a real package and a checked
published baseline. Do not create placeholder packages.
Qualification workspace members remain unpublished, outside the product policy,
and independently versioned.

`make deps-list` uses the installed Toko reader to display group membership.
Types records the `candidate-types` alias used by Runtime. Runtime records its
`candidate-runtime-guard` and `candidate-macros` aliases; Protocol has no local
catalog consumers. Do not add unused
catalog entries just to display membership; the JSON policy owns that mapping.

This change establishes changelog ownership and inventory only. It installs no
release or publication commands. Packages remain unpublished. Future execution
must explicitly select `RELEASE_GROUP`, admit all required
proof and qualify Toko's supported release adapter before release readiness can
be claimed. Existing full release obligations remain open.

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
