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
`releases/`. Protocol, Types and Runtime are registered now; the other groups are added with their
real implementation packages. Package versions remain independent.

The first vertical slice is bounded Cloud Engine EngineView/EngineSubnet
projection decoding, using published Types initially where practical. Establish
its consumer contract and independent acceptance fixtures before broadening the
codec or adding other protocol families. Create packages only for real slices;
do not add empty packages to carry tooling.

The Rust workspace now contains an unpublished `ic-platform-protocol` package
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
Protocol and its private qualification packages start at 0.1.0;
publication remains disabled.

The local `ic-platform-types` 0.21.0 candidate now implements Principal and
Account/Subaccount admission and Serde, including rendering into caller-owned storage. Its
[qualification record](docs/qualification/types-principal.md) documents narrow
source reuse and independent fixtures; [Account qualification](docs/qualification/types-account.md)
covers effective identity and explicit Serde presence. Opt-in self-authentication
and [independent native/Wasm consumer checks](docs/qualification/types-consumer.md)
are implemented. Production consumer migration remains pending.
Protocol still uses registry Types 0.20.4 until the replacement
cohort is qualified; these are distinct Rust type identities.

Runtime `0.23.3` is the current GitHub source snapshot, with registry publication
disabled. It provides opt-in inert Call
preparation and independent method/argument ownership, using local Types. Its
[qualification boundary](docs/qualification/runtime-call.md) distinguishes this
implemented request model from the experimental IC0/executor and pending installed
qualification. A test-only custody model supplies supporting evidence for
first-poll submission and post-drop callback ownership.
Completion records now retain exact responses, unknown rejection codes and refund
snapshots, with explicit uncertain outcomes and no inferred retry policy.
An opt-in wasm32 `call-ic0` slice now reads system call costs through the approved
IC-only unsafe boundary and supports consuming one-way submission, with an
independently executed ABI harness. Experimental awaited transport now submits on
first poll and captures responses through integer callback handles. Its
[callback evidence](docs/qualification/runtime-awaited.md) covers real Wasm callback
execution and an experimental method-attached executor: callbacks resume their
owning task, cleanup terminates it, and handles expose local lifecycle state.
Installed-canister execution and rollback remain pending.
Runtime's `system::read_args(maximum)` now supplies bounded raw argument capture
for that entrypoint work, refusing oversized messages before allocation/copy.
`entrypoints::prepare` composes a caller-supplied pre-copy guard and decoder
around that capture, stopping at the first failure. Runtime selects neither
authorization policy nor a codec; see the
[argument boundary](docs/qualification/runtime-arguments.md).
The [manual update adapter](docs/qualification/runtime-update.md) now composes
that admission with one executor-owned handler. `system::reply` supplies an
explicit raw reply with one ordinary reply authority per task. Broader export
modes, automatic encoding/reply ownership and installed execution remain pending.
The opt-in `entrypoint-macros` feature now exposes an explicit manual-reply
`update` attribute from `ic-platform-macros` 0.23.3. Its
[generated exports](docs/qualification/runtime-exports.md) run in a separate Wasm
consumer, including rejection and compile-fail checks. Queries and automatic
reply generation remain unsupported in this initial macro slice.
The separate `ic-platform-runtime-guard` now joins the Runtime group at 0.23.3.
Its exact version check and shared Cargo `links` identity reject incompatible
guarded executor graphs; see the [guard evidence](docs/qualification/runtime-guard.md).

An opt-in `registry-settings` feature also projects selected Registry subnet
features and resource limits, preserving presence and raw defaults under shared
byte/field bounds. Its [contract and qualification status](docs/qualification/registry-settings.md)
describe the local tests and remaining consumer gates.

The response feature also decodes raw network fees and Registry-copy status as
the first part of the [marketplace/availability work](docs/qualification/cloud-engine-observations.md).
The opt-in `cloud-engine-marketplace` feature adds [daily price rows](docs/qualification/marketplace-prices.md)
with all six node classes and arbitrary naturals, plus [available-node page responses](docs/qualification/available-nodes.md).
Bounded page argument encoding is also implemented; consumer adoption remains pending.

Toko 0.44.4 is installed through its authenticated installer, including canonical
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

All sibling checkouts remain read-only. The maintainer authorized initial source
delivery to the public `dragginzgame/ic-platform-new` GitHub repository. Registry
publication, releases, deletion and consumer migration remain excluded. All
packages retain `publish = false`.
