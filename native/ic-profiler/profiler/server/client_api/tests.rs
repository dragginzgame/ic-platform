use super::*;
#[test]
fn clear_guards_keep_wide_values_and_missing_vs_null_runtime_cursor() {
    let absent: Options = serde_json::from_value(json!({})).unwrap();
    assert_eq!(absent.expected_last_runtime_cost_sequence_number, None);
    let empty: Options =
        serde_json::from_value(json!({"expected_last_runtime_cost_sequence_number":null})).unwrap();
    assert_eq!(empty.expected_last_runtime_cost_sequence_number, Some(None));
    let wide: Options = serde_json::from_value(json!({"expected_trace_generation":u64::MAX.to_string(),"expected_last_sequence_number":"0","expected_last_runtime_cost_sequence_number":u64::MAX.to_string()})).unwrap();
    assert_eq!(wide.expected_trace_generation, Some(u64::MAX));
    assert_eq!(
        wide.expected_last_runtime_cost_sequence_number,
        Some(Some(u64::MAX))
    );
    assert!(serde_json::from_value::<Options>(json!({"expected_trace_generation":1.5})).is_err());
}
