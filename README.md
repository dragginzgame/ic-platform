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

Linux x86-64 and macOS 15 on Intel and Apple Silicon are supported hosts.
CI configures all three; native execution qualifies each separately. Wasm compile
checks establish target compatibility, not installed executor/trap behavior.

MIT licensed. The imported decoder retains IC Testkit's copyright notice.
