use super::*;
use serde_json::json;

fn legacy_execution() -> serde_json::Value {
    json!({
        "sequence_number": 0,
        "api_type": "update",
        "function": "canister_update test",
        "call_context_id": 7,
        "time_nanos": 123,
        "instructions": 500,
        "page_access_instructions": 0,
        "outcome": "reply",
        "message": null,
        "wasm_memory_size": 1,
        "dirty_pages_tracked": true,
        "accessed_pages": [[0, 1]],
        "dirty_pages": [[0, 1]]
    })
}

#[test]
fn stable_coverage_defaults_to_unknown_and_roundtrips_exact_ranges() {
    let mut execution: TracedExecution = serde_json::from_value(legacy_execution()).unwrap();
    assert!(!execution.stable_pages_tracked);
    assert!(execution.stable_accessed_pages.is_empty());
    execution.stable_pages_tracked = true;
    execution.stable_memory_size = u64::MAX;
    execution.stable_accessed_pages = vec![(u64::MAX - 1, u64::MAX)];
    execution.stable_dirty_pages = execution.stable_accessed_pages.clone();
    let value = serde_json::to_value(&execution).unwrap();
    assert_eq!(value["stable_memory_size"], u64::MAX.to_string());
    assert_eq!(value["stable_accessed_pages"][0][1], u64::MAX.to_string());
    assert_eq!(
        value["stable_dirty_pages"][0][0],
        (u64::MAX - 1).to_string()
    );
    assert_eq!(
        serde_json::from_value::<TracedExecution>(value).unwrap(),
        execution
    );
    let schema = serde_json::to_value(schemars::schema_for!(TracedExecution)).unwrap();
    assert_eq!(schema["properties"]["stable_memory_size"]["type"], "string");
}

#[test]
fn legacy_trace_numbers_remain_readable() {
    let trace: MemoryAccessTrace = serde_json::from_value(json!({
        "wasm_page_size": 65_536,
        "executions": [legacy_execution()]
    }))
    .unwrap();
    assert_eq!(trace.format_version, 0);
    assert_eq!(trace.trace_generation, 0);
    assert_eq!(trace.executions[0].call_context_id, Some(7));
    assert_eq!(trace.executions[0].accessed_pages, vec![(0, 1)]);
    assert!(trace.executions[0].module_hash.is_none());
    assert!(trace.executions[0].instruction_profile.is_none());
    assert!(!trace.executions[0].detail_truncated);
    assert!(trace.executions[0].attempt.is_none());
    assert!(trace.executions[0].disposition.is_none());
    assert!(!trace.dropped_instructions_incomplete);
    assert!(trace.logical_executions.is_empty());
    assert!(trace.modules.is_empty());
    assert!(trace.charge_rules.is_empty());
    assert!(trace.runtime_costs.is_empty());
    assert!(trace.subnet_runtime_costs.is_empty());
    assert!(trace.runtime_cost_totals.is_empty());
    assert!(trace.subnet_runtime_cost_totals.is_empty());
    assert!(trace.runtime_configurations.is_empty());
    assert!(!trace.runtime_costs_truncated);
    assert!(!trace.subnet_runtime_costs_truncated);
    assert_eq!(trace.last_runtime_cost_sequence_number, None);
    let mut legacy_profile = serde_json::to_value(WasmInstructionProfile::default()).unwrap();
    legacy_profile.as_object_mut().unwrap().remove("rules");
    assert!(
        serde_json::from_value::<WasmInstructionProfile>(legacy_profile)
            .unwrap()
            .rules
            .is_empty()
    );
}

#[test]
fn aborted_attempt_lifecycle_preserves_exact_slice_counts_and_availability() {
    let mut value = legacy_execution();
    value["outcome"] = json!("aborted");
    value["attempt"] = json!({
        "attempt_id": u64::MAX.to_string(), "status": "aborted",
        "slice_count": u64::MAX.to_string(),
        "total_round_instructions": u128::MAX.to_string(),
        "slices": [{"slice_index": u64::MAX.to_string(),
            "event": "paused_instruction_budget", "instructions": u64::MAX.to_string(),
            "instruction_kind": "round_instructions"}],
        "slice_details_truncated": true, "memory_details_available": false,
        "instructions_available": false
    });
    value["disposition"] = json!({
        "logical_execution_id": u64::MAX.to_string(),
        "state": "discarded", "reason": "paused attempt aborted"
    });
    let execution: TracedExecution = serde_json::from_value(value).unwrap();
    assert_eq!(execution.outcome, TracedExecutionOutcome::Aborted);
    let attempt = execution.attempt.as_ref().unwrap();
    assert_eq!(attempt.attempt_id, u64::MAX);
    assert_eq!(attempt.slice_count, u64::MAX);
    assert!(!attempt.memory_details_available);
    assert!(!attempt.instructions_available);
    let serialized = serde_json::to_value(&execution).unwrap();
    assert_eq!(serialized["attempt"]["attempt_id"], u64::MAX.to_string());
    assert_eq!(
        serialized["attempt"]["total_round_instructions"],
        u128::MAX.to_string()
    );
    assert_eq!(
        serialized["attempt"]["slices"][0]["instructions"],
        u64::MAX.to_string()
    );
    assert_eq!(
        serialized["disposition"]["logical_execution_id"],
        u64::MAX.to_string()
    );
    assert_eq!(
        serde_json::from_value::<TracedExecution>(serialized).unwrap(),
        execution
    );
    let numeric: WasmExecutionSlice = serde_json::from_value(json!({
        "slice_index": 7, "event": "finished", "instructions": 900,
        "instruction_kind": "round_instructions"
    }))
    .unwrap();
    assert_eq!(numeric.slice_index, 7);
    let schema = serde_json::to_value(schemars::schema_for!(WasmExecutionAttempt)).unwrap();
    assert_eq!(schema["properties"]["attempt_id"]["type"], "string");
}

#[test]
fn trace_counters_roundtrip_above_javascript_precision() {
    let mut execution: TracedExecution = serde_json::from_value(legacy_execution()).unwrap();
    execution.sequence_number = u64::MAX;
    execution.call_context_id = Some(u64::MAX);
    execution.time_nanos = u64::MAX;
    execution.instructions = u64::MAX;
    execution.page_access_instructions = u64::MAX;
    execution.wasm_memory_size = u64::MAX;
    execution.accessed_pages = vec![(u64::MAX - 1, u64::MAX)];
    execution.dirty_pages = vec![(u64::MAX - 1, u64::MAX)];
    execution.instruction_profile = Some(WasmInstructionProfile {
        module_origin_map_truncated: true,
        module_origin_map_unavailable: true,
        blocks: vec![WasmInstructionProfileBlock {
            block_id: u64::MAX,
            instructions: u64::MAX,
            ..Default::default()
        }],
        dynamic_charges: vec![WasmInstructionDynamicCharge {
            body_offset: Some(u64::MAX),
            ..Default::default()
        }],
        rules: vec![WasmInstructionRuleCharge {
            rule_id: "heap_first_access".to_string(),
            events: u64::MAX,
            instructions: u64::MAX,
        }],
        page_charges: vec![WasmInstructionPageCharge {
            region_index: u64::MAX,
            instructions: u64::MAX,
            native_pc: Some(u64::MAX),
            module_offset: Some(u64::MAX),
            ..Default::default()
        }],
        stacks: vec![WasmInstructionStack {
            stack_id: 7,
            frames: vec![WasmInstructionStackFrame {
                module_offset: Some(u64::MAX),
                body_offset: Some(u64::MAX),
                ..Default::default()
            }],
            ..Default::default()
        }],
        stack_charges: vec![WasmInstructionStackCharge {
            stack_id: Some(7),
            events: u64::MAX,
            instructions: u64::MAX,
            ..Default::default()
        }],
        budget_events: vec![WasmInstructionBudgetEvent {
            before: i64::MIN,
            after: Some(i64::MAX),
            cause: "instruction counter refill".to_string(),
        }],
        ledger: WasmInstructionChargeLedger {
            adjustment: i128::MIN,
            unexplained_instructions: i128::MAX,
            ..Default::default()
        },
        ..Default::default()
    });
    let runtime_cost = RuntimeCostEvent {
        sequence_number: u64::MAX,
        amount: u128::MAX,
        nominal_cycles: Some(u128::MAX),
        time_nanos: Some(u64::MAX),
        round: Some(u64::MAX),
        call_context_id: Some(u64::MAX),
        callback_id: Some(u64::MAX),
        payload_bytes: Some(u64::MAX),
        instructions: Some(u64::MAX),
        details: BTreeMap::from([("bytes".to_string(), u64::MAX.to_string())]),
        ..Default::default()
    };
    let trace = MemoryAccessTrace {
        format_version: 2,
        trace_generation: u64::MAX,
        dropped_executions: u64::MAX,
        dropped_instructions: u128::MAX,
        dropped_instructions_incomplete: true,
        recorded_bytes: u64::MAX,
        runtime_costs: vec![runtime_cost.clone()],
        subnet_runtime_costs: vec![runtime_cost],
        runtime_cost_totals: vec![RuntimeCostTotal {
            events: u64::MAX,
            amount: u128::MAX,
            nominal_cycles: u128::MAX,
            ..Default::default()
        }],
        subnet_runtime_cost_totals: vec![RuntimeCostTotal {
            events: u64::MAX,
            amount: u128::MAX,
            nominal_cycles: u128::MAX,
            ..Default::default()
        }],
        runtime_configurations: BTreeMap::from([(
            "subnet".to_string(),
            BTreeMap::from([("limit".to_string(), u128::MAX.to_string())]),
        )]),
        runtime_costs_truncated: true,
        subnet_runtime_costs_truncated: true,
        runtime_costs_recording: false,
        runtime_cost_totals_incomplete: false,
        subnet_runtime_cost_totals_incomplete: false,
        last_runtime_cost_sequence_number: Some(u64::MAX),
        wasm_page_size: 65_536,
        charge_rules: BTreeMap::from([(
            "heap_first_access".to_string(),
            InstructionChargeRule {
                version: 1,
                id: "heap_first_access".to_string(),
                title: "Heap first access".to_string(),
                condition: "The execution first accesses an aligned region".to_string(),
                source: vec![InstructionChargeRuleSource {
                    file: "runtime.rs".to_string(),
                    symbol: "charge_first_access".to_string(),
                    line: 123,
                    code: "charge(region_fee);\n".to_string(),
                    sha256: "abc123".to_string(),
                }],
                ..Default::default()
            },
        )]),
        modules: BTreeMap::from([(
            "module".to_string(),
            WasmInstructionModuleMetadata {
                code_section_offset: u64::MAX,
                instrumented_code_section_offset: u64::MAX,
                instrumented_ranges: vec![WasmInstructionOriginRange {
                    start_offset: u64::MAX - 1,
                    end_offset: u64::MAX,
                    ..Default::default()
                }],
                functions: vec![WasmInstructionFunctionOrigin {
                    function_index: 1,
                    body_start: u64::MAX - 1,
                    body_end: u64::MAX,
                }],
                operators: vec![WasmInstructionOperator {
                    origin_id: 1,
                    function_index: 1,
                    body_offset: u64::MAX,
                    operator_ordinal: 0,
                    weight: u64::MAX,
                }],
                ..Default::default()
            },
        )]),
        executions: vec![execution],
        ..Default::default()
    };
    let value = serde_json::to_value(&trace).unwrap();
    let max = u64::MAX.to_string();
    assert_eq!(value["trace_generation"], max);
    assert_eq!(value["dropped_executions"], max);
    assert_eq!(value["dropped_instructions"], u128::MAX.to_string());
    assert_eq!(value["dropped_instructions_incomplete"], true);
    assert_eq!(value["recorded_bytes"], max);
    assert_eq!(value["wasm_page_size"], "65536");
    assert_eq!(value["last_runtime_cost_sequence_number"], max);
    for stream in ["runtime_costs", "subnet_runtime_costs"] {
        for field in [
            "sequence_number",
            "time_nanos",
            "round",
            "call_context_id",
            "callback_id",
            "payload_bytes",
            "instructions",
        ] {
            assert_eq!(value[stream][0][field], max, "{stream}.{field}");
        }
        assert_eq!(value[stream][0]["amount"], u128::MAX.to_string());
        assert_eq!(value[stream][0]["nominal_cycles"], u128::MAX.to_string());
        assert_eq!(value[stream][0]["details"]["bytes"], max);
    }
    assert_eq!(value["runtime_cost_totals"][0]["events"], max);
    assert_eq!(
        value["runtime_cost_totals"][0]["amount"],
        u128::MAX.to_string()
    );
    assert_eq!(
        value["runtime_cost_totals"][0]["nominal_cycles"],
        u128::MAX.to_string()
    );
    assert_eq!(
        value["subnet_runtime_cost_totals"][0]["amount"],
        u128::MAX.to_string()
    );
    assert_eq!(
        value["runtime_configurations"]["subnet"]["limit"],
        u128::MAX.to_string()
    );
    assert_eq!(value["modules"]["module"]["code_section_offset"], max);
    assert_eq!(
        value["modules"]["module"]["instrumented_code_section_offset"],
        max
    );
    assert_eq!(
        value["modules"]["module"]["instrumented_ranges"][0]["end_offset"],
        max
    );
    assert_eq!(value["modules"]["module"]["functions"][0]["body_end"], max);
    assert_eq!(value["modules"]["module"]["operators"][0]["weight"], max);
    let execution_json = &value["executions"][0];
    for field in [
        "sequence_number",
        "call_context_id",
        "time_nanos",
        "instructions",
        "page_access_instructions",
        "wasm_memory_size",
    ] {
        assert_eq!(execution_json[field], max, "{field}");
    }
    assert_eq!(execution_json["accessed_pages"][0][1], max);
    assert_eq!(execution_json["dirty_pages"][0][1], max);
    let profile = &execution_json["instruction_profile"];
    assert_eq!(profile["module_origin_map_truncated"], true);
    assert_eq!(profile["module_origin_map_unavailable"], true);
    assert_eq!(profile["stack_details_truncated"], false);
    assert_eq!(profile["blocks"][0]["block_id"], max);
    assert_eq!(profile["dynamic_charges"][0]["body_offset"], max);
    assert_eq!(profile["rules"][0]["events"], max);
    assert_eq!(profile["rules"][0]["instructions"], max);
    for field in ["region_index", "instructions", "native_pc", "module_offset"] {
        assert_eq!(profile["page_charges"][0][field], max, "{field}");
    }
    assert_eq!(profile["stacks"][0]["frames"][0]["module_offset"], max);
    assert_eq!(profile["stacks"][0]["frames"][0]["body_offset"], max);
    assert_eq!(profile["stack_charges"][0]["events"], max);
    assert_eq!(profile["stack_charges"][0]["instructions"], max);
    assert_eq!(
        value["charge_rules"]["heap_first_access"]["source"][0]["line"],
        123
    );
    assert_eq!(profile["budget_events"][0]["before"], i64::MIN.to_string());
    assert_eq!(profile["budget_events"][0]["after"], i64::MAX.to_string());
    assert_eq!(profile["ledger"]["adjustment"], i128::MIN.to_string());
    assert_eq!(
        profile["ledger"]["unexplained_instructions"],
        i128::MAX.to_string()
    );
    assert_eq!(
        serde_json::from_value::<MemoryAccessTrace>(value).unwrap(),
        trace
    );
}

#[test]
fn invalid_counters_are_rejected() {
    for invalid in [json!(-1), json!(1.5), json!("18446744073709551616")] {
        let mut execution = legacy_execution();
        execution["instructions"] = invalid;
        assert!(serde_json::from_value::<TracedExecution>(execution).is_err());
    }
}

#[test]
fn profile_window_defaults_and_clear_guards_read_both_formats() {
    let request: RawStartInstructionProfiling =
        serde_json::from_value(json!({"canister_id": ""})).unwrap();
    assert_eq!(
        request.max_executions,
        DEFAULT_INSTRUCTION_PROFILING_EXECUTIONS
    );
    for generation in [json!(7), json!("7")] {
        let request: RawClearMemoryAccessTrace = serde_json::from_value(json!({
            "canister_id": "",
            "expected_server_generation": "server",
            "expected_trace_generation": generation,
            "expected_last_sequence_number": null
        }))
        .unwrap();
        assert_eq!(request.expected_trace_generation, Some(7));
        assert_eq!(request.expected_last_sequence_number, None);
        assert_eq!(request.expected_last_runtime_cost_sequence_number, None);
        assert_eq!(
            request.expected_server_generation.as_deref(),
            Some("server")
        );
    }
}

#[test]
fn runtime_snapshot_clear_guard_preserves_absent_null_and_values() {
    let absent: RawClearMemoryAccessTrace =
        serde_json::from_value(json!({"canister_id": ""})).unwrap();
    assert_eq!(absent.expected_last_runtime_cost_sequence_number, None);
    assert!(
        serde_json::to_value(&absent)
            .unwrap()
            .get("expected_last_runtime_cost_sequence_number")
            .is_none()
    );
    for (value, expected) in [
        (json!(null), None),
        (json!(7), Some(7)),
        (json!(u64::MAX.to_string()), Some(u64::MAX)),
    ] {
        let request: RawClearMemoryAccessTrace = serde_json::from_value(json!({
            "canister_id": "",
            "expected_last_runtime_cost_sequence_number": value,
        }))
        .unwrap();
        assert_eq!(
            request.expected_last_runtime_cost_sequence_number,
            Some(expected)
        );
        let serialized = serde_json::to_value(&request).unwrap();
        assert_eq!(
            serialized["expected_last_runtime_cost_sequence_number"],
            expected.map_or(serde_json::Value::Null, |value| json!(value.to_string()))
        );
    }
}

#[test]
fn trace_schema_exposes_strings_for_counters() {
    let schema = serde_json::to_value(schemars::schema_for!(MemoryAccessTrace)).unwrap();
    assert_eq!(schema["properties"]["trace_generation"]["type"], "string");
    assert_eq!(
        schema["definitions"]["TracedExecution"]["properties"]["instructions"]["type"],
        "string"
    );
    assert_eq!(
        schema["definitions"]["WasmInstructionChargeLedger"]["properties"]["adjustment"]["type"],
        "string"
    );
    assert_eq!(
        schema["definitions"]["WasmInstructionRuleCharge"]["properties"]["events"]["type"],
        "string"
    );
    assert_eq!(
        schema["definitions"]["RuntimeCostEvent"]["properties"]["amount"]["type"],
        "string"
    );
    assert_eq!(
        schema["definitions"]["RuntimeCostTotal"]["properties"]["events"]["type"],
        "string"
    );
}

#[test]
fn read_profile_preserves_exact_counters_and_legacy_request_default() {
    let profile = WasmReadProfile {
        ranges: vec![WasmReadRange {
            memory: "stable".to_string(),
            start: 9_007_199_254_740_993,
            end: 9_007_199_254_741_001,
            stable_direct_reads: 1,
            stable_native_reads: u64::MAX,
            ..Default::default()
        }],
        observed_reads: u64::MAX,
        observed_bytes: u64::MAX,
        ..Default::default()
    };
    let json = serde_json::to_value(&profile).unwrap();
    assert_eq!(json["ranges"][0]["start"], "9007199254740993");
    assert_eq!(
        json["ranges"][0]["stable_native_reads"],
        u64::MAX.to_string()
    );
    assert_eq!(
        serde_json::from_value::<WasmReadProfile>(json).unwrap(),
        profile
    );
    let legacy: RawStartInstructionProfiling =
        serde_json::from_value(serde_json::json!({"canister_id": "", "max_executions": 1}))
            .unwrap();
    assert!(!legacy.read_mask);
    assert!(!legacy.write_mask);
}

#[test]
fn write_profile_preserves_exact_counters_and_addresses() {
    let profile = WasmWriteProfile {
        ranges: vec![WasmWriteRange {
            memory: "heap".to_string(),
            start: 9_007_199_254_740_993,
            end: 9_007_199_254_741_001,
            heap_writes: u64::MAX,
            stable_writes: 0,
        }],
        observed_writes: u64::MAX,
        observed_bytes: u64::MAX,
        ..Default::default()
    };
    let json = serde_json::to_value(&profile).unwrap();
    assert_eq!(json["ranges"][0]["start"], "9007199254740993");
    assert_eq!(json["ranges"][0]["heap_writes"], u64::MAX.to_string());
    assert_eq!(
        serde_json::from_value::<WasmWriteProfile>(json).unwrap(),
        profile
    );
}

#[tokio::test]
async fn stale_clear_conflicts_preserve_the_error_message() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn conflict(body: &'static str) -> ApiResponse<()> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut connection, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            connection.read(&mut request).await.unwrap();
            let response = format!(
                "HTTP/1.1 409 Conflict\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            connection.write_all(response.as_bytes()).await.unwrap();
        });
        let response = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get(format!("http://{address}/"))
            .send()
            .await
            .unwrap();
        let result = ApiResponse::from_response(response).await;
        server.await.unwrap();
        result
    }

    match conflict(r#"{"message":"MemoryAccessTraceChanged"}"#).await {
        ApiResponse::Error { message } => assert_eq!(message, "MemoryAccessTraceChanged"),
        other => panic!("expected stale-clear error, got {other:?}"),
    }
    match conflict(r#"{"state_label":"state","op_id":"operation"}"#).await {
        ApiResponse::Busy { state_label, op_id } => {
            assert_eq!(state_label, "state");
            assert_eq!(op_id, "operation");
        }
        other => panic!("expected retryable busy response, got {other:?}"),
    }
}
