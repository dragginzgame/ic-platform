//! Explanations for the charge and decision IDs emitted by the diagnostic meter.
//! Source excerpts are compiled into the server, so an exported trace remains
//! explainable after the checkout changes or the server is gone.

use ic_crypto_sha2::Sha256;
use pocket_ic::common::rest::{InstructionChargeRule, InstructionChargeRuleSource};
use std::collections::BTreeMap;
use std::sync::OnceLock;

const METER: (&str, &str) = (
    "rs/embedders/src/wasm_utils/instrumentation.rs",
    include_str!("../../../embedders/src/wasm_utils/instrumentation.rs"),
);
const REPLACEMENTS: (&str, &str) = (
    "rs/embedders/src/wasm_utils/system_api_replacements.rs",
    include_str!("../../../embedders/src/wasm_utils/system_api_replacements.rs"),
);
const LINKER: (&str, &str) = (
    "rs/embedders/src/wasmtime_embedder/linker.rs",
    include_str!("../../../embedders/src/wasmtime_embedder/linker.rs"),
);
const TRACKER: (&str, &str) = (
    "rs/memory_tracker/src/deterministic.rs",
    include_str!("../../../memory_tracker/src/deterministic.rs"),
);
const EMBEDDER: (&str, &str) = (
    "rs/embedders/src/wasmtime_embedder.rs",
    include_str!("../../../embedders/src/wasmtime_embedder.rs"),
);
const COLLECTOR: (&str, &str) = (
    "rs/embedders/src/wasmtime_embedder/profiling.rs",
    include_str!("../../../embedders/src/wasmtime_embedder/profiling.rs"),
);

const CAM: (&str, &str) = (
    "rs/cycles_account_manager/src/cycles_account_manager.rs",
    include_str!("../../../cycles_account_manager/src/cycles_account_manager.rs"),
);
const FEES: (&str, &str) = (
    "rs/config/src/subnet_config.rs",
    include_str!("../../../config/src/subnet_config.rs"),
);
const EXEC_CONFIG: (&str, &str) = (
    "rs/config/src/execution_environment.rs",
    include_str!("../../../config/src/execution_environment.rs"),
);
const EMBED_CONFIG: (&str, &str) = (
    "rs/config/src/embedders.rs",
    include_str!("../../../config/src/embedders.rs"),
);
const SCHEDULER: (&str, &str) = (
    "rs/execution_environment/src/scheduler.rs",
    include_str!("../../../execution_environment/src/scheduler.rs"),
);
const MANAGER: (&str, &str) = (
    "rs/execution_environment/src/canister_manager.rs",
    include_str!("../../../execution_environment/src/canister_manager.rs"),
);
const ENVIRONMENT: (&str, &str) = (
    "rs/execution_environment/src/execution_environment.rs",
    include_str!("../../../execution_environment/src/execution_environment.rs"),
);
const QUERY: (&str, &str) = (
    "rs/execution_environment/src/query_handler/query_context.rs",
    include_str!("../../../execution_environment/src/query_handler/query_context.rs"),
);
const INSTALL: (&str, &str) = (
    "rs/execution_environment/src/execution/install.rs",
    include_str!("../../../execution_environment/src/execution/install.rs"),
);
#[cfg(test)]
const UPGRADE: (&str, &str) = (
    "rs/execution_environment/src/execution/upgrade.rs",
    include_str!("../../../execution_environment/src/execution/upgrade.rs"),
);
#[cfg(test)]
const INSTALL_CODE: (&str, &str) = (
    "rs/execution_environment/src/execution/install_code.rs",
    include_str!("../../../execution_environment/src/execution/install_code.rs"),
);
#[cfg(test)]
const COMMON: (&str, &str) = (
    "rs/execution_environment/src/execution/common.rs",
    include_str!("../../../execution_environment/src/execution/common.rs"),
);
#[cfg(test)]
const RESPONSE: (&str, &str) = (
    "rs/execution_environment/src/execution/response.rs",
    include_str!("../../../execution_environment/src/execution/response.rs"),
);
const SANDBOX_STATE: (&str, &str) = (
    "rs/embedders/src/wasmtime_embedder/system_api/sandbox_safe_system_state.rs",
    include_str!(
        "../../../embedders/src/wasmtime_embedder/system_api/sandbox_safe_system_state.rs"
    ),
);
const EXECUTOR: (&str, &str) = (
    "rs/embedders/src/wasm_executor.rs",
    include_str!("../../../embedders/src/wasm_executor.rs"),
);
const HTTP_PRICING: (&str, &str) = (
    "rs/https_outcalls/pricing/src/fees.rs",
    include_str!("../../../https_outcalls/pricing/src/fees.rs"),
);
const HTTP_SETTLEMENT: (&str, &str) = (
    "rs/messaging/src/canister_http_spent.rs",
    include_str!("../../../messaging/src/canister_http_spent.rs"),
);
const ROUTING_STATE: (&str, &str) = (
    "rs/messaging/src/state_machine.rs",
    include_str!("../../../messaging/src/state_machine.rs"),
);
const REPLICATED_STATE: (&str, &str) = (
    "rs/replicated_state/src/replicated_state.rs",
    include_str!("../../../replicated_state/src/replicated_state.rs"),
);
#[cfg(test)]
const LIMITS: (&str, &str) = (
    "rs/limits/src/lib.rs",
    include_str!("../../../limits/src/lib.rs"),
);
const CONFIG_CAPTURE: (&str, &str) = (
    "rs/execution_environment/src/lib.rs",
    include_str!("../../../execution_environment/src/lib.rs"),
);

pub(crate) fn source(
    file: (&str, &str),
    symbol: &str,
    marker: &str,
    lines: usize,
) -> InstructionChargeRuleSource {
    let (path, contents) = file;
    let offset = contents
        .find(marker)
        .expect("charge rule source marker must exist in compiled source");
    let start = contents[..offset]
        .bytes()
        .filter(|&byte| byte == b'\n')
        .count();
    // Host event IDs sit beside existing debit sites; retain preceding code
    // as well as the observer so the excerpt shows the operation being recorded.
    let start = if marker.starts_with("\"ic.") || marker.starts_with("ic.cycles.") {
        start.saturating_sub(24)
    } else {
        start
    };
    InstructionChargeRuleSource {
        file: path.to_string(),
        symbol: symbol.to_string(),
        line: (start + 1) as u32,
        code: contents
            .lines()
            .skip(start)
            .take(lines)
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end_matches('\n')
            .to_string(),
        sha256: hex::encode(Sha256::hash(contents.as_bytes())),
    }
}

/// Rule charges partition the raw debit ledger. Decision rows carry zero
/// instructions; neither projection is an additional charge to the canister.
pub(crate) fn charge_rules() -> &'static BTreeMap<String, InstructionChargeRule> {
    static RULES: OnceLock<BTreeMap<String, InstructionChargeRule>> = OnceLock::new();
    RULES.get_or_init(|| {
        let mut rules = BTreeMap::new();
        let mut add = |id: &str, title: &str, kind: &str, condition: &str, formula: &str, description: &str, tip: &str, sources| {
            rules.insert(id.to_string(), InstructionChargeRule {
                version: 1, id: id.to_string(), title: title.to_string(),
                description: description.to_string(), condition: condition.to_string(),
                formula: formula.to_string(), tip: tip.to_string(), charge_kind: kind.to_string(), source: sources,
            });
        };
        add("ic.wasm.static.v1", "Weighted Wasm block", "debit",
            "The existing static metering point is reached.",
            "Sum of the original operators' IC weights, excluding the separately recorded function-entry minimum.",
            "IC prepays a block. A trap or failed API may stop execution before all prepaid operators run. Wasm64 and Wasm32 can have different weights. This allocation is not an executed-opcode count.",
            "Reduce work on the complete message path. Inspect original blocks and optimized code; counting source statements or wall-clock samples does not predict IC charges.",
            vec![source(METER, "instruction_to_cost", "pub fn instruction_to_cost(", 24), source(METER, "injections", "fn injections(", 45)]);
        add("ic.wasm.function_entry.v1", "Function entry minimum", "debit",
            "A function-entry static meter is reached.", "The entry_weight recorded in that block, once per entry.",
            "The meter adds a minimum function-entry charge. It is already included in the block weight; this rule separates it from the operators so the rule totals remain additive.",
            "Many tiny helper calls can add work. Compare optimized release artifacts; do not assume an inline annotation changes the emitted calls.",
            vec![source(METER, "injections", "fn injections(", 45)]);
        add("ic.wasm.bulk_length.v1", "Bulk operation length", "debit",
            "An original bulk memory or table operation reaches its existing dynamic debit.",
            "The runtime length operand; memory operations count bytes, table operations count elements. Static operation weights remain in the prepaid block.",
            "The IC charges the dynamic length in addition to the static weight (100 for the relevant bulk operations in this instrumenter). Synthetic stable copies are accounted under their replacement rule instead.",
            "Avoid redundant copies and oversized fills. A copy can also trigger source access and destination access/dirty taxes; compare the whole ledger.",
            vec![source(METER, "injections", "fn injections(", 70)]);
        add("ic.api.base.v1", "System API base charge", "debit",
            "A linked system API reaches the actual fixed/base debit.", "The configured base charge supplied for that API at this debit point.",
            "API calls have operation-specific overhead. The api_calls table retains API identity and its base, byte and direct components. Native stable-grow charges appear here as well.",
            "Reduce repeated calls when semantics permit. Batching saves fixed overhead but can increase copied bytes and memory regions; measure both.",
            vec![source(LINKER, "charge_for_system_api_call", "fn charge_for_system_api_call(", 18), source(LINKER, "charge_direct_fee", "fn charge_direct_fee(", 80)]);
        add("ic.api.bytes.v1", "System API bytes and variable fee", "debit",
            "A linked system API reaches a length-dependent or variable instruction debit.",
            "The actual API-specific byte quantity: ordinary copies use length; reply/reject/call-data transmission uses 50 × length; logging uses 2 × min(length, capacity) + 50 × min(length, remaining). Trap and enabled debug_print additionally charge their input length.",
            "Not every byte has the same price. Reply/reject/call-data transmission and logging can apply different multipliers or caps. The observed api_calls breakdown is authoritative; fixed cycle fees are separate.",
            "Avoid serializing and transmitting unused data. Cache coherent decoded data only when the saved API work exceeds extra heap and memory taxes.",
            vec![source(LINKER, "charge_for_system_api_call", "fn charge_for_system_api_call(", 18), source(LINKER, "logging_charge_bytes", "fn logging_charge_bytes(", 18), source(LINKER, "msg_reply_data_append", ".func_wrap(\"ic0\", \"msg_reply_data_append\"", 18), source(LINKER, "BYTE_TRANSMISSION_COST_FACTOR", "const BYTE_TRANSMISSION_COST_FACTOR:", 1)]);
        add("ic.api.direct.v1", "Direct system API instruction fee", "debit",
            "A system API charges an explicitly computed instruction fee outside the base/byte pair.",
            "The actual direct fee recorded by the linker; this is an instruction debit, not a fixed cycle fee.",
            "Some API work has a directly computed instruction price. Zero observed events means this path was not reached, not that all direct fees are free.",
            "Inspect the API identity and rule source before optimizing; do not subtract unrelated fixed cycle costs from the instruction ledger.",
            vec![source(LINKER, "charge_direct_fee", "fn charge_direct_fee(", 80)]);
        add("ic.stable.api.v1", "Stable read or write replacement fee", "debit",
            "A generated stable read/write replacement reaches its own debit.",
            "Nonempty read/write: 20 + byte length in this implementation. The 32-bit stable API charges its base before the zero-length guard; the 64-bit stable API returns before that debit for length zero.",
            "This is the stable API/copy charge. Native stable faults and heap source/destination taxes are additional. The generated copy is not charged again as an original bulk operation. Bounds checks and debit ordering differ between the 32-bit and 64-bit APIs.",
            "Avoid duplicate reads and writes. Batch only useful spans: larger buffers may touch extra heap pages or force native stable mapping through overlap.",
            vec![source(REPLACEMENTS, "replacement_functions", "SystemApiFunc::StableRead,", 42), source(REPLACEMENTS, "replacement_functions", "value: overhead_native::STABLE64_READ.get() as i64,", 14)]);
        for (memory, kind, symbol, marker) in [
            ("heap", "access", "mark_wasm_page_accessed", "fn mark_wasm_page_accessed(\n        &self,"),
            ("heap", "dirty", "mark_wasm_page_dirty", "fn mark_wasm_page_dirty(&self,"),
            ("stable", "access", "mark_wasm_page_accessed", "fn mark_wasm_page_accessed(\n        &self,"),
            ("stable", "dirty", "mark_wasm_page_dirty", "fn mark_wasm_page_dirty(&self,"),
        ] {
            let accessed = kind == "access";
            let stable = memory == "stable";
            add(&format!("ic.memory.{memory}.{kind}.v1"),
                &format!("{} first {}", if stable { "Stable memory" } else { "Heap" }, if accessed { "access" } else { "dirtying" }), "debit",
                if accessed { "The deterministic native-memory tracker first accesses an aligned 64 KiB region in this physical execution, by a read or write." }
                else { "The deterministic native-memory tracker first dirties an aligned 64 KiB region in this physical execution, with dirty tracking enabled." },
                "OS pages in the 64 KiB region × configured page_overhead. With 4 KiB OS pages and page_overhead 5,000, each event is 80,000 instructions.",
                if stable { "This counts actual native stable-memory tracker debits. The direct first-read path can avoid mapping the stable region, so applying a heap-style formula to stable API ranges would be wrong. An accessed region is charged once; first dirtying is an additional tax." }
                else { "Host copies and Wasm operations can both trigger this tax. A first write can pay access and dirtying together. Repeated accesses of the same kind in this physical execution do not pay again; later messages/callbacks start a new epoch, while DTS slices preserve it." },
                if stable { "Keep stable reads coherent and avoid unnecessary overlap that switches to native memory. Include API/copy and heap-buffer taxes in the comparison; do not count every stable read as a new 80,000 fee." }
                else { "Pack data used together into fewer regions and reuse buffers within one execution. A warmup in an earlier message does not remove this execution's tax. Batching must preserve behavior and stay within budgets." },
                vec![source(TRACKER, symbol, marker, if accessed { 29 } else { 22 })]);
        }
        add("ic.stable.read.direct.v1", "Untouched stable read uses PageMap", "decision",
            "Every requested 4 KiB stable bytemap granule was untouched in this execution and the generated code calls stable_read_first_access.",
            "0 additional instructions for this explanatory decision row; applicable stable API, heap and static fees remain.",
            "The helper copies from PageMap into the heap without accessing the native stable mapping for this copy. This is the limited meaning of a free first stable read. It is not a guarantee of 80,000 instructions saved: that native region might already be mapped by another operation.",
            "Read useful untouched spans once where possible. Another untouched 4 KiB span can still take this path even in the same 64 KiB region; API-call ordinal alone does not decide it.",
            vec![source(REPLACEMENTS, "replacement_functions", "// We need to call the first read API if this", 26), source(LINKER, "stable_read_first_access", ".func_wrap(\"__\", \"stable_read_first_access\"", 27)]);
        add("ic.stable.read.native.v1", "Overlapping stable read uses native copy", "decision",
            "At least one requested 4 KiB stable bytemap granule was previously accessed or written, selecting the native memory.copy branch.",
            "0 for the decision itself. Actual native stable access/dirty debits, if triggered, appear in their memory rules.",
            "Overlap switches the whole requested read to native stable memory. The first native access of a 64 KiB region can pay its access tax; later native accesses of that region in the same execution do not repeatedly pay it. The second read is therefore not universally expensive.",
            "Reuse already-read bytes in a coherent heap cache when it wins overall. Compare nonoverlapping reads, overlap, rereads and heap destination reuse; do not assume a global one-free-read allowance.",
            vec![source(REPLACEMENTS, "replacement_functions", "// perform the copy, calling API if it's the first access.", 48)]);
        for operation in ["read", "write"] {
            add(&format!("ic.stable64.{operation}.zero_length.v1"),
                &format!("Empty stable64 {operation}"), "decision",
                "The stable64 replacement takes its explicit zero-length early return.",
                "0 replacement copy/base instructions on this path. The original call and surrounding Wasm block can still have prepaid charges.",
                "The 64-bit API skips the replacement debit for an empty span. This does not describe the 32-bit stable API, whose fixed debit precedes its empty-span check.",
                "Skip empty operations in application code when convenient; that can save the original call and surrounding work too. Do not use an empty operation to validate nonempty bounds.",
                vec![source(REPLACEMENTS, "replacement_functions", if operation == "read" { "SystemApiFunc::Stable64Read," } else { "SystemApiFunc::Stable64Write," }, 38)]);
        }
        add("ic.runtime.other.v1", "Other observed runtime debit", "debit",
            "An actual instrumented debit uses a known runtime category without a more specific rule.",
            "The observed debit, preserved separately from unexplained reconciliation differences.",
            "This row accounts for a known debit point. It must not absorb the difference between the raw ledger and the runtime's reported total.",
            "Inspect the original operation/category and archived code. Keep unavailable source origins unmapped rather than assigning them to a nearby line.",
            vec![source(COLLECTOR, "observe_dynamic", "pub fn observe_dynamic(", 57)]);
        add("ic.budget.message_limit_clamp.v1", "Terminal instruction budget clamp", "decision",
            "The runtime reports a message-limit clamp with recorded instruction-counter evidence.",
            "0 additional debit. The signed adjustment is in ledger.adjustment, with its identified reason.",
            "A prepaid block or memory/API debit may overshoot before the next budget check. The runtime's final reported instruction total can clamp to the message limit, so raw debits can exceed that total. An unexplained difference remains a separate failure.",
            "Leave budget margin for API and memory taxes and inspect debit-boundary failures. Do not spread a clamp difference over source lines or call it compute.",
            vec![source(EMBEDDER, "set_instance_stats", "fn set_instance_stats(", 46)]);
        add("ic.budget.dts_refill.v1", "Deterministic time slicing refill", "decision",
            "The existing out-of-instructions handler returns a continuation counter for another slice.",
            "0 additional debit for the refill; execution work and carried overshoot remain accounted by actual charge rules.",
            "DTS changes the available slice budget, not the memory-access epoch. The same physical execution retains page state and profile counters across paused/resumed slices. A later message or callback is a new epoch.",
            "Distinguish slice limits from message limits. Do not expect yielding into a new DTS slice to reset or duplicate a first-access tax.",
            vec![source(COLLECTOR, "observe_budget_refill", "pub fn observe_budget_refill(", 28)]);
        add_runtime_rules(&mut rules);
        rules
    })
}

/// Host charges use their own units and transaction semantics, never the Wasm ledger.
fn add_runtime_rules(rules: &mut BTreeMap<String, InstructionChargeRule>) {
    let mut add = |id: &str,
                   title: &str,
                   condition: &str,
                   formula: &str,
                   description: &str,
                   tip: &str,
                   file,
                   marker: &str| {
        rules.insert(
            id.to_string(),
            InstructionChargeRule {
                version: 1,
                id: id.to_string(),
                title: title.to_string(),
                condition: condition.to_string(),
                formula: formula.to_string(),
                description: description.to_string(),
                tip: tip.to_string(),
                charge_kind: "host_cost".to_string(),
                source: {
                    let mut references = vec![
                        source(file, marker, marker, 36),
                        source(
                            FEES,
                            "CyclesAccountManagerConfig",
                            "pub struct CyclesAccountManagerConfig",
                            55,
                        ),
                    ];
                    match id {
                        "ic.cycles.dropped_message.v1" => {
                            references.push(source(ROUTING_STATE, "actual round loss", "let dropped_message_cycles_after =", 50));
                            references.push(source(REPLICATED_STATE, "observe_lost_cycles_due_to_dropped_messages", "pub fn observe_lost_cycles_due_to_dropped_messages(", 15));
                        }
                        "ic.cycles.canister_creation.v1" => {
                            references.push(source(MANAGER, "create_canister", "system_state.consume_cycles(creation_fee);", 28));
                            references.push(source(MANAGER, "committed creation observation", "\"ic.cycles.canister_creation.v1\"", 60));
                        }
                        "ic.cycles.http.v1" => references.push(source(ENVIRONMENT, "HTTP payment deduction", "// The payment deduction differs per pricing version.", 74)),
                        "ic.cycles.ecdsa.v1" | "ic.cycles.schnorr.v1" | "ic.cycles.vetkd.v1" => references.push(source(ENVIRONMENT, "sign_with_threshold", "// If the request isn't from the NNS, then we need to charge for it.", 100)),
                        _ => {}
                    }
                    if id.contains("http") {
                        references.push(source(
                            HTTP_PRICING,
                            "base_fee_amount",
                            "fn base_fee_amount(",
                            110,
                        ));
                    }
                    if id.contains("compilation") {
                        references.push(source(
                            EMBED_CONFIG,
                            "compilation prices",
                            "pub(crate) const DEFAULT_COST_TO_COMPILE_WASM_INSTRUCTION",
                            14,
                        ));
                    }
                    references
                },
            },
        );
    };
    add(
        "ic.cycles.ingress.v1",
        "Signed ingress induction",
        "Ingress is accepted or its deferred debt is settled.",
        "Normal 13-node/reference-13 application: 1,200,000 cycles + 2,000 × complete signed ingress envelope bytes.",
        "Deferred debit is a liability; the later debit is the balance charge. Never count both as paid. Normal fees scale with subnet size/reference size and cost schedule.",
        "Batch useful work when appropriate; argument length alone underestimates the signed envelope.",
        CAM,
        "pub fn charge_ingress_induction_cost(",
    );
    add(
        "ic.cycles.execution.v1",
        "Replicated execution and callback settlement",
        "An execution or callback prepays, adjusts prepayment, or refunds unused execution cycles.",
        "Normal reference-13 application: 5,000,000 cycles per execution + 1 × Wasm32 charged instructions or 2 × Wasm64 charged instructions.",
        "Prepayment is a worst-case debit, not final consumption. Subtract refunds once. Callbacks, heartbeat, global timer and low-memory tasks have their own execution base. A skipped callback can still retain the base. Install/reinstall/upgrade and snapshot load use one execution prepayment with a Wasm32 variable price even for Wasm64; multiple lifecycle-hook traces do not each add a new execution base.",
        "Reduce physical executions as well as guest instructions. Follow callback IDs and prepayments through settlement.",
        CAM,
        "pub fn prepay_execution_cycles(",
    );
    add(
        "ic.cycles.management_instructions.v1",
        "Native management instruction cycles",
        "Committed native management work consumes its instruction fee.",
        "Wasm32 instruction-cycle price × native instructions; no additional 5,000,000 native-management execution base.",
        "Management instruction rows explain the same work in instruction units; this cycle row converts that work into cycles. Do not add unlike units. Installation uses its execution prepayment path.",
        "Check the management-specific charge and failure ordering before retrying expensive operations.",
        CAM,
        "pub fn consume_cycles_for_final_instructions(",
    );
    add(
        "ic.cycles.transmission.v1",
        "Inter-canister request transport",
        "A successful outgoing call commits its request/base transport debit.",
        "Normal reference-13 application: 260,000 cycles per request/response pair + 1,000 × request bytes. Request bytes = UTF-8 method name + arguments.",
        "Same-subnet and self calls pay too. Response transmission and callback execution are separate prepayments; actual response bytes settle later.",
        "Send only necessary bytes. Include caller, callee and callback execution when comparing batching.",
        CAM,
        "pub fn xnet_total_transmission_fee(",
    );
    add(
        "ic.cycles.response_transmission.v1",
        "Response transport reserve and refund",
        "The sender reserves maximum response transport, then receives any unused portion.",
        "Normal reference-13 application: 1,000 cycles/byte; reserve 2,097,152,000 cycles for the 2 MiB maximum response, then refund unused bytes.",
        "Reply bytes are charged; rejection includes message bytes and RejectCode size. This is part of transport cost, not another fixed request base.",
        "Distinguish peak required balance from final fee; a tiny reply releases most of this reserve.",
        CAM,
        "pub fn refund_for_response_transmission(",
    );
    add(
        "ic.cycles.storage.v1",
        "Persistent memory and canister rent",
        "Batched resource charging bills elapsed time and allocated/used persistent memory.",
        "Normal reference-13 application: 317,500 cycles/GiB/second plus 10,000 cycles/second when accounted memory is nonzero.",
        "Memory includes heap, stable, Wasm, globals, logs, history, chunks and snapshots. Reserved memory allocation can exceed actual usage. Integer arithmetic and elapsed interval determine the observed amount.",
        "Remove unnecessary retained data and choose allocations deliberately; stopping computation does not stop rent.",
        CAM,
        "pub fn charge_canister_for_resource_allocation_and_usage(",
    );
    add(
        "ic.cycles.message_memory.v1",
        "In-flight message memory rent",
        "Batched resource charging bills queued messages and response reservations over elapsed time.",
        "Normal reference-13 application: 317,500 cycles/GiB/second of accounted message memory.",
        "Guaranteed-response calls reserve roughly maximum-response memory plus structures, while best-effort messages account stored bytes. This byte-time rent is separate from transmission cycles.",
        "Avoid unnecessary outstanding calls and long response waits; small payloads can still hold large guaranteed-response reservations.",
        CAM,
        "fn idle_cycles_burned_rate_by_resource(",
    );
    add(
        "ic.cycles.compute_allocation.v1",
        "Reserved compute allocation rent",
        "Batched resource charging bills the reserved compute percentage over elapsed time.",
        "Normal reference-13 application: 10,000,000 cycles per percentage point per second.",
        "Allocation rent is additional to executed instruction fees. A 1% reservation costs this rate even while idle.",
        "Reserve compute only when the scheduling guarantee is useful.",
        CAM,
        "pub fn compute_allocation_cost(",
    );
    add(
        "ic.cycles.storage_reservation.v1",
        "Storage balance reservation",
        "A committed memory allocation/usage increase moves cycles from liquid to reserved balance.",
        "Observed reservation amount; saturation-dependent marginal rent, up to configured reservation horizon. Default horizon maximum is 300,000,000 seconds; default reserved limit 5T cycles.",
        "This moves balance rather than immediately burning it. Reserved cycles subsequently pay resource rent. Freezing threshold is a required balance, not another fee.",
        "Inspect liquid and reserved balances separately; avoid treating reserved funds as consumed execution cycles.",
        CAM,
        "pub fn storage_reservation_cycles(",
    );
    add(
        "ic.cycles.transfer.v1",
        "Attached cycles transfer",
        "A committed call attaches/accepts cycles or returns unused attached payment.",
        "Observed transferred amount; not an IC resource tariff.",
        "Transfers change ownership and can be refunded. A service may later charge its own fee; attached payment is not automatically net consumption.",
        "Follow sender, receiver and callback; keep transfers separate from resource costs.",
        SANDBOX_STATE,
        "\"ic.cycles.transfer.v1\"",
    );
    add(
        "ic.cycles.burn.v1",
        "Explicit cycle burning",
        "A successful execution commits its explicit cycles_burn request.",
        "Actual burned amount, bounded by available cycles above required balance.",
        "This is a requested balance destruction, distinct from the API instruction charge and normal resource fees. Tentative execution burns are reported only at commit.",
        "Use only for intentional burning; inspect actual returned amount rather than assuming the requested amount was burned.",
        SANDBOX_STATE,
        "\"ic.cycles.burn.v1\"",
    );
    add(
        "ic.cycles.uninstall.v1",
        "Out-of-cycles uninstall loss",
        "Resource rent cannot be paid and the canister is uninstalled.",
        "Remaining liquid and reserved balances consumed by the uninstall path.",
        "A loss event, not a fixed uninstall price. It is separate from the rent attempted before uninstall.",
        "Maintain enough balance for rent and freezing thresholds.",
        SCHEDULER,
        "\"ic.cycles.uninstall.v1\"",
    );
    add(
        "ic.cycles.deleted_canister.v1",
        "Deletion balance loss",
        "A canister is deleted with a remaining balance.",
        "Remaining balances discarded by deletion.",
        "A loss event, not a fixed deletion service price.",
        "Withdraw or transfer desired balances before deletion where the API permits.",
        CAM,
        "ic.cycles.deleted_canister.v1",
    );
    add(
        "ic.cycles.dropped_message.v1",
        "Dropped message/refund loss",
        "Cycles in an undeliverable message or refund are discarded.",
        "Actual lost amount recorded by the drop path.",
        "A loss event, not ordinary per-byte transport. Best-effort late responses can involve dropped refunds.",
        "Inspect callback deadlines and late responses when balances fail to match expected refunds.",
        CAM,
        "ic.cycles.dropped_message.v1",
    );
    add(
        "ic.cycles.canister_creation.v1",
        "Canister creation",
        "A successful creation commits its creation fee.",
        "Normal reference-13 application: 500,000,000,000 cycles, scaled by subnet/reference size and cost schedule.",
        "Remaining attached funds initialize the new canister; they are not part of the creation fee.",
        "Separate creation fee, initial funding and storage reservation.",
        CAM,
        "pub fn canister_creation_fee(",
    );
    add(
        "ic.cycles.ecdsa.v1",
        "Threshold ECDSA signature",
        "An accepted signing request consumes its service fee.",
        "Normal reference-13: 10,000,000,000 cycles, scaled by subnet/reference size; NNS-origin caller exemption applies.",
        "System-subnet resource exemptions do not universally exempt chain-key service fees. Execution/transport and attached payment are separate.",
        "Use cost APIs for the effective quote; a quote itself does not consume the quoted service fee.",
        CAM,
        "pub fn ecdsa_signature_fee(",
    );
    add(
        "ic.cycles.schnorr.v1",
        "Threshold Schnorr signature",
        "An accepted signing request consumes its service fee.",
        "Normal reference-13: 10,000,000,000 cycles, scaled by subnet/reference size; NNS-origin caller exemption applies.",
        "Signing is separately priced from ordinary execution and transport.",
        "Check the effective quote and key identity before attaching payment.",
        CAM,
        "pub fn schnorr_signature_fee(",
    );
    add(
        "ic.cycles.vetkd.v1",
        "vetKD derivation",
        "An accepted derivation request consumes its service fee.",
        "Normal reference-13: 10,000,000,000 cycles, scaled by subnet/reference size; NNS-origin caller exemption applies.",
        "Derivation is separately priced from ordinary execution and transport.",
        "Check the effective quote and avoid unnecessary repeated derivations.",
        CAM,
        "pub fn vetkd_fee(",
    );
    add(
        "ic.cycles.http.v1",
        "HTTP outcall payment",
        "An accepted HTTP request consumes legacy payment or pays base plus service allowance.",
        "Legacy: N × (3,000,000 + 60,000N + 400Q + 800L). PAYG base and allowances use the archived pricing functions.",
        "Q includes URL, headers, body and transform identity/context; L is declared max response, default 2,000,000 decimal bytes. Legacy fees do not shrink with the actual response. PAYG service settlement/refunds use a separate accounting domain. HTTP formulas already include N; do not scale twice.",
        "Set a realistic response limit. Inspect pricing/replication details; a maximum allowance is not final PAYG usage.",
        CAM,
        "pub fn http_request_fee(",
    );
    add(
        "ic.cycles.http_allowance.v1",
        "HTTP service allowance funding",
        "An accepted PAYG HTTP call reserves per-replica usage allowances from its attached payment.",
        "Actual refundable usage allowance, bounded by payment beyond the base and the configured maximum usage quote, divided into whole per-node allowances.",
        "A reservation inside the HTTP service allowance domain, not another liquid-balance debit or a consumed fee. The base fee is separate; actual usage later consumes the allowance and unused funds are released for routing.",
        "Compare peak attached funding with settled usage. Do not add an allowance reservation to its usage as two resource bills.",
        ENVIRONMENT,
        "\"ic.cycles.http_allowance.v1\"",
    );
    add(
        "ic.cycles.http_usage.v1",
        "HTTP allowance usage settlement",
        "Initial or asynchronous reports settle actual network, transform and consensus work from prepaid service allowances.",
        "Per participating replica: 50 × raw bytes + 300 × response milliseconds + floor(transform instructions / 13); replication-specific gossip/consensus/base formulas are archived.",
        "This consumes an already transferred service allowance, without another liquid-balance debit. Reported usage is capped to the allowance; raw report and accounting domain remain in details. Free schedule preserves nominal values while actual cycles are zero.",
        "Compare actual service use with its prepaid allowance and pending releases. Do not add settlement to the original allowance as two balance debits.",
        HTTP_SETTLEMENT,
        "impl HttpCostTracing",
    );
    add(
        "ic.cycles.http_refund.v1",
        "HTTP unused allowance release",
        "Unused or timed-out service allowance is queued for refund routing.",
        "Actual unused allowance released, bounded by remaining refundable balance.",
        "Release is pending routing, not immediate liquid-balance credit. No duplicate report refunds the same allowance twice.",
        "Follow the refund through routing before expecting it in liquid balance.",
        HTTP_SETTLEMENT,
        "pub(crate) fn refund_timed_out_canister_http_contexts_with_tracing(",
    );
    add(
        "ic.scheduler.slice.v1",
        "Executed slice scheduler overhead",
        "An initial or resumed replicated slice performs charged work.",
        "Default 2,000,000 round-budget instructions per executed slice.",
        "A scheduler capacity debit, separate from guest execution instructions and cycles. Finished zero-instruction work skips this overhead.",
        "Reduce unnecessary physical executions; DTS slices can each consume scheduling capacity.",
        SCHEDULER,
        "\"ic.scheduler.slice.v1\"",
    );
    add(
        "ic.scheduler.canister.v1",
        "Canister pass scheduler overhead",
        "A canister pass accumulates positive instructions from completed messages.",
        "Default 8,000,000 round-budget instructions per eligible canister pass.",
        "Paused-only work with no completed-message instruction total does not trigger this debit.",
        "Inspect completed messages and scheduler passes before estimating capacity from slice counts.",
        SCHEDULER,
        "\"ic.scheduler.canister.v1\"",
    );
    add(
        "ic.scheduler.finalization.v1",
        "Inner-round finalization overhead",
        "Each inner scheduler iteration finishes.",
        "Default 12,000 round-budget instructions × every canister in replicated state.",
        "Shared subnet cost; includes inactive canisters and is not attributed entirely to the selected canister.",
        "Account for subnet-wide population when reasoning about available round capacity.",
        SCHEDULER,
        "\"ic.scheduler.finalization.v1\"",
    );
    add(
        "ic.scheduler.dirty_page_copy.v1",
        "Large dirty heap copy overhead",
        "Successful DTS execution dirties more than 262,144 heap OS pages.",
        "Default 3,000 round-budget instructions × dirty heap OS pages.",
        "This capacity accounting begins above 1 GiB with 4 KiB OS pages. It is not a guest-meter or additional cycle debit.",
        "Large dirty working sets consume scheduler capacity even when guest instruction totals are similar.",
        EXECUTOR,
        "\"ic.scheduler.dirty_page_copy.v1\"",
    );
    add(
        "ic.query.call_overhead.v1",
        "Nested query graph overhead",
        "A valid nested ordinary or management query callee is entered.",
        "Default 50,000,000 query-graph instructions per nested callee, from the shared 5,000,000,000 graph budget.",
        "The initial user query has no such call overhead. This limits graph execution; it is separate from Wasm instructions and replicated execution cycles.",
        "Batch query work and reduce unnecessary fanout; track depth and wall-time limits too.",
        QUERY,
        "\"ic.query.call_overhead.v1\"",
    );
    add(
        "ic.management.compilation.v1",
        "Wasm compilation charge",
        "Compilation/create-execution-state returns an actual instruction charge.",
        "20,000,000 fixed + 6,000 × counted instrumented operators/global initializer work; expected-compiled variable cost / 100. Failure uses byte-length-based work.",
        "Diagnostic additions are excluded. This is native compilation work and its conversion to cycles is accounted separately. Events retain actual budget delta and requested amount where a limit clamps.",
        "Reuse already compiled modules where applicable, and inspect failure charges before repeated invalid installations.",
        INSTALL,
        "\"ic.management.compilation.v1\"",
    );
    add(
        "ic.management.wasm_assembly.v1",
        "Chunked Wasm assembly",
        "Install or upgrade assembles listed Wasm chunks.",
        "1,048,576 execution instructions × listed chunk count, even for a short final chunk; ordinary unchunked assembly costs zero here.",
        "Compilation and guest initialization are additional.",
        "Avoid unnecessary chunks and repeated assembly; preserve module semantics.",
        INSTALL,
        "\"ic.management.wasm_assembly.v1\"",
    );
    add(
        "ic.management.upload_chunk.v1",
        "Wasm chunk upload",
        "A chunk upload reaches its upfront charge, including duplicates.",
        "Default 200,000,000 execution instructions per upload.",
        "Charging precedes validation; failed work can retain this fee. A duplicate upload still costs the fixed amount.",
        "List/reuse stored chunks rather than uploading known duplicates.",
        MANAGER,
        "\"ic.management.upload_chunk.v1\"",
    );
    add(
        "ic.management.log_resize.v1",
        "Canister log resize",
        "A management update resizes existing log storage.",
        "Default 32 execution instructions × previously stored log bytes.",
        "Read-log-content byte instructions are charged through their API. The cycle conversion is separate.",
        "Avoid unnecessary repeated resizing of a populated log.",
        MANAGER,
        "\"ic.management.log_resize.v1\"",
    );
    add(
        "ic.management.snapshot_memory.v1",
        "Snapshot take/load memory work",
        "A snapshot take or load succeeds.",
        "Default 2,000,000,000 execution instructions + logical snapshot bytes; load also has compilation work.",
        "Failure handling and upfront upload/read charges are recorded separately.",
        "Compare snapshot size and retained storage rent; repeated snapshots cost more than their call envelope.",
        ENVIRONMENT,
        "\"ic.management.snapshot_memory.v1\"",
    );
    add(
        "ic.management.snapshot_read_data.v1",
        "Snapshot data read",
        "A snapshot read reaches its upfront instruction charge.",
        "Default 5,000,000 execution instructions + requested maximum return bytes.",
        "The maximum requested length matters even when fewer bytes are returned.",
        "Request only the range needed.",
        MANAGER,
        "\"ic.management.snapshot_read_data.v1\"",
    );
    add(
        "ic.management.snapshot_upload_metadata.v1",
        "Snapshot metadata upload",
        "A metadata upload reaches its upfront instruction charge.",
        "Default 2,000,000,000 execution instructions + declared logical snapshot bytes.",
        "The declared snapshot size determines the variable component.",
        "Avoid repeated failed metadata uploads and unrealistic declarations.",
        MANAGER,
        "\"ic.management.snapshot_upload_metadata.v1\"",
    );
    add(
        "ic.management.snapshot_upload_data.v1",
        "Snapshot data upload",
        "A module/heap/stable snapshot chunk reaches its upfront charge.",
        "Default 2,000,000,000 execution instructions + chunk bytes.",
        "Executable code uses the snapshot baseline, even though a configuration comment mentions 5M for uploading/downloading. Wasm chunk upload has its separate 200M charge.",
        "Choose chunking with fixed overhead in mind and validate data before retrying.",
        MANAGER,
        "\"ic.management.snapshot_upload_data.v1\"",
    );
    add(
        "ic.management.failed_operation.v1",
        "Management work retained after failure",
        "A failed management operation rolls back state but retains already performed work.",
        "Actual retained execution-instruction debit after rollback.",
        "Tentative events are discarded before this committed failure accounting is published. Do not add discarded successful-path estimates.",
        "Inspect the failure and fix its cause before retrying expensive native operations.",
        ENVIRONMENT,
        "\"ic.management.failed_operation.v1\"",
    );
    add_configuration_rules(rules);
}

fn add_configuration_rules(rules: &mut BTreeMap<String, InstructionChargeRule>) {
    // Exact captured keys double as lookup IDs in the Limits view. Values are
    // supplied by each running subnet; these descriptions never override them.
    // Enumerate the actual captured keys rather than reparsing the producer's
    // Rust macro invocations. The existing config constructors own these facts.
    let config = ic_config::execution_environment::Config::default();
    let subnet = ic_config::subnet_config::SubnetConfig::new(
        ic_registry_subnet_type::SubnetType::Application,
    );
    let capacity = (subnet.scheduler_config.scheduler_cores - 1) * 100;
    let defaults = ic_execution_environment::runtime_configuration_defaults(
        &config,
        &subnet,
        ic_registry_subnet_type::SubnetType::Application,
        capacity,
    );
    let keys = defaults.into_keys().chain([
        "subnet_size".into(),
        "reference_subnet_size".into(),
        "cost_schedule".into(),
        "subnet_type".into(),
    ]);
    for key in keys {
        let field = key.rsplit('.').next().unwrap_or(&key);
        let declaration = format!("pub {field}:");
        let declared_in = match key.split('.').next() {
            Some("cycles" | "scheduler") => FEES,
            Some("embedder") => EMBED_CONFIG,
            _ => EXEC_CONFIG,
        };
        let literal = format!("\"{key}\"");
        let (file, marker) = if !key.contains('.') {
            (FEES, "pub struct CyclesAccountManagerConfig".to_string())
        } else if declared_in.1.contains(&declaration) {
            (declared_in, declaration)
        } else if CONFIG_CAPTURE.1.contains(&literal) {
            (CONFIG_CAPTURE, literal)
        } else {
            (
                CONFIG_CAPTURE,
                "pub fn runtime_configuration_defaults(".to_string(),
            )
        };
        let explanation = configuration_explanation(&key);
        rules.entry(key.clone()).or_insert_with(|| InstructionChargeRule {
            version: 1, id: key.clone(), title: key.clone(), charge_kind: "configuration".to_string(),
            condition: "Runtime configuration captured from the selected subnet; default-prefixed entries explicitly describe source defaults rather than an overridden effective value.".to_string(),
            formula: "Use the captured value and unit in the key; archived source defines its application. Fees in the cycles group are unscaled configured prices unless stated otherwise.".to_string(),
            description: explanation.to_string(),
            tip: "Treat a budget as capacity, not a fee or a throughput guarantee. Compare the actual subnet configuration, cost schedule and source version.".to_string(),
            source: vec![source(file, &key, &marker, 18)],
        });
    }
}

fn configuration_explanation(key: &str) -> &'static str {
    match key {
        "scheduler.max_instructions_per_round" => {
            "Instruction budget per execution thread per round: application default 4 billion, system 7 billion. This counts instructions, not executions. Scheduler overhead and fairness reduce usable capacity."
        }
        "scheduler.max_instructions_per_message" => {
            "Replicated message budget: application 40 billion instructions, system 50 billion. Deterministic slices share this message budget."
        }
        "scheduler.max_instructions_per_slice" => {
            "Deterministic execution slice budget: default 2 billion instructions. Resuming does not start a new memory first-access epoch."
        }
        "scheduler.max_instructions_per_query_message" => {
            "Per-query message budget: application 5 billion, system 50 billion; composite graph and time/depth limits also apply."
        }
        "scheduler.max_instructions_per_install_code" => {
            "Install/upgrade total instruction budget: application 300 billion, system 1 trillion; compilation and initialization consume it."
        }
        "scheduler.max_instructions_per_install_code_slice" => {
            "Install slicing budget: application 2 billion, system 5 billion; total installation budget is separate."
        }
        "scheduler.page_overhead" => {
            "Instructions per OS page at a first native 64 KiB region fault; with 4 KiB OS pages and value 5,000, access costs 80,000 and first dirtying adds another 80,000."
        }
        "cycles.update_message_execution_fee" => {
            "Fixed cycle price per replicated execution, application default 5 million; this is separate from the 2 million scheduler instruction overhead."
        }
        "cycles.xnet_byte_transmission_fee" => {
            "Cycles per request/response byte, application default 1,000, including same-subnet and self calls. Queue byte-time rent is separate."
        }
        "cycles.ingress_byte_reception_fee" => {
            "Cycles per byte of the entire signed ingress envelope, application default 2,000; argument length alone is insufficient."
        }
        "scheduler.scheduler_cores" => {
            "Replicated execution threads, default four. Each owns its round budget; concurrency does not establish guaranteed 16-billion-instruction throughput."
        }
        "scheduler.max_paused_executions" => {
            "Maximum simultaneously paused long executions, default four. This limits DTS concurrency rather than messages admitted into a block."
        }
        "scheduler.instruction_overhead_per_execution" => {
            "Round-budget instructions per executed initial or resumed slice, default two million. This is separate from guest instructions and the five-million-cycle execution base."
        }
        "scheduler.instruction_overhead_per_canister" => {
            "Round-budget instructions per canister pass with completed positive instruction work, default eight million; paused-only work is excluded."
        }
        "scheduler.instruction_overhead_per_canister_for_finalization" => {
            "Round-budget instructions per canister in state for each inner scheduler iteration, default twelve thousand. This is shared subnet work."
        }
        "embedder.max_wasm_memory_size" => {
            "Hard Wasm32 heap limit in bytes, default four GiB. The canister's separate wasm_memory_limit setting can be lower for update/init/start."
        }
        "embedder.max_wasm64_memory_size" => {
            "Hard Wasm64 heap limit in bytes, default six GiB; a 64-bit address space does not imply unlimited usable memory."
        }
        "embedder.max_stable_memory_size" => {
            "Hard stable-memory byte limit, default 500 GiB. Per-execution accessed/dirty page caps are independent and much smaller."
        }
        "memory.default_wasm_memory_limit" => {
            "Default canister wasm_memory_limit, three GiB. It applies to start/init/update and install validation, is configurable per canister, and zero disables this setting. Hard engine heap limits remain."
        }
        "memory.canister_queue_capacity" => {
            "Default 500 request slots and 500 response/reservation slots separately per peer in each direction, not 500 total messages per canister. The ingress queue has no count bound here."
        }
        "memory.allocatable_compute_capacity_in_percent" => {
            "Portion of scheduler capacity allocatable as reservations, default 50%. Four threads provide (4−1)×100=300% nominal capacity, then 150% allocatable; manager preserves one percentage point of headroom."
        }
        "query.max_query_call_graph_depth" => {
            "Maximum composite query depth, default six. The initial query, nested-call overhead, shared graph budget and wall time also constrain execution."
        }
        "query.instruction_overhead_per_call" => {
            "Query-graph instruction debit per valid nested callee, default fifty million. The initial user query does not pay this graph-call overhead."
        }
        "query.max_call_walltime_millis" | "query.effective_max_call_walltime_millis_at_setup" => {
            "Composite query wall-time ceiling in milliseconds, default ten seconds; effective setup snapshot includes registry override."
        }
        "query.query_execution_threads_total" => {
            "Configured query worker count, default four; this is separate from the replicated scheduler's four threads."
        }
        "query.execution_threads_per_canister" => {
            "Maximum concurrent query executions per canister, default two."
        }
        "query.max_inspect_message_instructions" => {
            "Instruction budget for ingress inspect_message admission, default 200 million. This is not the normal replicated execution-cycle prepayment path."
        }
        "network.default.max_ingress_messages_per_block" => {
            "Default consensus-block ingress admission ceiling, one thousand. Messages can execute in later rounds; this is neither a per-round execution guarantee nor an ingress queue bound."
        }
        "network.default.initial_notary_delay_millis" => {
            "Initial consensus notary delay, default 300 milliseconds. Rank delays and backlog adjustments also apply, so this does not guarantee three rounds per second. PocketIC ticks use their own simulated progression."
        }
        "network.max_same_subnet_request_payload_bytes" => {
            "Maximum local request method-name plus argument bytes, ten MiB by default. Remote requests and all inter-canister responses retain their separate two-MiB limit."
        }
        "network.max_inter_canister_payload_bytes" => {
            "Maximum remote request or response payload, default two MiB; local requests have a separate five-times allowance. Response and signed-ingress limits are different quantities."
        }
        "network.max_xnet_payload_bytes_with_metadata" => {
            "Wire-envelope allowance of about 2.1 MiB including metadata; it does not increase the two-MiB application payload limit."
        }
        "network.max_best_effort_call_timeout_seconds" => {
            "Upper bound for best-effort call timeout, default 300 seconds. Caller-provided deadlines are clamped to this bound."
        }
        _ if key.starts_with("embedder.stable_memory_accessed_page_limit.")
            || key.starts_with("embedder.stable_memory_dirty_page_limit.") =>
        {
            "Unique OS-page count per physical execution: default regular message 524,288 (2 GiB), init/preupgrade 2,097,152 (8 GiB), query/inspect/composite callbacks 262,144 (1 GiB), using 4-KiB pages. DTS slices retain the epoch. These are distinct from 64-KiB native memory taxes."
        }
        _ if key.ends_with("_at_setup") => {
            "Registry-resolved limit captured when this execution environment was constructed. A later registry update can change effective limits; this historical snapshot is not a live guarantee."
        }
        "subnet_size" => "Actual node count N used by this subnet's cost calculations.",
        "reference_subnet_size" => {
            "Reference R used for ordinary fee scaling floor(price × N / R); normally 13, or 7 for SEV non-system subnets."
        }
        "cost_schedule" => {
            "Normal or Free resource charging. Free preserves nominal cycle costs while actual charges are zero; explicit burns and balance losses remain separate."
        }
        _ => {
            "Archived source explains this limit or price; the captured value belongs to this subnet and build. It is configuration rather than an observed charge."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_emitted_host_cost_has_an_archived_explanation() {
        let rules = charge_rules();
        for (path, contents) in [
            CAM,
            SANDBOX_STATE,
            EXECUTOR,
            SCHEDULER,
            MANAGER,
            ENVIRONMENT,
            QUERY,
            INSTALL,
            UPGRADE,
            INSTALL_CODE,
            COMMON,
            RESPONSE,
            HTTP_SETTLEMENT,
            ROUTING_STATE,
            REPLICATED_STATE,
        ] {
            for id in contents.split('"').skip(1).step_by(2).filter(|id| {
                ["ic.cycles.", "ic.scheduler.", "ic.management.", "ic.query."]
                    .iter()
                    .any(|p| id.starts_with(p))
                    && id.ends_with(".v1")
                    && !id.contains(char::is_whitespace)
            }) {
                assert!(
                    rules.contains_key(id),
                    "missing rule {id} emitted in {path}"
                );
            }
        }
        for key in [
            "scheduler.scheduler_cores",
            "query.instruction_overhead_per_call",
            "embedder.stable_memory_accessed_page_limit.query",
            "memory.default_wasm_memory_limit",
            "network.default.max_ingress_messages_per_block",
        ] {
            assert_eq!(
                rules[key].charge_kind, "configuration",
                "missing actual configuration explanation {key}"
            );
        }
    }

    #[test]
    fn memory_access_rule_dictionary_has_exact_compiled_sources() {
        let files = [
            METER,
            REPLACEMENTS,
            LINKER,
            TRACKER,
            EMBEDDER,
            COLLECTOR,
            CAM,
            FEES,
            EXEC_CONFIG,
            EMBED_CONFIG,
            SCHEDULER,
            MANAGER,
            ENVIRONMENT,
            QUERY,
            INSTALL,
            UPGRADE,
            INSTALL_CODE,
            COMMON,
            RESPONSE,
            SANDBOX_STATE,
            EXECUTOR,
            HTTP_PRICING,
            HTTP_SETTLEMENT,
            LIMITS,
            ROUTING_STATE,
            REPLICATED_STATE,
            CONFIG_CAPTURE,
        ];
        let rules = charge_rules();
        assert!(rules.len() >= 90);
        for (id, rule) in rules {
            assert_eq!(id, &rule.id);
            assert!(!rule.condition.is_empty() && !rule.tip.is_empty());
            assert!(matches!(
                rule.charge_kind.as_str(),
                "debit" | "decision" | "host_cost" | "configuration"
            ));
            assert!(!rule.source.is_empty());
            if id.starts_with("ic.memory.") {
                assert!(
                    rule.source
                        .iter()
                        .any(|reference| reference.code.contains("subtract_instruction_counter")),
                    "memory rule must cite its actual debit, not only bitmap bookkeeping: {id}"
                );
            }
            for reference in &rule.source {
                let (_, contents) = files
                    .iter()
                    .find(|(path, _)| path == &reference.file)
                    .unwrap();
                assert_eq!(
                    reference.sha256,
                    hex::encode(Sha256::hash(contents.as_bytes()))
                );
                let excerpt = contents
                    .lines()
                    .skip(reference.line as usize - 1)
                    .take(reference.code.lines().count())
                    .collect::<Vec<_>>()
                    .join("\n");
                assert_eq!(reference.code, excerpt);
            }
        }
    }
}
