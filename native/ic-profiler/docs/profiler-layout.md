# PocketIC and IC profiling integration

This directory collects the dedicated profiling implementation, report formats,
server interfaces, viewer and tests. The reusable `ic-system-profiler` package
does not depend on `pocket-ic-server`; its location beneath the server directory
is organization, not a Cargo or Bazel dependency on the server.

See [profilingic.md](../../../profilingic.md) for the system-wide accounting and
coverage design, and the [viewer guide](viewer/README.md) for existing canister
memory and instruction workflows.

## Layout

| Path | Responsibility |
| --- | --- |
| `mod.rs` | Rust module integration for the PocketIC server |
| `server/` | Profiling APIs, CLI/MCP, metrics import, reports/rendering, charge dictionary, instruction configuration and memory snapshots |
| `viewer/` | Browser UI, Python client, agent manual, design references and browser/client tests |
| `system/` | Independent `ic-system-profiler` crate: bounded recording, CPU scopes, host collection, causal records, observer accounting, reports, import/export and contract tests |

`rs/pocket_ic_server/src/lib.rs` retains the established public module names
through an integration facade and re-exports. Existing callers do not need to
know the physical locations of the server implementation files.

The core package remains `ic-system-profiler`, imported as
`ic_system_profiler`. Its Bazel library is
`//rs/pocket_ic_server/src/profiling/system:system_profiler`; the independent test
target is `//rs/pocket_ic_server/src/profiling/system:system_profiler_test`.
Cargo workspace membership and dependent crates' path declarations point to
`system/`; this package has no server, execution-engine or replicated-state
dependency.

## Source baselines and upstream integration

The original checkout baseline is `ee8086a9773725fd1058832a9400297cf0990112`.
Commit `49a94778d9` adds the earlier canister instruction/memory profiler, charge
catalog, snapshots and viewer. The whole-system profiler and this directory
consolidation build on that commit. These source references are distinct from
the actual binary hash recorded for each measurement.

When updating IC source, review both layers:

```sh
git diff --stat ee8086a977 -- rs/pocket_ic_server/src/profiling
git diff --stat 49a94778d9 -- . ':(exclude)rs/pocket_ic_server/src/profiling'
git diff ee8086a977 -- . ':(exclude)rs/pocket_ic_server/src/profiling'
```

The first command covers central implementation. The second isolates newer
integration work outside this directory; the third includes the earlier
canister-profiler integration too. A single implementation directory does not
eliminate the need to instrument the components being measured.

## Changes that remain with their owning components

| Owning code | Integration that must be preserved |
| --- | --- |
| Workspace Cargo files and instrumented crates' Cargo/Bazel declarations | Register and depend on the independent core library; no dependency cycle through the server |
| `packages/pocket-ic` | Public schemas, client methods and profiling/storage tests; these belong to the client package |
| `rs/canister_sandbox` | Child activation/transport propagation, execution/IPC boundaries, and the earlier instruction/memory observer protocol |
| `rs/embedders` and `rs/memory_tracker` | Native execution and stable-memory scopes; existing Wasm instrumentation, source/origin metadata and deterministic page-debit observation |
| `rs/execution_environment` | Scheduler/message/query/management boundaries, call identities, queue and budget events |
| `rs/replicated_state`, `rs/messaging`, `rs/xnet` | Request/response identity, routing/stream events and native XNet work |
| `rs/state_manager` | State commit, checkpoint and background-worker boundaries |
| Ingress, validator and standalone-signature crates | Admission/selection/verification stages, including work outside a canister |
| Consensus, certification, DKG/IDKG, P2P and adapters | Native subsystem scopes on real exercised paths; PocketIC coverage remains path-dependent |
| `rs/state_machine_tests` and server entry points | Runtime integration, configuration, transport startup and existing canister-trace collection |

Most whole-system hooks are small scopes or events. The earlier canister
profiler additionally needs substantive owner-private support, including
`rs/embedders/src/wasmtime_embedder/profiling.rs`, Wasm instrumentation,
memory-tracker observation, sandbox protocols and REST schemas. Keep those
changes visible in upstream diffs; do not describe them as a directory-only
move or assume they can all be removed with the UI.

Hooks must remain observational: CPU time, residency, I/O and host clocks never
feed instruction charges, cycle debits, replicated state, admission decisions
or consensus. Signal-handler observation must retain its existing restricted
mechanism; general allocating/locking event recording does not belong there.

## Capture and disabled paths

System run capture, canister instruction profiling, canister memory tracing,
byte read/write masks and snapshots are separate opt-ins. Starting one does
not authorize silently enabling all of the others. In particular, a system-only
performance run should keep legacy per-canister tracing disabled unless it is
part of the explicitly matched experiment.

Ordinary canister execution compiles the original instrumentation path, without
instruction-block or read/write-mask hooks and without a page-charge collector.
A diagnostic Wasm variant is compiled lazily for source profiling or byte masks;
its native and disk caches are separate from ordinary code. Page-only tracing
attaches a signal-safe observer to ordinary code only when requested. Stopping
tracing uses the ordinary variant again, preserving installed Wasm, globals,
memory contents and deterministic instruction/cycle accounting. The first
source-profiled call may include diagnostic compilation; warm that variant
separately when measuring steady-state execution.

Recording-idle and process-disabled are different conditions. A build can have
hooks compiled in while no run is active; this must not be called a vanilla
upstream binary. To disable whole-system profiling for a process and its
inherited children, set this environment variable before starting the server:

```sh
IC_SYSTEM_PROFILER_DISABLED=1 /absolute/path/to/pocket-ic [server arguments]
```

The setting is cached at startup/first use. Hard-off skips system-profiler
transport creation and child transport propagation; starting a system capture
returns an explicit disabled error. It does not change the separate canister
instruction/memory profiler settings. Changing the environment after a process
has initialized is not a supported mode transition.

With hard-off unset and no recording active, initialized system hooks check
atomic activation state without per-hook file reads, clock calls, profiler
thread-local initialization or record allocation. Warm children map a shared,
read-only host control word once, allowing capture to start after canisters are
installed. The receiver blocks for new records; an initialized sampler waits
when no run is active. Private socket/control files use short `/tmp/icp-*`
paths independently of a possibly long state-directory `TMPDIR`. These controls
are host-only and never become replicated state. See
[`system/src/control.rs`](system/src/control.rs) and
[`system/src/lib.rs`](system/src/lib.rs).

Do not infer disabled status from an empty report or a failed transport
connection. That failure is missing coverage, not a successful disabled-mode
test.

Active capture adds CPU scopes, observer accounting, sampling, bounded retention
and optional metrics work. Reports keep observer CPU, residuals, dropped data,
counter availability and scope boundaries visible. Physical counters and
instrumented component coverage are distinct: a complete process-tree CPU total
does not imply complete per-component attribution.

The private child-process transport uses packed CBOR with an `ICP2` version
prefix and a reused encoding buffer. Its 60 KiB frame bound includes the prefix;
an oversized or partial frame is never sent. Receivers also accept the earlier
JSON frames. Packed struct-field positions are private protocol identifiers:
reordering fields requires a new version prefix. Public APIs, exported bundles
and image evidence retain their JSON schemas and exact numeric values.

Recording still updates component and pricing aggregates after generic detail
retention fills. It skips unnecessary detail-size estimation at that point,
avoids observer-batch processing for frames with none, and updates only changed
inherited scope context. Host identity checks parse the necessary process
fields without constructing a complete resource sample. These optimizations do
not reduce sampling cadence, suppress events, change observer flush boundaries,
or change deterministic IC work and charges.

An idle or disabled profiler still requires an empirical overhead comparison.
Use the same Wasm, workload, server build, storage path, runtime limits and
verification checks. Compare the candidate's explicitly disabled, idle and
recording modes independently; a historical release built with different
compiler settings is a separate reference. Record actual binary hashes, clock
windows and initialization success. A missing/failed transport is not a valid
properly initialized idle baseline.

## Scheduler round budgets

The viewer's **System → Rounds** view reads completed native scheduler
snapshots. Select a logical round, then use Workers, Canisters, Slices or Limits
to inspect its instruction progression, overheads, stopping reasons and captured
configuration. Parallel workers share an iteration origin; the global counter
debits the maximum worker consumption per iteration plus finalization, rather
than summing all worker work. The separate dirty-memory gauge reports logical
heap delta in bytes, with its own captured gate and coverage.

Application defaults here are 40B instructions per message, 2B per DTS slice
and 4B nominal per worker round. The initial dispatch counter is
`round - max(update_slice, install_slice) + 1`, leaving a final-slice reserve;
there is no separate hard 2B per-canister round quota. Values come from the
captured scheduler configuration. The outer heap-delta gate compares cumulative
round work between iterations, so a completed iteration can overshoot it.

Agents can use `pocket-ic profile rounds`, MCP `profile_rounds`, or
`GET /api/profiling/runs/{id}/rounds`. Request `round_details=false` for a compact
overview, then `round_id` and `limit=1` for one full snapshot. Selected-round
SVG/PNG/JSON exports preserve immutable query identity. Round retention is
bounded independently of generic events: dropped generic events do not by
themselves truncate retained round accounting. Missing or old data remains
explicitly unavailable, including unknown heap-delta measurements.

## Validation boundaries

Use the repository-qualified Rust toolchain to run the independent core tests
(`cargo test -p ic-system-profiler --lib`) and relevant server/client tests.
Browser tests live beside the viewer; the core contract and fixtures live in
`system/`. A relocation needs dependency/module/asset validation even when the
Rust source contents are unchanged. Native benchmark results from an earlier
binary remain historical evidence until the relocated build is measured.

Avoid parallel builds sharing one target/cache. Keep canister artifacts and
source receipts immutable during profiling. Whole-system native captures can
measure real scheduler, sandbox and storage work in PocketIC, while ordinary
direct ingress, simulated cross-subnet transport and local threshold replies
do not establish production authentication, P2P, consensus or distributed
signing costs. Missing capability and partial attribution stay explicit.
