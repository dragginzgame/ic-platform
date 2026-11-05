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
