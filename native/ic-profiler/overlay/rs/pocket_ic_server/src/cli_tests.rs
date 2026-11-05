use super::*;
use serde_json::{Value, json};

fn sample_catalog() -> Value {
    let mut catalog = json!({
        "format_version": 1,
        "binary": { "version": "test", "sha256": "binary-hash" },
        "defaults": { "instruction_limit": "18446744073709551615" },
        "limitations": ["Configured instances can override defaults"],
        "charge_rules": {
            "ic.compute.v1": {
                "id": "ic.compute.v1",
                "source": [{"file": "runtime.rs", "code": "charge(amount);\n"}]
            },
            "ic.compute.v10": { "id": "ic.compute.v10" }
        }
    });
    let digest = hex::encode(Sha256::hash(&serde_json::to_vec(&catalog).unwrap()));
    catalog["catalog_sha256"] = json!(digest);
    catalog
}

#[test]
fn explain_options_are_optional_and_explicit() {
    let parsed = Args::try_parse_from(["pocket-ic", "explain"]).unwrap();
    let Some(CliCommand::Explain(options)) = parsed.command else {
        panic!("expected standalone explain command");
    };
    assert_eq!(options.rule, None);
    assert!(!options.compact);
    let parsed = Args::try_parse_from([
        "pocket-ic",
        "explain",
        "--rule",
        "ic.compute.v1",
        "--compact",
    ])
    .unwrap();
    let Some(CliCommand::Explain(options)) = parsed.command else {
        panic!("expected standalone explain command");
    };
    assert_eq!(options.rule.as_deref(), Some("ic.compute.v1"));
    assert!(options.compact);
    assert!(Args::try_parse_from(["pocket-ic", "explain", "--rule"]).is_err());
    assert!(Args::try_parse_from(["pocket-ic", "--rule", "ic.compute.v1"]).is_err());
}

#[test]
fn server_flags_remain_available_and_are_separate_from_explain() {
    let parsed = Args::try_parse_from([
        "pocket-ic",
        "--port",
        "8081",
        "--ttl",
        "17",
        "--port-file",
        "server.port",
        "--memory-access-viewer-port",
        "0",
    ])
    .unwrap();
    assert!(parsed.command.is_none());
    assert_eq!(parsed.port, 8081);
    assert_eq!(parsed.ttl, 17);
    assert_eq!(parsed.port_file, Some(PathBuf::from("server.port")));
    assert_eq!(parsed.memory_access_viewer_port, Some(0));
    assert!(Args::try_parse_from(["pocket-ic", "--port", "8081", "explain"]).is_err());
    assert!(Args::try_parse_from(["pocket-ic", "explain", "--port", "8081"]).is_err());
    assert!(Args::try_parse_from(["pocket-ic", "explain", "--port-file", "server.port"]).is_err());
}

#[test]
fn explain_pretty_json_preserves_exact_values_and_source() {
    let catalog = sample_catalog();
    let mut output = Vec::new();
    write_computation_catalog(
        catalog.clone(),
        &ExplainArgs {
            rule: None,
            compact: false,
        },
        &mut output,
    )
    .unwrap();
    assert!(output.windows(2).any(|bytes| bytes == b"\n "));
    assert!(output.ends_with(b"\n"));
    assert_eq!(serde_json::from_slice::<Value>(&output).unwrap(), catalog);
}

#[test]
fn explain_compact_json_is_one_complete_line() {
    let mut output = Vec::new();
    write_computation_catalog(
        sample_catalog(),
        &ExplainArgs {
            rule: None,
            compact: true,
        },
        &mut output,
    )
    .unwrap();
    assert_eq!(output.iter().filter(|byte| **byte == b'\n').count(), 1);
    assert!(output.ends_with(b"\n"));
    assert_eq!(
        serde_json::from_slice::<Value>(&output).unwrap(),
        sample_catalog()
    );
}

#[test]
fn explain_rule_filter_is_exact_and_retains_the_envelope() {
    let mut output = Vec::new();
    write_computation_catalog(
        sample_catalog(),
        &ExplainArgs {
            rule: Some("ic.compute.v1".to_string()),
            compact: false,
        },
        &mut output,
    )
    .unwrap();
    let mut filtered: Value = serde_json::from_slice(&output).unwrap();
    let mut expected = sample_catalog();
    assert_eq!(filtered["charge_rules"].as_object().unwrap().len(), 1);
    assert_eq!(filtered["full_catalog_sha256"], expected["catalog_sha256"]);
    let filtered_hash = filtered
        .as_object_mut()
        .unwrap()
        .remove("catalog_sha256")
        .unwrap();
    assert_eq!(
        filtered_hash,
        hex::encode(Sha256::hash(&serde_json::to_vec(&filtered).unwrap()))
    );
    filtered
        .as_object_mut()
        .unwrap()
        .remove("full_catalog_sha256");
    expected.as_object_mut().unwrap().remove("catalog_sha256");
    expected["charge_rules"]
        .as_object_mut()
        .unwrap()
        .remove("ic.compute.v10");
    assert_eq!(filtered, expected);
}

#[test]
fn explain_unknown_rule_does_not_write_a_partial_document() {
    let mut output = Vec::new();
    let error = write_computation_catalog(
        sample_catalog(),
        &ExplainArgs {
            rule: Some("ic.compute".to_string()),
            compact: true,
        },
        &mut output,
    )
    .unwrap_err();
    assert!(error.contains("unknown charge rule 'ic.compute'"));
    assert!(output.is_empty());
}

#[test]
fn explain_compiled_catalog_is_available_without_a_tokio_runtime() {
    assert!(tokio::runtime::Handle::try_current().is_err());
    let mut output = Vec::new();
    write_computation_catalog(
        pocket_ic_server::computation_catalog::catalog(),
        &ExplainArgs {
            rule: None,
            compact: true,
        },
        &mut output,
    )
    .unwrap();
    let catalog: Value = serde_json::from_slice(&output).unwrap();
    assert!(!catalog["charge_rules"].as_object().unwrap().is_empty());
    assert!(
        catalog["charge_rules"]
            .as_object()
            .unwrap()
            .values()
            .any(
                |rule| rule["source"].as_array().is_some_and(|sources| sources
                    .iter()
                    .any(|source| source["code"].as_str().is_some_and(|code| !code.is_empty())))
            )
    );
    assert!(tokio::runtime::Handle::try_current().is_err());
}

#[test]
fn explain_filtered_compiled_catalog_has_its_own_verifiable_hash() {
    let catalog = pocket_ic_server::computation_catalog::catalog();
    let full_hash = catalog["catalog_sha256"].clone();
    let rule = catalog["charge_rules"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    let mut output = Vec::new();
    write_computation_catalog(
        catalog,
        &ExplainArgs {
            rule: Some(rule.clone()),
            compact: true,
        },
        &mut output,
    )
    .unwrap();
    let mut filtered: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(filtered["full_catalog_sha256"], full_hash);
    assert_eq!(filtered["charge_rules"].as_object().unwrap().len(), 1);
    assert!(filtered["charge_rules"][rule].is_object());
    let reported = filtered
        .as_object_mut()
        .unwrap()
        .remove("catalog_sha256")
        .unwrap();
    assert_eq!(
        reported,
        hex::encode(Sha256::hash(&serde_json::to_vec(&filtered).unwrap()))
    );
}

#[test]
fn instruction_profiler_limits_are_opt_in_cli_settings() {
    let defaults = Args::try_parse_from(["pocket-ic"]).unwrap();
    assert!(defaults.instruction_profile_max_origin_ranges.is_none());
    assert!(defaults.memory_access_trace_limit_mib.is_none());
    let configured = Args::try_parse_from([
        "pocket-ic",
        "--instruction-profile-max-origin-ranges",
        "1048576",
        "--memory-access-trace-limit-mib",
        "256",
    ])
    .unwrap();
    assert_eq!(
        configured.instruction_profile_max_origin_ranges,
        Some(1_048_576)
    );
    assert_eq!(configured.memory_access_trace_limit_mib, Some(256));
    assert!(
        Args::try_parse_from(["pocket-ic", "--instruction-profile-max-origin-ranges", "-1"])
            .is_err()
    );
    assert!(
        Args::try_parse_from([
            "pocket-ic",
            "--instruction-profile-max-origin-ranges",
            "1048576",
            "explain"
        ])
        .is_err()
    );
}

#[test]
fn ram_cli_options_are_opt_in_and_do_not_change_explain_or_version_parsing() {
    let default = Args::try_parse_from(["pocket-ic"]).unwrap();
    assert!(!default.ram);
    assert!(default.ram_dir.is_none());
    let ram = Args::try_parse_from(["pocket-ic", "--ram", "--ram-dir", "/dev/shm"]).unwrap();
    assert!(ram.ram);
    assert_eq!(ram.ram_dir, Some(PathBuf::from("/dev/shm")));
    let directory_only = Args::try_parse_from(["pocket-ic", "--ram-dir", "/dev/shm"]).unwrap();
    assert_eq!(directory_only.ram_dir, Some(PathBuf::from("/dev/shm")));
    assert!(Args::try_parse_from(["pocket-ic", "--ram", "explain"]).is_err());
    let version = Args::try_parse_from(["pocket-ic", "--version"]).unwrap_err();
    assert_eq!(version.kind(), clap::error::ErrorKind::DisplayVersion);
}

#[test]
fn profile_subcommand_keeps_ordinary_clap_argument_discovery() {
    let parsed = Args::try_parse_from([
        "pocket-ic",
        "profile",
        "summary",
        "--input",
        "saved-profile.json",
    ])
    .unwrap();
    assert!(matches!(parsed.command, Some(CliCommand::Profile(_))));
    assert!(Args::try_parse_from(["pocket-ic", "profile", "unknown-action"]).is_err());
}
