# IC Platform vNext

A ground-up rebuild of IC Platform to reduce technical debt and give identity,
execution, external formats and authenticated evidence explicit owners.

Start with [DESIGN.md](docs/DESIGN.md), which contains the replacement architecture
and migration/qualification sequence. It supersedes the earlier tentative drafts.
[Consumer requirements](docs/requirements.md) connect the design to existing
Platform issues and the first implementation slice.

The rebuild permits deliberate internal Rust API breaks while preserving required
external bytes, canonical identity, security checks and resource refusals. Its
main changes are one consuming call API, explicit task/callback custody,
single-declaration protocol mappings, immutable authenticated evidence and one
qualification obligation catalog. Existing implementations are evidence to assess;
reuse requires a destination contract and focused proof.

Four product owners are planned: Types, Runtime, Protocol and Platform. Runtime's
macros and singleton guard are support packages. Protocol has no Runtime
dependency; Platform has no Protocol or Runtime dependency. Browser remains a
separate ESM package. Prefer existing crate names for retained concepts. Existing
crates.io packages take their next SemVer version (next minor for breaking pre-1.0
APIs); new packages start at 0.1.0. Publisher selection remains separate.

[Changelogs](CHANGELOG.md) are separated by product release group under
`releases/`. Protocol, Types, Runtime and Platform are registered with real
implementation packages. Package versions remain independent.

The first vertical slice is bounded Cloud Engine EngineView/EngineSubnet
projection decoding, using published Types initially where practical. Establish
its consumer contract and independent acceptance fixtures before broadening the
codec or adding other protocol families. Create packages only for real slices;
do not add empty packages to carry tooling.

The Rust workspace now contains the published `ic-platform-protocol` package
with bounded `getEngineViews` request encoding and selected response decoding.
The response records, schema and decode dispatch come from one declaration,
without requiring Encode. Defaults are empty; the request-only feature has no
production dependencies. A separate `qualification/` workspace checks a public
consumer, canonical Principal identity and Serde representation. The
[Query integration](docs/qualification/query-engine-view.md) now passes in its
actual working tree after authorized local application; consumer delivery remains
pending. A separate native heap
profiler compares allocation against the published decoder; see its
[criteria and results](docs/qualification/engine-view-heap.md).
Protocol 0.2.3 is published; private qualification packages
remain at 0.1.0. Changed product crates advance after each GitHub source snapshot,
independently of registry publication. Each implemented group has now been published independently.

The published `ic-platform-types` 0.21.1 release now implements Principal and
Account/Subaccount admission and Serde, including rendering into caller-owned storage. Its
[qualification record](docs/qualification/types-principal.md) documents narrow
source reuse and independent fixtures; [Account qualification](docs/qualification/types-account.md)
covers effective identity and explicit Serde presence. Opt-in self-authentication
and [independent native/Wasm consumer checks](docs/qualification/types-consumer.md)
are implemented. Production consumer migration remains pending.
Protocol 0.2.3 now uses local Types 0.21.1, matching Runtime. Its public Principal
identity changes from the registry type; consumers must migrate as one cohort.
Registry Types remains only in qualification oracles. See the
[cohort boundary](docs/qualification/types-consumer.md#protocol-cohort-020).

Runtime `0.23.11`, macros and its singleton guard are published together. It provides opt-in inert Call
preparation and independent method/argument ownership, using local Types. Its
[qualification boundary](docs/qualification/runtime-call.md) distinguishes this
implemented request model from the experimental IC0/executor and pending installed
qualification. Tests exercise the production completion-state transitions and
real Wasm callbacks for first-poll submission and post-drop callback ownership.
Completion records now retain exact responses, unknown rejection codes and refund
snapshots, with explicit uncertain outcomes and no inferred retry policy.
An opt-in wasm32 `call-ic0` slice now reads system call costs through the approved
IC-only unsafe boundary and supports consuming one-way submission, with an
independently executed ABI harness. Experimental awaited transport now submits on
first poll and captures responses through integer callback handles. Its
[callback evidence](docs/qualification/runtime-awaited.md) covers real Wasm callback
execution and an experimental method-attached executor: callbacks resume their
owning task, cleanup terminates it, and handles expose local lifecycle state.
The first [installed rollback probe](docs/qualification/runtime-installed.md)
covers immediate updates and inter-canister awaits, including traps before/after
reply, successful callback cleanup and rollback when cleanup itself traps.
It also checks committed task observations, destructor re-entry, stale wakes,
notification rollback, resource refusals and refund capture across callbacks.
A controlled cross-subnet test observes SYS_UNKNOWN despite a committed remote
effect, preserving the distinction between timeout and nonexecution.
Upgrades follow the IC lifecycle: **stop → wait for Stopped → upgrade → start**.
The installed probe verifies outstanding callbacks drain before installation.
Use ICP CLI (`icp`) exclusively. The preview-first [operator tool](scripts/upgrade-canister.mjs) and
[upgrade instructions](docs/development.md#canister-upgrades) provide status reporting,
timeouts and artifact verification. Runtime does not preserve suspended futures
across upgrades or automatically cancel/retry remote calls.
Retained custody after failed cleanup and the remaining execution/rollback cases
remain pending.
Runtime's `system::read_args(maximum)` now supplies bounded raw argument capture
for that entrypoint work, refusing oversized messages before allocation/copy.
`entrypoints::prepare` composes a caller-supplied pre-copy guard and decoder
around that capture, stopping at the first failure. Runtime selects neither
authorization policy nor a codec; see the
[argument boundary](docs/qualification/runtime-arguments.md).
Runtime 0.23.10 adds explicit automatic updates, synchronous queries and
init/pre-upgrade/post-upgrade exports alongside manual updates. All generated
updates share one executor and reply authority; codecs and error behavior remain
explicit application choices. Caller/canister identity, IC time, controller checks,
bounded query certificates and certified-data publication are available through
`system`. The [Auth execution qualification](docs/qualification/runtime-auth.md)
records installed consumer and byte-boundary evidence and remaining migration
limits. The separate `ic-platform-runtime-guard` shares Runtime's published 0.23.11 version.
Its exact version check and shared Cargo `links` identity reject incompatible
guarded executor graphs; see the [guard evidence](docs/qualification/runtime-guard.md).

Runtime's opt-in `system-raw` feature also supplies dependency-free synchronous
operations for caller-owned exports, without a task or executor. See the
[Storage migration contract](docs/qualification/runtime-storage.md).

An opt-in `registry-settings` feature also projects selected Registry subnet
features and resource limits, preserving presence and raw defaults under shared
byte/field bounds. Its [contract and qualification status](docs/qualification/registry-settings.md)
describe the local tests and remaining consumer gates.

The response feature also decodes raw network fees and Registry-copy status as
the first part of the [marketplace/availability work](docs/qualification/cloud-engine-observations.md).
The opt-in `cloud-engine-marketplace` feature adds [daily price rows](docs/qualification/marketplace-prices.md)
with all six node classes and arbitrary naturals, plus [available-node page responses](docs/qualification/available-nodes.md).
Bounded page argument encoding is also implemented; consumer adoption remains pending.

Platform `0.34.3` now has an opt-in `certificate-decode` capability and empty
defaults. It strictly decodes untrusted certificate grammar, retains the original
bytes by borrow and exposes an immutable, explicitly unverified view. See the
[certificate boundary](docs/qualification/platform-certificate.md) for reuse
provenance and tests. The separate opt-in `certificate-authentication` capability
verifies direct-root or singly delegated signatures through upstream BLS, with
explicit root, target, separate time windows and resource limits. Its evidence is
immutable. Delegation verifies certified subnet authority and complete selected
legacy/sharded ranges; nested delegations refuse. Fresh PocketIC observations and
frozen direct/delegated certificate replays now pass through the public API.
`make qualify-platform-wasm` also executes authentication and selected refusals
in standalone Wasm, beyond the compile-only checks.
The same verifier now supplies immutable selected-subnet signing time for Query
issue #1, with explicit root-subnet binding when no delegation exists.
Independently observed mainnet direct-root and delegated certificates now pass
native and Wasm replay, including explicit root-subnet binding, separate
signing/delegation timestamps and authority refusals. Broader mainnet interoperability and production consumer qualification
remain pending before the full authentication gate.

Toko 0.44.6 is installed through its authenticated installer, including canonical
company files. Rust is pinned to 1.100.0. Use `make deps-list` to list local
packages and `make deps-update` for deliberate Cargo lock updates. See
[development setup and focused checks](docs/development.md) for prerequisites,
package-specific updates and the encoder checks.
`make check-protocol-graphs` enforces the current projection's dependency
boundaries and single Types identity for the selected Wasm graphs.

The design's source-review baseline is IC Platform commit
`c24e3a59583eabcf5b6bb87987269c13a209a51f`. Its findings and issue discussions are
historical evidence, not successful qualification of this replacement. No new
execution-speed, security or installed-runtime qualification is claimed.
The [standalone Wasm comparison](docs/qualification/engine-view-wasm.md) currently
reports size differences with acceptance undecided; it is not deployment qualification.

All sibling checkouts remain read-only. The maintainer authorized source delivery
to `dragginzgame/ic-platform` and subsequently requested starting crates.io
deployment. The [initial release plan](docs/releases.md) proposes Types followed
by Platform and records the remaining acceptance and execution blockers.
All six product crates are now published; qualification packages remain private.
Deletion and sibling consumer migration remain excluded. See the
[publication record](docs/qualification/2026-11-16-publication.md) and the
[Runtime/Protocol update](docs/qualification/2026-11-16-runtime-protocol-publication.md).

The opt-in `canister-signature-decode` capability now admits bounded tagged
signature envelopes and reuses strict certificate tree grammar. It retains exact
original bytes, an opaque embedded certificate and an explicitly unverified
witness. `canister-signature-authentication` now adds upstream DER key parsing,
message/seed/witness binding and direct/delegated certificate verification with
separate explicit time windows and shared traversal bounds. Independently observed
PocketIC signatures now pass public-API replay on native and standalone Wasm.
An isolated Auth issuer-policy adapter also passes on both targets using its
exact message domain, protected signer/seed and caller-owned freshness checks.
Mainnet signer interoperability and Auth migration remain pending; see
[the consumer contract](docs/qualification/canister-signature.md).
