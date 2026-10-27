//! Self-contained, binary-scoped IC cost and capacity reference.
//!
//! The CLI and HTTP/MCP interfaces share this catalog. It reads compiled source
//! and configuration constructors only; no instance, routing table or Markdown
//! file is required.

use crate::instruction_charge_rules::{charge_rules, source};
use ic_config::{execution_environment::Config, subnet_config::SubnetConfig};
use ic_crypto_sha2::Sha256;
use ic_embedders::wasmtime_embedder::system_api_complexity::{overhead, overhead_native};
use ic_registry_subnet_type::SubnetType;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

const API_COSTS: (&str, &str) = (
    "rs/embedders/src/wasmtime_embedder/system_api_complexity.rs",
    include_str!("../../../embedders/src/wasmtime_embedder/system_api_complexity.rs"),
);
const METER: (&str, &str) = (
    "rs/embedders/src/wasm_utils/instrumentation.rs",
    include_str!("../../../embedders/src/wasm_utils/instrumentation.rs"),
);

/// Returns the same compiled reference used by `pocket-ic explain`.
///
/// Default values describe this build, rather than registry overrides or the
/// effective settings of an arbitrary running instance. All quantity values are
/// decimal strings; schema/rule version numbers remain small JSON integers.
pub fn catalog() -> Value {
    static CATALOG: OnceLock<Value> = OnceLock::new();
    CATALOG.get_or_init(build_catalog).clone()
}

/// Available on the server without creating an instance or starting the viewer.
pub async fn handler() -> axum::Json<Value> {
    axum::Json(catalog())
}

fn build_catalog() -> Value {
    let config = Config::default();
    let mut defaults = BTreeMap::new();
    let mut metadata = BTreeMap::new();
    for (name, subnet_type) in [
        ("application", SubnetType::Application),
        ("verified_application", SubnetType::VerifiedApplication),
        ("system", SubnetType::System),
        ("cloud_engine", SubnetType::CloudEngine),
    ] {
        let subnet = SubnetConfig::new(subnet_type);
        // Same DTS capacity formula as RoundSchedule::compute_capacity_percent.
        let capacity = (subnet.scheduler_config.scheduler_cores - 1) * 100;
        let values = ic_execution_environment::runtime_configuration_defaults(
            &config,
            &subnet,
            subnet_type,
            capacity,
        );
        for (key, value) in &values {
            metadata.entry(key.clone()).or_insert_with(|| {
                let (unit, scope) = configuration_quantity(key, value);
                json!({
                    "unit": unit,
                    "scope": scope,
                    "value_kind": "source_default",
                    "rule_id": key,
                })
            });
        }
        defaults.insert(
            name,
            json!({
                "configuration_values": values,
                "value_kind": "source_default",
                "cost_schedule": "Not selected: Normal/Free is a runtime choice.",
                "subnet_size": Value::Null,
                "reference_subnet_size": Value::Null,
            }),
        );
    }
    let app = SubnetConfig::new(SubnetType::Application);
    let native_region_bytes = ic_embedders::WASM_PAGE_SIZE as u64;
    let os_page_bytes = ic_sys::PAGE_SIZE as u64;
    let access = app.scheduler_config.page_overhead.get() * (native_region_bytes / os_page_bytes);
    let marker = "pub fn instruction_to_cost(";
    let tail = &METER.1[METER.1.find(marker).unwrap()..];
    let cost_model_lines = tail[..tail.find("\n}\n").unwrap() + 2].lines().count();
    let mut result = json!({
        "catalog_version": 1,
        "title": "Game of Compute and Memory",
        "catalog_kind": "compiled_source_defaults",
        "server_version": env!("CARGO_PKG_VERSION"),
        "server_binary_sha256": crate::memory_access_tracing::server_identity().binary_hash,
        "numeric_encoding": "Quantities are decimal strings; version fields are small JSON integers.",
        "charge_rules": charge_rules(),
        "default_configurations": defaults,
        "configuration_metadata": metadata,
        "pricing_parameters": {
            "ordinary_reference_subnet_nodes": ic_config::subnet_config::DEFAULT_REFERENCE_SUBNET_SIZE.to_string(),
            "sev_reference_subnet_nodes": ic_config::subnet_config::SEV_REFERENCE_SUBNET_SIZE.to_string(),
            "ordinary_scaling": "floor(component_price * N / R), at the actual component/settlement site; HTTP has its own N-dependent formulas.",
            "runtime_parameters": ["subnet_size", "reference_subnet_size", "cost_schedule"],
            "free_schedule": "Nominal fees can remain nonzero when actual resource charges are zero. Explicit burns and balance losses are separate.",
        },
        "memory_tax_example": {
            "value_kind": "derived_source_default",
            "unit": "execution_instructions",
            "scope": "First native access/dirtying per region in one physical execution; DTS resumes preserve the epoch.",
            "os_page_bytes": os_page_bytes.to_string(),
            "native_region_bytes": native_region_bytes.to_string(),
            "first_access": access.to_string(),
            "first_dirty_additional": access.to_string(),
            "first_write_to_untouched_region": (2 * access).to_string(),
            "formula": "scheduler.page_overhead * native_region_bytes / os_page_bytes",
            "stable_opening_rule_ids": ["ic.stable.read.direct.v1", "ic.stable.read.native.v1", "ic.stable.api.v1"],
            "stable_opening": "A read over entirely untouched 4-KiB stable pages can use the direct PageMap path and avoid the stable native surcharge. API/copy work and heap destination charges remain. Overlap switches the whole read to native memory; a later untouched range can still use the direct path.",
        },
        "system_api_costs": system_api_costs(),
        "rust_usage": rust_usage(),
        "profiling_tools": crate::profiling::profiling_manual::manual(),
        "instruction_profiler_limits": {
            "module_origin_ranges": "The final instrumented origin map is bounded per compiled module, default 262144 ranges. Set --instruction-profile-max-origin-ranges RANGES or POCKET_IC_INSTRUCTION_PROFILE_MAX_ORIGIN_RANGES. For direct Rust embedder use Config.instruction_profile_max_origin_ranges. Zero intentionally retains no ranges.",
            "retained_trace": "The recorder retains traces and shared module dictionaries under a per-instance conservative size budget, default 64 MiB. Set --memory-access-trace-limit-mib MIB or POCKET_IC_MEMORY_ACCESS_TRACE_LIMIT_MIB (minimum 1, checked for host-size overflow). Larger maps can require both budgets to be increased; this is not a process RSS cap.",
            "example": "pocket-ic --instruction-profile-max-origin-ranges 1048576 --memory-access-trace-limit-mib 256",
            "lifetime": "Start a fresh server before loading/installing the module so it is recompiled with the selected limit. Shorter trace windows do not repair a module origin map truncated during compilation. These diagnostic settings do not change guest instructions, page taxes, IC limits or cycle prices.",
            "flags": "module.origin_map_truncated and profile.module_origin_map_truncated describe incomplete module coordinate maps. profile.module_origin_map_unavailable describes missing/rejected/omitted dictionaries, including retention-budget omissions. profile.stack_details_truncated independently describes bounded execution-stack capture. Missing source attribution is not proof that a physical stack or reconciled accounting total was truncated."
        },
        "ram_backed_benchmarks": {
            "enable": "Linux: pocket-ic --ram, POCKET_IC_RAM=1 in the benchmark environment, or Rust PocketIcBuilder::new().with_ram_storage().with_application_subnet().build() (build_async().await for async). Use the matching updated client/server. A benchmark that pins a server binary hash must update its pin for a new build.",
            "storage": "Temporary subnet state, checkpoints, imported working copies and compilation caches use a private executable-capable tmpfs directory, defaulting to /dev/shm. --ram-dir PATH or POCKET_IC_RAM_DIR selects another tmpfs. The server overrides TMPDIR for itself and its sandbox children. Unsupported filesystems fail; there is no disk fallback.",
            "charging": "PageMap, memory tracking, first-access/first-dirty heap and stable taxes, instruction limits, checkpoint semantics and cycle prices are unchanged. Storage changes host performance, not simulated IC time. Compare exact charges from identical Wasm, initial state, inputs and subnet configuration.",
            "state": "with_read_only_state imports a saved seed into RAM. Writable with_state_dir paths must already be on tmpfs in RAM mode. PocketIcState::new_in_ram() creates an owned RAM state handle. Default temporary instances do not acquire an extra teardown checkpoint round.",
            "inspection": "GET /storage returns ram_backed and temporary_directory for the running server. RAM-requesting Rust clients verify this endpoint before creating an instance, including when with_server_url connects to an existing server.",
            "lifetime": "Temporary RAM directories are cleaned on normal shutdown; forced termination can leave directories. Capacity is bounded by host memory and tmpfs. Hosts with swap can swap tmpfs pages. Explicit logs, trace/image exports, binary downloads and benchmark result files remain at caller-selected locations."
        },
        "agent_memory_images": {
            "browser": "The viewer has System, Memory, Compute and Executions workspaces with a shared selected execution and sequence timeline. Memory Pages displays heap/stable coverage; Byte snapshots captures or imports explicit before/after IDs and displays Data/Reads/Writes using the same page order. Fit is a scaled overview; 1:1 is one byte per CSS pixel. Offline access traces do not contain memory contents. Compute Blocks has a weighted mosaic of retained prepaid block charges, separate from API/page fees; full metadata is under inspector Details.",
            "purpose": "Get an actual colored PNG and exact JSON summary to inspect post-install layout and before/after memory changes without a browser.",
            "workflow": [
                "Use start_profiling or the Rust start_instruction_profiling API before installation/workload when access/charge overlays are wanted.",
                "Call capture_memory_snapshot(instance_id, canister_id, label) after installation or before the workload; save its snapshot_id.",
                "Run the intended canister call(s), then capture another snapshot. These are committed operation boundaries, not a promise that only one message ran.",
                "Call get_memory_snapshot_image(snapshot_id=AFTER, compare_to=BEFORE). It defaults to all observed touched heap and stable blocks, with optional attempt_id or logical_execution_id to select one call/install. MCP returns an actual image/png block plus the JSON color key and evidence.",
                "Each tile is one 64 KiB block at 256x256 byte pixels. H:page and S:page identify heap/stable. A thin reserved magenta grid separates blocks. No legend, title, zoom inset or descriptive labels are printed in the PNG; pass the JSON palette as separate text to the viewing LLM. If more than 256 blocks are touched, follow more/next_page_offset explicitly; no blocks are silently omitted. Release captures with delete_memory_snapshot(snapshot_id)."
            ],
            "http": "Viewer: POST /api/instances/{instance}/canisters/{canister}/snapshots; GET /api/memory-snapshots/{id} for JSON or /image.png for PNG. compare_to=BEFORE is required for default selection=touched,memory=both. Optional attempt_id/logical_execution_id filters and page_offset/page_count paginate the sorted block list. selection=range,memory=heap|stable,start_page=N requests a manual window. DELETE /api/memory-snapshots/{id} releases a capture.",
            "cli": "memory_access_viewer.py snapshot --server URL --instance 0 --canister ID --label before -o before.json; memory_access_viewer.py image --viewer URL --snapshot-id AFTER --compare-to BEFORE --memory both -o call.png (also writes call.json with the color legend).",
            "image_semantics": "Each block is256x256pixels with exact byte addresses. Dark=zero, light=nonzero, amber=changed, purple=cleared, green=new allocation, checker=outside allocation; the precise RGB palette is in JSON. Thin magenta grid lines and H:/S:page identifiers are the only image decoration. Touched selection includes read-only blocks and unchanged written blocks, not just blocks with byte differences.",
            "read_masks": {
                "enable": "Rust: pic.start_read_profiling(canister_id, max_executions). MCP start_profiling(..., read_mask=true). CLI start --server URL --canister ID --read-mask --max-executions 1. Enable before the intended operation; existing traces cannot reconstruct byte reads.",
                "export": "Use the identical before/after snapshot IDs and selection with get_memory_snapshot_image(..., view=reads), HTTP view=reads, or CLI image --view reads. Data and reads use the same touched page list, coordinates, H:/S: IDs and magenta grid; write-only pages stay blank in the read mask.",
                "colors": "Black means no recorded logical read. Cyan=heap read; green=direct stable read; orange=native stable read, which takes precedence when both stable paths cover a byte. Fixed logarithmic brightness represents per-byte read frequency. The JSON palette gives exact RGB/count buckets. No data values, legend or extra labels appear in the mask.",
                "costs": "Read frequency is separate from actual instruction fees. Native first access to a 64 KiB region is charged once per execution, not once per read. Stable direct eligibility depends on all overlapping 4 KiB granules being untouched; it avoids the stable native page surcharge but API/copy and heap destination fees remain.",
                "coverage": "Opt-in successful logical Wasm scalar/SIMD loads, memory.copy source ranges, stable reads by selected path, stable_write heap source, and validated host API heap read buffers. Not CPU cache accesses or observer-internal copying. Sparse bounded counters export exact byte RLE with explicit truncation/saturation and coverage. Black remains unknown when coverage is incomplete. Same-value writes and net byte differences are independent."
            },
            "write_masks": {
                "enable": "Rust pic.start_write_profiling(canister_id, max_executions), MCP start_profiling(..., write_mask=true), or CLI start --write-mask --max-executions 1. Read/write selection is independent; use Rust start_memory_profiling(canister_id, max_executions, true, true), both MCP booleans, or both CLI flags to collect both from one call.",
                "export": "Use the same snapshots and selection with get_memory_snapshot_image(..., view=writes), HTTP view=writes, or CLI image --view writes. Data/read/write share the same touched page list, H:/S: IDs, grid and pixel coordinates. Logical profile ranges are included in selection even if a JIT eliminates a native access; physical fee evidence stays separate.",
                "colors": "Black=no recorded completed write; pink=heap; orange-red=stable. Fixed logarithmic brightness means per-byte write frequency. The JSON palette gives precise RGB/count buckets. Memory values and legends do not appear in the mask.",
                "coverage": "Successful logical scalar/SIMD stores, memory.copy/fill/init destinations, stable writes, stable-read heap destinations, and successful host API destination writes. Includes same-value writes and writes subsequently restored. Zero-length and trapping incomplete operations do not create completed write events. Exact sparse byte RLE has explicit truncation, saturation and missing-execution flags; black stays unknown when coverage is incomplete. Native dirty-page fees and final byte differences are separate from logical write frequency."
            },
            "limits": "Eight immutable captures globally, 128MiB conservative retained-delta estimate (not an RSS bound), no silent eviction. Both memories remain lazy; reads and PNGs select bounded address windows.",
            "precision": "Contents and value differences are exact. Zero does not mean allocator-free; unchanged bytes can have been written or read. Stable touch sets include direct first-read paths via the runtime bytemap; exact byte read/write ranges and frequencies require their independent opt-in mask profiling; snapshots alone do not establish accesses. Check touch coverage flags for unavailable or truncated attempts. Aborted/trapped attempts can pay fees while committed snapshots remain unchanged."
        },
        "wasm_instruction_cost_model": {
            "unit": "execution_instructions",
            "rule_id": "ic.wasm.static.v1",
            "description": "Complete compiled operator-weight function for Wasm32/Wasm64. Block prepayment, function-entry minimums and dynamic length debits are additional aspects of metering; original module dictionaries carry per-operator weights.",
            "source": source(METER, "instruction_to_cost", marker, cost_model_lines),
        },
        "interpretation": crate::computation_catalog_notes::interpretation_notes(),
        "supplemental_limits": crate::computation_catalog_notes::supplemental_limits(),
        "runtime_configuration_access": {
            "description": "Trace snapshots contain runtime_configurations keyed by subnet. *_at_setup values are historical registry-resolved setup snapshots. Per-canister settings and subsequent registry updates can differ.",
            "server": "POST /instances/{instance_id}/read/get_memory_access_trace",
            "viewer": "GET /api/instances/{instance_id}/canisters/{canister_id}/runtime-costs",
            "mcp_tool": "get_runtime_costs",
        },
    });
    // Content identity excludes this field itself and includes the executable's
    // hash plus all compiled source excerpts/defaults. It never includes a clock
    // or process generation, so repeated exports of one binary are identical.
    let bytes = serde_json::to_vec(&result).expect("catalog is JSON serializable");
    result["catalog_sha256"] = json!(hex::encode(Sha256::hash(&bytes)));
    result
}

fn rust_usage() -> Value {
    json!({
        "title": "Using the IC charge profiler with Rust",
        "client": {
            "dependency": "Use the updated packages/pocket-ic crate from the same IC checkout, plus candid and serde_json. A matching version number alone does not establish that a released client includes these profiling extensions.",
            "cargo_toml_example": "[dev-dependencies]\npocket-ic = { path = \"/path/to/ic/packages/pocket-ic\" }\ncandid = \"0.10\"\nserde_json = \"1\"",
            "run": "POCKET_IC_BIN=\"$HOME/.local/bin/pocket-ic\" cargo test -- --nocapture",
            "example": "use candid::Principal;\nuse pocket_ic::PocketIc;\n\n// Use an instance with your optimized Rust Wasm already installed.\nfn profile(pic: &PocketIc, canister_id: Principal, sender: Principal, payload: Vec<u8>) {\n    pic.start_instruction_profiling(canister_id, 20);\n    pic.update_call(canister_id, sender, \"transfer\", payload).unwrap();\n    pic.stop_memory_access_tracing(canister_id);\n    let trace = pic.get_memory_access_trace(canister_id);\n    std::fs::write(\"trace.json\", serde_json::to_vec(&trace).unwrap()).unwrap();\n    let viewer = pic.start_memory_access_viewer(None);\n    println!(\"{viewer}#{}:{canister_id}\", pic.instance_id());\n}",
            "workload": "Create/fund/install the canister normally with PocketIcBuilder and the optimized final Wasm. Replace transfer and payload with the method and Candid-encoded arguments you want to measure. Use the same artifacts, subnet configuration and inputs for profiled/control comparisons.",
            "window": "The selected window is 1–10000 physical attempts, normally 20. Profiling starts memory tracing; tracing continues after the profile window is exhausted. Stop keeps existing records and does not cancel an in-flight attempt. Use clear_memory_access_trace only after persisting the export.",
            "async_client": "The nonblocking PocketIC client exposes the same methods with .await. Do not call the blocking client inside an async runtime.",
            "read_results": "Start with the viewer's internal-function ranking, exclusive rule/API/page breakdown, Origins and Call paths. Join each rule_id to charge_rules. Inspect ledger.reconciled, availability flags, detail truncation, installed module hash and actual commit/discard disposition. Reservations/prepayments and their refunds are separate from final consumption."
        },
        "optimized_source_artifacts": {
            "status": "Required work in the canonical Rust artifact builder and SDK transformer; PocketIC does not build these artifacts or create the SDK origin map.",
            "compiler_settings": ["-C debuginfo=2", "-C strip=none"],
            "preserve": "Keep release optimization, LTO, features, panic behavior, SIMD and memory layout equal to the normal build. Debug information does not require a development build. Apply debug settings to dependencies/build-std where needed for their inline/library frames.",
            "build_order": [
                "Produce one optimized debug-bearing raw Rust Wasm and archive it before stripping.",
                "Archive generated and dependency sources with content hashes, build/compiler/options identity, and a physical-to-logical path remap manifest.",
                "Strip only debug sections for deployment, then use the normal canonical SDK prepare/transformation pipeline.",
                "Preserve the SDK raw-to-transformed operator/function origin map and hashes through each transformation; install the resulting final Wasm.",
                "Keep matching debug Wasm, source archive, origin maps and final installed bytes as one identified artifact set."
            ],
            "old_artifacts": "Do not attach symbols from an approximate rebuild to an older binary. Prefer full deployed-byte equality after permitted debug-only stripping. Equal code alone does not establish equal imports, data, memory configuration or execution-relevant metadata. Qualify a changed program separately.",
            "path_handling": "Use stable --remap-path-prefix roots and record generated directories. Resolve source by archive/source hash and line; open current checkout files only if their content hash matches."
        },
        "origin_and_inline_resolution": {
            "status": "PocketIC emits original Wasm origins, final instrumented ranges, bounded physical Wasm stacks and proven page-debit origins. Matching Rust source-line and DWARF inline symbolization belongs to the artifact/analyzer integration; this server does not claim to render recovered Rust source today.",
            "identity": "Require the supported installed_uncompressed_sha256 hash kind and exact installed-module hash before joining metadata. Compressed module hashes, current-checkout paths and nearby PCs are insufficient. Keep source/configuration/instrumentation identities alongside the server binary hash.",
            "coordinate_chain": [
                "Use instrumented_ranges to translate proven final instrumented operator locations to original installed operator IDs.",
                "Use the SDK transformation origin map to translate installed origins back to raw optimized Rust Wasm.",
                "Use the matching raw Wasm DWARF to recover source lines, caller locations and all inline frames, including libraries and distinct monomorphizations."
            ],
            "offsets": "body_offset is relative to FunctionBody.range().start, including locals. Installed module-file offset = body_start + body_offset. Code-relative offset additionally subtracts code_section_offset, which already includes the function-count LEB. Function indices can shift after imports are inserted; use the explicit origin maps.",
            "allocation": "Allocate exclusive charges to source rows once; inclusive caller totals are a separate view. Block prepayment does not prove every operator executed. A first-touch page charge describes the operation that triggered a state-dependent fee, rather than an inherent source-line price.",
            "unknowns": "Leave SDK/generated, runtime, unmapped helper PCs, missing ranges and unavailable caller chains visible. Never infer an exact line from the last block, a neighboring PC or the nearest prior call context.",
            "exports": "Weighted physical call paths can be exported as Speedscope unit none, labeled IC instructions. This is charged work, not CPU samples or an invented wall-clock timeline."
        },
        "qualification": "For the same installed bytes and inputs, compare profiling off/on instructions, outcomes/replies, heap/stable state, accessed/dirty sets and scheduler slice boundaries. Test first reads/writes, overlap/rereads, host copies, bulk and zero-length operations, traps, callbacks, queries, upgrades and DTS abort/retry. Record host CPU time, peak profiler memory, compile time and trace size separately; this catalog makes no performance-overhead claim."
    })
}

fn configuration_quantity(key: &str, value: &str) -> (&'static str, &'static str) {
    if value.parse::<u128>().is_err() {
        return ("enum_or_configuration", "configuration");
    }
    let unit = match key {
        "cycles.ten_update_instructions_execution_fee"
        | "cycles.ten_update_instructions_execution_fee_wasm64" => {
            "cycles_per_10_execution_instructions"
        }
        "cycles.gib_storage_per_second_fee" => "cycles_per_gibibyte_second",
        "cycles.base_per_second_fee" => "cycles_per_second",
        "cycles.compute_percent_allocated_per_second_fee" => "cycles_per_percent_second",
        "cycles.ingress_byte_reception_fee"
        | "cycles.xnet_byte_transmission_fee"
        | "cycles.http_request_per_byte_fee"
        | "cycles.http_response_per_byte_fee" => "cycles_per_byte",
        "scheduler.page_overhead" => "execution_instructions_per_os_page",
        "scheduler.canister_log_resize_instructions_per_byte" => "execution_instructions_per_byte",
        "embedder.cost_to_compile_wasm_instruction" => {
            "execution_instructions_per_counted_operator"
        }
        "embedder.dirty_page_copy_overhead" => "round_instructions_per_os_page",
        "scheduler.instruction_overhead_per_execution"
        | "scheduler.instruction_overhead_per_canister"
        | "scheduler.instruction_overhead_per_canister_for_finalization"
        | "scheduler.max_instructions_per_round" => "round_instructions",
        "query.instruction_overhead_per_call" | "query.max_call_graph_instructions" => {
            "query_graph_instructions"
        }
        "scheduler.accumulated_priority_reset_interval" => "rounds",
        "memory.canister_queue_capacity" => "slots",
        "embedder.max_sum_exported_function_name_lengths"
        | "memory.max_environment_variable_name_length"
        | "memory.max_environment_variable_value_length" => "bytes",
        _ if key.contains("stable_memory_accessed_page_limit")
            || key.contains("stable_memory_dirty_page_limit") =>
        {
            "os_pages"
        }
        _ if key.ends_with("_seconds") => "seconds",
        _ if key.ends_with("_millis") => "milliseconds",
        _ if key.starts_with("cycles.") => "cycles",
        _ if key.contains("instructions") || key == "embedder.create_execution_state_base_cost" => {
            "execution_instructions"
        }
        _ if key.ends_with("_percent") || key.ends_with("_percent_before_allocation_cap") => {
            "percent"
        }
        _ if key.ends_with("_bytes")
            || key.contains("_bytes_")
            || key.ends_with("_size")
            || key.ends_with("_capacity")
            || key.ends_with("_reserve")
            || key.ends_with("_reservation")
            || key.ends_with("_memory_limit")
            || key.ends_with("_memory_threshold")
            || key.ends_with("_heap_delta_per_iteration")
            || key.ends_with("_heap_delta_rate_limit")
            || key == "scheduler.heap_delta_rate_limit"
            || key == "embedder.wasm_max_size"
            || key == "memory.ingress_history_memory_capacity"
            || key.ends_with("aggregate_log_memory_limit")
            || key.ends_with("delta_log_memory_limit") =>
        {
            "bytes"
        }
        _ => "count_or_source_defined",
    };
    let scope = match key {
        "scheduler.max_instructions_per_round" => "per_execution_thread_per_round",
        "scheduler.instruction_overhead_per_execution" => "per_executed_slice",
        "scheduler.instruction_overhead_per_canister" => "per_eligible_canister_pass",
        "scheduler.instruction_overhead_per_canister_for_finalization" => {
            "per_canister_in_state_per_inner_iteration"
        }
        "network.default.max_ingress_messages_per_block"
        | "network.default.max_ingress_bytes_per_block"
        | "network.default.max_block_payload_bytes" => "per_consensus_block_admission",
        "memory.canister_queue_capacity" => {
            "per_peer_per_direction_separately_for_requests_and_responses"
        }
        _ if key.contains("page_limit.") => "per_physical_execution",
        _ => "See matching charge_rules entry and compiled source.",
    };
    (unit, scope)
}

fn variable_api_model(name: &str) -> &'static str {
    match name {
        "call_new" => {
            "Configured base + callee principal bytes + UTF-8 method name bytes, at 1 instruction per byte."
        }
        "call_data_append" | "msg_reply_data_append" | "msg_reject" => {
            "Configured base + 50 * input bytes."
        }
        "debug_print" => {
            "Always configured base + log work 2*min(length,capacity) + 50*min(length,remaining). Enabled stdout printing additionally charges the input length."
        }
        "trap" => "Configured base + input length + applicable log work; the call traps.",
        "stable_read" | "stable_write" => {
            "Native replacement: 20 + length, including length zero; native memory page surcharges and static callsite charges are separate."
        }
        "stable64_read" | "stable64_write" => {
            "Native replacement: 20 + length for nonempty operations; length zero returns before this replacement debit. Page surcharges and static callsite charges are separate."
        }
        "stable_grow" | "stable64_grow" => {
            "Native grow uses overhead_native::STABLE_GROW (100), not the historical configured overhead (500). Growth/allocation limits and static callsite work remain."
        }
        "stable_size" | "stable64_size" => {
            "Native replacement has no separate host API base; original prepaid callsite and surrounding Wasm charges remain. The generated replacement helper is not separately metered."
        }
        "cost_call"
        | "cost_create_canister"
        | "cost_http_request"
        | "cost_http_request_v2"
        | "cost_ecdsa"
        | "cost_schnorr"
        | "cost_vetkd" => {
            "An estimator pays its API instruction work and applicable copies, not the quoted service fee."
        }
        _ if name.ends_with("_copy") => {
            "Configured base plus actual copied bytes where the linked implementation debits length; inspect source and observed api_calls for caps and failed paths."
        }
        _ => {
            "Configured base; API-specific direct/byte charges, early failures and execution context are defined by the linked implementation and measured api_calls."
        }
    }
}

fn system_api_costs() -> BTreeMap<String, Value> {
    let mut costs = BTreeMap::new();
    macro_rules! api {
        ($($id:ident),+ $(,)?) => {$({
            let name = stringify!($id).to_ascii_lowercase();
            costs.insert(name.clone(), json!({
                "configured_overhead": overhead::$id.get().to_string(),
                "unit": "execution_instructions",
                "value_kind": "compiled_constant",
                "variable_model": variable_api_model(&name),
                "rule_ids": ["ic.api.base.v1", "ic.api.bytes.v1", "ic.api.direct.v1", "ic.stable.api.v1"],
                "source": source(API_COSTS, stringify!($id), concat!("pub const ", stringify!($id), ":"), 1),
            }));
        })+};
    }
    api!(
        ACCEPT_MESSAGE,
        CALL_CYCLES_ADD,
        CALL_CYCLES_ADD128,
        CALL_DATA_APPEND,
        CALL_NEW,
        CALL_ON_CLEANUP,
        CALL_PERFORM,
        CALL_WITH_BEST_EFFORT_RESPONSE,
        CYCLES_BURN128,
        CANISTER_CYCLE_BALANCE,
        CANISTER_CYCLE_BALANCE128,
        CANISTER_LIQUID_CYCLE_BALANCE128,
        CANISTER_SELF_COPY,
        CANISTER_SELF_SIZE,
        CANISTER_STATUS,
        CANISTER_VERSION,
        ROOT_KEY_SIZE,
        ROOT_KEY_COPY,
        CERTIFIED_DATA_SET,
        CONTROLLER_COPY,
        CONTROLLER_SIZE,
        COST_CALL,
        COST_CREATE_CANISTER,
        COST_HTTP_REQUEST,
        COST_HTTP_REQUEST_V2,
        COST_ECDSA,
        COST_SCHNORR,
        COST_VETKD,
        DATA_CERTIFICATE_COPY,
        DATA_CERTIFICATE_PRESENT,
        DATA_CERTIFICATE_SIZE,
        DEBUG_PRINT,
        GLOBAL_TIMER_SET,
        IS_CONTROLLER,
        IN_REPLICATED_EXECUTION,
        MSG_ARG_DATA_COPY,
        MSG_ARG_DATA_SIZE,
        MSG_CALLER_COPY,
        MSG_CALLER_INFO_DATA_COPY,
        MSG_CALLER_INFO_DATA_SIZE,
        MSG_CALLER_INFO_SIGNER_COPY,
        MSG_CALLER_INFO_SIGNER_SIZE,
        MSG_CALLER_SIZE,
        MSG_CYCLES_ACCEPT,
        MSG_CYCLES_ACCEPT128,
        MSG_CYCLES_AVAILABLE,
        MSG_CYCLES_AVAILABLE128,
        MSG_CYCLES_REFUNDED,
        MSG_CYCLES_REFUNDED128,
        MSG_DEADLINE,
        MSG_METHOD_NAME_COPY,
        MSG_METHOD_NAME_SIZE,
        MSG_REJECT_CODE,
        MSG_REJECT_MSG_COPY,
        MSG_REJECT_MSG_SIZE,
        MSG_REJECT,
        MSG_REPLY_DATA_APPEND,
        MSG_REPLY,
        PERFORMANCE_COUNTER,
        SUBNET_SELF_SIZE,
        SUBNET_SELF_COPY,
        SUBNET_SELF_NODE_COUNT,
        STABLE_GROW,
        STABLE_READ,
        STABLE_SIZE,
        STABLE_WRITE,
        STABLE64_GROW,
        STABLE64_READ,
        STABLE64_SIZE,
        STABLE64_WRITE,
        ENV_VAR_COUNT,
        ENV_VAR_NAME_SIZE,
        ENV_VAR_NAME_COPY,
        ENV_VAR_NAME_EXISTS,
        ENV_VAR_VALUE_SIZE,
        ENV_VAR_VALUE_COPY,
        TIME,
        TRAP,
    );
    for (name, fee) in [
        ("stable_grow", overhead_native::STABLE_GROW),
        ("stable64_grow", overhead_native::STABLE_GROW),
        ("stable_read", overhead_native::STABLE_READ),
        ("stable_write", overhead_native::STABLE_WRITE),
        ("stable64_read", overhead_native::STABLE64_READ),
        ("stable64_write", overhead_native::STABLE64_WRITE),
    ] {
        costs.get_mut(name).unwrap()["native_replacement_base"] = json!(fee.get().to_string());
    }
    for name in ["stable_size", "stable64_size"] {
        costs.get_mut(name).unwrap()["native_replacement_base"] = json!("0");
    }
    costs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_share_live_capture_keys_and_retain_units_and_scopes() {
        let result = catalog();
        let app = &result["default_configurations"]["application"]["configuration_values"];
        let system = &result["default_configurations"]["system"]["configuration_values"];
        assert_eq!(app["scheduler.max_instructions_per_round"], "4000000000");
        assert_eq!(system["scheduler.max_instructions_per_round"], "7000000000");
        assert_eq!(
            app["network.default.max_ingress_messages_per_block"],
            "1000"
        );
        assert_eq!(app["network.default.initial_notary_delay_millis"], "300");
        assert_eq!(result["memory_tax_example"]["first_access"], "80000");
        assert_eq!(
            result["memory_tax_example"]["first_write_to_untouched_region"],
            "160000"
        );
        for (name, defaults) in result["default_configurations"].as_object().unwrap() {
            for (key, _) in defaults["configuration_values"].as_object().unwrap() {
                assert!(!key.ends_with("_at_setup"), "{name}: {key}");
                assert!(
                    result["charge_rules"][key].is_object(),
                    "missing rule {key}"
                );
                assert!(result["configuration_metadata"][key]["unit"].is_string());
            }
            assert!(defaults["subnet_size"].is_null());
        }
        assert_eq!(
            result["configuration_metadata"]["scheduler.max_instructions_per_round"]["scope"],
            "per_execution_thread_per_round"
        );
        assert_eq!(
            result["configuration_metadata"]["network.default.max_ingress_messages_per_block"]["scope"],
            "per_consensus_block_admission"
        );
    }

    #[test]
    fn api_catalog_covers_every_configured_overhead_and_explains_native_exceptions() {
        let costs = system_api_costs();
        let overhead_source = API_COSTS.1.split("pub mod overhead_native").next().unwrap();
        for line in overhead_source.lines() {
            if let Some(rest) = line.trim().strip_prefix("pub const ") {
                let name = rest.split(':').next().unwrap().to_ascii_lowercase();
                assert!(costs.contains_key(&name), "missing API constant {name}");
            }
        }
        assert_eq!(costs["call_new"]["configured_overhead"], "1500");
        assert_eq!(costs["stable_grow"]["configured_overhead"], "500");
        assert_eq!(costs["stable_grow"]["native_replacement_base"], "100");
        assert_eq!(costs["stable64_size"]["native_replacement_base"], "0");
        assert!(
            costs["stable64_read"]["variable_model"]
                .as_str()
                .unwrap()
                .contains("length zero")
        );
        assert!(
            costs["cost_http_request_v2"]["variable_model"]
                .as_str()
                .unwrap()
                .contains("quoted service")
        );
    }

    #[test]
    fn content_hash_is_repeatable_and_full_operator_source_is_embedded() {
        let mut result = catalog();
        assert_eq!(result, catalog());
        let hash = result
            .as_object_mut()
            .unwrap()
            .remove("catalog_sha256")
            .unwrap();
        assert_eq!(
            hash,
            hex::encode(Sha256::hash(&serde_json::to_vec(&result).unwrap()))
        );

        let src = result["wasm_instruction_cost_model"]["source"]["code"]
            .as_str()
            .unwrap();
        assert!(src.starts_with("pub fn instruction_to_cost("));
        assert!(src.contains("Operator::I32DivS"));
        assert!(src.contains("_ => 1,"));
        assert!(src.ends_with('}'));
    }
}
