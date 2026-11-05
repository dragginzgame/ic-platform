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
