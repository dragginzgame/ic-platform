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
