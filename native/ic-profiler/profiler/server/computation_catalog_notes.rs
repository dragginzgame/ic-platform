//! Interpretation shared by the standalone computation catalog.
//! Values here describe units and evidence; active subnet settings remain in
//! instance configuration snapshots, independently of this source catalog.

use crate::instruction_charge_rules::source;
use serde_json::{Value, json};

const INTERFACES: (&str, &str) = (
    "rs/interfaces/src/execution_environment.rs",
    include_str!("../../../interfaces/src/execution_environment.rs"),
);
const SCHEDULER: (&str, &str) = (
    "rs/execution_environment/src/scheduler.rs",
    include_str!("../../../execution_environment/src/scheduler.rs"),
);
const SUBNET_CONFIG: (&str, &str) = (
    "rs/config/src/subnet_config.rs",
    include_str!("../../../config/src/subnet_config.rs"),
);
const STATE_MACHINE: (&str, &str) = (
    "rs/state_machine_tests/src/lib.rs",
    include_str!("../../../state_machine_tests/src/lib.rs"),
);
const API_STATE: (&str, &str) = (
    "rs/pocket_ic_server/src/state_api/state.rs",
    include_str!("../../src/state_api/state.rs"),
);
const COLLECTOR: (&str, &str) = (
    "rs/embedders/src/wasmtime_embedder/profiling.rs",
    include_str!("../../../embedders/src/wasmtime_embedder/profiling.rs"),
);
const RECORDER: (&str, &str) = (
    "rs/pocket_ic_server/profiling/server/memory_access_tracing.rs",
    include_str!("memory_access_tracing.rs"),
);
const HTTP: (&str, &str) = (
    "rs/types/types/src/canister_http.rs",
    include_str!("../../../types/types/src/canister_http.rs"),
);
const ROUTING: (&str, &str) = (
    "rs/config/src/message_routing.rs",
    include_str!("../../../config/src/message_routing.rs"),
);
const CHUNKS: (&str, &str) = (
    "rs/replicated_state/src/canister_state/system_state/wasm_chunk_store.rs",
    include_str!("../../../replicated_state/src/canister_state/system_state/wasm_chunk_store.rs"),
);
const MANAGER: (&str, &str) = (
    "rs/execution_environment/src/canister_manager.rs",
    include_str!("../../../execution_environment/src/canister_manager.rs"),
);
const SYSTEM_STATE: (&str, &str) = (
    "rs/replicated_state/src/canister_state/system_state.rs",
    include_str!("../../../replicated_state/src/canister_state/system_state.rs"),
);

pub(crate) fn interpretation_notes() -> Value {
    let resource_poll_every_rounds =
        ic_execution_environment::resource_charge_interval_rounds().to_string();
    json!({
        "version": 1,
        "value_authority": {
            "source_default": "A value compiled into this binary; no instance is required. Registry overrides, instance options, canister settings and cost schedules can change the applicable value.",
            "hard_limit": "A boundary enforced by this source implementation, with the stated scope. Other limits can reject an operation first.",
            "derived_example": "Arithmetic under explicit assumptions, not a measurement or a deployment guarantee.",
            "effective_at_setup": "A historical registry-resolved configuration snapshot from instance setup, not a continuously refreshed active limit.",
            "observed_amount": "An amount recorded at the actual accounting point. An unobserved rule is not evidence of a zero price.",
            "integer_encoding": "Quantities are exact base-10 integer strings. Bytes, OS pages, Wasm pages, instructions, cycles, rounds and time units remain distinct."
        },
        "quantity_domains": [
            {"unit": "execution_instructions", "scope": "physical_wasm_execution_or_native_management_operation",
             "meaning": "Weighted guest work and actual API/memory debits, or separately identified native management work.",
             "combination": "Sum exclusive components within one operation. A cycle conversion of these instructions is another unit view of the same work."},
            {"unit": "round_instructions", "scope": "replicated_scheduler_round",
             "meaning": "Scheduler capacity consumed by execution/canister/finalization/copy overhead.",
             "combination": "Do not add to the guest instruction ledger or convert into an extra execution-cycle invoice."},
            {"unit": "query_graph_instructions", "scope": "nonreplicated_query_call_graph",
             "meaning": "Graph work and nested-call admission overhead under query graph limits.",
             "combination": "Keep distinct from round instructions and replicated execution-cycle fees. Nonreplicated queries do not pay the ordinary replicated execution-cycle invoice."},
            {"unit": "cycles", "scope": "explicit_accounting_domain",
             "meaning": "Balance charges or movements, classified by kind. Nominal cycles preserve the normal-price comparison on a free schedule.",
             "combination": "Group by accounting domain and scope before netting. Never mix actual and nominal quantities."}
        ],
        "accounting": {
            "kinds": [
                {"kind": "debit", "meaning": "Consumption or a recorded loss in the stated accounting domain."},
                {"kind": "deferred_debit", "meaning": "A liability recorded for later induction, not another paid debit."},
                {"kind": "prepayment", "meaning": "Upfront funding which can later be partly refunded; it is not final consumption."},
                {"kind": "refund", "meaning": "An amount returned from the corresponding prepayment; subtract once in that domain."},
                {"kind": "reservation", "meaning": "An amount set aside, including storage reserve or service allowance, rather than immediate consumption."},
                {"kind": "release", "meaning": "Reservation/allowance becomes available in its destination domain; this need not immediately credit the caller's liquid balance."},
                {"kind": "transfer", "meaning": "Ownership or accounting-domain movement, separately from fees."}
            ],
            "booked_net_formula": "sum(debit) + sum(prepayment) - sum(refund)",
            "booked_net_preconditions": "Use one unit, accounting domain and scope; inspect transfers between domains and recording completeness. This formula alone does not reconcile a canister balance.",
            "exclude_from_booked_net": ["deferred_debit", "reservation", "release", "transfer"],
            "overlapping_views": [
                "Profile blocks, rule charges, function allocation, page detail and weighted stacks project existing charges; they are not independent bills.",
                "Management instruction events and their cycle-conversion events describe the same native work in different units.",
                "HTTP allowance funding, consumption and routing refunds are distinct stages. Usage does not debit the liquid balance a second time.",
                "A DTS slice continuation is part of the same physical execution; a callback or later message starts another execution and memory first-touch epoch."
            ],
            "sources": [source(INTERFACES, "RuntimeCostUnit / RuntimeCostKind", "pub enum RuntimeCostUnit", 45)]
        },
        "pricing": {
            "ordinary_scaling": "Apply the charging subnet's N and reference R at each runtime component: floor(configured_price * N / R). R is normally 13; SEV non-system pricing can use 7. Settlement sites can round separately.",
            "http_scaling": "HTTP formulas already include subnet node count N; do not apply ordinary N/R scaling again.",
            "free_schedule": "Actual resource fees can be zero while nominal costs remain nonzero. Explicit burns and lost balances remain separate operations.",
            "system_subnet": "System defaults waive many ordinary fees. Do not infer that every service is free: chain-key service exemptions depend on caller origin as well as the charging configuration.",
            "estimators": "cost_* APIs quote service costs and incur their own instruction work. Calling an estimator does not perform or purchase the quoted service."
        },
        "installation_accounting": {
            "operations": ["install", "reinstall", "upgrade", "load_canister_snapshot"],
            "cycle_pricing": "These operations use execution prepayment: one configured message base plus consumed instructions at the Wasm32 variable cycle rate, including when the installed module is Wasm64. The normal reference-13 application message base is 5,000,000 cycles.",
            "lifecycle_hooks": "Several lifecycle-hook Wasm traces within one installation do not each incur another message base. Include compilation and host management work without adding their cycle conversion as a second instruction bill.",
            "other_management_work": "Other native management instruction charges use their applicable Wasm32 variable pricing without this execution-message base.",
            "rule_ids": ["ic.cycles.execution.v1", "ic.cycles.management_instructions.v1"],
            "sources": [
                source(MANAGER, "installation prepayment", "// For installing code, Wasm64 does not differ from Wasm32.", 24),
                source(MANAGER, "snapshot loading prepayment", "module outside the sandbox, so we default to Wasm32 (undercharging", 27)
            ]
        },
        "cadence": {
            "resource_poll_every_rounds": resource_poll_every_rounds,
            "resource_poll_on_checkpoints": true,
            "resource_elapsed_gate_configuration_key": "cycles.allocation_charge_interval_seconds",
            "resource_skip_paused_update_or_install": true,
            "resource_charge_polling": format!("The scheduler checks resource charging every {resource_poll_every_rounds} rounds or on checkpoint rounds. A canister is charged only after its configured allocation-charge interval has elapsed, and paused update/install executions defer charging."),
            "allocation_charge_interval_source_default": "Normal application and system source defaults are 10 seconds; read the actual configured interval. Elapsed whole seconds and integer rounding determine charges.",
            "consensus": "A round is an execution batch, not a fixed duration. Initial notary delay, rank delays, backlog and execution affect progress; no fixed rounds-per-second guarantee follows from these constants.",
            "pocketic_implicit_round_time_increment_nanoseconds": ic_state_machine_tests::StateMachine::EXECUTE_ROUND_TIME_INCREMENT.as_nanos().to_string(),
            "pocketic_time": "Ticks advance simulated execution. The implicit round increment applies when time otherwise does not advance; explicit advance_time and auto-progress control time separately.",
            "auto_progress": "Auto-progress sleeps at least MIN_OPERATION_DELAY after its operations, with optional artificial delay. This is a local test-driver cadence, not mainnet consensus throughput.",
            "sources": [
                source(SCHEDULER, "resource polling interval", "const CHARGE_INTERVAL_ROUNDS:", 1),
                source(SCHEDULER, "resource charging", "fn charge_canisters_for_resource_allocation_and_usage(", 57),
                source(SUBNET_CONFIG, "application allocation interval", "duration_between_allocation_charges: Duration::from_secs(10)", 1),
                source(STATE_MACHINE, "implicit round time", "pub const EXECUTE_ROUND_TIME_INCREMENT:", 1),
                source(API_STATE, "auto-progress sleep", "let sleep_duration = std::cmp::max(artificial_delay, MIN_OPERATION_DELAY);", 2),
                source(API_STATE, "minimum operation delay", "const MIN_OPERATION_DELAY:", 1)
            ]
        },
        "timer_example": {
            "value_kind": "derived_example",
            "sdk": "ic-cdk-timers", "sdk_version": "1.0.0",
            "scope": "One simple SDK timer firing with a successful self-call and empty callback response; shared dispatch, retries, asynchronous tasks and SDK versions can change the path.",
            "execution_path": ["global_timer_dispatch", "self_call_update", "callback"],
            "parameters": {"reference_subnet_nodes": "13", "reference_size": "13", "method_name_bytes": "32", "timer_id_argument_bytes": "8", "response_bytes": "0", "physical_executions": "3"},
            "arithmetic": "3 * configured_execution_base + configured_call_base + configured_byte_fee * (32 + 8 + 0), before actual instruction-cycle charges; do not add prepayment and settlement twice.",
            "interpretation": "Setting the IC global timer pays its API instruction fee and schedules a one-shot system task. It does not buy a recurring subscription. SDK cancellation can leave a dispatcher wakeup already armed.",
            "external_sdk_source": "https://docs.rs/crate/ic-cdk-timers/1.0.0/source/src/global_timer.rs",
            "external_sdk_source_scope": "Version-pinned illustrative SDK source, separately from the replica source compiled into this catalog."
        },
        "evidence_and_coverage": {
            "run_identity": "Use server-generation UUID and binary hash with instance/canister identity. Instance IDs and sequence numbers can repeat after a restart.",
            "module_identity": "Installed-module hashes identify exact uncompressed instrumentation input bytes. Join an origin dictionary only when its module hash and supported hash kind match; retain instrumentation/configuration/source hashes.",
            "source_identity": "Rust names, source lines and inline frames need exact matching SDK origin maps, debug artifacts and archived source contents. Current-checkout files are not a replacement for a matching artifact.",
            "prepaid_blocks": "Block charges allocate prepaid operators; an early trap does not prove every operator in the block executed.",
            "physical_stacks": "Frames describe physical Wasm calls and final instrumented module-file offsets. Tail-elided or asynchronous causal frames are not recovered Rust stacks. Exported frame/detail limits are visible; Wasmtime's underlying capture still walks full physical stack depth before the retained-frame bound.",
            "memory_origins": "A native fault PC and a proven scoped host/bulk operation are different evidence. Unmapped helpers, missing ranges and unavailable caller chains remain unmapped; do not use the last block or a neighboring PC as an exact origin.",
            "attempt_lifecycle": "Physical attempt IDs survive DTS resumes; aborted attempts retain host-owned debit evidence and exact available counters without reading destroyed guest memory. Slice records and their exact retained total describe scheduler round instructions, including explicitly labeled dirty-copy yields; never sum them into the guest ledger. Availability flags distinguish missing memory ranges/counters from zero work. Logical IDs join retries only through actual message/task identity; commit or discard comes from the execution environment after state application or complete install/upgrade settlement. Published records wait for final disposition and remain immutable. A selected profile is not a complete instruction history for all work in an instance.",
            "retention": "Detail truncation and accounting-total completeness are separate. Missing refund/settlement history prevents a final net-consumption claim even when retained events are exact.",
            "balance_reconciliation_complete": false,
            "known_balance_credit_gaps": ["full installation prepayment refunds after subnet-split abort", "late or duplicate best-effort response credits", "routing refund pool to liquid balance", "cycle deposits", "provisional cycle minting"],
            "balance_credit_gap_scope": "These balance-changing paths are outside the currently observed accounting inventory. An exact sum of retained fee events does not establish complete liquid/reserved balance reconciliation.",
            "correlation": "Host events carry only correlations supplied at their accounting site; they are not assigned to the most recent Wasm execution.",
            "sources": [
                source(COLLECTOR, "bounded selected stack observation", "pub(crate) fn observe_charge_stack(", 45),
                source(RECORDER, "origin dictionary identity", "// Intern immutable origins once per installed module.", 22),
                source(INTERFACES, "host correlation contract", "/// An observed host accounting operation.", 4),
                source(SYSTEM_STATE, "split-aborted installation refund", "pub fn drop_in_progress_management_calls_after_split(", 12),
                source(MANAGER, "cycle deposit credit", "pub(crate) fn deposit_cycles(", 11)
            ]
        },
        "external_service_boundary": {
            "services": ["Bitcoin service canister", "Dogecoin service canister", "EVM RPC canister", "SOL RPC canister"],
            "price_authority": "Separately deployed and versioned service canisters, not this replica source catalog.",
            "required_quote_evidence": ["service_principal_and_network", "service_interface_or_implementation_version", "request_parameters", "quote_or_configuration_timestamp", "attached_payment", "actual_refund_or_settlement"],
            "interpretation": "Fetch the service's current cost quote and pin its configuration before budgeting. Minimum cycle attachment, retained collateral, service fees and miner/network fees can differ. SDK attachment helpers and local privileged adapters do not establish a deployed net tariff.",
            "documentation_reference": "https://docs.internetcomputer.org/references/cycle-costs/",
            "documentation_reference_scope": "A discovery reference, not a live price assertion or a substitute for a versioned service quote."
        }
    })
}

/// Public source constants absent from the per-subnet setup capture. Every
/// numeric value comes from the implementation, not a copied paper table.
pub(crate) fn supplemental_limits() -> Value {
    use ic_types::canister_http::*;
    let limit = |id: &str,
                 value: String,
                 unit: &str,
                 kind: &str,
                 scope: &str,
                 note: &str,
                 file: (&str, &str),
                 symbol: &str| {
        let marker = format!("pub const {symbol}:");
        let declaration = file.1.split_once(&marker).unwrap().1;
        let lines = declaration.split_once(';').unwrap().0.lines().count();
        json!({"id": id, "value": value, "unit": unit, "value_kind": kind,
            "scope": scope, "description": note,
            "source": [source(file, symbol, &marker, lines)]})
    };
    json!([
        limit(
            "http.max_request_header_body_bytes",
            MAX_CANISTER_HTTP_REQUEST_BYTES.to_string(),
            "bytes",
            "hard_limit",
            "http_request_headers_and_body",
            "This validation quantity is headers plus body; HTTP pricing Q additionally includes URL and transform identity/context.",
            HTTP,
            "MAX_CANISTER_HTTP_REQUEST_BYTES"
        ),
        limit(
            "http.max_response_bytes",
            MAX_CANISTER_HTTP_RESPONSE_BYTES.to_string(),
            "bytes",
            "hard_limit",
            "http_raw_response",
            "The request's selected max_response_bytes can be lower. Candid encoding and maximum rejection size have separate allowances.",
            HTTP,
            "MAX_CANISTER_HTTP_RESPONSE_BYTES"
        ),
        limit(
            "http.max_url_bytes",
            MAX_CANISTER_HTTP_URL_SIZE.to_string(),
            "bytes",
            "hard_limit",
            "http_request_url",
            "URL byte limit, independent of the headers/body limit.",
            HTTP,
            "MAX_CANISTER_HTTP_URL_SIZE"
        ),
        limit(
            "http.max_header_count",
            MAX_CANISTER_HTTP_HEADER_NUM.to_string(),
            "headers",
            "hard_limit",
            "http_request",
            "Maximum header count.",
            HTTP,
            "MAX_CANISTER_HTTP_HEADER_NUM"
        ),
        limit(
            "http.max_header_name_or_value_bytes",
            MAX_CANISTER_HTTP_HEADER_NAME_VALUE_LENGTH.to_string(),
            "bytes",
            "hard_limit",
            "individual_http_header_name_or_value",
            "Name and value are checked independently, rather than as one combined field.",
            HTTP,
            "MAX_CANISTER_HTTP_HEADER_NAME_VALUE_LENGTH"
        ),
        limit(
            "http.max_total_header_bytes",
            MAX_CANISTER_HTTP_HEADER_TOTAL_SIZE.to_string(),
            "bytes",
            "hard_limit",
            "http_request_headers",
            "Sum of all header name and value byte lengths.",
            HTTP,
            "MAX_CANISTER_HTTP_HEADER_TOTAL_SIZE"
        ),
        limit(
            "http.max_error_message_bytes",
            MAXIMUM_CANISTER_HTTP_ERROR_MESSAGE_BYTES.to_string(),
            "bytes",
            "hard_limit",
            "http_reject_message",
            "A rejection is not bounded by the caller's successful max_response_bytes.",
            HTTP,
            "MAXIMUM_CANISTER_HTTP_ERROR_MESSAGE_BYTES"
        ),
        limit(
            "http.max_counted_reject_bytes",
            MAX_CANISTER_HTTP_REJECT_BYTES.to_string(),
            "bytes",
            "hard_limit",
            "http_reject_accounting",
            "Reject code plus maximum reject message bytes, using the runtime's actual counted size.",
            HTTP,
            "MAX_CANISTER_HTTP_REJECT_BYTES"
        ),
        limit(
            "http.candid_encoding_reserve_bytes",
            CANDID_OVERHEAD_RESERVE_BYTES.to_string(),
            "bytes",
            "hard_limit",
            "http_delivered_response_allowance",
            "Additional delivery allowance above raw max_response_bytes, not an extra raw-response limit.",
            HTTP,
            "CANDID_OVERHEAD_RESERVE_BYTES"
        ),
        limit(
            "http.max_responses_per_block",
            CANISTER_HTTP_MAX_RESPONSES_PER_BLOCK.to_string(),
            "responses",
            "hard_limit",
            "consensus_block_http_payload",
            "Timeouts and asynchronous receipts do not count towards this limit.",
            HTTP,
            "CANISTER_HTTP_MAX_RESPONSES_PER_BLOCK"
        ),
        limit(
            "http.timeout_interval_seconds",
            CANISTER_HTTP_TIMEOUT_INTERVAL.as_secs().to_string(),
            "seconds",
            "hard_limit",
            "http_request_context_timeout",
            "Request budget and adapter timeout can end work earlier; this is not a response latency guarantee.",
            HTTP,
            "CANISTER_HTTP_TIMEOUT_INTERVAL"
        ),
        limit(
            "stream.target_size_bytes",
            ic_config::message_routing::TARGET_STREAM_SIZE_BYTES.to_string(),
            "bytes",
            "source_default",
            "outgoing_xnet_stream",
            "Enqueuing stops once the target is reached; the final message can overshoot this byte target.",
            ROUTING,
            "TARGET_STREAM_SIZE_BYTES"
        ),
        limit(
            "stream.max_messages",
            ic_config::message_routing::MAX_STREAM_MESSAGES.to_string(),
            "messages",
            "hard_limit",
            "outgoing_xnet_stream",
            "Message count and byte target constrain the stream independently.",
            ROUTING,
            "MAX_STREAM_MESSAGES"
        ),
        limit(
            "stream.max_advert_body_bytes",
            ic_config::message_routing::ADVERT_MAX_BODY_BYTES.to_string(),
            "bytes",
            "hard_limit",
            "xnet_header_only_advert",
            "Receiver-enforced stream advert allowance; not an application message-payload limit.",
            ROUTING,
            "ADVERT_MAX_BODY_BYTES"
        ),
        limit(
            "wasm_chunk.slot_bytes",
            ic_replicated_state::canister_state::system_state::wasm_chunk_store::CHUNK_SIZE
                .to_string(),
            "bytes",
            "hard_limit",
            "stored_wasm_chunk",
            "Each distinct stored hash occupies a full slot even when its chunk is smaller. Total chunk-store capacity is separately configured.",
            CHUNKS,
            "CHUNK_SIZE"
        )
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn accounting_vocabulary_matches_runtime_and_keeps_movements_out_of_net() {
        use ic_interfaces::execution_environment::{RuntimeCostKind, RuntimeCostUnit};
        let notes = interpretation_notes();
        let units: BTreeSet<_> = notes["quantity_domains"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["unit"].as_str().unwrap())
            .collect();
        for unit in [
            RuntimeCostUnit::Cycles,
            RuntimeCostUnit::RoundInstructions,
            RuntimeCostUnit::ExecutionInstructions,
            RuntimeCostUnit::QueryGraphInstructions,
        ] {
            assert!(units.contains(unit.as_str()));
        }
        let kinds: BTreeSet<_> = notes["accounting"]["kinds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["kind"].as_str().unwrap())
            .collect();
        for kind in [
            RuntimeCostKind::Debit,
            RuntimeCostKind::DeferredDebit,
            RuntimeCostKind::Prepayment,
            RuntimeCostKind::Refund,
            RuntimeCostKind::Reservation,
            RuntimeCostKind::Release,
            RuntimeCostKind::Transfer,
        ] {
            assert!(kinds.contains(kind.as_str()));
        }
        assert_eq!(
            notes["accounting"]["exclude_from_booked_net"],
            json!(["deferred_debit", "reservation", "release", "transfer"])
        );
    }

    #[test]
    fn supplemental_limits_have_unique_ids_exact_values_and_archived_sources() {
        let limits = supplemental_limits();
        let mut ids = BTreeSet::new();
        for row in limits.as_array().unwrap() {
            assert!(ids.insert(row["id"].as_str().unwrap()));
            row["value"].as_str().unwrap().parse::<u128>().unwrap();
            let archived = &row["source"][0];
            let path = archived["file"].as_str().unwrap();
            let contents = [HTTP, ROUTING, CHUNKS]
                .into_iter()
                .find(|file| file.0 == path)
                .unwrap()
                .1;
            assert_eq!(
                archived["sha256"],
                hex::encode(ic_crypto_sha2::Sha256::hash(contents.as_bytes()))
            );
            let line = archived["line"].as_u64().unwrap() as usize;
            let code = archived["code"].as_str().unwrap();
            assert_eq!(
                code,
                contents
                    .lines()
                    .skip(line - 1)
                    .take(code.lines().count())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            assert!(code.contains(';'), "constant excerpt is incomplete: {path}");
        }
        let request = limits
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == "http.max_request_header_body_bytes")
            .unwrap();
        assert_eq!(
            request["value"],
            ic_types::canister_http::MAX_CANISTER_HTTP_REQUEST_BYTES.to_string()
        );
        assert_eq!(request["scope"], "http_request_headers_and_body");
    }

    #[test]
    fn notes_keep_external_examples_and_historical_snapshots_explicit() {
        let notes = interpretation_notes();
        assert_eq!(notes["timer_example"]["value_kind"], "derived_example");
        assert_eq!(
            notes["timer_example"]["parameters"]["method_name_bytes"],
            "<ic-cdk internal> timer_executor".len().to_string()
        );
        assert_eq!(
            notes["cadence"]["pocketic_implicit_round_time_increment_nanoseconds"],
            ic_state_machine_tests::StateMachine::EXECUTE_ROUND_TIME_INCREMENT
                .as_nanos()
                .to_string()
        );
        assert!(
            notes["value_authority"]["effective_at_setup"]
                .as_str()
                .unwrap()
                .contains("historical")
        );
        assert!(
            notes["external_service_boundary"]["documentation_reference_scope"]
                .as_str()
                .unwrap()
                .contains("not a live price assertion")
        );
    }
}
