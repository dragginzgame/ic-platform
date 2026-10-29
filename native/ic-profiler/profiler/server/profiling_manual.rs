//! Binary-scoped discovery/manual for the complete profiling toolkit. Kept
//! outside the charge-rule dictionary so exact rule filtering remains unchanged.
use crate::system_profile_cli::ProfileAction;
use clap::ValueEnum;
use serde_json::{Value, json};

fn command(action: &ProfileAction) -> Value {
    let name = action.to_possible_value().unwrap();
    let (purpose, example) = match action {
        ProfileAction::Start => (
            "Begin a bounded whole-system capture on a server",
            "pocket-ic profile start --server URL --name workload --metrics-level standard --sample-interval-ms 100 --memory-detail-interval-ms 1000",
        ),
        ProfileAction::Stop => (
            "Finish a capture; optional bounded drain observes later work",
            "pocket-ic profile stop --server URL --run RUN --drain-ms 0",
        ),
        ProfileAction::Delete => (
            "Release a stopped capture and its retained query snapshots",
            "pocket-ic profile delete --server URL --run RUN",
        ),
        ProfileAction::List => (
            "List retained captures without expanding heavy datasets",
            "pocket-ic profile list --server URL",
        ),
        ProfileAction::Status => (
            "Read capture status, quality and a small resource summary",
            "pocket-ic profile status --server URL --run RUN",
        ),
        ProfileAction::Summary => (
            "Read measured CPU, memory, storage and coverage first",
            "pocket-ic profile summary --server URL --run RUN",
        ),
        ProfileAction::Components => (
            "Rank measured native components; exclusive CPU and wall time remain distinct",
            "pocket-ic profile components --server URL --run RUN --query '{\"limit\":100}'",
        ),
        ProfileAction::Resources => (
            "Read bounded host-resource samples and explicit measurement sources",
            "pocket-ic profile resources --server URL --run RUN --query '{\"resolution\":250,\"limit\":250}'",
        ),
        ProfileAction::Metrics => (
            "Read bounded lifecycle snapshots of existing raw IC counters, gauges and histograms",
            "pocket-ic profile metrics --server URL --run RUN --query '{\"limit\":100}'",
        ),
        ProfileAction::Events => (
            "Read native spans, logical events and separately paged observer evidence",
            "pocket-ic profile events --server URL --run RUN --query '{\"component\":\"ic.execution\",\"limit\":100}'",
        ),
        ProfileAction::Flows => (
            "Inspect observed ingress, request, callback, same-subnet and XNet links",
            "pocket-ic profile flows --server URL --run RUN --query '{\"limit\":100}'",
        ),
        ProfileAction::Limits => (
            "Inspect actual scheduling/admission limits and recorded exhaustion evidence",
            "pocket-ic profile limits --server URL --run RUN",
        ),
        ProfileAction::Rounds => (
            "Inspect dedicated scheduler round budgets, per-worker consumption and stopping reasons",
            "pocket-ic profile rounds --server URL --run RUN --query '{\"limit\":1000,\"round_details\":false}'",
        ),
        ProfileAction::Pricing => (
            "Inspect deterministic instruction/cycle evidence separately from physical CPU",
            "pocket-ic profile pricing --server URL --run RUN",
        ),
        ProfileAction::Explain => (
            "Print the built-in system component, unit, coverage, defaults and observer dictionary",
            "pocket-ic profile explain",
        ),
        ProfileAction::Compare => (
            "Compare two captures with their scope and quality evidence",
            "pocket-ic profile compare --input before.icprofile --other-input after.icprofile",
        ),
        ProfileAction::Export => (
            "Save a versioned compressed bundle of all retained evidence",
            "pocket-ic profile export --server URL --run RUN --output run.icprofile",
        ),
        ProfileAction::Import => (
            "Read a compressed/plain versioned bundle offline",
            "pocket-ic profile import --input run.icprofile",
        ),
        ProfileAction::Render => (
            "Render SVG/PNG and JSON evidence from one immutable query",
            "pocket-ic profile render --input run.icprofile --view components --format png --output components.png",
        ),
        ProfileAction::Capture => (
            "Observe an independently owned process without stopping it",
            "pocket-ic profile capture --pid PID --duration 10 --output run.icprofile",
        ),
        ProfileAction::Record => (
            "Launch an instrumented command with an established capture baseline",
            "pocket-ic profile record --backend replica --duration 30 --output run.icprofile -- /path/to/instrumented-replica ARGS",
        ),
    };
    json!({"name":name.get_name(),"purpose":purpose,"example":example})
}

pub(crate) fn manual() -> Value {
    json!({
        "manual_version":1,
        "entry_points":{
            "all_tools":"pocket-ic explain; pocket-ic explain --compact; pocket-ic explain --rule EXACT_RULE_ID",
            "system_dictionary":"pocket-ic profile explain; GET /api/profiling/explain; MCP profile_explain",
            "catalog_http":"GET /computation_catalog on the native server; the viewer/MCP exposes the same compiled catalog",
            "viewer":"pocket-ic --memory-access-viewer-port 8093; or Rust pic.start_memory_access_viewer(None); viewer /llms.txt describes JSON and MCP /mcp",
            "help":"pocket-ic --help; pocket-ic profile --help; python3 rs/pocket_ic_server/profiling/viewer/memory_access_viewer.py --help"
        },
        "choose_a_tool":[
            {"tool":"canister_instruction_profile","question":"Which Wasm operations, APIs, physical stacks and state-dependent memory page taxes account for one canister's IC instructions?","enable":"Rust start_instruction_profiling; optional start_read_profiling/start_write_profiling/start_memory_profiling; viewer/MCP start_profiling"},
            {"tool":"memory_access_trace","question":"Which heap/stable 64 KiB pages were touched or dirtied during complete physical attempts?","enable":"Rust start_memory_access_tracing; Python start; no byte masks are implied"},
            {"tool":"memory_byte_snapshot","question":"What bytes exist after installation, and what changed/read/wrote during a selected call?","enable":"Capture before/after snapshot IDs; request data, reads or writes PNG plus exact JSON sidecar"},
            {"tool":"system_profile","question":"How much real host CPU/RAM/storage and native subsystem work did an entire multi-canister run use?","enable":"Rust start_system_profile; pocket-ic profile start/capture/record; viewer System; MCP profile_start"}
        ],
        "command_inventory":{
            "server":"Running pocket-ic without a subcommand starts the ordinary native API; --memory-access-viewer-port adds the compact viewer. Existing canister operations remain on /instances/{instance}/read|update/...",
            "explain":{"purpose":"Compiled costs, game rules, source defaults, limits and this manual; no server or Markdown files required","flags":["--rule EXACT_RULE_ID","--compact"],"hash":"catalog_sha256 covers the full emitted catalog except itself. A filtered export retains full_catalog_sha256 and has its own verifiable hash; rule IDs/fees are unchanged."},
            "profile":ProfileAction::value_variants().iter().map(command).collect::<Vec<_>>(),
            "canister_python":{"path":"rs/pocket_ic_server/profiling/viewer/memory_access_viewer.py","commands":["start","stop","clear","fetch","snapshot","image","snapshot-delete","viewer","html","video"],"examples":["memory_access_viewer.py start --server URL --instance 0 --canister ID --read-mask --write-mask --max-executions 1","memory_access_viewer.py fetch --server URL --instance 0 --canister ID --clear -o trace.json","memory_access_viewer.py snapshot --server URL --instance 0 --canister ID --label before -o before.json","memory_access_viewer.py image --viewer URL --snapshot-id AFTER --compare-to BEFORE --view reads --memory both -o reads.png","memory_access_viewer.py html trace.json -o report.html"]}
        },
        "server_options":[
            {"environment":"IC_SYSTEM_PROFILER_DISABLED=1","effect":"Explicit whole-system observer hard-off: disables instrumentation/collection fast paths and rejects new system-profile captures. Normal idle observation already takes the inexpensive inactive path; this switch does not change IC fees or silently turn absent observations into zero."},
            {"flag":"--instruction-profile-max-origin-ranges RANGES","environment":"POCKET_IC_INSTRUCTION_PROFILE_MAX_ORIGIN_RANGES","default_ranges":ic_config::embedders::DEFAULT_INSTRUCTION_PROFILE_MAX_ORIGIN_RANGES.to_string(),"effect":"Per-compiled-module origin map capacity. Zero keeps no ranges. Set before install/compilation; a shorter execution trace cannot repair a truncated module dictionary."},
            {"flag":"--memory-access-trace-limit-mib MIB","environment":"POCKET_IC_MEMORY_ACCESS_TRACE_LIMIT_MIB","default_mib":crate::instruction_profiler_config::DEFAULT_TRACE_LIMIT_MIB.to_string(),"effect":"Per-instance retained canister trace/module dictionary estimate; at least 1 MiB, not a process RSS cap. Increase alongside origin capacity when necessary."},
            {"flag":"--memory-access-viewer-port PORT","environment":"POCKET_IC_MEMORY_ACCESS_VIEWER_PORT","effect":"Serve the shared System/Memory/Compute/Executions viewer and agent JSON/MCP API; zero chooses a port."},
            {"flag":"--ram","environment":"POCKET_IC_RAM=1","effect":"Executable-capable tmpfs temporary storage; host disk behavior changes while deterministic instructions, page taxes and cycle prices do not."},
            {"flag":"--ram-dir DIRECTORY","environment":"POCKET_IC_RAM_DIR","effect":"Select a tmpfs parent; implies RAM mode, validates backing and has no disk fallback. Inspect GET /storage and profile scope.metadata.server_storage; imported persistent paths may have different backing."}
        ],
        "system_profile":{
            "built_in_reference":crate::profiling::backend_description::explain(),
            "capture_configuration":{"name":"Label","sample_interval_ms":"Host-resource sample cadence, default 100; CPU/RSS/I/O independent of PSS scans","memory_detail_interval_ms":"Default 1000; 0 disables all PSS scans including boundaries while CPU/RSS/I/O remain","metrics_level":"standard (default): raw scalars/labels/units and histogram count/sum; full adds bounded raw buckets/quantiles; none explicitly disables registry gathering","max_events":"Bounded retained detail, maximum 200000","max_samples":"Bounded retained samples, maximum 20000","max_retained_bytes":"Default 64 MiB, 1–128 MiB per run; 256 MiB global across retained runs. Export/delete stopped captures when needed; budgets are conservative retained allocation estimates, not a measured RSS cap","root_pid":"Defaults to server process; capture can attach a different root","extra_pids":"Additional independently launched service roots; not invented child relationships","include_descendants":"Observe available descendants; true by default","instance_id":"Select logical evidence/IC registries. Shared process CPU/memory is still the observed server tree","cgroup_path":"Optional explicit host cgroup resource boundary","backend":"Descriptive deployment label, not a switch that creates production consensus","metadata":"Bounded caller provenance; server records actual binary/generation/storage context"},
            "cli_flags":["--server URL","--run RUN","--other-run RUN","--name LABEL","--backend LABEL","--pid PID","--extra-pid PID (repeatable)","--sample-interval-ms N","--metrics-level none|standard|full","--memory-detail-interval-ms N","--duration SECONDS","--drain-ms N","--input FILE","--other-input FILE","--output FILE","--view VIEW","--query JSON","--format png|svg|json","--keep-command-running","-- COMMAND ARGS"],
            "lifecycle":"Start before the intended work, stop with drain_ms=0 unless a later observation interval is wanted, query a small summary first, then export. Initial+stop IC metric snapshots; positive drain adds foreground_end. Drain is not a quiescence, durability or kernel-writeback barrier.",
            "process_lifetime":"capture never terminates attached targets. record gates the owned Unix command until its capture baseline exists, preserves argv/stdin, routes workload stdout to stderr, and stops its owned process group at the duration limit unless --keep-command-running. Instrumented children inherit transport; an uninstrumented attached binary has host counters without invented component spans.",
            "native_api":"Both native server and viewer: GET/POST /api/profiling/runs; GET/DELETE /api/profiling/runs/{id}; POST /{id}/stop; GET /{id}/{summary,components,resources,events,flows,limits,rounds,pricing,metrics,status,explain}; GET /{id}/export; POST /api/profiling/import; GET /api/profiling/compare?a=A&b=B; GET /{id}/render.svg|render.png|render.json",
            "mcp_tools":["profile_start","profile_stop","profile_list","profile_status","profile_summary","profile_components","profile_resources","profile_trace_slice","profile_evidence","profile_flows","profile_limits","profile_rounds","profile_pricing","profile_metrics","profile_compare","profile_render","profile_export","profile_delete","profile_explain"],
            "queries":{"filters":["start_ns","end_ns","component","canister_id","process_id","subnet_id","round_id","round_number","round_details","limit","cursor","resolution"],"time":"Capture-relative nanoseconds as exact decimal strings, not a wall timestamp or a universal cross-process clock","pagination":"Inspect next_cursor, truncation and coverage; view rows are bounded. Metrics phase snapshots declare unapplied time/component/canister/process filters explicitly","identity":"A query_id identifies immutable selected data. Render that retained query rather than silently re-querying a running capture; raw bundles export the whole retained run; stale IDs and mismatched run/view fail explicitly","images":"PNG is an actual MCP image block with JSON evidence metadata; SVG embeds source metadata. Range selection changes the query; image crop changes presentation only."},
            "round_budget_evidence":{
                "scheduler_model":"The shared explanation is system_profile.built_in_reference.rounds.scheduler_model in this manual, rounds.scheduler_model in profile explain, and rounds.scheduler_model in every rounds query response. It covers deterministic worker assignment, the iteration barrier/no work stealing, dispatch reservation, chart interpretation and the distinct message/slice/worker limits.",
                "query":"pocket-ic profile rounds --server URL --run RUN --query '{\"limit\":1000}'; Rust query_system_profile(run, \"rounds\", &query); MCP profile_rounds",
                "retention":"Dedicated newest scheduler round records and versioned configuration snapshots are retained independently of generic events, within their own bounded fraction of the run budget. Read observed/dropped/rejected/boundary-crossing round counts and omitted iteration/worker/canister/slice counters. Generic dropped_events cannot establish or repair this coverage; older bundles explicitly report dedicated rounds unavailable.",
                "round_capacity":"max_instructions_per_round is the configured nominal per-worker/round instruction allowance, not a subnet-wide pool multiplied into every message. Actual initial/remaining counters include scheduler reservation rules. Shared global decrement is the maximum worker consumption in each parallel inner iteration plus finalization; summed worker work is a distinct useful quantity. Negative remaining counters or reserve use alone do not prove a limit violation.",
                "distinct_limits":"Message completion totals, DTS slice caps, query/install caps, actual scheduler overhead and application-defined turn budgets are separate. A completed message can span older rounds/slices: do not add its whole-message total to each slice or current-round guest total. Declared harness application budgets (for example 500M) are provenance, not an inferred IC 2B slice cap.",
                "heap_delta":"Nullable heap_delta_bytes at round/iteration/worker/pass/slice level records the scheduler logical dirty-memory proxy. Despite the configuration name max_heap_delta_per_iteration_bytes, the scheduler checks round-cumulative delta between iterations. It can stop a round while instruction allowance remains. It is not RSS, allocated heap size, bytes written to disk, or a guaranteed device write. Summary known/unknown round counts and null values preserve missing older capture data. Read actual stop reasons and threshold outcome separately from nominal instruction reserve/exhaustion.",
                "placement":"The worker timeline places each iteration at round.initial_instructions - iteration.budget_before. Worker bar width is worker.initial_instructions - worker.remaining_instructions. Parallel workers share this logical offset; widths are not elapsed CPU or summed capacities.",
                "filters":"Use round_details=false for a compact many-round overview, then round_id+limit=1+round_details=true for selected full detail. None/true keeps full detail for compatibility. Projection flags and retained counts distinguish intentionally omitted iterations from source truncation; aggregate totals/source completeness stay unchanged. subnet_id, exact round_id, round_number, process_id and capture-relative time select retained completed rounds. canister_id selects whole rounds with a retained matching pass and preserves global denominators; omitted passes can censor matching. Summary covers all matching retained rows before pagination.",
                "images":"Use view=rounds plus an immutable query_id for SVG/PNG/JSON. Optional render_round_id focuses a row already present in that query as a presentation selection, without changing accounting. Nominal limits come only from captured configurations; absent data remains unavailable."
            },
            "separate_accounting_planes":"Physical CPU/wall/RSS/PSS/storage, logical guest instructions, round allowances and cycle debit/prepayment/refund/reservation/transfer rows have separate units. Never sum round allowances into the canister instruction ledger, treat prepayment as final consumption, or equate counters/histogram wall time with component CPU.",
            "coverage":"Missing process/perf/device/network counters remain null with reasons. RSS sums can double-count shared pages; PSS is sampled. /proc storage bytes are not stable API requested bytes or device latency. scheduler_compute_utilization_per_core is an instruction-budget ratio, not measured CPU; unknown metric units stay null. Flow links require recorded identities and preserve unresolved/censored/ambiguous joins. PocketIC coverage distinguishes real execution/XNet code from emulated/absent consensus, gossip and signing.",
            "overhead":"Profiler collection, scope creation/attributes/drop, events, transport encoding/receiving, retention and flush are measured explicitly where available. Native exclusive CPU excludes measured observer work; residual CPU is not assigned by guess. Check dropped observer segments, epoch exclusions, mismatches, stop-flush timeouts and residuals. Probe costs and undelivered tails can remain unattributed. Compare identical off/on workloads and report Bun/agent harness CPU separately from the server tree."
        },
        "rust_system_example":"use pocket_ic::{PocketIc, SystemProfileConfig, SystemProfileQuery};\nfn measure(pic: &PocketIc) {\n    let started = pic.start_system_profile(SystemProfileConfig { name: \"workload\".into(), ..Default::default() });\n    let run = started[\"run_id\"].as_str().unwrap();\n    pic.tick(); // Replace with the intended ordinary multi-canister work.\n    pic.stop_system_profile(run);\n    let rows = pic.query_system_profile(run, \"components\", &SystemProfileQuery { limit: 100, ..Default::default() });\n    std::fs::write(\"run.icprofile\", pic.export_system_profile(run)).unwrap();\n    std::fs::write(\"components.png\", pic.render_system_profile(run, rows[\"query_id\"].as_str().unwrap(), \"components\", \"png\")).unwrap();\n    // Async client exposes the same APIs with .await; delete_system_profile releases a stopped run.\n}",
        "diagnostic_compilation":"Ordinary execution uses the original Wasm instrumentation without source or byte-mask hooks. A separately cached diagnostic variant is compiled lazily for instruction/read/write profiling. Exhausting a bounded profile returns later execution to ordinary Wasm; page memory tracing can continue through lazily attached host observers. Explicit stop_memory_access_tracing stops page tracing and runtime-cost recording too. Starting profiling after installation remains supported, but the first enabled execution may compile/warm a diagnostic variant. Warm the relevant variants outside a performance window using a matched/resettable state, or include compilation explicitly as setup; do not change the measured initial state or workload to hide this cost. Whole-system capture alone does not enable byte/instruction instrumentation.",
        "memory_and_origin_notes":"The existing instruction_profiler_limits, rust_usage and agent_memory_images sections contain complete module/stack truncation distinctions, exact matching Rust DWARF/artifact requirements, data/read/write palettes and same-layout H:/S: page tiles. Enabling an image alone does not retroactively record byte reads/writes; inspect incomplete coverage before interpreting black as untouched.",
        "source_tree":{"root":"rs/pocket_ic_server/profiling","server":"profiling/server: catalogs, operations/routes, tracing, snapshots, API, CLI, metrics, images","viewer":"profiling/viewer: HTML, agent guide, Python tooling, browser/unit tests and references","system":"profiling/system: independent ic-system-profiler crate used by native IC hooks","integration":"Existing root module/API aliases remain usable. Replica/scheduler/sandbox hook sites necessarily stay in their owning IC components."}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_tracks_actual_cli_actions_and_embeds_the_live_dictionary() {
        let manual = manual();
        let rows = manual["command_inventory"]["profile"].as_array().unwrap();
        assert_eq!(rows.len(), ProfileAction::value_variants().len());
        for action in ProfileAction::value_variants() {
            let name = action.to_possible_value().unwrap();
            assert!(rows.iter().any(|row| row["name"] == name.get_name()));
        }
        assert_eq!(
            manual["system_profile"]["built_in_reference"],
            crate::profiling::backend_description::explain()
        );
        assert!(
            manual["rust_system_example"]
                .as_str()
                .unwrap()
                .contains("start_system_profile")
        );
        assert!(
            manual["server_options"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["flag"] == "--instruction-profile-max-origin-ranges RANGES")
        );
    }
}
