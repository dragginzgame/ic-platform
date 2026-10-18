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
| `candid` | Bounded single-result, exact-result and argument-list decoding; external DTO derives and encoding |
| `candid-parser` | Native host parsing/type checking, service equality, dynamic refusal fixtures and JavaScript/TypeScript generation |
| `certification` | Upstream certificate/hash-tree types, constructors, lookup and hashing; independent of Candid and executor |
| `executor` | Shared CDK executor task/context primitives; no Candid |
| `ic0` | Qualified raw IC 1.2.0 system calls used by Timers and Testkit, including debug output; independent of Candid and executor |
| `icrc-ledger-types` | Explicit upstream ICRC account, metadata, allowance, transfer-error and block/archive/certificate types; no storage integration |
| `management-canister-types` | Upstream 0.11.0 HTTP outcall and canister history DTOs; transitive Candid, no executor or host transport |

The full CDK call/entrypoint boundary remains unfinished; consumer migration is incomplete.
See [the migration contract](docs/migration.md) for ownership and adoption conditions.
The public GitHub repository exists; registry availability requires separate verification.

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
`make check-consumer` tests separate Candid-only, certification-only, parser-only, management/Candid, ledger/Candid, ic0-only and Agent/certification consumers
with only Platform declared, on Rust 1.99 and 1.88. Candid and certification
consumers also compile for Wasm, as do the management, ledger and raw-system consumers.
The ic0-only caller compiles without executing a system call on native hosts.
Management qualification covers fixed HTTP/history wire bytes and change variants.
Ledger qualification covers account semantics, fixed account/block wire bytes,
representation-independent hashes, arbitrary-precision amounts and archive callbacks.
It covers DTOs, argument helpers and wire vectors, plus hash-tree constructors,
lookup outcomes, witness hashes and certificate/delegation fields. The parser
consumer runs natively and covers checking, binding generation, service drift
and typed textual-argument encoding. The Agent consumer checks offline signing,
request identity, effective-ID routing, typed errors and shared certificate types.
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
`ic-platform` is the sole production package. Principal, Candid, the host parser, Agent, certification, raw IC calls,
management/ledger DTOs, the executor and Serde are production dependencies, including optional and derive use; there are no
tests/tooling-only catalog entries or build/dev declarations. Optional parser
and Agent dependencies are restricted to non-Wasm targets.
The root catalog has `Production dependencies` and `Tests and tooling only`
sections. All current entries serve production, so the second section is empty.
`fmt`, `fmt-check`, the shared hook and release metadata checks all preserve
sections with grouped ordering. Shared 0.24.0 supports this through the paired
local formatting commands; newer uncommitted Shared defaults are not adopted.
Keep a blank line after each section heading so sorting preserves its position.
The common Rust tool set now contains only cargo-sort and cargo-sort-derives;
this repository has no Candid extraction command requiring a selected extractor.

`make check-features`, included in CI/release validation, checks normal/build
transitive graphs for the native host and `wasm32-unknown-unknown`, with no
features and all combinations of `candid`, `executor`, `certification` and
`candid-parser`, `agent`, `ic0`, `management-canister-types` and `icrc-ledger-types`, plus the actual default selection. It enforces empty defaults
and exact qualified executor/system requirements,
rejects `pocket-ic`, `ic-testkit` and `ic-testkit-candid`, and checks Candid/CDK-free defaults and Candid-free
executor-only graphs. Cargo reports canonical package names, so dependency
aliases do not bypass these exclusions. `check-boundary` independently checks
direct dependency ownership; role comments themselves are not executable policy.
Certification-only builds are Candid/CDK-free. Native Agent itself requires
transitive Candid and certification; it does not select those Platform features.
Management DTOs also require transitive Candid and are admitted only when selected.
Their `=0.11.0` requirement preserves the maintainer-selected external type contract;
enable `candid` separately when using Platform's encoding and bounded decoding.
Ledger DTOs similarly bring transitive Candid without selecting Platform's codec,
executor, parser or Agent. Upstream defaults/storage integration are disabled;
the graph gate rejects `ic-stable-structures` and unselected ledger dependencies.
Without certification or native Agent, builds exclude `ic-certification`.
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
remaining CDK/Testkit direct-dependency exceptions.

This library has no deployment entrypoint. Missing deployment enforcement belongs
at each consuming application: run exclusions against its exact final feature,
target and lockfile selection, retain successful builds, and bind source/lockfile
identity and artifact digest to deployed bytes. This repository does not provide
that artifact admission or infer new exclusions automatically from role comments.
`make publish` does not run the graph gate; registry publication is separate from
the release gate and from deployment qualification. Native macOS must run its own
checks; Linux/Wasm graph results do not qualify macOS or installed canisters.

MIT licensed. The imported decoder retains IC Testkit's copyright notice.
