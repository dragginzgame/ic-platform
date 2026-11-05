use super::*;
#[test]
fn backend_examples_use_compiled_defaults_and_retain_source_identity() {
    let config = ic_config::subnet_config::SubnetConfig::new(
        ic_registry_subnet_type::SubnetType::Application,
    )
    .scheduler_config;
    let result = explain();
    let example = &result["rounds"]["scheduler_model"]["example_application_defaults"];
    assert_eq!(
        example["nominal_round_instructions_per_worker"],
        config.max_instructions_per_round.get().to_string()
    );
    assert_eq!(
        example["max_regular_message_instructions"],
        config.max_instructions_per_message.get().to_string()
    );
    assert_eq!(
        result["backend_description"]["source_sha256"]
            .as_object()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        result["components"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["component_id"] == "ic.chain_key")
            .unwrap()["pocketic_implementation"],
        "emulated"
    );
    assert!(
        result["coverage_notes"][0]
            .as_str()
            .unwrap()
            .contains("PocketIC")
    );
}
