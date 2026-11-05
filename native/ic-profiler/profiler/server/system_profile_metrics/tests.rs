use super::*;
#[test]
fn explicit_units_do_not_mislabel_idkg_or_budget_utilization_as_cpu() {
    assert_eq!(
        unit("idkg_create_transcript_duration"),
        (
            Some("milliseconds"),
            "explicit_metric_mapping",
            "elapsed_wall_duration"
        )
    );
    assert_eq!(
        unit("scheduler_compute_utilization_per_core").2,
        "instruction_budget_utilization_not_measured_cpu"
    );
    assert_eq!(unit("mystery_duration").0, None);
}
#[test]
fn registry_snapshot_preserves_counter_labels_histograms_and_unknown_units() {
    let registry = ic_metrics::MetricsRegistry::new();
    let counter = registry.int_counter_vec("test_requests_total", "Request count", &["result"]);
    counter.with_label_values(&["ok"]).inc_by(3);
    let histogram = registry.histogram(
        "test_latency_seconds",
        "Elapsed wall latency",
        vec![0.1, 1.0],
    );
    histogram.observe(0.25);
    let mut values = Vec::new();
    for family in registry.prometheus_registry().gather() {
        for metric in family.get_metric() {
            values.push(row(&family, metric, MetricsLevel::Standard));
        }
    }
    let count = values
        .iter()
        .find(|r| r["name"] == "test_requests_total")
        .unwrap();
    assert_eq!(count["value"], "3");
    assert_eq!(count["labels"]["result"], "ok");
    assert!(count["unit"].is_null());
    let latency = values
        .iter()
        .find(|r| r["name"] == "test_latency_seconds")
        .unwrap();
    assert_eq!(latency["unit"], "seconds");
    assert_eq!(latency["histogram"]["sample_count"], "1");
    assert_eq!(latency["histogram"]["sample_sum"], "0.25");
}
#[test]
fn standard_keeps_raw_values_and_counts_while_full_preserves_buckets() {
    let registry = ic_metrics::MetricsRegistry::new();
    let histogram = registry.histogram(
        "test_latency_seconds",
        "Elapsed wall latency",
        (1..=64).map(|n| n as f64).collect(),
    );
    histogram.observe(20.5);
    let families = registry.prometheus_registry().gather();
    let family = &families[0];
    let metric = &family.get_metric()[0];
    let standard = row(family, metric, MetricsLevel::Standard);
    let full = row(family, metric, MetricsLevel::Full);
    assert_eq!(
        standard["histogram"]["sample_count"],
        full["histogram"]["sample_count"]
    );
    assert_eq!(
        standard["histogram"]["sample_sum"],
        full["histogram"]["sample_sum"]
    );
    assert_eq!(standard["series_id"], full["series_id"]);
    assert!(standard["histogram"]["buckets"].is_null());
    assert_eq!(standard["histogram"]["buckets_collected"], false);
    assert_eq!(
        full["histogram"]["buckets"].as_array().unwrap().len(),
        MAX_BUCKETS
    );
    assert_eq!(full["histogram"]["buckets_truncated"], true);
    assert!(standard.to_string().len() * 2 < full.to_string().len());
    let labels: Vec<_> = metric
        .get_label()
        .iter()
        .map(|label| (label.name(), label.value()))
        .collect();
    assert_eq!(
        series_id(family.name(), metric.get_label()),
        hex::encode(ic_crypto_sha2::Sha256::hash(
            &serde_json::to_vec(&(family.name(), labels)).unwrap()
        ))
    );
}

#[test]
fn allocation_free_json_bound_covers_escaping_and_numeric_fields() {
    for value in [
        json!({"quotes\\\n\u{0}":"é\u{2028}\u{1f}\n\\\""}),
        json!([u64::MAX, -123456789, true, false, null, 1.3e-100]),
        json!({"nested":[{"text":"plain ascii"}]}),
    ] {
        assert!(json_upper_bound(&value) >= serde_json::to_vec(&value).unwrap().len());
    }
    assert!(MetricsLevel::parse("none").is_ok());
    assert!(MetricsLevel::parse("standard").is_ok());
    assert!(MetricsLevel::parse("full").is_ok());
    assert!(MetricsLevel::parse("invalid").is_err());
}
