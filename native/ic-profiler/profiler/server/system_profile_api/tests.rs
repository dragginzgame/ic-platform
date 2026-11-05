use super::*;

#[test]
fn pricing_ledger_differences_keep_transfers_and_currencies_separate() {
    let make = |generation: &str, amount: &str| json!({"canisters":[{"instance_id":0,"canister_id":"aaaaa-aa","trace_generation":generation,"recording":true,"incomplete":false,"totals":[{"rule_id":"fee","unit":"cycles","kind":"debit","amount":amount},{"rule_id":"transfer","unit":"cycles","kind":"transfer","amount":"900000"},{"rule_id":"round","unit":"round_instructions","kind":"debit","amount":"2000000"}]}],"truncated":false});
    let initial = make("1", "9007199254740993");
    let finished = make("1", "9007199254741013");
    let delta = pricing_delta(&initial, &finished);
    assert_eq!(delta["totals"]["cycles_debited"], "20");
    assert_eq!(delta["rows"].as_array().unwrap().len(), 3);
    assert!(
        pricing_delta(&initial, &make("2", "9007199254741013"))["totals"]["cycles_debited"]
            .is_null()
    );
}

#[test]
fn oversized_single_legacy_row_cannot_escape_snapshot_byte_budget() {
    let mut values = Vec::new();
    let mut bytes = 0;
    assert!(!retain_pricing_fragment(
        &mut values,
        json!({"configuration":"x".repeat(MAX_PRICING_SNAPSHOT_BYTES)}),
        &mut bytes
    ));
    assert!(values.is_empty());
    assert_eq!(bytes, 0);
    assert!(retain_pricing_fragment(
        &mut values,
        json!({"amount":"9"}),
        &mut bytes
    ));
    let before = bytes;
    assert!(!retain_pricing_fragment(
        &mut values,
        json!({"totals":"x".repeat(MAX_PRICING_SNAPSHOT_BYTES)}),
        &mut bytes
    ));
    assert_eq!(bytes, before);
    assert_eq!(values.len(), 1);
}
