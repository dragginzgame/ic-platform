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

`scripts/ic_profiler.py native-verify` owns the complete native gate: prerequisite
and graph admission, server build, recorder tests and the five profiling tests
against that exact server. Profiling tests serialize their whole-server recording
windows; independent workloads must not contaminate one another's cycle totals.

The provider's `profiler/` sources and the consumer's `profiler/` sources merge at
the declared IC destination. Overlapping files are rejected. Their separate
origins and digests enter the assembly identity, together with the IC pin and all
patch/overlay inputs. Command receipts record both provider and consumer commits.
Adapters stay inside the provider's IC workspace, below the public Platform
boundary; they never depend back on the facade and create a package cycle.

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
the instrumented server build and all five `system_profiling` regressions against
that new executable's recorded hash. Fixture passes do not establish native or
Bazel qualification.

Cargo builds embed the IC pin's real `mainnet-routing-table.json` rather than its
compile-only empty placeholder. Explicit routing overrides are parsed and hashed
in command receipts. The build script observes routing/canister input changes.
NNS genesis still needs real canister artifacts supplied by the selected IC build
environment; application-subnet profiling tests do not qualify those artifacts.
