# IC Platform

One controlled dependency boundary for IC runtime integration and required IC
Candid interfaces. All application endpoints we own use IC Wire, including public
endpoints called by third parties. Candid is the adapter for management canisters,
NNS/SNS, ledgers and other interfaces that require it; caller location does not
select a codec. Existing Candid checkpoints remain a legacy compatibility exception.

The pending 0.5 hard cut removes the unused `cdk::storage` checkpoint helpers;
existing data stays with its consumer and raw stable-memory passthroughs remain.
See [the retirement contract](docs/migration.md#pending-05-retirement).

Platform has no default features. Its owned runtime is Candid-free; transport,
parsing and PocketIC are native opt-ins. Consumers retain their Wire schemas,
authentication, budgets and retry policy. See the [maintenance and recovery
contract](docs/migration.md#runtime-maintenance-and-recovery).

Select `principal` for owned Principal/Account types without depending directly
on `ic_principal`. Runtime, Candid and external client/DTO features include it;
executor/IC0-only builds omit application types. The 0.3 type is Platform-owned;
external clients and DTOs retain `principal::ExternalPrincipal`. Convert explicitly
with `.into()` at those boundaries. Default builds include Principal and its
dependencies, but neither Candid nor CDK. Self-authentication continues to delegate
to the original upstream implementation. See the [Principal migration](docs/migration.md#owned-principal-migration-03).

| Feature | Implemented contract |
| --- | --- |
| `principal` | Owned Principal/Account types, text/Serde and external conversions; no Candid |
| `agent` | Native Agent transport/builders, PEM identities, typed errors and signed-request operations |
| `bls` | Upstream BLS keys/signatures, typed errors and verification, with alloc and no RNG feature; Candid-free |
| `candid` | Bounded single-result, exact-result and argument-list decoding; external DTO derives and encoding |
| `candid-service` | Candid service declaration macros and printer; implies `candid` |
| `runtime` | Raw calls, system and task adapters at `cdk::*`, without entrypoint macros |
| `candid-parser` | Native host parsing/type checking, service equality, dynamic refusal fixtures and JavaScript/TypeScript generation |
| `canister-sig-creation` | Imported signature public-key formats, root-key constants and hashing; no Candid/ic0 or store API |
| `cdk` | Owned explicit-codec entrypoints, raw calls, system helpers and protected spawning; Candid-free runtime with one registry executor |
| `certification` | Upstream certificate/hash-tree and mutable tree types, constructors, lookup and hashing; independent of Candid and executor |
| `executor` | Shared CDK executor task/context primitives; no Candid |
| `ic0` | Qualified raw IC 1.2.0 calls, including replicated/controller flags, reply/reject bytes and debug output; independent of Candid and executor |
| `icrc-ledger-types` | Explicit upstream ICRC account, metadata, allowance, transfer-error and block/archive/certificate types; no storage integration |
| `management-canister-types` | Upstream 0.11.0 HTTP, history, snapshot and status DTOs; transitive Candid, no executor or host transport |
| `signature-verification` | Imported canister-signature verification; certification/BLS, no Candid/ic0, executor or host transport |
| `pocket-ic` | Native PocketIC client identities; Testkit retains server lifecycle and policy |
| `ecdsa` | Native/Wasm K256 secp256k1 ECDSA keys, signatures and prehash traits; no RNG or runtime |
| `ed25519` | Native/Wasm Dalek keys, signatures and canonical scalars; no RNG, Candid or runtime |
| `secp256k1` | Native offline public-key derivation from exactly 0.3.0; no Candid/CDK, excluded from Wasm |

The optional `cdk` surface uses maintained, non-cryptographic runtime adapters
and the owned generator. It does not depend on the full `ic-cdk` package.
All controlled endpoints use explicit Wire hooks; select `candid` separately
for required external management/system interfaces and existing checkpoint bytes.
These breaking runtime and transport changes are released in the 0.4 line.
See [the migration contract](docs/migration.md) for ownership and adoption conditions.
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
[the benchmark report](docs/benchmarks/0.4/README.md).
`bash scripts/ci/fuzz-parsers.sh` runs a separate 60-second coverage-guided
Principal/Account differential campaign. Prepare nightly-2026-07-29, a C++
compiler and registry sources explicitly first; the command installs nothing.
It retains its corpus, crash artifacts and disposable lock under `target/`.
This tooling-only libFuzzer selection does not enter consumer dependency graphs.

Consumers own schemas, authentication, byte/work budgets, deadlines, retries and
recovery. IC Wire owns controlled application protocols. Candid remains required
where external protocols specify it. Moving a dependency behind this boundary
does not itself remove upstream code or shrink a canister.

## Development

The optional `stable-structures` boundary supplies original substrate identities
for Storage/Testkit; see the [contract](docs/migration.md#stable-structures-substrate).

Prepare common tools explicitly with `make install-tools`; `make tools-check`
verifies them offline. Prepare locked dependency sources with
`cargo fetch --locked` before validation, including the Wasm target dependencies
used by frozen standalone consumer checks. Checks never install executables. Rust 1.99.1 is the
development compiler; Rust 1.88.0 is the minimum. Prepare both toolchains with the
`wasm32-unknown-unknown` target for each compiler before their checks. For an
existing MSRV toolchain, use `rustup target add --toolchain 1.88.0 wasm32-unknown-unknown`.

Tool setup includes the pinned IC Build 0.18.0 release editor, which requires
Rust 1.91 or later to build. `make install-release-editor` prepares it separately;
`make release-editor-check` admits it offline. Standard release commands prepare
tools during admitted preflight; standalone metadata checks never install them.
Pending changelog notes use `## [X.Y.Z]` without a date or `Unreleased` suffix.

Focused commands: `make check`, `make test`, `make clippy`, `make wasm-check`,
`make msrv`, `make check-boundary`, `make check-doc-links`, and
`make shared-tooling-check`. `make ci` is the full release gate.
Run full `make clippy` before longer tests; its all-target/all-feature check denies
warnings. The fail-fast release gate also runs Clippy before its test targets.
`make check-upstream-scaffolding` applies the experimental packaging patches to
disposable registry-source copies and checks source preservation and licenses.
It performs no dependency adoption, publication or runtime qualification.
`make check-consumer` tests separate Candid-only, certification-only, parser-only,
dynamic, management/Candid, ledger/Candid, ic0-only, Agent/certification three signature/BLS consumers and a CDK canister
with only Platform declared, on Rust 1.99 and 1.88. The CDK standalone
consumer proves raw entrypoints/calls without Candid or full CDK dependencies.
The `external-candid` profile separately qualifies required external Candid
entrypoints, explicit service registration and fixed wire bytes without a direct
Candid dependency; both compilers link its Wasm library.
Candid and certification
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
sections with grouped ordering. The reviewed Shared 0.25.1 snapshot supports
this through the paired local formatting commands.
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
Canister-signature creation and verification bring upstream certification but
no Candid or raw ic0; only verification brings BLS and selects the creation surface.
BLS alone excludes creation, Candid, certification, CDK and
host tooling. Its upstream defaults/RNG are disabled and alloc is explicit.
Graph admission rejects retired registry signature packages and checks BLS and
certification identities; other consumer graphs still need their own qualification.
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

The facade combines MIT and Apache-2.0 sources; its imported decoder retains
IC Testkit's copyright notice. The maintained generator is Apache-2.0.

### Maintained runtime regression

`make check-runtime` builds and installs Wire canisters through Testkit-managed
PocketIC on Rust 1.99 and 1.91. The pure `cdk` selection rejects Candid, parser,
full CDK and native host clients in its canister graph. A separate
`external-candid` selection tests an outbound management `raw_rand` call with
explicit Candid arguments and preserves the original persisted checkpoint bytes.
All RPCs and installation/upgrade arguments use Wire in both selections.

The fixture checks raw awaits, composite queries, guard/malformed-frame refusal,
all four explicit request allowances, response-encoding traps, callback cleanup,
weak/protected/migratory spawning, submission-pressure and insufficient-cycle
errors, timer cancellation/rearm, upgrade, cycle/refund counters, stable-byte
rollback and certificate lookup. It installs no alternative executor.

Run `make check-runtime-lock` for coordinated lock, graph and imported-source
checks, or `bash scripts/ci/check-platform-runtime.sh --check-graph` without
compiling/starting a server. Source fingerprints protect the imported call,
callback/trap cleanup and protected-spawn regions; mutation cases test refusal.
The sole registry executor remains =2.0.0 and ic0 remains =1.2.0.

Prepare/admit PocketIC using Testkit, then set `POCKET_IC_BIN` or
`IC_TESTKIT_POCKET_IC_URL`. The harness retains Testkit's bounded startup and
teardown. It never installs a server implicitly. Artifacts, sources and metadata
remain in `target/cdk-runtime.*`. Normal runs use the maintained frozen selection
in `scripts/fixtures/cdk-runtime/Cargo.lock`; explicit `--refresh-lock` is a
reviewed dependency refresh, not normal validation. `--build-only` compiles both
protocol selections without starting a server. Runtime qualification remains an
explicit target outside local full CI. Native macOS and live IC are separate
qualification from Linux PocketIC or Wasm compilation.

The Platform-only fixture remains a Rust 1.88 gate with no Wire-version MSRV
requirement. The installed Wire/Timers/Testkit fixture has Rust 1.91 inputs.
No measured Wasm-size reduction is claimed.

## Source licenses

Original Platform code is MIT. Imported non-cryptographic runtime adapters and
the generator retain Apache-2.0. The combined `ic-platform` package declares
MIT AND Apache-2.0; included source notices identify the upstream owner/version.
