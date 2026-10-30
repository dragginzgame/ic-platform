# Profiling the whole Internet Computer

Research, design and implementation notes, 2026-10-26. The earlier canister profiling implementation was committed locally, without pushing, as `49a94778d9` on top of `ee8086a977`. The implementation status below supersedes the original proposal wording in the design sections.

## Implemented system profiler

The implementation adds the [`ic-system-profiler` crate](rs/pocket_ic_server/src/profiling/system), [runtime instrumentation](rs/pocket_ic_server/src/profiling/system/src/flow.rs), [HTTP/MCP API](rs/pocket_ic_server/src/profiling/server/system_profile_api/mod.rs), [CLI](rs/pocket_ic_server/src/profiling/server/system_profile_cli.rs), [IC metric snapshots](rs/pocket_ic_server/src/profiling/server/system_profile_metrics.rs), [image renderer](rs/pocket_ic_server/src/profiling/server/system_profile_render.rs) and the System workspace in the existing viewer. It does not change deterministic instruction or cycle prices.

Available now:

- Run start/status/stop/delete, process-tree or declared-cgroup observations, bounded sampling/retention, compression, offline bundles, comparison and explicit loss/missingness.
- Measured synchronous exclusive thread CPU and per-poll async CPU, inherited execution identities, and bounded child-process transport. Compiler/sandbox process lifetimes and ancestry retain short-lived observations.
- Instrumentation across ingress authentication, consensus/payloads, P2P/QUIC, XNet, routing, scheduler, execution/setup/DTS/queries, compilation/sandboxes, state persistence and adapters. A path must actually execute before its costs can be observed.
- Explicit request/response/induction/callback lifecycle links. Structural span parents and shared-identity correlations are separate from causal edges; ambiguous attempts stay unresolved.
- Committed host-accounting events with rule IDs, prepayments/refunds/reservations/transfers kept distinct, and independent price aggregates that survive detail retention. This works without enabling detailed Wasm or byte-mask tracing.
- Bounded initial/stop IC metrics snapshots, with foreground-end added when a drain is requested: raw counters, gauges, histogram counts/sums, labels, registry generations, units and truncation. The `full` metrics level adds bounded buckets and quantiles; `none` explicitly disables registry collection. Snapshot gathering does not hold instance/state locks.
- System Overview, Calls, Components, Resources and Pricing/limits views, with context and source labels, agent-readable JSON/MCP, and query-linked PNG/SVG plus evidence sidecars.
- Sync and async Rust client methods, described with examples in [HOWTO](packages/pocket-ic/HOWTO.md), and built-in `profile explain`/agent documentation.

Quick Rust usage:

```rust
use pocket_ic::{PocketIc, SystemProfileConfig, SystemProfileQuery};

let pic = PocketIc::new();
// Install/configure your canisters, or start the capture before that work.
let started = pic.start_system_profile(SystemProfileConfig {
    name: "multi-canister workload".into(),
    ..Default::default()
});
let run_id = started["run_id"].as_str().unwrap();
// Execute the application workload here.
let summary = pic.stop_system_profile(run_id);
let components = pic.query_system_profile(run_id, "components", &SystemProfileQuery::default());
let metrics = pic.query_system_profile(run_id, "metrics", &SystemProfileQuery::default());
std::fs::write("workload.icprofile", pic.export_system_profile(run_id)).unwrap();
```

A runnable [ICP ledger example](packages/pocket-ic/examples/system_profile_ledger.rs) captures 30 transfers and a balance query after preloading the real ledger. Run `cargo run -p pocket-ic --example system_profile_ledger -- ledger.icprofile [SERVER_URL]`. It uses RAM-backed storage and records the NNS system-subnet fee schedule explicitly; it does not enable detailed instruction traces or byte masks.

The `profile` CLI can inspect the same runs online or offline:

```bash
pocket-ic profile explain
pocket-ic profile summary --server http://127.0.0.1:PORT/ --run RUN_ID
pocket-ic profile metrics --server http://127.0.0.1:PORT/ --run RUN_ID
pocket-ic profile export --server http://127.0.0.1:PORT/ --run RUN_ID --output run.icprofile
pocket-ic profile components --input run.icprofile
pocket-ic profile render --input run.icprofile --view components --format png --output components.png
pocket-ic profile capture --pid PID --duration 5 --output process.icprofile
pocket-ic profile record --duration 10 --output command.icprofile -- ./instrumented-replica ARGUMENTS
```

On Unix, `record` gates workload execution until the capture baseline is ready, preserving argv without shell interpolation. Workload stdout is sent to stderr so the profiler's stdout remains JSON. By default it stops only the command group it launched after its observation window; `--keep-command-running` changes that behavior. `capture --pid` never stops the attached target. An instrumented launched replica and descendants inherit component-trace transport; attaching to an already-running process without that transport provides host observations, not retroactive component traces.

Current collection boundaries remain explicit:

- Process-tree CPU is OS-tick quantized and snapshots are nontransactional. CPU scopes use thread clocks. RSS sums are not unique RAM; PSS is sampled. A declared cgroup is not automatically an isolated run.
- Storage values are process/cgroup storage accounting, not per-request SSD IOPS, device-completion latency or proof of cache residency. Device-layer selection is needed before treating stacked-device totals as physical hardware bytes.
- Owned wire traffic, perf/JIT sampled stacks, kernel cache-hit tracing, HostOS/QEMU attribution and multi-node clock alignment are not collected by this version. They are unavailable, not zero.
- PocketIC's distributed consensus/P2P, transport and cryptographic emulations remain labeled. Full-replica hooks can be exercised with a suitably built replica; local PocketIC tests are not a 13-node hardware benchmark.
- Drain is a bounded observation window, not a state-manager/kernel durability barrier. Exact critical paths remain unavailable when the captured causal evidence is insufficient.
- Bundles currently use bounded compressed JSON. Native stack profiles, OTLP/Perfetto projections, chunked random-access offline storage and a deterministic logical-cache simulator remain follow-on extensions. Actual cache misses never affect billing.

The original research and full target design follow; proposed capabilities beyond this status are not claimed to be available.

### Collector overhead and attribution, 2026-10-26

The Linux collector now discovers only the owned process tree through every thread's `children` file. CPU/RSS/I/O sampling is independent of PSS detail (default 1,000 ms); a zero detail interval disables PSS, including boundary scans. Periodic and final samples are serialized, their timestamps follow the scan, and final process counters are refreshed after memory collection. Retention uses conservative allocation-aware estimates instead of serializing each record to measure its size. Records move into a single capture, child encoding reuses a bounded buffer, and receiver bookkeeping is batched with a bounded stop flush.

Measured profiler CPU now has its own categories for scope creation/labels/finishing, event construction, recording, transport encoding/receiving and batch publication. This CPU is excluded from native parent exclusive totals. Batches are discontinuous evidence under `events.observer_segments`, separate from native timeline spans and causal edges. Capture epochs prevent previous tails merging into new captures; excluded old tails and flush failures are explicit quality fields. `pocket-ic profile explain` includes the category dictionary and collection defaults.

Additional scopes cover native HTTP/JSON handling, operation dispatch/publication, sandbox IPC, Wasm teardown, query bookkeeping and background state metrics. A separate PocketIC fix shares its read-only OpenAPI schema with `Arc`, avoiding deep schema clones during ordinary HTTP requests. The `/api.json` response remains byte-identical.

An existing warm ICP ledger can reproduce the comparison with [system_profile_overhead.rs](packages/pocket-ic/examples/system_profile_overhead.rs):

```bash
cargo run -p pocket-ic --example system_profile_overhead -- SERVER_URL INSTANCE_ID SERVER_PID OUTPUT_DIR
```

The example rotates recording-off, 20 ms and 100 ms modes, verifies every transfer and the resulting balance, and observes CPU independently through Linux `/proc`. `COUNT` and `REPETITIONS` default to 30 and three. Exports occur after the measured workload. Do not run unrelated operations on the observed server during a comparison.

Local medians for 30 transfers, three repetitions, the same unoptimized development-build settings, RAM storage and standard metrics:

| Observation | Before | After |
| --- | ---: | ---: |
| Recording-off workload CPU | 590 ms | 430 ms |
| 20 ms recording: workload CPU | 1,080 ms | 750 ms |
| 20 ms recording: workload wall | 698 ms | 516 ms |
| 20 ms recording: full capture CPU, including start/stop | 1,850 ms | 920 ms |
| 100 ms recording: workload CPU | 850 ms | 720 ms |
| 100 ms recording: workload wall | 677 ms | 514 ms |
| 100 ms recording: full capture CPU | 1,540 ms | 900 ms |
| Unattributed CPU in the 20 ms profiler window | 516 ms | 84 ms |

The original 30-transfer capture's 582 ms residual cannot be reconstructed exactly after the fact. Reproducing the workload exposed omitted profiler bookkeeping/transport, request handling and sandbox lifecycle work. A fresh 30-transfer-plus-query demo leaves 77 ms unattributed: approximately 67 ms in the server and 10 ms in its executing sandbox. Runtime/allocator work, probe clocks/tails and OS-tick quantization remain limits; these values are not invented per-component allocations.

Recording still has substantial cost in this short workload: after these changes, workload CPU is about 67% above its own off baseline at 100 ms sampling and 74% at 20 ms, with about 22% extra wall time. Coverage now includes roughly 2.6 times as many native events. These descriptive three-repetition results are not confidence intervals or mainnet throughput claims. Workload, full-capture and profiler windows differ and must not be subtracted as an exact CPU decomposition. Independent boundary measurements support the performance comparison despite the old collector's ordering bug. Detailed receipts and screenshots are saved locally in `/srv/shared/demo/pocketic-system-profile/`.

## 1. Purpose and accounting boundaries

Extend the existing canister instruction, cycle and memory profiler into a run-wide profiler. It should explain the resources used to complete a workload across canisters, subnets, replica components and supporting processes. It should support both application optimization and investigation of IC implementation, pricing and safety limits.

The main questions are:

- Where did CPU time go: authentication, consensus, scheduling, execution setup, guest code, memory tracking, persistence, cryptography, adapters, or something not yet attributed?
- Is latency caused by computation, queueing, network waits, scheduling, resource limits, checkpointing, or an external service?
- How much actual RAM, storage traffic and network traffic did the run consume, including work outside its foreground calls?
- Which canisters and call chains cause shared overhead, contention or resource amplification?
- Do instruction weights, fixed fees and scheduling allowances track physical resource costs well enough across realistic workloads?

Keep four accounting planes distinct and join them through identities:

| Plane | Examples | Meaning |
| --- | --- | --- |
| Physical observations | CPU seconds, resident memory, storage I/O, packets, faults | Nondeterministic diagnostics of a particular observed machine and interval |
| Logical work | Calls, validation attempts, unique pages, bytes copied, hashes, stream messages | Deterministic or explicitly scoped implementation events |
| Pricing | Weighted guest instructions, cycle debits/refunds, pricing rule IDs | The charge model already exposed by our profiler |
| Admission and scheduling | Round deductions, slice limits, page limits, queues, throttling | Why work was admitted, delayed, rejected, paused or stopped |

Do not rename a wall timer as CPU time, an instruction allowance as CPU consumption, or a fee as physical work. Report both productive throughput and resource use; a lower price does not by itself establish a faster or more efficient implementation.

**Observation boundary is part of every result.** Distinguish isolated PocketIC, a shared PocketIC server, an attached replica, a multi-node testnet, and optional GuestOS/HostOS or gateway collectors. One observed replica is not a measured 13-node subnet. Summing physical CPU across observed nodes is valid when labeled; summing replicated copies of the same cycle debit would count the same logical charge repeatedly.

## 2. Coverage: PocketIC and real replicas

Build one report model with different collection backends. PocketIC is a useful first implementation, but a whole-IC profiler also needs a real-replica backend. The capability manifest must describe the **actual operation path**, rather than assigning one blanket realism flag to a backend.

| Path | What PocketIC can exercise | What must not be inferred |
| --- | --- | --- |
| Direct client ingress | Real canister dispatch/execution through `submit_ingress_as` | Ordinary signed-envelope admission, pool authentication and consensus validation were measured |
| HTTP/gateway ingress | Can construct a real `IngressValidatorBuilder` with `StandaloneIngressSigVerifier` | All later production ingress validation stages also ran |
| Round payload assembly | `do_execute_round` invokes production `PayloadBuilderImpl`; direct payload execution is a different path | Full block proposal, propagation, notarization, quorum and finalization cost |
| Execution and queries | Real scheduler, hypervisor, sandboxes, query handling, memory tracking | Production hardware throughput or production contention |
| Inter-subnet streams | Production XNet payload builder, certified slice pool, stream encoding, routing and induction | Real network pulls, peers, TLS/RTT or distributed certification-signature verification |
| Certified data processing | Some real tree, witness, encoding and hash work | `FakeVerifier` performing real certification-signature verification |
| HTTPS, BTC and DOGE adapters | Real adapter tasks on configured paths; BTC/DOGE can use configured regtest endpoints | External network/blockchain behavior when responses are mocked; production process topology |
| Chain-key requests | Native management handling and locally synthesized test-key replies | Distributed DKG/IDKG, presignature generation, shares, aggregation and production signing throughput |
| State persistence | Real state manager, checkpoint, overlay and hashing paths when invoked | Disk behavior from RAM-backed state, or background work that the run did not observe |

Sources: [PocketIC operation and adapter wiring](rs/pocket_ic_server/src/pocket_ic.rs), [StateMachine setup, PocketXNet and round execution](rs/state_machine_tests/src/lib.rs). Relevant anchors are `PocketXNetImpl::refill`, `StateMachine::do_execute_round`, `FakeVerifier`, and the HTTP call-service construction around `IngressValidatorBuilder`.

For each component/path record independent fields:

```text
implementation: real | emulated | external | absent
availability: enabled | disabled | unsupported | permission_denied
observation: counter | cpu_scope | wall_scope | sampled | unavailable
completeness: complete | sampled | truncated | unknown
attribution: direct | causal_deferred | shared | allocated | residual
activity: observed_count, observation_start, observation_end
```

An unexercised feature can still consume background resources. An unavailable counter is `null` with a reason, never zero. Public node metrics proxies may filter out most internal metrics; a reachable endpoint does not prove full coverage.

## 3. Subsystem inventory

Use a versioned component dictionary with stable identifiers, descriptions, ownership, units, source references, existing metric mappings and required new hooks. Groupings below are coverage requirements; they are not mutually additive if one includes another. Store exclusive resource ownership separately from inclusive workflow views.

| Family | Required subcomponents and observations | Existing source footholds |
| --- | --- | --- |
| Public interfaces | Gateway/boundary routing and caching; HTTP connection/parsing/serialization; update admission and synchronous waiting; query responses/signing; `read_state` witnesses/certificates/delegations; polling traffic | [Public HTTP](rs/http_endpoints/public/src/lib.rs), [HTTP metrics](rs/http_endpoints/public/src/metrics.rs) |
| Ingress authentication and pools | Signature scheme, delegation checks, each validation stage, actual verify/cache/skip outcome, rejected requests, pool waits/history/deduplication, selection count/byte limits | [Validator](rs/validator/src/ingress_validation.rs), [pool handling](rs/ingress_manager/src/ingress_handler.rs), [selector](rs/ingress_manager/src/ingress_selector.rs) |
| P2P | Peer discovery/topology, adverts, duplicates, artifact assembly/download, stripped artifacts, retries, connection management, QUIC congestion/loss, queued bytes | [Consensus manager](rs/p2p/consensus_manager/src/metrics.rs), [QUIC](rs/p2p/quic_transport/src/metrics.rs) |
| Consensus | Component dispatch; block/payload creation and validation; ingress/XNet/HTTP/query-stats/adapter/DKG/IDKG payload sections; proposals/ranks; notary/finalizer; random beacon/tape; batch delivery; finalized/certified backlog | [Consensus metrics](rs/consensus/src/consensus/metrics.rs), [block maker](rs/consensus/src/consensus/block_maker.rs) |
| Certification | State-tree/witness preparation separately from shares, signing, validation, aggregation and serving certificates | [Certifier](rs/consensus/certification/src/certifier.rs), [state manager](rs/state_manager/src/lib.rs) |
| Message routing and local calls | Batch demultiplexing, induction, input/output queues, same-subnet direct induction, request/response reservation, timeout, rejection, shedding and callback delivery | [Message routing](rs/messaging/src/message_routing.rs), [scheduler](rs/execution_environment/src/scheduler.rs) |
| XNet | Per-destination streams; certified slice serving/pulling/pooling/building/validation; witnesses and certificates; transport; incoming induction; acknowledgements/reject signals/GC; capacity/backpressure | [Stream builder](rs/messaging/src/routing/stream_builder.rs), [stream handler](rs/messaging/src/routing/stream_handler.rs), [XNet builder](rs/xnet/payload_builder/src/lib.rs), [XNet endpoint](rs/http_endpoints/xnet/src/lib.rs) |
| Scheduler and admission | Canister selection/preparation/finalization; worker distribution and skew; round/slice deductions; runnable versus blocked queues; heap-delta limits; stopping reasons and remaining budgets | [Scheduler](rs/execution_environment/src/scheduler.rs), [scheduler metrics](rs/execution_environment/src/scheduler/scheduler_metrics.rs) |
| Execution lifecycle | System API construction, Wasmtime Store/instance setup, globals/memory restoration, wrapper entry/exit, execution, delta extraction, commit/rollback; updates, callbacks, cleanup, heartbeat, global timer and low-memory tasks | [Wasm executor](rs/embedders/src/wasm_executor.rs), [execution dispatch](rs/execution_environment/src/execution_environment.rs), [response execution](rs/execution_environment/src/execution/response.rs) |
| DTS | Logical execution, physical attempt and active slice; pause/resume waits; abort/restart and wasted work; dirty-page copying | [DTS](rs/canister_sandbox/src/dts.rs), [Wasm executor](rs/embedders/src/wasm_executor.rs) |
| Compilation and sandbox infrastructure | Validation, instrumentation, compile/link/deserialize; code cache; sandbox spawn/reuse/eviction; launcher/compiler CPU; IPC serialization/copy/dispatch/wait | [Hypervisor](rs/execution_environment/src/hypervisor.rs), [sandbox controller](rs/canister_sandbox/src/replica_controller/sandboxed_execution_controller.rs), [launcher](rs/canister_sandbox/src/launcher.rs) |
| Guest code and memory | Existing instruction/source/block/page profiler; actual CPU samples; SDK/Candid/allocator attribution only with symbols/markers; direct/native stable paths; requested bytes and unique 4 KiB/64 KiB regions; tracker faults, maps/protection and copies | [Instruction profiler](rs/embedders/src/wasmtime_embedder/profiling.rs), [stable replacements](rs/embedders/src/wasm_utils/system_api_replacements.rs), [memory tracker](rs/memory_tracker/src/deterministic.rs) |
| Queries and inspection | Query worker queues/fairness, result-cache lookup/hit/invalidation, composite queries and callbacks; `inspect_message`, including rejected ingress | [Query scheduler](rs/execution_environment/src/query_handler/query_scheduler.rs), [query cache](rs/execution_environment/src/query_handler/query_cache.rs), [execution metrics](rs/execution_environment/src/metrics.rs) |
| Native management operations | Create/delete/start/stop/settings/status; install/upgrade hooks; chunk store; snapshot creation/load/upload/download; cycles and async contexts; account nested compilation/storage separately | [Canister manager](rs/execution_environment/src/canister_manager.rs), [execution environment](rs/execution_environment/src/execution_environment.rs) |
| State and persistence | PageMap delta/zero/overlay/checkpoint sources; flush/merge/amplification; state commit/hash/manifest; checkpoint create/load/validate; syncfs; retention/GC; state-sync chunks/reuse/download/validation | [PageMap](rs/replicated_state/src/page_map.rs), [storage](rs/replicated_state/src/page_map/storage.rs), [state manager](rs/state_manager/src/lib.rs), [tip workers](rs/state_manager/src/tip.rs) |
| Crypto and CSP vault | Algorithm/operation/result; basic/BLS verification and caches; replica-side work versus remote vault RPC/serialization/locks; key generation and rotation | [Crypto metrics](rs/crypto/internal/logmon/src/metrics.rs), [basic signatures](rs/crypto/src/sign/mod.rs), [vault codec](rs/crypto/internal/crypto_service_provider/src/vault/remote_csp_vault/codec.rs) |
| DKG and chain-key protocols | DKG/IDKG/resharing, transcript/share production and validation, threshold BLS/ECDSA/Schnorr/vetKD, presignature inventory/replenishment, aggregation and request completion | [DKG metrics](rs/consensus/dkg/src/metrics.rs), [IDKG metrics](rs/consensus/idkg/src/metrics.rs), [chain-key metrics](rs/consensus/chain_key/src/metrics.rs) |
| External adapters | HTTPS queue/DNS/connect/TLS/proxy/reuse/upload/download, transform execution, response agreement; BTC/DOGE peer/header/block/cache work, payload construction and transaction submission | [HTTPS adapter](rs/https_outcalls/adapter/src/metrics.rs), [HTTPS consensus](rs/https_outcalls/consensus/src/metrics.rs), [Bitcoin adapter](rs/bitcoin/adapter/src/metrics.rs), [Bitcoin payloads](rs/bitcoin/consensus/src/metrics.rs) |
| Control plane and infrastructure | Registry/topology changes, routing/key changes, orchestrator/restarts/upgrades/CUP, firewall/trim, crypto service, GuestOS/QEMU/HostOS, filesystem/encryption/device contention | [Orchestrator](rs/orchestrator/src/processes.rs), [orchestrator metrics](rs/orchestrator/src/metrics.rs), [GuestOS service](ic-os/components/guestos/ic-replica.service) |
| Observability | Collector/metrics/logging/tracing CPU and memory; dictionary retention; serialization/compression/export bytes; UI/API serving; lost events and collector backpressure | [Monitoring](rs/monitoring), [existing viewer/API](rs/pocket_ic_server/src/profiling/server/memory_access_viewer.rs) |

Chain Fusion is a composed workflow through canisters, adapters, chain-key protocols and external networks, not an extra additive resource bucket. System canisters such as ledgers, registry and governance should likewise appear as workload identities through the same component model, rather than being double-counted as both guest work and another resource family.

## 4. Reuse existing telemetry, with its actual meaning

Start by importing current counters and histograms; add causal context and CPU attribution where missing. Do not rebuild an entire parallel metrics stack.

- `scheduler_compute_utilization_per_core` is computed from instructions and a configured slice limit. It is **instruction-budget utilization**, not measured CPU utilization: [calculation](rs/execution_environment/src/scheduler.rs), around line 765 at the baseline commit.
- Existing scheduler queue latency uses the IC clock. PocketIC virtual time must not be plotted as elapsed host time without a separate axis/domain.
- Sandbox metrics already separate replica-side prepare/wait/finish and sandbox execution/run wall durations, and count accessed/dirty pages, SIGSEGV, mmap/mprotect and copied pages. Controller waiting overlaps sandbox work; their durations are not additive CPU.
- `crypto_duration_seconds` observes elapsed duration with operation labels. Some IDKG duration metrics use milliseconds without a unit suffix. Import units from definitions, never infer all units from names.
- `mr_time_in_backlog`, `mr_time_in_stream`, stream counts/bytes, XNet build/validation/query durations and ingress limit counters are useful starting points. Histogram deltas cannot reconstruct individual call paths.
- `/_/tracing/flamegraph` provides a tracing-span view. In this checkout, `/_/pprof/profile` and `/_/pprof/flamegraph` are **disabled placeholders**; the available pprof collector would be in-process even if enabled. It would not cover sandbox, compiler, CSP or adapter processes automatically: [endpoint implementation](rs/http_endpoints/public/src/pprof.rs), [span flamegraph](rs/http_endpoints/public/src/tracing_flamegraph.rs), [CPU collector](rs/monitoring/pprof/src/lib.rs).
- Adapter metrics can be gathered through the existing adapter registry. PocketIC adapters can instead be tasks inside the server process; discover roles and actual topology, not a fixed executable list.
- State manager metrics cover checkpoint steps, hashing, manifest/tip queues, state sync and disk/storage work. These continue outside canister execution spans.

The metric importer needs a versioned registry: source name, source code reference, unit, counter/gauge/histogram semantics, labels, reset behavior, clock domain, aggregation rules, scope and known coverage limitations. Keep request/canister/peer identities in bounded traces, not unbounded Prometheus label sets.

## 5. How to collect physical resources

### Collection levels and ownership

1. **Portable baseline:** process/task registration, existing IC metrics, monotonic lifecycle markers, available process/thread CPU counters, faults, context switches, memory and I/O snapshots. Export missing platform support and permissions. Cheap counter sampling should be configurable; expensive mapping scans run less frequently and at boundaries.
2. **Isolated Linux run:** launch the server/replica inside a delegated cgroup v2 before creating descendants. Use its CPU, memory and I/O envelope to include short-lived children. Keep the collector and load generator outside, while measuring their own overhead separately. Register independently launched services such as CSP and adapters through an explicit ownership manifest; they need not be replica descendants.
3. **Diagnostic mode:** optional native/JIT stack sampling, perf events, scheduling/off-CPU traces, and block/socket tracing where available. Record event losses, unwind failures, event multiplexing, permissions and unsupported features. Basic capture must remain useful without these facilities.

Isolated-run, shared-server observation and attached-replica modes must be distinct. Moving an existing parent into a cgroup does not move existing children or reassign old page-cache charges. A shared PocketIC process can serve multiple instances and adapter tasks, so its process-level memory and CPU cannot automatically be divided exactly among them.

### CPU and component attribution

Collect user and system CPU time, process/thread identity and lifetime, affinity, eligible CPUs, cgroup quota/ancestor restrictions, throttling, scheduler runnable delay, context switches and available pressure/steal/frequency context.

```text
average core-equivalents = observed CPU seconds / host elapsed seconds
capacity utilization = observed CPU seconds / eligible CPU-seconds
```

For changing quotas/affinity, integrate eligible capacity over time. State the denominator; SMT logical CPUs are not a promise of linear physical throughput. Hardware-retired instructions, IC-priced instructions and CPU seconds are three different quantities. An idle execution worker can coexist with a saturated hash/crypto worker or a stalled disk queue.

Add low-cost CPU scopes around synchronous component work using thread CPU clocks. For asynchronous work, account only each entered/polled segment on its current thread. Never read a thread clock before an `await` and subtract one from another thread after resumption. Propagate scope context into worker pools and sandbox/compiler/CSP IPC.

Use the existing Rust `tracing` infrastructure with a dedicated, opt-in profiling sink and an explicit component registry. Logging filters must not silently decide profiling coverage. CPU-segment accounting can be attached to span enter/exit or dedicated synchronous guards; worker jobs and IPC carry optional local profiling context. Register process and detailed thread birth/exit, including lifetime IDs. Hot signal-handler paths keep bounded counters/preallocated events rather than ordinary tracing machinery. Independently sample native/JIT stacks when supported to find work outside explicit scopes; report missing JIT symbols/unwind coverage.

Maintain exclusive component CPU ownership per running thread. Inclusive scopes remain useful for navigation but are not additive. A controller's wait span and a sandbox's active execution overlap intentionally. A DTS `instance.run` span can contain condition-variable waits across rounds; split active slices from pause/queue time.

Use process/cgroup counters as the independent reconciliation envelope:

```text
observed CPU = exclusive attributed CPU + runtime/shared CPU + unattributed residual
```

Include synchronization uncertainty and snapshot skew. Show discrepancies; do not force reconciliation by spreading residual CPU according to wall-span proportions. Sampling provides estimates with sample counts, interval, losses and estimator metadata; it does not automatically provide valid confidence intervals or instruction-level precision. Guest/JIT source attribution needs supported JIT symbol registration and available source maps; missing symbols remain unknown.

### Memory and storage

Expose these separately:

- Logical Wasm heap/stable capacity, reservations and memory rent.
- Virtual address-space/mappings, including code caches and sandbox/allocator pools.
- Resident anonymous/file-backed/shared memory, page cache/shmem, dirty/writeback, swap and pressure.
- Cgroup charged memory where available; PSS/private/shared mapping breakdown for attribution. Summed process RSS double-counts shared pages and is not unique physical RAM.
- Logical file sizes, allocated physical blocks, overlays/checkpoints/snapshots, and deleted-but-open backing files. Directory size alone misses deleted-open `.mem` files and memfd-backed allocations.
- Peaks and retained footprint, with measurement resolution. Do not sum per-process or per-node peaks and call that a simultaneous aggregate peak.

Collect `memory.current`, available peak/stat/event counters, CPU/memory/I/O pressure, and selected `/proc` mapping data. Shared file-cache charging has ownership limitations: cgroup memory is the measured charged footprint, not perfect marginal or globally unique memory cost. Preserve the metric's source and boundary in the UI.

For storage, distinguish **requested application bytes**, **syscall bytes**, **storage-layer accounting**, and **device-completed bytes**. Capture read/write IOPS, throughput, request/flush latency where available, queue depth, faults, dirty/writeback and filesystem/device topology. Preserve logical 4 KiB stable-granule counts and native 64 KiB region events alongside physical observations.

Mmap faults, readahead, page-cache hits, delayed writeback, merges and asynchronous checkpointing prevent exact physical I/O attribution from a canister's syscall deltas alone. A SIGSEGV from intentional IC memory tracking is not itself an SSD miss; Linux minor/major faults are separate counters. Fault counts are not bytes read. Never interpret the 80,000-instruction regional charge as evidence of a 64 KiB SSD transfer.

Linux pitfalls to encode in collectors and tests:

- `/proc/<pid>/io` includes waited-for children; summing parent and child values can double-count. `rchar/wchar` differ from storage `read_bytes/write_bytes`; canceled writes complicate interpretation.
- `RUSAGE_CHILDREN` covers terminated/waited descendants, not all live descendants. `ru_maxrss` is a peak; its child value is not the simultaneous tree footprint.
- Device-mapper/striped-volume and underlying-drive counters describe layers of the same I/O. Select a topology layer instead of adding both.
- Busy-time percentage alone cannot establish saturation of parallel NVMe storage. Combine it with latency, queueing, throughput and pressure.
- Detect process exit/reuse and all counter resets. Register birth/exit at the sandbox launcher; polling alone loses short-lived compilers and sandboxes.

### Network

Collect payload bytes, encoded request/slice/witness bytes, transport/RPC bytes, retransmissions and physical-interface bytes as separate layers. Per-peer/subnet queueing and flow-control limits matter as much as throughput. Network work includes loopback/IPC and external adapter traffic, but each retains its transport kind.

`/proc/PID/net/dev` is network-namespace data, not the PID's personal traffic. Owned network totals require a suitable namespace/veth boundary or socket/cgroup-aware instrumentation. A host NIC counter can include unrelated work; report it as host context, not automatically as this run's network bill.

### Observation and drain windows

Each run has setup, optional warmup, measured foreground, optional bounded drain, and post-run observation phases. Keep their resources separate. Record whether warmup is included, and how checkpoints, code/query caches and storage-cache state were established.

Report both **requested operations finished** and **selected drain finished**. Drain policies can wait for callbacks, stream acknowledgements, queued hash/manifest/checkpoint tasks and selected flush barriers, with deadlines and pending-work counts. An application-visible reply or `flush_tip_channel` is not proof that all kernel writeback is complete or that storage is durable/quiescent. A stronger flush changes the workload: record it and account its cost separately. Pending maintenance is a result, not silently discarded work.

Background jobs carry origin run/height/canister links when known. Shared block construction, certification, merges and presignature replenishment retain measured batch/job totals. Optional allocations to requests must name a policy such as bytes or operations; they are labeled **allocated**, not directly measured. Keep unattributed/shared totals visible.

## 6. Causal trace model

Use a causal graph, not one enormous nested span. A batch may have many initiating messages; a deferred worker can outlive the originating call. Parent spans express ownership where appropriate, while explicit links express batch membership, fork/join and asynchronous causality.

Required identities and event fields:

| Group | Fields |
| --- | --- |
| Capture and topology | Schema/run/topology IDs, server generation, backend and operation path, subnet/node IDs, observed-node set, hardware/config/binary hashes |
| Process/task | Host boot ID, PID plus process start time/incarnation, TID plus thread lifetime/incarnation, process role, async task/job/IPC correlation IDs |
| Logical work | Ingress request ID, separate envelope/authentication fingerprint, canister incarnation/module hash, call context and callback IDs, logical execution ID, physical attempt ID, slice ID |
| Consensus/XNet | Height/round, block/artifact identity, registry version, source/destination subnet, stream index/range, certification height, pull/pool attempt |
| Event | Stable component/operation ID, span ID, parent and causal links, phase/result/reason, local monotonic timestamp, optional UTC mapping plus uncertainty, distinct IC virtual time |
| Resource/evidence | Counter values and units, CPU segments or samples, attribution kind, collector/source version, coverage/missingness, sampling/loss details, evidence ID |

Message IDs identify request content, not necessarily the entire authentication envelope. A verification investigation/cache key must also distinguish scheme, key/signature/delegation material and validation context. Record safe fingerprints rather than secret data or full arguments by default.

Outgoing requests already contain sender/receiver and `sender_reply_callback`; responses carry `originator_reply_callback`. Combine these with run and canister incarnation, and record parent mappings at enqueue. Existing `RequestMetadata` contains depth/start time, not a unique trace ID. Instrument sidecars and existing boundaries first; do not require changing consensus-visible payloads just to obtain local tracing.

The XNet lifecycle must remain navigable in both directions:

```text
caller execution -> output enqueue -> routing/stream position
  -> certified slice construction/pull/validation -> incoming induction
  -> callee input queue -> execution/commit -> response output
  -> return stream/induction -> callback queue -> callback/cleanup
```

Add transport/pool/verification events only when those real paths occur. Include acknowledgement, rejection, expiry, retry, dropped work and reservation/refund events. Same-subnet direct induction has its own path; it must not look like a network XNet hop. Keep attached/transferred/refunded cycles distinct from fees actually burned.

Collect enqueue, worker-start, active segment, suspend, resume, finish, commit/result-publication and cancellation events. Separate queue wait, runnable scheduling delay, protocol wait, network wait, blocking I/O and active CPU. Thread sampling/off-CPU data refines these labels; a long unexplained wall span remains unexplained.

Use host monotonic clocks for local duration. Maintain clock snapshots with uncertainty for cross-host alignment; do not infer one-way network latency when clock error is comparable to the interval. Causality remains useful without exact clock alignment. Show an **observed path/lower bound** when missing edges or incomparable clocks prevent establishing an exact critical path. Async child CPU is not automatically on the parent's wall critical path.

Deduplicate logical work when combining replicated traces, but retain each node's physical observations. Sum observed node CPU only across explicit nodes/windows; show missing nodes. Never multiply one node's counters by thirteen and label the result measured. If displaying a model-based extrapolation, expose that model and label it estimated.

## 7. UI design

Extend the existing compact viewer instead of making a second unrelated dashboard. The starting references are [Memory](rs/pocket_ic_server/src/profiling/viewer/design/reference-memory.png), [Compute](rs/pocket_ic_server/src/profiling/viewer/design/reference-compute.png), and the current [live-viewer capture](../demo/pocketic-live/memory-20261026-data.png). Preserve the large memory visualization and existing Data/Reads/Writes behavior when drilling into a canister.

Add one primary **System** workspace beside **Memory / Compute / Executions**. System contains **Overview / Calls / Components / Resources / Pricing**. Deep resource details have secondary CPU / Memory / Storage / Network views rather than one dashboard showing every chart.

Proposed frame:

```text
PocketIC   System | Memory | Compute | Executions       Run v   Export
Scope: observed nodes/processes · subnet · interval · clock · coverage
Overview | Calls | Components | Resources | Pricing
CPU core·s / avg core-equivalents | RAM peak | storage R/W | ops/s + p95
+---------------------------------------------------+------------------+
| Active visualization: timeline, graph or table     | Summary/Evidence |
|                                                   | selected item    |
| One compact companion chart where useful          |                  |
+---------------------------------------------------+------------------+
Shared time-range brush · selected track · zoom · live/pause
```

The scope strip includes host elapsed time, measured node/process boundary, logical subnet, selected interval, clock domain and a concise coverage badge. A 13-node pricing configuration must never resemble thirteen measured replicas. Use four fixed summary-card slots; the default emphasizes actual CPU, actual memory, storage traffic and useful throughput. Users can choose other metrics without changing the layout. Every card names its unit/source, and an unsupported metric shows unavailable rather than zero. Prefer cgroup memory when present; do not label summed RSS as aggregate RAM.

Throughput and latency name their operation/cohort: root requests, application-defined useful operations, attempts, XNet messages and callbacks are different counts. Default request throughput counts completed root requests per host elapsed interval; application batch-item throughput requires an explicit workload counter/definition. Latency results report N, start/completion-window policy, timed-out, censored and still-incomplete operations, including calls crossing the selected interval. Do not silently exclude slow pending calls and present a biased p95. Label empirical versus histogram/sample estimates; unsupported quantiles remain unavailable.

### Views and visual encodings

| View | Primary visual | Questions answered |
| --- | --- | --- |
| Overview | One observed component/process timeline, compact exclusive-CPU allocation, resource summary and visible shared/residual segment | Where did this interval go? What is incomplete? |
| Calls | Stable subnet/canister swimlanes with request/response arrows; expandable attempts/slices; queue/wait intervals | Which call chain or XNet stage delayed completion? |
| Components | Sortable exclusive CPU table and optional native sampled flamegraph, expanded by process/thread/operation | Authentication, scheduling, setup, guest code, hashing or adapters: which consumed CPU? |
| Resources | Synchronized CPU, memory, storage or network plots, with optional canister×time footprint heatmap | Which resource is saturated or growing, and when? |
| Pricing | Aligned logical charges, round-budget use and physical observations; controlled-run scatter/ratio comparisons | Which workload/path has high physical cost per charged unit, or the reverse? |

Use filled segments for measured active work and outlined segments for waits. Native CPU sample stacks and the existing priced-instruction mosaic are different visualizations with different units. In Calls, prefer stable swimlanes to a force-directed graph that rearranges on every update. Collapse repeated edges with explicit counts, preserving drilldown to exact calls and stream positions. Graph edge widths may encode counts or bytes, with the selected encoding stated; never infer CPU from edge width or latency.

Show shared/background work, unattributed residual, missing collectors, unobserved nodes and truncated trace intervals explicitly. Use subdued gray/hatching plus text or icons, so missingness and categories do not rely on color alone. Red denotes errors or observed limit violations, not empty padding. Keep component colors stable; resource colors must not silently change the established byte-mask palette. Tooltips and Evidence show units, clock, collector, attribution method, source and uncertainty.

Keep the body within **1440×900 and 1366×768** for the primary workflow. Use compact 12–13 px interface text where legible, tabular numerals, bounded internal table/panel scrolling and a fixed inspector width. Advanced fields live in Evidence or their relevant tab. Selecting an item, changing tabs, live updates and opening details must not shift chart geometry. Freeze ordering during live inspection unless the user requests a resort. Pausing preserves the window and selection.

Click/Enter selects an item; keyboard navigation moves among items, and Escape returns from drilldown. Carry run, window, canister, request, attempt and slice context into Memory/Compute/Executions and back. The same selected interval drives charts, summaries, tables and exports. Distinguish host monotonic time, cross-host aligned time, IC virtual time, round ordinal and instruction position; they are not interchangeable axes.

First paint must load a small run summary and aggregated timeline. Fetch detail, source maps and dictionaries lazily by content hash; use incremental cursors, server-side aggregation/decimation, compression and explicit loading states. Do not repeat the earlier 143 MB initial-response failure. A clear empty state distinguishes “no matching activity”, “loading”, “path emulated”, “collector unavailable” and “trace details dropped”.

The UI consumes the same immutable query results as agent tools. Rendering must not invent events, CPU allocations or causal edges. Decimation reports its resolution and aggregation method; short expensive events remain discoverable through tables and exact queries.

## 8. AI-agent interfaces and capture format

Make structured inspection available with the first useful capture implementation, not after the UI. Agents should not need to read Markdown or download a complete raw trace to answer a bounded question. Extend the built-in `explain`, schema discovery, JSON API, MCP and offline reader consistently.

The following operations are **proposed API/tool names**:

| Operation | Result |
| --- | --- |
| `profile_start`, `profile_status`, `profile_stop` | Capture configuration, capabilities, run ID, current windows, bounded drain result and pending work |
| `profile_summary` | Resource envelope, useful-work throughput/latency, logical charges, major components, shared/residual and coverage |
| `profile_components` | Filtered component/process/thread/operation rows with exclusive CPU, separate inclusive scope and supporting evidence |
| `profile_flows` | Cross-canister/subnet requests, callbacks, transport stages, attempts and observed critical-path links |
| `profile_resources` | CPU/memory/storage/network series with units, resolution, source and sampling/aggregation metadata |
| `profile_limits` | Actual configured limits and events: requested/remaining amount, scope, stopping reason, backlog and binding limit |
| `profile_pricing` | Existing instruction/cycle/rule ledger joined to physical observations at comparable scopes |
| `profile_compare` | Comparable run/window/workload deltas; configuration differences, uncertainty and missing coverage |
| `profile_evidence`, `profile_trace_slice` | Bounded exact records, source references and causal neighborhood for a cited finding |
| `profile_render` | SVG/PNG of a named view/query plus machine-readable legend, layout and element-to-evidence mapping |
| `profile_explain` | Self-contained component/metric dictionary, units, accounting rules, source hooks, capabilities and known limitations |

The HTTP equivalent can live under `/api/profiling/runs`, alongside the existing canister APIs. CLI commands should offer the same start/status/stop/summary/components/flows/resources/limits/compare/render/export operations. A Rust client should support run-scoped start/finish guards and structured results; cancellation/drop must report an incomplete run rather than fabricate successful drain. No existing API is claimed to provide these new operations yet.

Every query accepts a bounded selection: run, observed nodes/subnets/process roles, canisters, request/callback/execution/attempt/slice, component, interval/clock and desired detail. Supply limit/cursor and resolution controls, stable ordering, batched independent lookups and a query ID/hash. Summary calls return bounded top rows and totals, with access to the omitted remainder. Do not support only top-N tables that make residual costs disappear.

Every result includes:

```text
schema_version, run_id, query_id, capture/binary/config identity
scope: observed nodes/processes, logical subnet/canister/call selection
window: phase, clock domain, start/end, alignment uncertainty
metrics: value, unit, aggregation, source, attribution kind
coverage: implementation/path, availability, completeness, missing reasons
quality: sampling, dropped data, unresolved joins, counter resets, overhead
accounting: included/excluded categories, shared/unattributed totals
evidence_refs, dictionary_hashes, pagination/decimation metadata
```

Large integers use lossless encodings compatible with the existing decimal-string JSON conventions. Missing values are explicit nulls with reasons. Separate event loss, sampling, source-map truncation, execution-stack truncation, call-link gaps and host-collector absence; one generic `truncated` bit is insufficient.

Store a versioned `.icprofile` bundle containing a small manifest, summary/index, typed event/counter/sample chunks, deduplicated dictionaries and source/config provenance. Compression and random access are required design properties. Preserve raw imported counters and unit mappings so future analysis can be reproduced. Streaming retention must be bounded and configurable by data class; publish dropped counts/ranges while retaining independent aggregate counters where possible. Signals and hot paths use preallocated/bounded recording rather than allocations, heavy locks or synchronous export.

Provide interoperable exports rather than requiring this viewer for every analysis: OpenTelemetry-compatible spans/links for causal operations, Perfetto tracks/flows/counters for timelines and scheduling evidence, and a supported CPU-profile format for native sampled stacks. These are projections of the same capture, with coverage and unsupported fields documented; none replaces the logical pricing ledger or independent host envelope. Choose exporter/collector dependencies after an overhead pilot, keeping summary capture usable without a heavyweight tracing service.

Suggested agent workflow:

1. Capture a named workload over selected canisters/subnets; record setup/warmup and drain policy.
2. Read summary and coverage. Determine whether the question can be answered by this backend.
3. Query the largest exclusive components or the slowest observed call chain.
4. Retrieve exact evidence for the relevant validation, execution, XNet, storage or limit events.
5. Join physical observations to existing instruction/cycle rules; label hypotheses and allocation estimates.
6. Compare a controlled alternative and render a targeted image if spatial/temporal patterns help.

Rendered images are a first-class agent output. Accept an immutable query ID, view, dimensions, selected IDs, range and scale; return the image plus its legend/units, coverage, binning, geometry and event-ID map in JSON. Offer compact timeline, component CPU, call swimlane, resource chart, pricing comparison and memory-footprint heatmap views. Repeated renders preserve ordering/scale when requested for comparison. Existing byte snapshots retain their exact page/address geometry and separate Data/Reads/Writes views. Legends can live in the sidecar for image-based agent analysis rather than filling the image with explanatory prose.

For an immutable query, render-only range/selection options crop or highlight without changing its underlying filters or totals, and the sidecar identifies the visible subset. Any option that changes filtering, aggregation or accounting scope produces a new query ID/hash with revised scope/totals. This prevents an image and its accompanying evidence from describing different measurements.

Use stable component/metric/rule identifiers for evidence citations. A finding such as “authentication dominates” must name the observed interval, CPU attribution method, validation stages, verification counts, sharing/coverage and evidence IDs. Automated findings are candidates with alternative explanations, not unsupported declarations that pricing or a limit is wrong. Low whole-node CPU alone does not prove a bad scheduler limit: a single hot canister is sequential, other workers can be idle, and the actual bottleneck may be memory, I/O, locks or insufficient offered load.

## 9. Experiments tied to the current suspicions

Run controlled comparisons against the same application build, state, offered load and useful operations. Record rejected/failed work too. Report source defaults versus actual captured configuration explicitly; no experiment should silently assume the defaults are every subnet's live settings.

| Question | Comparison and required evidence |
| --- | --- |
| Is authentication repeated unnecessarily? | Signed versus anonymous requests, by native scheme and validation stage; count actual verification invocations, cache/skipped results, CPU, bytes and delegation checks. Distinguish request-content identity from envelope identity. Compare any future bounded crypto-result cache while retaining context-dependent validation. |
| Is repeated message entry the bottleneck? | Empty raw-Wasm entry, small real application operation, N separate ingress calls and an explicit application batch. Measure Store/instance/System API setup, IPC, wrapper work, page taxes, guest CPU, commit/rollback and queueing. Match semantics or state their differences; one atomic batch is not N independent transactions. |
| Are round limits conservative for useful workloads? | Capture actual round budget/configuration, per-slice deductions, worker CPU and queues, stopping reasons and offered demand. Defaults include 2M round instructions per executed slice, 8M per canister scheduling pass, and an effective initial short-message budget near 2B with a nominal 4B maximum. Ingress's 1,000/block admission cap is separate. Test a measured implementation optimization before interpreting a lowered allowance as increased physical capacity. |
| Which SDK/layout pages dominate? | Join existing page charges/masks to enough source attribution, source markers or symbols to distinguish wrappers, allocator/stack, SDK and business state. Compare repeated messages to batching. Do not infer “four SDK pages” from a page image alone. |
| Does stable-read pricing oppose locality? | Same-granule header/body reads versus a combined buffered read and distinct-granule reads; direct/native path counts, requested bytes, logical granules, native regional charges and measured CPU/I/O. A distinct untouched granule can retain the direct path when a repeated access would not. Record both the pricing benefit and any extra physical footprint. |
| Are sparse first reads under-accounted for physically? | Populated persisted pages; hot versus verified cold/oversized working sets; random versus sequential locality; varying useful read sizes, competing canisters and pressure. Observe faults, storage traffic, CPU, p95/p99 and useful throughput alongside instructions. Do not mistake holes/zero pages, a new execution, or disabled query-result caching for cold physical storage. |
| Where do multi-canister costs appear? | Same-subnet versus cross-subnet request/response chains, fan-out/fan-in, varying message sizes, queues, callbacks, failures, best-effort expiry and refunds. Separate encoding/verification/CPU from transport/round waits and external latency. Include real-replica transport tests rather than treating PocketXNet transfers as wire measurements. |
| Does background work explain regressions? | Quiet versus checkpoint/hash/merge/state-sync/GC load, signature presignature replenishment, registry activity, query competition and adapter work. Observe foreground completion plus bounded maintenance and retained footprint. |
| Which lifecycle events are hidden by steady-state tests? | Cold/warm compilation, sandbox reuse/eviction, install/upgrade/snapshots, query cache state and adapter connection reuse. Show setup, foreground and drain separately. |

Existing tests and benchmarks give useful baselines but do not settle these questions. The real ICP ledger canbench performs 10,000 direct transfer-function calls inside one execution: [benchmark](rs/ledger_suite/icp/ledger/src/canbench/benches.rs). That amortizes the message boundaries under investigation. Stable benchmarks repeatedly query retained checkpointed state: [stable-memory cases](rs/embedders/benches/stable_memory.rs), [driver](rs/embedders/benches/embedders_bench/src/lib.rs). This is not proof of node-scale cold-cache behavior. Conversely, the presence of these benchmarks contradicts a claim that IC testing has no real applications.

Our measured PocketIC ledger transfer is a useful initial fixture: 1,428,474 instructions, including 720,000 heap-page and 320,000 stable-page charges, or 72.8% page charges. It establishes a priced-work distribution, not a CPU-time distribution, and its disabled origin map prevents SDK/source blame. RAM-backed PocketIC preserves logical charges but cannot establish SSD sustainability.

### Observer effects

Record profiler feature flags separately: aggregate system capture, native CPU scopes/sampling, instruction details, source maps, read/write byte masks and export. **Do not automatically enable expensive per-instruction or byte tracing for every physical-resource experiment.** Such instrumentation changes CPU, cache, memory and I/O behavior even when logical charges reconcile.

Compare profiler-off, baseline and detailed modes over repeated runs, including run-to-run variance, latency, throughput, CPU, footprint and trace bytes. Keep exporter/collector/UI work identifiable; avoid exporting during a measured interval unless that is intentional. Do not subtract one constant “profiling overhead” and claim exact uninstrumented timings: instrumentation can change interactions nonlinearly. Publish the measured overhead and scope of each result.

Record state backing, log/export locations and swap configuration independently. RAM-backed state does not imply zero host disk traffic: logs, exported profiles, unrelated workloads or swapping can still perform I/O.

## 10. Deterministic cache-aware pricing: research enabled by the tool

The determinism constraint is a starting requirement: actual disk-cache hits, wall time and OS scheduling must not directly determine replicated billing. The useful proposal is to test **deterministic expected reuse**, such as a page reused by consecutive calls, against measured physical behavior.

Equal hardware does not synchronize Linux page-cache residency, query-induced eviction, readahead, background writeback, state-sync/restart state or host contention. A deterministic logical discount can nevertheless be designed independently of those physical differences.

The profiler should export the logical access sequence, reuse distance, page identity/version, direct/native path, bytes and configured memory budgets alongside physical observations. An offline logical-cache simulator can then evaluate candidate policies without changing execution or billing.

A candidate model must specify:

- Scope and capacity: per-canister, subnet-wide or another explicit domain, with bounded state and metadata cost.
- Page granularity and identity, including canister incarnation, writes/version changes and invalidation.
- Admission, eviction and recency order based on deterministic logical events. Concurrent worker completion order and measured time cannot determine shared LRU order.
- Lifetime across calls, rounds, traps/rollback, upgrades, checkpoints, restart and state sync; recovery must reproduce the same model state.
- Interaction with DTS attempts, retries and state that was read before a trap.
- Nonreplicated queries: local query access cannot silently mutate a replicated pricing-cache state.
- Whether a discount predicts likely reuse or guarantees residency; physical caches remain observations, not synchronized replicas of the logical model.
- Independent physical safety bounds and the resource cost of maintaining the model itself.

Compare candidate logical hit/miss predictions with physical faults/I/O and throughput across nodes. Evaluate adversarial and ordinary locality patterns, fairness between canisters, metadata cost, and whether the model rewards efficient packing rather than artificial page spreading. Keep economic coefficients, deterministic resource budgets and emergency safety limits separately justified. This document proposes measurement and offline modeling, not an enacted pricing change.

## 11. Implementation sequence and acceptance criteria

| Stage | Deliverable | Exit criteria |
| --- | --- | --- |
| A: run envelope | Run lifecycle/capabilities/configuration, host/process ownership, available CPU/memory/I/O counters and imported IC metrics; isolated and attached modes; bounded bundle; summary API/MCP/export and System overview | Honest full-run totals and missingness; short-lived child resources retained where supported; no false real-network/crypto claims; no first-load raw-trace download |
| B: causal execution and storage | Component CPU segments, scheduler/entry/DTS/callback identities, persistence jobs, stopping reasons and joins to existing charges; Components/Resources and existing canister drilldowns | Exclusive CPU plus residual reconciles within documented measurement error; async/migration/nesting tests pass; deferred work/pending drain remains visible |
| C: network and replica detail | Ingress-stage authentication, same-subnet/XNet flow graph, real-replica consensus/P2P/certification, CSP/adapters/control-plane collectors; Calls and cross-node coverage | Requests/responses/timeout/refund paths join correctly; shared batch costs remain explicit; simulated and actual transport are distinguishable; missing nodes/clocks do not produce false exact paths |
| D: calibration and comparison | Controlled experiment suite, Pricing comparisons, optional perf/off-CPU/block diagnostics and logical-cache simulator | Reproducible workload/configuration comparisons with uncertainty and observer-effect results; no automatic “fair price” or arbitrary limit-change conclusion |

Keep all subsystem families in the capability dictionary from stage A, including not-yet-instrumented families. Stage boundaries prioritize implementation; they must not hide missing systems.

Validation requirements:

1. **Accounting:** nested scopes, parallel workers, async task migration, task cancellation, process/thread ID reuse, short-lived children, parent-child resource double counting, counter resets, clock skew and partial nodes.
2. **Coverage:** direct versus HTTP ingress; direct payload execution versus payload building; real versus emulated XNet verification/transport; mocked versus configured adapters; local chain-key replies versus real distributed signing.
3. **Execution:** updates, queries/cache/inspection, callbacks/cleanup, timers/heartbeats, DTS pause/restart/trap, management operations and compilation-cache states.
4. **Storage:** shared mappings, memfd and deleted files, logical versus resident memory, read/write amplification, asynchronous checkpoint/manifest/hash/GC, drain timeout and outstanding writeback.
5. **Robustness:** unsupported OS facilities, missing permissions, sample loss, broken joins, retention overflow, module/stack truncation, stale generations, concurrent runs and collector failure. Failures must not change canister semantics or silently erase coverage warnings.
6. **UI/agents:** primary workflow fits both target viewports without body scrolling; no selection/live-update layout shifts; keyboard/drilldown context survives; identical query results drive JSON, tables and rendered images; zero/missing/sampled/truncated states remain distinguishable. Test latency cohort/window/censoring behavior, and ensure render crops preserve query identity while actual query changes receive a new identity.
7. **Performance:** startup/steady-state capture overhead, first paint, bounded memory/disk retention, lazy dictionaries, compressed/paginated exports, and profiler-off/baseline/detailed comparisons. Set numerical performance budgets from a recorded pilot, not invented confidence thresholds.

## 12. Reference material and provenance

The source links throughout refer to the committed local baseline. Source defaults describe that revision, not an independently audited live subnet configuration. Four subsystem researchers mapped networking, execution/storage, host telemetry and peripheral/control-plane services; separate host-accounting and viewer reviews checked measurement semantics and interface design. This is a research specification, not a new whole-system benchmark result or an implementation claim.

Primary external references for collection and trace semantics:

- [Linux cgroup v2](https://docs.kernel.org/admin-guide/cgroup-v2.html), [PSI](https://docs.kernel.org/accounting/psi.html), [block statistics](https://docs.kernel.org/block/stat.html), [I/O statistics](https://docs.kernel.org/admin-guide/iostats.html), [perf security](https://docs.kernel.org/admin-guide/perf-security.html).
- Linux manual pages for [process I/O](https://man7.org/linux/man-pages/man5/proc_pid_io.5.html), [resource usage](https://man7.org/linux/man-pages/man2/getrusage.2.html), [CPU clocks](https://man7.org/linux/man-pages/man2/clock_gettime.2.html), [smaps/PSS](https://man7.org/linux/man-pages/man5/proc_pid_smaps.5.html), [network namespaces in proc](https://man7.org/linux/man-pages/man5/proc_pid_net.5.html).
- [Tracing async span lifecycle](https://docs.rs/tracing/latest/tracing/span/index.html), [OpenTelemetry span links](https://opentelemetry.io/docs/specs/otel/overview/), [Perfetto CPU scheduling](https://perfetto.dev/docs/data-sources/cpu-scheduling), [Perfetto query model](https://perfetto.dev/docs/analysis/perfetto-sql-getting-started).
- IC [cycle costs](https://docs.internetcomputer.org/references/cycle-costs/), [resource limits](https://docs.internetcomputer.org/references/resource-limits/), and [message execution properties](https://docs.internetcomputer.org/references/message-execution-properties/).


## 13. Consolidation and inactive profiling (2026-10-27)

The implementation is now collected under
[`rs/pocket_ic_server/src/profiling`](rs/pocket_ic_server/src/profiling/README.md):
`server/` holds APIs, operations, CLI, catalogs and snapshots; `viewer/` holds
browser/agent tooling; `system/` is the independent `ic-system-profiler` crate.
Existing public module aliases and REST interfaces remain usable. The integration
README maps the compiler, execution, schema and component hooks that necessarily
remain with their owning IC crates. Eight async integration files now differ
from the original by 37 added lines and no removed lines, replacing the earlier
601 additions/545 deletions caused by wrapping/reindenting their bodies.

Inactive canister execution uses ordinary Wasm instrumentation. Source profiling
and byte masks select a separate diagnostic compilation/cache namespace lazily;
page-only tracing attaches a host debit observer lazily. Bounded source/mask
exhaustion selects ordinary Wasm again, while page tracing can continue until
explicitly stopped. Ordinary instances allocate no page-charge collector.
Recorder checks avoid the history lock when inactive, and fee/logical-context
metadata is evaluated only when requested.

Whole-system child activation uses a read-only shared mmap control word.
Initialized inactive hooks do not read files/clocks, initialize profiler TLS or
allocate records; idle collectors park. `IC_SYSTEM_PROFILER_DISABLED=1` explicitly
disables whole-system captures. No physical observation feeds deterministic
charges or replicated state. PocketIC also shares its immutable OpenAPI schema
through an Arc instead of cloning the full schema for every HTTP request.

`pocket-ic explain` now embeds all 185 charge rules and the 20 system-profile
commands, Rust examples, limits, recording modes, RAM/origin/retention flags,
metrics/PSS configuration and coverage guidance. `pocket-ic profile explain`
continues to provide the system dictionary. The installed local binary is
PocketIC 16.1.0, SHA256
`99318a0ce5eb538a2cbd7d3435138ca9a60525ddb332413b23f7d498342d93ad`.

Validation completed with 227 focused Rust checks, 48 Python tests and 38 browser
tests. The 43 instrumentation checks use snapshot files unchanged from upstream.
The unrelated full serialized-module round-trip proptest was stopped; it is not
counted as a completed suite.

For a direct inactive-path comparison, untouched upstream
`ee8086a9773725fd1058832a9400297cf0990112` and the modified build used the same
Rust 1.99 development profile, linker flags, host and disk backing. Nine
alternating pairs ran 400 calls per build, with identical guest instruction
counter replies and final cycles. Mean paired wall time was 16.1% lower and
process-tree CPU was 15.0% lower in the modified build. This validates that
workload; it is not a claim of mathematically zero overhead for all workloads.
A separate idle-versus-hard-off comparison measured +0.7% wall time with an
approximate paired interval of -0.3% to +1.7%.

The frozen Heavy16 replay also passes: 18,000 roots, 19,200 total workflows,
594,000 steps and 2,434 batches, with zero traps/rejects and exact previous cycle
totals (363,878,013,600 primary; 363,897,284,838 including maintenance). Its primary
phase took 131.65 seconds in this development build. The historical 41-second
result has not been qualified against matching compiler/configuration settings.

Local receipts, raw paired samples, build/test logs and the Heavy16 validation
are in `/srv/shared/demo/pocketic-refactor-20261026/verification-summary.json`
and adjacent files. The clean upstream build and benchmark are separate from
production-node performance or instruction-price calibration claims above.

## 14. Implemented scheduler round view (2026-10-27)

System → Rounds now displays captured subnet scheduler rounds, parallel worker
instruction progression, canister passes and individual DTS slices. It separates
guest instructions, per-slice overhead, canister preparation, dirty-page-copy
charges and finalization. Captured configuration, remaining counters, stopping
reasons and detail coverage accompany the charts. A separate byte gauge shows
cumulative logical heap delta against the scheduler's dirty-memory gate; this is
neither RSS nor physical disk traffic.

The application defaults are **40B per message**, **2B per DTS slice** and
**4B nominal per worker round**, with four scheduler workers. The initial
dispatch counter is 2,000,000,001 after reserving a final slice. Each inner
iteration charges the maximum worker consumption plus global finalization,
rather than summing all parallel worker work. The outer scheduler's 200,000,000
byte heap-delta gate is checked between iterations against cumulative round
work, despite the configuration field's per-iteration name; finishing an
iteration can overshoot it. These are captured configuration values, not a
claim of one pooled 40B subnet budget or a hard 2B canister-round quota.

Dedicated bounded round snapshots preserve accounting independently of generic
event loss. CLI `pocket-ic profile rounds`, MCP `profile_rounds`, the Rust client
and the rounds JSON API expose the same data. Compact overview queries and
selected full snapshots keep large runs usable; immutable SVG/PNG/JSON exports
support agent inspection. Old captures report missing measurements explicitly.

The frozen RRT Heavy16 replay passed its original assertions, exact work/cycle
comparisons and source-integrity checks: 18,000 roots, 19,200 workflows, 594,000
steps and 2,434 batches, with no traps or rejected ingress. All 48 primary rounds
have complete, reconciled budget details. Peak dispatch usage was 94.89%
(round 317); no round exhausted that counter. Nine rounds stopped at the
heap-delta gate, with a maximum of 249,921,536 bytes (round 321). The largest
slice used 208,759,474 instructions, 10.44% of its nominal 2B slice limit.
Cycle totals exactly match the frozen control: 363,878,013,600 primary and
363,897,284,838 including maintenance. The profiled primary phase took 219.24
host seconds in this development build; it is not a production throughput
measurement or an unprofiled timing comparison.

Validation passed 59 core, five scheduler, 90 server, five native integration
and 43 browser checks. Browser inspection covered both 1366×768 and 1440×900
without outer-page overflow or JavaScript errors. The installed 16.1.0 binary
is SHA256 `0ab8cc4254581fbb13a8fca2c36054b6b1f830c0706d4eac093477c06215dde7`.
Receipts, the complete capture, round analysis and screenshots are under
`/srv/shared/demo/pocketic-round-limits-20261027/`. The live viewer is on port
8095; select Heavy16 primary → System → Rounds.

## 15. Active recording overhead reduction (2026-10-27)

The private child-process transport now uses versioned packed CBOR with a reused
buffer and the same 60 KiB frame cap. Public JSON APIs and exported evidence keep
their formats; receivers also accept earlier JSON transport frames. Empty
observer batches avoid registry/run locking. Detail retention size is computed
only when a run has room for a record, while component/pricing aggregates still
update after detail retention fills. Scope context updates copy only the changed
inherited attribute; string bounds and direct event-field construction avoid
repeated allocation. Process identity checks parse only the necessary fields.
Observer flush boundaries, CPU/RSS/I/O/PSS cadence, metrics level and complete
bounded scheduler round capture are preserved.

One fresh matched Heavy16 pair, using identical development compiler settings,
disk backing, frozen Wasm and recording configuration, measured:

| Primary measurement | Before | After |
| --- | ---: | ---: |
| Host wall time | 180.527 s | 163.758 s |
| Independent process-tree CPU | 479.56 s | 391.99 s |
| Measured observer CPU | 185.35 s | 105.08 s |
| Dropped private transport records | 410,919 | 251,150 |
| Ingested generic records | 2,746,159 | 2,864,919 |

This pair used 9.29% less primary wall time and 18.26% less process CPU, with
43.3% less measured observer CPU and more received records. Generic detail is
still bounded at 5,000 retained events; it is not a complete event trace. Both
runs retained complete accounting for all 48 rounds, and every nested round
counter and captured limit matched after excluding host identities/timing.
Both completed the same 18,000 roots, 19,200 workflows, 594,000 steps and 2,434
batches with zero traps/rejects/reoffers. Cycles remained exactly
363,878,013,600 primary and 363,897,284,838 with maintenance. Source and frozen
artifacts remained unchanged. Overflow/mismatch counters were zero and neither
stop flush timed out. No unprofiled speedup or statistical confidence interval
is inferred from this single active-recording pair.

The candidate passed 66 core, 94 server and five native integration checks,
including disabled-path allocation/clock probes, transport bounds/lossless
round-trips, context inheritance, pricing, rollback, XNet and DTS accounting.
Installed executable SHA256:
`82e186a2af9db59666be3213ebdb7dca088c4252ca6e397094609a74afb59604`.
Canonical captures and `/proc` boundary readings are preserved under
`/srv/shared/demo/pocketic-performance-20261027/`; `comparison.json` contains
the exact checks, scope/quality caveats and separate binary identities.
