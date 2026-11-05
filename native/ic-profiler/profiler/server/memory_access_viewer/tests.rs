use super::*;

#[test]
fn numeric_surfaces_preserve_their_distinct_lexical_contracts() {
    let query: ExecutionSelection = serde_json::from_value(json!({"after": "+1"})).unwrap();
    assert_eq!(query.after, Some(1));
    assert!(tool_selection(&json!({"after": "+1"})).is_err());
    assert!(
        crate::memory_snapshot_api::SnapshotImageQuery::from_arguments(
            &json!({"start_page": "+1"})
        )
        .is_err()
    );
    assert_eq!(
        integer_argument(&json!({"n": u64::MAX.to_string()}), "n").unwrap(),
        Some(u64::MAX)
    );
    for invalid in [
        json!("18446744073709551616"),
        json!("１２"),
        json!(-1),
        json!(1.5),
    ] {
        assert!(integer_argument(&json!({"n": invalid}), "n").is_err());
    }
}

#[test]
fn profile_requests_preserve_defaults_limits_masks_and_refusals() {
    let defaults = ProfileRequest {
        max_executions: 20,
        read_mask: false,
        write_mask: false,
    };
    assert_eq!(ProfileRequest::parse(&json!({})).unwrap(), defaults);
    assert_eq!(ProfileRequest::parse(&json!({"max_executions": "18446744073709551615", "read_mask": true, "write_mask": true})).unwrap(),
        ProfileRequest { max_executions: pocket_ic::common::rest::MAX_INSTRUCTION_PROFILING_EXECUTIONS, read_mask: true, write_mask: true });
    for input in [
        json!({"max_executions": 0}),
        json!({"max_executions": -1}),
        json!({"max_executions": "1.5"}),
        json!({"read_mask": "true"}),
        json!({"write_mask": null}),
    ] {
        assert!(ProfileRequest::parse(&input).is_err(), "{input}");
    }
}
use pocket_ic::common::rest::TracedExecutionOutcome;

fn execution(
    sequence_number: u64,
    api_type: &str,
    function: &str,
    call_context_id: Option<u64>,
    accessed_pages: Vec<(u64, u64)>,
    dirty_pages: Vec<(u64, u64)>,
) -> TracedExecution {
    let page_accesses = pages(&accessed_pages) + pages(&dirty_pages);
    TracedExecution {
        sequence_number,
        attempt: None,
        disposition: None,
        module_hash: Some("0123456789abcdef".to_string()),
        module_hash_kind: Some("installed_uncompressed_wasm".to_string()),
        instruction_profile: None,
        read_profile: None,
        write_profile: None,
        detail_truncated: false,
        api_type: api_type.to_string(),
        function: function.to_string(),
        call_context_id,
        time_nanos: 1_000 + sequence_number,
        instructions: 1_000 + page_accesses * 80_000,
        page_access_instructions: page_accesses * 80_000,
        outcome: TracedExecutionOutcome::Reply,
        message: None,
        wasm_memory_size: 256,
        stable_memory_size: 0,
        stable_pages_tracked: false,
        stable_accessed_pages: vec![],
        stable_dirty_pages: vec![],
        dirty_pages_tracked: true,
        accessed_pages,
        dirty_pages,
    }
}

#[test]
fn union_merges_overlapping_and_adjacent_ranges() {
    let merged = union([(5, 6), (0, 2), (2, 3), (1, 2), (10, 12)].into_iter());
    assert_eq!(merged, vec![(0, 3), (5, 6), (10, 12)]);
    assert!(contains(&merged, 0));
    assert!(contains(&merged, 2));
    assert!(!contains(&merged, 3));
    assert!(contains(&merged, 11));
    assert!(!contains(&merged, 12));
}

#[test]
fn only_localhost_and_ip_addresses_are_local_hosts() {
    use crate::system_profile_api::is_local_host;
    for host in [
        "localhost",
        "localhost:8080",
        "LOCALHOST:1",
        "app.localhost:1",
        "127.0.0.1:80",
        "10.0.0.1",
        "[::1]:8080",
        "[::1]",
    ] {
        assert!(is_local_host(host), "{host}");
    }
    for host in [
        "example.com",
        "example.com:8080",
        "localhost.example.com",
        "localhost@evil.example",
        "localhost:99999",
        "[::1]evil.example",
        "[::1]:99999",
        "",
    ] {
        assert!(!is_local_host(host), "{host}");
    }
}

#[test]
fn callbacks_are_named_after_their_call() {
    let executions = vec![
        execution(0, "update", "canister_update f", Some(1), vec![], vec![]),
        execution(
            1,
            "reply callback",
            "callback (table index 0)",
            Some(1),
            vec![],
            vec![],
        ),
        execution(
            2,
            "reject callback",
            "callback (table index 3)",
            Some(9),
            vec![],
            vec![],
        ),
    ];
    assert_eq!(
        function_labels(&executions),
        vec![
            "canister_update f",
            "canister_update f ↳ reply callback",
            "callback (table index 3) (reject callback)",
        ]
    );
}

#[test]
fn callback_labels_follow_the_latest_initiating_context() {
    let executions = vec![
        execution(0, "update", "old", Some(1), vec![], vec![]),
        execution(1, "reply callback", "callback", Some(1), vec![], vec![]),
        execution(2, "update", "restored", Some(1), vec![], vec![]),
        execution(3, "reply callback", "callback", Some(1), vec![], vec![]),
    ];
    assert_eq!(
        function_labels(&executions),
        vec![
            "old",
            "old ↳ reply callback",
            "restored",
            "restored ↳ reply callback"
        ]
    );
}

#[test]
fn composite_and_replicated_contexts_have_separate_labels() {
    let executions = vec![
        execution(0, "update", "update", Some(1), vec![], vec![]),
        execution(1, "composite query", "query", Some(1), vec![], vec![]),
        execution(2, "reply callback", "callback", Some(1), vec![], vec![]),
        execution(
            3,
            "composite reply callback",
            "callback",
            Some(1),
            vec![],
            vec![],
        ),
    ];
    assert_eq!(
        function_labels(&executions),
        vec![
            "update",
            "query",
            "update ↳ reply callback",
            "query ↳ composite reply callback"
        ]
    );
}

#[test]
fn page_runs_count_executions_per_page() {
    let executions = vec![
        execution(0, "update", "a", None, vec![(0, 4)], vec![(1, 2)]),
        execution(1, "update", "a", None, vec![(2, 6)], vec![(2, 3)]),
        execution(2, "update", "a", None, vec![(6, 7), (9, 10)], vec![]),
    ];
    let run = |start, end, accessed_by, written_by| PageRun {
        start,
        end,
        accessed_by,
        written_by,
    };
    assert_eq!(
        page_runs(&executions),
        vec![
            run(0, 1, 1, 0),
            run(1, 2, 1, 1),
            run(2, 3, 2, 1),
            run(3, 4, 2, 0),
            run(4, 7, 1, 0),
            run(9, 10, 1, 0),
        ]
    );
}

#[test]
fn summary_lists_functions_pages_and_regions() {
    let executions = vec![
        execution(
            0,
            "update",
            "canister_update put",
            Some(1),
            vec![(0, 1), (10, 12)],
            vec![(10, 12)],
        ),
        execution(
            1,
            "update",
            "canister_update put",
            Some(2),
            vec![(0, 1), (12, 13)],
            vec![(12, 13)],
        ),
        execution(
            2,
            "update",
            "canister_update scan",
            Some(3),
            vec![(0, 1), (100, 200)],
            vec![],
        ),
    ];
    let summary = summarize(&executions, 10);
    assert_eq!(summary["executions"], "3");
    assert_eq!(summary["outcomes"]["reply"], "3");
    assert_eq!(summary["pages"]["distinct_accessed"], "104");
    assert_eq!(summary["pages"]["distinct_written"], "3");
    assert_eq!(summary["pages"]["accessed_max"], "101");
    assert_eq!(summary["instructions"]["per_page_access"], 80_000.0);
    assert!(
        summary["instructions"]["page_access_share"]
            .as_f64()
            .unwrap()
            > 0.99
    );

    // The scan takes the most instructions.
    assert_eq!(summary["function_count"], "2");
    let scan = &summary["functions"][0];
    assert_eq!(scan["function"], "canister_update scan");
    assert_eq!(scan["pages"]["accessed_mean"], 101.0);
    let put = &summary["functions"][1];
    assert_eq!(put["function"], "canister_update put");
    assert_eq!(put["executions"], "2");
    assert_eq!(put["pages"]["accessed_mean"], 2.5);
    assert_eq!(put["pages"]["distinct_accessed"], "4");
    assert_eq!(put["pages"]["distinct_written"], "3");
    assert_eq!(
        summary["most_expensive_executions"][0]["sequence_number"],
        "2"
    );

    // Page 0 is accessed by all executions.
    assert_eq!(summary["hot_pages"][0]["first_page"], "0");
    assert_eq!(summary["hot_pages"][0]["pages"], "1");
    assert_eq!(summary["hot_pages"][0]["accessed_by"], "3");

    // The regions are [0, 1), [10, 13), and [100, 200).
    assert_eq!(summary["region_count"], "3");
    let regions = &summary["regions"];
    assert_eq!(regions[0]["first_page"], "0");
    assert_eq!(regions[0]["accessed_by"], "3");
    assert_eq!(regions[1]["first_page"], "10");
    assert_eq!(regions[1]["last_page"], "12");
    assert_eq!(regions[1]["accessed_by"], "2");
    assert_eq!(regions[1]["written_pages"], "3");
    assert_eq!(regions[1]["offset"], "0xa0000");
    assert_eq!(regions[2]["first_page"], "100");
    assert_eq!(regions[2]["page_accesses"], "100");
}

#[test]
fn summary_of_no_executions_is_empty() {
    let summary = summarize(&[], 10);
    assert_eq!(summary["executions"], "0");
    assert_eq!(summary["pages"]["distinct_accessed"], "0");
    assert_eq!(summary["functions"], json!([]));
    assert_eq!(summary["regions"], json!([]));
    assert_eq!(summary["instructions"]["page_access_share"], 0.0);
}

#[test]
fn unknown_abort_counters_and_pages_are_excluded_from_known_summary_and_attempt_filter_is_exact() {
    let known = execution(
        1,
        "update",
        "canister_update known",
        None,
        vec![(0, 1)],
        vec![],
    );
    let mut unavailable = execution(
        2,
        "update",
        "canister_update unavailable",
        None,
        vec![(10, 20)],
        vec![(10, 20)],
    );
    unavailable.instructions = u64::MAX;
    unavailable.attempt = Some(pocket_ic::common::rest::WasmExecutionAttempt {
        attempt_id: 9_007_199_254_740_993,
        status: "aborted".to_string(),
        memory_details_available: false,
        instructions_available: false,
        ..Default::default()
    });
    unavailable.outcome = TracedExecutionOutcome::Aborted;
    let summary = summarize(&[known.clone(), unavailable.clone()], 10);
    assert_eq!(
        summary["instructions"]["total"],
        known.instructions.to_string()
    );
    assert_eq!(summary["instructions"]["detail_complete"], false);
    assert_eq!(summary["instructions"]["available_executions"], "1");
    assert_eq!(summary["pages"]["distinct_accessed"], "1");
    assert_eq!(summary["pages"]["detail_complete"], false);
    assert_eq!(summary["wasm_memory_size"], Value::Null);
    assert_eq!(
        summary["most_expensive_executions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let selected =
        tool_selection(&json!({"attempt_id": "9007199254740993", "outcome": "aborted"})).unwrap();
    assert!(selected.filter().matches(&unavailable));
    assert!(!selected.filter().matches(&known));
    assert!(tool_selection(&json!({"attempt_id": "-1"})).is_err());
    let parsed =
        Query::<ExecutionsQuery>::try_from_uri(&"/?attempt_id=9007199254740993".parse().unwrap())
            .unwrap()
            .0;
    assert_eq!(parsed.selection.attempt_id, Some(9_007_199_254_740_993));
}

#[test]
fn shared_image_envelope_preserves_bytes_and_exact_budget() {
    let metadata = json!({"label":"é"});
    let response = image_tool_response(&metadata, &[255], "too large").unwrap();
    assert_eq!(
        response,
        json!({"content":[{"type":"text","text":metadata.to_string()},{"type":"image","mimeType":"image/png","data":"/w=="}],"isError":false})
    );
    let at_limit = Value::String("x".repeat(MAX_TOOL_RESPONSE_BYTES - 6));
    assert!(image_tool_response(&at_limit, &[255], "too large").is_ok());
    let over_limit = Value::String("x".repeat(MAX_TOOL_RESPONSE_BYTES - 5));
    assert_eq!(
        image_tool_response(&over_limit, &[255], "too large").unwrap_err(),
        "too large"
    );
}

#[test]
fn immutable_mcp_catalog_preserves_its_content_hash_and_numeric_contract() {
    let catalog = crate::computation_catalog::catalog();
    let mut encoded = encode_tool_response(ToolOutput::Native(catalog.clone()));
    assert_eq!(encoded, catalog);
    assert!(encoded.get("server_generation").is_none());
    assert_eq!(encoded["catalog_version"], 1);
    let hash = encoded
        .as_object_mut()
        .unwrap()
        .remove("catalog_sha256")
        .unwrap();
    assert_eq!(
        hash,
        hex::encode(ic_crypto_sha2::Sha256::hash(
            &serde_json::to_vec(&encoded).unwrap()
        ))
    );

    // Ordinary trace tools retain their independent process/decimal encoding.
    let trace = encode_tool_response(ToolOutput::Trace(summarize(&[], 10)));
    assert_eq!(trace["executions"], "0");
    assert!(trace["server_generation"].is_string());
}

#[tokio::test]
async fn http_and_mcp_preserve_producer_owned_counter_encoding() {
    let mut observed = execution(0, "update", "f", Some(u64::MAX), vec![(0, 3)], vec![]);
    observed.sequence_number = 9_007_199_254_740_993;
    observed.time_nanos = u64::MAX;
    observed.instructions = u64::MAX;
    let summary = summarize(std::slice::from_ref(&observed), 10);
    assert_eq!(summary["first_sequence_number"], "9007199254740993");
    assert_eq!(summary["first_time_nanos"], u64::MAX.to_string());
    assert_eq!(summary["instructions"]["total"], u64::MAX.to_string());
    assert_eq!(
        summary["functions"][0]["instructions"]["max"],
        u64::MAX.to_string()
    );
    assert_eq!(summary["pages"]["distinct_accessed"], "3");
    assert_eq!(summary["page_size"], PAGE_SIZE);
    assert!(summary["instructions"]["mean"].is_f64());
    let bytes = axum::body::to_bytes(
        respond(Ok(summary.clone())).into_body(),
        MAX_TOOL_RESPONSE_BYTES,
    )
    .await
    .unwrap();
    let http: Value = serde_json::from_slice(&bytes).unwrap();
    let mcp = encode_tool_response(ToolOutput::Trace(summary.clone()));
    assert_eq!(http, mcp);
    for (key, value) in summary.as_object().unwrap() {
        assert_eq!(&http[key], value);
    }
    assert!(http["server_generation"].is_string());
    assert!(http.get("format_version").is_none());
    assert!(http.get("integer_encoding").is_none());

    // The canonical DTO serializer owns both wide counters and bounded
    // identifiers. HTTP/MCP cannot silently rewrite that typed contract.
    let mut document = json!({"executions": [observed], "module": pocket_ic::common::rest::WasmInstructionOriginRange {
        instrumented_function_index: 17,
        start_offset: u64::MAX,
        ..Default::default()
    }});
    let body = axum::body::to_bytes(
        respond(Ok(document.clone())).into_body(),
        MAX_TOOL_RESPONSE_BYTES,
    )
    .await
    .unwrap();
    let http: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(http["executions"], document["executions"]);
    assert_eq!(http["module"], document["module"]);
    assert_eq!(
        http["executions"][0]["call_context_id"],
        u64::MAX.to_string()
    );
    assert_eq!(http["executions"][0]["accessed_pages"], json!([["0", "3"]]));
    assert_eq!(http["module"]["instrumented_function_index"], 17);
    assert_eq!(http["module"]["start_offset"], u64::MAX.to_string());
    document["charge_rules"] = charge_rule_dictionary()["charge_rules"].clone();
    let mcp = encode_tool_response(ToolOutput::Trace(document.clone()));
    assert_eq!(mcp["charge_rules"], document["charge_rules"]);
}

#[test]
fn function_charge_allocation_uses_exact_origins_once_and_preserves_missing_remainder() {
    use pocket_ic::common::rest::{
        WasmInstructionChargeLedger, WasmInstructionModuleMetadata, WasmInstructionOperator,
        WasmInstructionPageCharge, WasmInstructionProfile, WasmInstructionStackCharge,
    };
    let hash = "ab".repeat(32);
    let modules = BTreeMap::from([(
        hash.clone(),
        WasmInstructionModuleMetadata {
            module_sha256: [0xab; 32],
            operators: vec![
                WasmInstructionOperator {
                    origin_id: 1,
                    function_index: 4,
                    body_offset: 0,
                    operator_ordinal: 0,
                    weight: 100,
                },
                WasmInstructionOperator {
                    origin_id: 2,
                    function_index: 5,
                    body_offset: 1,
                    operator_ordinal: 1,
                    weight: 1,
                },
            ],
            ..Default::default()
        },
    )]);
    let mut observed = execution(0, "update", "f", None, vec![], vec![]);
    observed.module_hash = Some(hash);
    observed.module_hash_kind = Some("installed_uncompressed_sha256".into());
    observed.instruction_profile = Some(WasmInstructionProfile {
        stack_charges: vec![
            WasmInstructionStackCharge {
                origin_id: Some(1),
                category: "static_prepaid".into(),
                instructions: 100,
                origin_evidence: "block".into(),
                ..Default::default()
            },
            WasmInstructionStackCharge {
                origin_id: Some(2),
                category: "api".into(),
                instructions: 900,
                origin_evidence: "exact_operation".into(),
                ..Default::default()
            },
            WasmInstructionStackCharge {
                origin_id: None,
                category: "heap_access".into(),
                instructions: 160_000,
                ..Default::default()
            },
        ],
        page_charges: vec![WasmInstructionPageCharge {
            origin_id: Some(1),
            memory: "heap".into(),
            kind: "access".into(),
            instructions: 160_000,
            origin_evidence: "native_fault_pc".into(),
            ..Default::default()
        }],
        ledger: WasmInstructionChargeLedger {
            raw_instructions: 161_000,
            ..Default::default()
        },
        ..Default::default()
    });
    let allocation = allocate_profile_functions(&[observed.clone()], &modules, 10);
    assert_eq!(allocation["allocated_instructions"], "161000");
    assert_eq!(allocation["raw_minus_allocated_instructions"], "0");
    assert_eq!(allocation["functions"][0]["function_index"], 4);
    assert_eq!(allocation["functions"][0]["instructions"], "160100");
    assert_eq!(allocation["detail_complete"], true);
    observed
        .instruction_profile
        .as_mut()
        .unwrap()
        .module_origin_map_truncated = true;
    let module_partial = allocate_profile_functions(&[observed.clone()], &modules, 10);
    assert_eq!(module_partial["allocated_instructions"], "161000");
    assert_eq!(module_partial["detail_complete"], false);
    assert_eq!(module_partial["module_origin_map_truncated"], true);
    assert_eq!(module_partial["module_origin_map_unavailable"], false);
    assert_eq!(module_partial["stack_details_truncated"], false);
    let aggregate = aggregate_profiles(&[observed.clone()], 10);
    assert_eq!(aggregate["module_origin_map_truncated"], true);
    assert_eq!(aggregate["stack_details_truncated"], false);
    assert_eq!(aggregate["truncated"], false);
    let profile = observed.instruction_profile.as_mut().unwrap();
    profile.module_origin_map_truncated = false;
    profile.page_charges.clear();
    profile.page_details_truncated = true;
    let partial = allocate_profile_functions(&[observed.clone()], &modules, 10);
    assert_eq!(partial["raw_minus_allocated_instructions"], "160000");
    assert_eq!(partial["detail_complete"], false);
    observed.module_hash = Some("cd".repeat(32));
    let mismatched = allocate_profile_functions(&[observed], &modules, 10);
    assert_eq!(mismatched["allocated_instructions"], "0");
    assert_eq!(mismatched["module_origin_map_unavailable"], true);
    assert_eq!(mismatched["stack_details_truncated"], false);
}

#[test]
fn aggregate_instructions_do_not_overflow_u64() {
    let mut first = execution(0, "update", "f", None, vec![], vec![]);
    first.instructions = u64::MAX;
    first.page_access_instructions = u64::MAX / 2;
    let mut second = first.clone();
    second.sequence_number = 1;
    let summary = summarize(&[first, second], 10);
    assert_eq!(
        summary["instructions"]["total"],
        (2 * u64::MAX as u128).to_string()
    );
    assert_eq!(
        summary["functions"][0]["instructions"]["total"],
        (2 * u64::MAX as u128).to_string()
    );
    assert_eq!(
        summary["instructions"]["page_access_total"],
        (2 * (u64::MAX / 2) as u128).to_string()
    );
}

#[test]
fn mcp_integer_arguments_accept_exact_decimals_and_reject_invalid_values() {
    assert_eq!(
        integer_argument(&json!({ "after": u64::MAX.to_string() }), "after").unwrap(),
        Some(u64::MAX)
    );
    assert_eq!(
        integer_argument(&json!({ "after": 0 }), "after").unwrap(),
        Some(0)
    );
    assert_eq!(integer_argument(&json!({}), "after").unwrap(), None);
    for invalid in [
        json!(-1),
        json!(1.5),
        json!("18446744073709551616"),
        json!("+1"),
        json!(null),
    ] {
        assert!(integer_argument(&json!({ "after": invalid }), "after").is_err());
    }
}

#[test]
fn mcp_rejects_malformed_json_rpc_envelopes() {
    for invalid in [
        json!({ "method": "ping", "id": 1 }),
        json!({ "jsonrpc": "1.0", "method": "ping", "id": 1 }),
        json!({ "jsonrpc": "2.0", "method": "ping", "id": {} }),
        json!({ "jsonrpc": "2.0", "method": 3, "id": 1 }),
        json!({ "jsonrpc": "2.0", "method": "ping", "params": false }),
        json!([]),
    ] {
        assert!(validate_rpc_message(&invalid).is_err(), "{invalid}");
    }
    assert!(validate_rpc_message(&json!({ "jsonrpc": "2.0", "method": "ping", "id": 1 })).is_ok());
    assert!(
        validate_rpc_message(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }))
            .is_ok()
    );
}

#[test]
fn profiles_keep_module_versions_and_ledger_categories_separate() {
    use pocket_ic::common::rest::{
        WasmInstructionChargeLedger, WasmInstructionProfile, WasmInstructionProfileBlock,
        WasmInstructionRuleCharge,
    };
    let mut first = execution(0, "update", "f", None, vec![], vec![]);
    first.instructions = 1_000;
    first.instruction_profile = Some(WasmInstructionProfile {
        rules: vec![
            WasmInstructionRuleCharge {
                rule_id: "static".to_string(),
                events: 2,
                instructions: 100,
            },
            WasmInstructionRuleCharge {
                rule_id: "api".to_string(),
                events: 1,
                instructions: 900,
            },
            WasmInstructionRuleCharge {
                rule_id: "decision".to_string(),
                events: 1,
                instructions: 0,
            },
        ],
        blocks: vec![WasmInstructionProfileBlock {
            block_id: 1,
            function_index: 4,
            instructions: 100,
            hits: 2,
            ..Default::default()
        }],
        ledger: WasmInstructionChargeLedger {
            static_instructions: 100,
            api_instructions: 900,
            raw_instructions: 1_000,
            reported_instructions: 1_000,
            reconciled: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let mut upgraded = first.clone();
    upgraded.sequence_number = 1;
    upgraded.module_hash = Some("different-installed-module".to_string());
    let profile = aggregate_profiles(&[first, upgraded], 10);
    assert_eq!(profile["profiled_executions"], "2");
    assert_eq!(profile["function_count"], "2");
    assert_eq!(profile["block_count"], "2");
    assert_eq!(profile["allocated_static_instructions"], "200");
    assert_eq!(profile["ledger"]["static_instructions"], "200");
    assert_eq!(profile["ledger"]["api_instructions"], "1800");
    assert_eq!(profile["ledger"]["reported_instructions"], "2000");
    assert_eq!(profile["ledger"]["reconciled"], true);
    assert_eq!(profile["observed_rule_instructions"], "2000");
    assert_eq!(profile["raw_minus_rule_instructions"], "0");
    assert_eq!(profile["rules_cover_raw_instructions"], true);
    let zero = profile["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|rule| rule["rule_id"] == "decision")
        .unwrap();
    assert_eq!(zero["instructions"], "0");
    assert_eq!(zero["events"], "2");
    assert_eq!(profile["source_coverage"], Value::Null);
}

#[test]
fn missing_page_detail_does_not_become_a_zero_page_execution() {
    let first = execution(0, "update", "f", None, vec![(0, 2)], vec![(1, 2)]);
    let mut truncated = execution(1, "update", "f", None, vec![(10, 15)], vec![(10, 15)]);
    truncated.instructions = first.instructions;
    truncated.page_access_instructions = first.page_access_instructions;
    truncated.detail_truncated = true;
    let summary = summarize(&[first, truncated], 10);
    assert_eq!(summary["pages"]["detail_complete"], false);
    assert_eq!(summary["pages"]["accessed_executions"], "1");
    assert_eq!(summary["pages"]["accessed_mean"], 2.0);
    assert_eq!(summary["pages"]["written_mean"], 1.0);
    assert_eq!(summary["instructions"]["per_page_access"], 80_000.0);
}

#[test]
fn conditional_clear_preserves_runtime_cursor_presence_and_exactness() {
    let request = json!({"trace_generation":"3", "last_sequence_number":null,
        "expected_last_runtime_cost_sequence_number":"18446744073709551615"});
    assert_eq!(
        clear_condition(&request).unwrap(),
        Some(ClearCondition {
            generation: 3,
            last_sequence: None,
            runtime_sequence: Some(Some(u64::MAX))
        })
    );
    assert_eq!(
        clear_condition(&json!({"trace_generation":"3", "last_sequence_number":null,
        "expected_last_runtime_cost_sequence_number":null}))
        .unwrap(),
        Some(ClearCondition {
            generation: 3,
            last_sequence: None,
            runtime_sequence: Some(None)
        })
    );
    assert!(clear_condition(&json!({"expected_last_runtime_cost_sequence_number":null})).is_err());
}

#[test]
fn conditional_clear_requires_both_snapshot_fields() {
    assert_eq!(clear_condition(&json!({})).unwrap(), None);
    assert_eq!(
        clear_condition(
            &json!({ "trace_generation": "3", "last_sequence_number": "9007199254740993" })
        )
        .unwrap(),
        Some(ClearCondition {
            generation: 3,
            last_sequence: Some(9_007_199_254_740_993),
            runtime_sequence: None
        })
    );
    assert_eq!(
        clear_condition(&json!({ "trace_generation": "3", "last_sequence_number": null })).unwrap(),
        Some(ClearCondition {
            generation: 3,
            last_sequence: None,
            runtime_sequence: None
        })
    );
    assert!(clear_condition(&json!({ "trace_generation": 3 })).is_err());
    assert!(clear_condition(&json!({ "last_sequence_number": 9 })).is_err());
    assert!(
        clear_server_generation(
            &json!({ "expected_server_generation": server_identity().generation })
        )
        .is_ok()
    );
    assert_eq!(
        clear_server_generation(&json!({ "expected_server_generation": "old-server" }))
            .unwrap_err()
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        clear_server_generation(&json!({ "expected_server_generation": 7 }))
            .unwrap_err()
            .0,
        StatusCode::BAD_REQUEST
    );
}

#[test]
fn execution_selection_accepts_url_query_numbers() {
    let uri = "/?after=0&before=9007199254740993&outcome=reply&limit=2"
        .parse()
        .unwrap();
    let query = Query::<ExecutionsQuery>::try_from_uri(&uri).unwrap().0;
    assert_eq!(query.selection.after, Some(0));
    assert_eq!(query.selection.before, Some(9_007_199_254_740_993));
    assert_eq!(query.selection.outcome, Some(TracedExecutionOutcome::Reply));
    assert_eq!(query.limit, Some(2));
    let summary = Query::<SummaryQuery>::try_from_uri(&"/?after=0&before=3&top=5".parse().unwrap())
        .unwrap()
        .0;
    assert_eq!(summary.selection.before, Some(3));
    assert_eq!(summary.top, Some(5));
}
