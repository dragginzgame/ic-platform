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
consumer, canonical Principal identity and Serde representation. Full Query
adoption remains pending. A separate native heap profiler compares allocation
against the published decoder; see its [criteria and results](docs/qualification/engine-view-heap.md).
Protocol and its private qualification packages start at 0.1.0;
publication remains disabled.

Toko 0.44.3 is installed through its authenticated installer, including canonical
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
