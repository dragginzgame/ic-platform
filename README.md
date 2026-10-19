# IC Platform

One controlled dependency boundary for external Candid protocols and IC runtime
integration. The initial library adopts Testkit's bounded Candid decoders and
exposes the selected executor through an explicit surface. It has no default
features or PocketIC/storage dependency. Transport and parsing are native opt-ins.

`ic_platform::Principal` is always available, without enabling a feature or
depending directly on `ic_principal`. It preserves the upstream type identity,
including the Principal exported by the Candid contract. Default builds include
Principal and its dependencies, but neither Candid nor CDK.

| Feature | Implemented contract |
| --- | --- |
| `agent` | Native Agent transport/builders, PEM identities, typed errors and signed-request operations |
| `bls` | Upstream BLS keys/signatures, typed errors and verification, with alloc and no RNG feature; Candid-free |
| `candid` | Bounded single-result, exact-result and argument-list decoding; external DTO derives and encoding |
| `candid-parser` | Native host parsing/type checking, service equality, dynamic refusal fixtures and JavaScript/TypeScript generation |
| `canister-sig-creation` | Canister-signature public-key formats, root-key constants and hashing; transitive Candid/ic0, no store API |
| `cdk` | Platform-owned entrypoint/service macros with upstream CDK 0.20.3 calls, system helpers and protected spawning |
| `certification` | Upstream certificate/hash-tree and mutable tree types, constructors, lookup and hashing; independent of Candid and executor |
| `executor` | Shared CDK executor task/context primitives; no Candid |
| `ic0` | Qualified raw IC 1.2.0 calls, including replicated/controller flags, reply/reject bytes and debug output; independent of Candid and executor |
| `icrc-ledger-types` | Explicit upstream ICRC account, metadata, allowance, transfer-error and block/archive/certificate types; no storage integration |
| `management-canister-types` | Upstream 0.11.0 HTTP, history, snapshot and status DTOs; transitive Candid, no executor or host transport |
| `signature-verification` | Upstream canister-signature verification; transitive Candid/ic0/certification/BLS, no executor or host transport |
| `pocket-ic` | Native PocketIC client identities; Testkit retains server lifecycle and policy |
| `ecdsa` | Native/Wasm K256 secp256k1 ECDSA keys, signatures and prehash traits; no RNG or runtime |
| `ed25519` | Native/Wasm Dalek keys, signatures and canonical scalars; no RNG, Candid or runtime |
| `secp256k1` | Native offline public-key derivation from exactly 0.3.0; no Candid/CDK, excluded from Wasm |

The optional CDK surface uses a maintained generator fork with Platform paths and
the published upstream runtime. Consumer runtime adoption still requires its own
qualification; no upstream generator change is required.
See [the migration contract](docs/migration.md) for ownership and adoption conditions.
The public GitHub repository exists; registry availability requires separate verification.

Consumers own schemas, authentication, byte/work budgets, deadlines, retries and
recovery. IC Wire owns controlled application protocols. Candid remains required
where external protocols specify it. Moving a dependency behind this boundary
does not itself remove upstream code or shrink a canister.

## Development

Prepare common tools explicitly with `make install-tools`; `make tools-check`
verifies them offline. Checks never install executables. Rust 1.99.0 is the
development compiler; Rust 1.88.0 is the minimum. Prepare both toolchains with the
`wasm32-unknown-unknown` target for each compiler before their checks. For an
existing MSRV toolchain, use `rustup target add --toolchain 1.88.0 wasm32-unknown-unknown`.

Tool setup includes the pinned IC Build 0.18.0 release editor, which requires
Rust 1.91 or later to build. `make install-release-editor` prepares it separately;
`make release-editor-check` admits it offline. Standard release commands prepare
tools during admitted preflight; standalone metadata checks never install them.
Pending changelog notes use `## [X.Y.Z]` without a date or `Unreleased` suffix.

Prepare both dependency target caches once before frozen consumer qualification
on a fresh checkout, as CI does before validation:

```bash
cargo fetch --locked --target "$(rustc -vV | sed -n 's/^host: //p')"
cargo fetch --locked --target wasm32-unknown-unknown
```

These fetches preserve the lockfile. Consumer builds remain frozen and do not
perform implicit network preparation.

Focused commands: `make check`, `make test`, `make clippy`, `make wasm-check`,
`make msrv`, `make check-boundary`, `make check-doc-links`, and
`make shared-tooling-check`. `make ci` is the full release gate.
`make check-consumer` tests separate Candid-only, certification-only, parser-only,
dynamic, management/Candid, ledger/Candid, ic0-only, Agent/certification three signature/BLS consumers and a CDK canister
with only Platform declared, on Rust 1.99 and 1.88. Candid and certification
consumers also compile for Wasm on both compilers, as do the management, ledger
and raw-system consumers all three signature/BLS selections and the CDK canister. `make msrv` runs native all-feature tests and checks
the all-feature Wasm library at Rust 1.88.
The ic0-only caller compiles without executing a system call on native hosts.
Management qualification covers fixed HTTP/history wire bytes and change variants.
Ledger qualification covers account semantics, fixed account/block wire bytes,
representation-independent hashes, arbitrary-precision amounts and archive callbacks.
It covers DTOs, argument helpers and wire vectors, plus hash-tree constructors,
lookup outcomes, witness hashes and certificate/delegation fields. The parser
consumer runs natively and covers checking, binding generation, service drift
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
They do not publish crates. Initial registry publication remains a separately
authorized operation after a clean committed source and native qualification.

Run `make publish-check` to package and verify the current version with all
features without uploading; this dry run allows working-tree changes. After
release and native qualification, `make publish` uploads the current committed
`ic-platform-macros` and `ic-platform` versions to crates.io using Cargo's
configured credentials and dependency ordering. It
requires a clean checkout and does not bump versions, commit, tag or push.
Both commands use the lockfile and retain evidence under `target/publication/`.
They require the pinned host tools (`make install-host-tools`). Cargo verification
is not the full release gate or native macOS qualification. No upload retries are
automatic: after an interrupted dispatch, reconcile both exact registry package versions
before invoking publication again. Partial publication is not rolled back. A successful upload command alone does not
prove registry visibility or consumer adoption.

Linux x86-64 and macOS 15 on Intel and Apple Silicon are supported hosts.
CI configures all three; native execution qualifies each separately. Wasm compile
checks establish target compatibility, not installed executor/trap behavior.

## Dependency roles and deployment admission

Shared Tooling 0.24.0 is pinned at `cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`.
`ic-platform` is the sole consumer library; `ic-platform-macros` is its production
proc-macro support package. Its Syn/Quote/Darling dependencies generate consumer
entrypoints. Principal, Candid, the host parser, Agent, certification, raw IC calls,
management/ledger DTOs, signature/BLS APIs, the executor and Serde are production
dependencies, including optional and derive use. Testkit, Timers and the runtime
driver's CBOR decoder are tests/tooling entries used by the independent fixture.
Optional parser
and Agent dependencies are restricted to non-Wasm targets.
The root catalog has `Production dependencies` and `Tests and tooling only`
sections. Fixture-only selections stay in the second section and outside the
consumer library's production graph.
`fmt`, `fmt-check`, the shared hook and release metadata checks all preserve
sections with grouped ordering. Shared 0.24.0 supports this through the paired
local formatting commands; newer uncommitted Shared defaults are not adopted.
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
names until the matrix and isolation policy are updated, admits the selected CDK runtime/generators only with `cdk`, and rejects additional executor/
system versions or non-registry sources in the transitive graph. It also
admits PocketIC only with its explicit native feature, rejects `ic-testkit` and `ic-testkit-candid`, and checks Candid/CDK-free defaults and Candid-free
executor-only graphs. Cargo reports canonical package names, so dependency
aliases do not bypass these exclusions. `check-boundary` independently checks
direct dependency ownership; role comments themselves are not executable policy.
`make check-features-exhaustive` manually audits every feature combination and
actual defaults using the same admission rules. It is not invoked by routine
CI or release validation. The routine matrix deliberately does not cover every
higher-order interaction; the exhaustive audit remains available for that scope.
Wasm cache keys omit the four native-only leaves
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
Canister-signature creation and verification also bring upstream certification,
Candid and raw ic0; only verification brings BLS. They do not select the related
Platform surfaces. BLS alone excludes creation, Candid, certification, CDK and
host tooling. Its upstream defaults/RNG are disabled and alloc is explicit.
Graph admission checks the reviewed signature, BLS and certification package
identities; other consumer graphs still need their own qualification.
Without certification, native Agent or canister-signature capabilities, builds exclude `ic-certification`.
`candid_parser` and `ic-agent` are admitted only with explicit native host
selection; no runtime feature enables them. Host retains strict bounded CBOR decoding;
trust roots, authentication and application interpretation remain consumer-owned.

The executor requirement is `=2.0.0`; raw `ic0` is `=1.2.0`. These explicit
compatibility constraints preserve Timers' qualified identities for downstream
consumers, rather than relying on this repository's lockfile. Changing either
requires renewed combined callback/trap/cancellation/upgrade qualification.
Use `candid::decoder` when preserving upstream configuration/defaults or observing
construction, value, finalization and drop separately. The `candid` feature also
supports explicit service registration/export and reflection; see the packaged
[consumer contract](crates/ic-platform/README.md) for generated paths and the
CDK adoption requirements and older Testkit direct-dependency exception.

This library has no deployment entrypoint. Missing deployment enforcement belongs
at each consuming application: run exclusions against its exact final feature,
target and lockfile selection, retain successful builds, and bind source/lockfile
identity and artifact digest to deployed bytes. This repository does not provide
that artifact admission or infer new exclusions automatically from role comments.
`make publish` does not run the graph gate; registry publication is separate from
the release gate and from deployment qualification. Native macOS must run its own
checks; Linux/Wasm graph results do not qualify macOS or installed canisters.

The facade is MIT licensed; its imported decoder retains IC Testkit's copyright
notice. The maintained CDK generator retains upstream's Apache-2.0 license.

### Maintained CDK runtime regression

`make check-runtime` builds and installs a small canister in PocketIC and checks
bounded/unbounded calls, raw replies, guard/malformed-input refusal, callback-trap
cleanup, timer await/trap cleanup, cancellation, subsequent scheduling and upgrade
reinitialization. It also checks partial cycle acceptance, reject/receiver-trap
refunds, callback-write rollback and instruction counters across awaits. Guard
refusal requires its exact rejection code and message; negative unit cases reject
missing methods and unrelated errors. Sources are maintained under `scripts/fixtures/cdk-runtime/`,
reusing the standalone CDK consumer. It declares Platform and published Timers;
the native driver declares Platform and published Testkit. Dependencies for this
qualification live in the root's tests/tooling catalog, outside production graphs.
The generated workspace, lock, metadata and isolated build outputs remain under
`target/cdk-runtime.*`; concurrent attempts cannot replace one another's artifacts.
Normal runs use the maintained `scripts/fixtures/cdk-runtime/Cargo.lock` with
`--locked`. Release preparation updates only the two local Platform identities;
`make check-runtime-lock` checks drift and tests that external selections stay fixed.
To refresh it after dependency changes,
explicitly run `bash scripts/ci/check-platform-runtime.sh --refresh-lock`, review
the lock diff, and rerun qualification. Refresh seeds from the production lock and
resolves the test-only additions; it may access the registry. For cached inputs,
set `CARGO_NET_OFFLINE=true`. Normal validation never refreshes the selection.

`bash scripts/ci/check-platform-runtime.sh --check-graph` validates the combined
canister's native/Wasm dependency graphs without compiling or starting PocketIC.
Every runtime mode except lock refresh rejects host parser, Agent and Testkit
dependencies in those normal/build graphs, and enforces the qualified CDK and
single executor/system identities. `make check-runtime-lock` also runs adversarial
graph admission tests. This covers the combined consumer graph in addition to
the separate Platform feature matrix; it does not inspect a Wasm binary's size.

The runtime fixture also checks exact checkpoint bytes across upgrade, stable
write/growth rollback after a trap, certified-data publication and certificate
lookup, cycle burning, costs and system helpers in their supported contexts.

Prepare and admit PocketIC with the published Testkit server setup/check workflow,
then set `POCKET_IC_BIN` to that admitted binary, or set `IC_TESTKIT_POCKET_IC_URL`
to an already managed local server. No sibling checkout is required by the target.
The harness uses Testkit's bounded startup and owned teardown; it never installs
a server implicitly. Its offline locked metadata check requires cached registry inputs.
Published Timers and Testkit require Rust 1.91, so install that toolchain with the
Wasm target before running this separate qualification. The combined fixture
builds on 1.99 and 1.91; `bash scripts/ci/check-platform-runtime.sh --build-only`
compiles without starting a server. Runtime qualification is an explicit target,
not silently skipped when a server is unavailable and not part of `make ci`.

The Platform-only fixture remains a Rust 1.88 gate: `make check-consumer` now
links its `cdylib` on both compilers. For focused work, run
`bash scripts/ci/check-platform-consumer.sh cdk`. Full CI still selects every
consumer. PocketIC success is separate from native macOS and live-IC acceptance.

For published-selection evidence, run
`bash scripts/ci/check-platform-consumer.sh --published 0.1.12 cdk`.
This selects exactly that crates.io facade and rejects path/Git replacements of
the facade or generator. The same maintained consumer tests and native/MSRV/Wasm
builds run with only Platform declared. Metadata and the resolved lock remain
in the printed attempt directory. Registry inputs must already be cached for
the offline metadata step; this command never publishes or changes the repository
lock. Omit `--published` to qualify local source. This is archive consumption
evidence, separate from consumer application tests and installed-canister behavior.
