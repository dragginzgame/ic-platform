# Instrumented PocketIC source provider

This opt-in native profile owns the pinned IC source, native adapters, patched
PocketIC client/DTOs, ordered patches, native prerequisites and build receipts.
It is outside Platform's default and Wasm dependency graphs. It does not install
or invoke Motoko. The independent recorder, retention/report policy, viewer and
workloads remain owned by IC Profiler.

Consumers pin a full Platform Git revision and select this profile through
`provider.json`. Initialize its `upstream/ic` submodule at the recorded gitlink.
Set `IC_PROFILER_CONSUMER_ROOT` to the physical consumer checkout, then invoke
`scripts/ic_profiler.py prepare|build|test|refresh-manifest|check-native-graph`. Explicit setup calls
`scripts/dev/native-tools.sh install`; `check` is read-only. No fetch occurs in
assembly, checks or builds. The consumer's `.build` holds generated integration,
targets and immutable receipts. Useful previous assemblies must be retained when
migrating from a consumer-owned backend.

Fresh and verified cached preparation publish the normal assembly's summary to
the consumer's `.build/receipts/latest-prepare.json`. Candidate probes keep that
summary unchanged. The normal assembly also retains its complete preparation receipt
at `.build/ic/.git/ic-profiler-receipt.json`; check its provider/consumer identities
before retaining it with measurements.

`scripts/ic_profiler.py native-verify` owns the complete native gate: prerequisite
and graph admission, strict primary-package Clippy, server build, recorder and server-library product tests,
and the installed profiling tests
against that exact server. Profiling tests serialize their whole-server recording
windows; independent workloads must not contaminate one another's cycle totals.

`scripts/ic_profiler.py clippy -- --no-deps --lib -- -D warnings` checks the
native server package without linting the full upstream dependency workspace.
Retain the pinned IC Clippy configuration; no blanket warning allowances apply.
The paired Profiler product sources also need the narrow CLI/rendering lint
correction tracked by Platform #98. Old source pins retain their prior gate.

The provider's `profiler/` sources and the consumer's `profiler/` sources merge at
the declared IC destination. Overlapping files are rejected. Their separate
origins and digests enter the assembly identity, together with the IC pin and all
patch/overlay inputs. Command receipts record both provider and consumer commits.
The paired consumer additionally supplies `server/system_profile_product/mod.rs`
and its tests. It owns query admission, immutable images, command metadata and
render/bundle plumbing. Native capture and scope cleanup stay in this provider.
Keep older consumers pinned until that product source is present.

Dependency updates compose in one current patch after the foundational workspace
patches. The separate compiled-fact patch exposes the existing resource-charge
cadence; it changes no scheduler calculation or charging behavior.

Adapters stay inside the provider's IC workspace. The recorder and server
select published IC Tooling Host Artifacts 0.72 with its `gzip` feature directly,
matching Host Process/FS. The generic codec adds no Platform facade or runtime
dependency. Profiler retains levels, import budgets and bundle semantics;
Metrics retains arithmetic and neither owns charging policy.

The PocketIC profile selects the `ic-admin` library without its CLI-only NNS
initialization dependency. The upstream CLI keeps its default feature and binary;
the native server retains its routing-table helper without building lifeline.
No dummy Wasm, ignored build failure or replacement cryptography is used.

The transferred sources originate from `dragginzgame/ic-profiler` at
`be3fce7f5d3ff18060c3679b377d803cc3295bc4`. Existing license/copyright
notices and IC provenance remain intact. The existing Python assembler is
transferred with its fixtures; this profile adds no second assembly engine.

Focused assembler tests:

```sh
python3 -m unittest discover -s native/ic-profiler/tests -p 'test_*.py'
```

Actual acceptance also requires the selected native graph, reproducible assembly,
the instrumented server build and the installed `system_profiling` suite against
that new executable's recorded hash. Fixture passes do not establish native or
Bazel qualification.

Cargo builds embed the IC pin's real `mainnet-routing-table.json` rather than its
compile-only empty placeholder. Explicit routing overrides are parsed and hashed
in command receipts. The build script observes routing/canister input changes.
NNS genesis still needs real canister artifacts supplied by the selected IC build
environment; application-subnet profiling tests do not qualify those artifacts.

Profiling routes on the main PocketIC listener and the separate viewer share
Host/Origin admission. This includes run APIs, memory trace/snapshot controls and
viewer startup. Loopback listeners admit localhost names and IP authorities;
explicit non-loopback binds also admit other valid hostnames. Present Origin
headers must use HTTP(S) with the exact requested authority. Local nonbrowser
clients may omit Origin; forwarded headers do not confer authority. This is
request admission, not authentication for a remotely exposed listener.

Focused native regression command, using an explicitly prepared consumer/provider:
`scripts/test -- -p pocket-ic-server --lib system_profile_api::admission`.
From the Platform repository root, `bash scripts/ci/check-native-profiler-http.sh`
additionally tests that exact
admission source and its Axum/Aide route integration without compiling the replica;
explicitly prepare its dependency cache with `--prepare` first. The source CI job
runs this proof alongside the assembler fixtures, with a separate build target.
Route-unit and standalone browser evidence do not replace a newly built native
server's qualification when the consumer adopts a changed provider pin.

## Paired product-source contract

This revision requires the consuming Profiler tree to supply
`server/system_profile_cli.rs`, `server/system_profile_render.rs` and
`rendering/{mod,png,ranges}.rs` with the PNG byte fixtures below that directory.
Those product modules have no provider compatibility copies. Older consumers
that omit them must retain their previous pin until adopting this paired source
change. Existing assembled module paths and native integration callers stay the
same; the provider still owns state access, trace capture, HTTP admission and
Cargo/Bazel dependencies. This source-profile change does not alter the published
Platform library API.

Server profile bundles import through Tooling's bounded gzip codec and export
unchanged core gzip bytes. API input/decoded limits remain 128 MiB; the recorder
retains its separate 256 MiB decoded limit. This paired provider/consumer change
supersedes the earlier decision to retain the Platform facade. Zlib for PNG is
a distinct native dependency; PNG mechanics and budgets stay product-owned.

## Host process and private Wire inputs

The paired Profiler consumer supplies the Host process policy adapter and the
IC Wire private process DTOs. This profile wires IC Wire 0.10.0 and
`ic-host-process` into both Cargo and Bazel, retaining the 60 KiB consumer frame
limit. Old ICP2/JSON process senders and receivers must be rebuilt together.
Saved reports and profile bundle formats do not change. Generic OS observations
and child/gate mechanics belong to Host; Profiler retains capture activation,
pinned root selection, cadence, aggregation, diagnostics and CLI output.

Process frames and Metrics 0.21 metadata share Wire 0.10. Only Metrics' native
TypeScript metadata feature is active; endpoints and Platform-type schemas stay
disabled. Arithmetic and saved layouts are unchanged; see the
[Metrics metadata adoption](https://github.com/dragginzgame/ic-observability/issues/103).

The native graph selects published Host Process/FS/Artifacts 0.72, Metrics 0.21
and Wire 0.10. Direct Host gzip removes the Platform 0.11 facade and its separate
Host Artifacts 0.71 instance from Cargo and Bazel. Cargo and Bazel locks retain
registry checksums; no downstream override or Git library supplier is added.
The native source profile still requires an immutable Platform Git pin for IC
source and adapters. Native macOS and installed acceptance remain separate.

## Textual client and compiled backend description

The protected profiling router exposes POST
`/api/profiling/instances/{instance_id}/canisters/{canister_id}/{action}` for
`start`, `stop`, `clear`, `trace`, `profile` and `snapshot`. It admits canister
text through IC's type, reuses native operations/DTO conversions and returns one
completed JSON result. Busy state is retried only before submission; Started
results are polled and successful results pruned. Operation errors, including
changed-snapshot conflicts, are never resubmitted. A timeout can leave an already
started operation running; the error says so. No client-side IC codec is needed.
Existing raw PocketIC endpoints retain their external contract.

The viewer JSON/MCP adapters add process generation/hash only. Counter producers
emit decimal strings directly; nested PocketIC values retain their canonical
Serde encoding. There is no recursive field-name rewrite or document-wide
`format_version` / `integer_encoding` label. Rebuild the paired viewer/provider
together; raw trace exports and saved Profiler report formats are unchanged.

The paired Profiler consumer supplies the query-product source interface from
Profiler #9 and the recorder's `explain_with_backend` projection. This provider
owns scheduler/source defaults, component implementation facts and PocketIC
coverage notes. It records descriptions only for local native captures. Source
hashes and compiled values accompany those facts; runtime configurations remain
independent captured evidence. Older consumers must retain their previous pin.
