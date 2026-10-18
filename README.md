# IC Platform

One controlled dependency boundary for external Candid protocols and IC runtime
integration. The initial library adopts Testkit's bounded Candid decoders and
exposes the selected executor through an explicit surface. It has no default
features, PocketIC, transport, host tooling or storage dependency.

| Feature | Implemented contract |
| --- | --- |
| `candid` | Bounded single-result, exact-result and argument-list decoding; external DTO derives and encoding |
| `executor` | Shared CDK executor task/context primitives; no Candid |

The full CDK call/entrypoint boundary and host parser migration are not implemented.
See [the migration contract](docs/migration.md) for ownership and adoption conditions.
The public GitHub repository exists; the crate remains unpublished and consumers
have not yet adopted it.

Consumers own schemas, authentication, byte/work budgets, deadlines, retries and
recovery. IC Wire owns controlled application protocols. Candid remains required
where external protocols specify it. Moving a dependency behind this boundary
does not itself remove upstream code or shrink a canister.

## Development

Prepare common tools explicitly with `make install-tools`; `make tools-check`
verifies them offline. Checks never install executables. Rust 1.99.0 is the
development compiler; Rust 1.88.0 is the minimum. Prepare both toolchains and the
`wasm32-unknown-unknown` target before their checks.

Focused commands: `make check`, `make test`, `make clippy`, `make wasm-check`,
`make msrv`, `make check-boundary`, `make check-doc-links`, and
`make shared-tooling-check`. `make ci` is the full release gate.
`make check-consumer` tests a standalone consumer without direct Candid/Serde
dependencies on Rust 1.99 and 1.88 and compiles it for Wasm. It covers generic
argument helpers, recursive DTOs, arbitrary-precision values and wire vectors.
`make check-features` verifies default, executor and parser-isolation graphs.
`make install-hooks` installs the shared formatting hook; `make fmt-check`
checks Rust and manifest formatting. Authorized releases use the shared
`make release-patch`, `make release-minor` or `make release-major` workflow.
They do not publish crates. Initial registry publication remains a separately
authorized operation after a clean committed source and native qualification.

Run `make publish-check` to package and verify the current version with all
features without uploading; this dry run allows working-tree changes. After
release and native qualification, `make publish` uploads the current committed
`ic-platform` version to crates.io using Cargo's configured credentials. It
requires a clean checkout and does not bump versions, commit, tag or push.
Both commands use the lockfile and retain evidence under `target/publication/`.
They require the pinned host tools (`make install-host-tools`). Cargo verification
is not the full release gate or native macOS qualification. No upload retries are
automatic: after an interrupted dispatch, reconcile the exact registry version
before invoking publication again. A successful upload command alone does not
prove registry visibility or consumer adoption.

Linux x86-64 and macOS 15 on Intel and Apple Silicon are supported hosts.
CI configures all three; native execution qualifies each separately. Wasm compile
checks establish target compatibility, not installed executor/trap behavior.

## Dependency roles and deployment admission

Shared Tooling 0.24.0 is pinned at `cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`.
`ic-platform` is the sole production package. Candid, the executor and Serde are
production dependencies, including their optional and derive use; there are no
tests/tooling-only catalog entries or build/dev/target-specific declarations.
Inline role labels retain global Cargo ordering: `fmt`, `fmt-check`, the shared
hook and release metadata checks all use the same ungrouped ordering policy.
The common Rust tool set now contains only cargo-sort and cargo-sort-derives;
this repository has no Candid extraction command requiring a selected extractor.

`make check-features`, included in CI/release validation, checks normal/build
transitive graphs for the native host and `wasm32-unknown-unknown`, with no
features, `candid`, `executor`, and both. It rejects `candid_parser`, `pocket-ic`,
`ic-testkit` and `ic-testkit-candid`, and checks empty defaults and Candid-free
executor-only graphs. Cargo reports canonical package names, so dependency
aliases do not bypass these exclusions. `check-boundary` independently checks
direct dependency ownership; role comments themselves are not executable policy.

This library has no deployment entrypoint. Missing deployment enforcement belongs
at each consuming application: run exclusions against its exact final feature,
target and lockfile selection, retain successful builds, and bind source/lockfile
identity and artifact digest to deployed bytes. This repository does not provide
that artifact admission or infer new exclusions automatically from role comments.
`make publish` does not run the graph gate; registry publication is separate from
the release gate and from deployment qualification. Native macOS must run its own
checks; Linux/Wasm graph results do not qualify macOS or installed canisters.

MIT licensed. The imported decoder retains IC Testkit's copyright notice.
