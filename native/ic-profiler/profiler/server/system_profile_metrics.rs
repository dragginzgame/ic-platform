//! Bounded projections of existing per-subnet IC metrics registries. These are
//! cumulative counters/gauges/wall histograms, not measured component CPU.
use crate::state_api::routes::AppState;
use prometheus::proto::{Metric, MetricFamily, MetricType};
use serde::{Serialize, Serializer, ser::SerializeSeq};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

const MAX_FAMILIES: usize = 512;
const MAX_ROWS: usize = 4096;
const MAX_BYTES: usize = 768 * 1024;
const MAX_BUCKETS: usize = 32;
const MAX_LABELS: usize = 16;

fn stamp() -> Option<u64> {
    #[cfg(unix)]
    {
        let mut value = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut value) } == 0 {
            return Some(
                (value.tv_sec as u64)
                    .saturating_mul(1_000_000_000)
                    .saturating_add(value.tv_nsec as u64),
            );
        }
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MetricsLevel {
    None,
    Standard,
    Full,
}
impl MetricsLevel {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "none" => Ok(Self::None),
            "standard" => Ok(Self::Standard),
            "full" => Ok(Self::Full),
            _ => Err("metrics_level must be none, standard, or full".into()),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Standard => "standard",
            Self::Full => "full",
        }
    }
}

fn short(value: &str, max: usize) -> String {
    if value.len() <= max {
        value.to_owned()
    } else {
        value.chars().take(max).collect()
    }
}

/// The JSON tuple is identical to the previous series-fingerprint encoding,
/// but writes straight into SHA-256 without allocating a label vector or JSON.
fn series_id(name: &str, labels: &[prometheus::proto::LabelPair]) -> String {
    struct Labels<'a>(&'a [prometheus::proto::LabelPair]);
    impl Serialize for Labels<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
            for label in self.0 {
                sequence.serialize_element(&(label.name(), label.value()))?;
            }
            sequence.end()
        }
    }
    let mut hash = ic_crypto_sha2::Sha256::new();
    serde_json::to_writer(&mut hash, &(name, Labels(labels)))
        .expect("string tuples and an infallible SHA writer serialize");
    hex::encode(hash.finish())
}

/// Allocation-free upper bound; avoids serializing every metric row just to
/// test the byte budget. Only the final snapshot is serialized for a hard check.
fn json_upper_bound(value: &Value) -> usize {
    fn string_bytes(text: &str) -> usize {
        2 + text
            .bytes()
            .map(|b| match b {
                b'"' | b'\\' => 2,
                0..=0x1f => 6,
                _ => 1,
            })
            .sum::<usize>()
    }
    match value {
        Value::Null => 4,
        Value::Bool(_) => 5,
        Value::Number(_) => 32,
        Value::String(s) => string_bytes(s),
        Value::Array(a) => 2usize
            .saturating_add(
                a.iter()
                    .map(json_upper_bound)
                    .fold(0usize, usize::saturating_add),
            )
            .saturating_add(a.len()),
        Value::Object(o) => 2usize.saturating_add(
            o.iter()
                .map(|(key, value)| {
                    string_bytes(key)
                        .saturating_add(json_upper_bound(value))
                        .saturating_add(2)
                })
                .fold(0usize, usize::saturating_add),
        ),
    }
}

fn float(value: f64) -> String {
    if value.is_nan() {
        "NaN".into()
    } else if value == f64::INFINITY {
        "+Inf".into()
    } else if value == f64::NEG_INFINITY {
        "-Inf".into()
    } else {
        value.to_string()
    }
}

fn unit(name: &str) -> (Option<&'static str>, &'static str, &'static str) {
    if matches!(
        name,
        "idkg_support_validation_duration"
            | "idkg_support_validation_total_duration"
            | "idkg_support_aggregation_duration"
            | "idkg_support_aggregation_total_duration"
            | "idkg_create_transcript_duration"
            | "idkg_create_transcript_total_duration"
            | "threshold_sig_share_validation_duration"
            | "threshold_sig_share_validation_total_duration"
            | "threshold_sig_share_aggregation_duration"
            | "threshold_sig_share_aggregation_total_duration"
    ) {
        return (
            Some("milliseconds"),
            "explicit_metric_mapping",
            "elapsed_wall_duration",
        );
    }
    if name == "scheduler_compute_utilization_per_core" {
        return (
            Some("ratio"),
            "explicit_metric_mapping",
            "instruction_budget_utilization_not_measured_cpu",
        );
    }
    if name == "execution_round_instructions" {
        return (
            Some("round_instructions"),
            "explicit_metric_mapping",
            "logical_round_budget",
        );
    }
    if name.ends_with("_seconds") || name.ends_with("_seconds_total") {
        return (
            Some("seconds"),
            "suffix",
            "source_defined_time_not_automatically_cpu",
        );
    }
    if name.ends_with("_bytes") || name.ends_with("_bytes_total") {
        return (Some("bytes"), "suffix", "source_defined_bytes");
    }
    if name.ends_with("_count") || name.ends_with("_count_total") {
        return (Some("count"), "suffix", "source_defined_count");
    }
    (None, "unknown", "read_metric_help_and_source")
}

fn priority(name: &str) -> (usize, &str) {
    let rank = if name == "scheduler_compute_utilization_per_core"
        || name == "execution_round_instructions"
    {
        0
    } else if [
        "execution_",
        "scheduler_",
        "sandboxed_",
        "mr_",
        "xnet_",
        "state_manager_",
        "crypto_",
        "ingress_",
        "consensus_",
    ]
    .iter()
    .any(|p| name.starts_with(p))
    {
        1
    } else {
        2
    };
    (rank, name)
}

fn row(family: &MetricFamily, metric: &Metric, level: MetricsLevel) -> Value {
    let name = family.name();
    let kind = family.get_field_type();
    let (unit, unit_source, interpretation) = unit(name);
    let mut labels = BTreeMap::new();
    let mut labels_truncated = metric.get_label().len() > MAX_LABELS;
    let series_id = series_id(name, metric.get_label());
    for label in metric.get_label().iter().take(MAX_LABELS) {
        labels_truncated |= (label.name().len() > 128 && label.name().chars().nth(128).is_some())
            || (label.value().len() > 256 && label.value().chars().nth(256).is_some());
        labels.insert(short(label.name(), 128), short(label.value(), 256));
    }
    let value = match kind {
        MetricType::COUNTER => metric.get_counter().as_ref().map(|v| float(v.value())),
        MetricType::GAUGE => metric.get_gauge().as_ref().map(|v| float(v.value())),
        MetricType::UNTYPED => metric.untyped.as_ref().map(|v| float(v.value())),
        _ => None,
    };
    let detailed = level == MetricsLevel::Full;
    let histogram=metric.get_histogram().as_ref().map(|h|{
        let buckets=h.get_bucket();
        let selected=detailed.then(||buckets.iter().enumerate().filter(|(i,_)|*i<MAX_BUCKETS-1||*i==buckets.len().saturating_sub(1)).map(|(_,b)|json!({"upper_bound":float(b.upper_bound()),"cumulative_count":b.cumulative_count().to_string()})).collect::<Vec<_>>());
        json!({"sample_count":h.sample_count().to_string(),"sample_sum":float(h.sample_sum()),"buckets":selected,"buckets_collected":detailed,"buckets_truncated":detailed&&buckets.len()>MAX_BUCKETS,"source_bucket_count":buckets.len()})
    });
    let summary=metric.get_summary().as_ref().map(|s|json!({"sample_count":s.sample_count().to_string(),"sample_sum":float(s.sample_sum()),"quantiles":detailed.then(||s.get_quantile().iter().take(MAX_BUCKETS).map(|q|json!({"quantile":float(q.quantile()),"value":float(q.value())})).collect::<Vec<_>>()),"quantiles_collected":detailed,"quantiles_truncated":detailed&&s.get_quantile().len()>MAX_BUCKETS}));
    json!({"name":short(name,256),"help":short(family.help(),1024),"metric_type":format!("{kind:?}"),"series_id":series_id,"labels":labels,"labels_truncated":labels_truncated,"unit":unit,"unit_source":unit_source,"interpretation":interpretation,"value":value,"histogram":histogram,"summary":summary,"precision":"prometheus_f64/counts_u64"})
}

pub(crate) fn snapshot(
    app: &AppState,
    selected: Option<usize>,
    phase: &str,
    level: MetricsLevel,
) -> Value {
    let _scope = ic_system_profiler::scope("ic.observability.metrics");
    let start = stamp();
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let snapshot_id = format!(
        "metrics-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    if level == MetricsLevel::None {
        return json!({"kind":"ic_metrics_snapshot","phase":phase,"snapshot_id":snapshot_id,"metrics_level":"none","clock":"host_monotonic","gather_start_ns":start.map(|n|n.to_string()),"gather_end_ns":stamp().map(|n|n.to_string()),"observation":"disabled_by_capture_configuration","rows":[],"coverage":{"status":"disabled","truncated":false,"unavailable":[{"reason":"metrics_level_none"}]}});
    }
    let (registries, mut unavailable) = app.api_state.profiling_metrics_registries(selected);
    let unavailable_omitted = unavailable.len().saturating_sub(64);
    unavailable.truncate(64);
    let total_registries = registries.len();
    let retained_registries = total_registries.min(32);
    let family_quota = (MAX_FAMILIES / retained_registries.max(1)).max(1);
    let row_quota = (MAX_ROWS / retained_registries.max(1)).max(1);
    // Leave a fixed allowance for bounded top-level provenance and missingness.
    let row_byte_limit = MAX_BYTES - 32 * 1024;
    let byte_quota = row_byte_limit / retained_registries.max(1);
    let mut rows = Vec::new();
    let mut family_count = 0usize;
    let mut bytes = 0usize;
    let mut source_family_count = 0usize;
    let mut truncated = unavailable_omitted > 0;
    for (index, (instance_id, subnet_id, generation, registry)) in
        registries.into_iter().enumerate()
    {
        if index >= 32 {
            truncated = true;
            if unavailable.len() < 64 {
                unavailable.push(json!({"instance_id":instance_id,"subnet_id":subnet_id,"reason":"registry_limit"}));
            }
            continue;
        }
        let mut families = {
            let _scope = ic_system_profiler::scope("ic.observability.metrics.gather");
            registry.prometheus_registry().gather()
        };
        let _conversion = ic_system_profiler::scope("ic.observability.metrics.convert");
        source_family_count += families.len();
        families.sort_by(|a, b| priority(a.name()).cmp(&priority(b.name())));
        truncated |= families.len() > family_quota;
        let mut source_bytes = 0usize;
        let mut source_rows = 0usize;
        'families: for family in families.into_iter().take(family_quota) {
            family_count += 1;
            for metric in family.get_metric() {
                if source_rows >= row_quota {
                    truncated = true;
                    break 'families;
                }
                let mut value = row(&family, metric, level);
                value["instance_id"] = json!(instance_id);
                value["subnet_id"] = json!(subnet_id);
                value["registry_generation"] = json!(generation);
                let size = json_upper_bound(&value);
                if source_bytes.saturating_add(size) > byte_quota
                    || bytes.saturating_add(size) > row_byte_limit
                {
                    truncated = true;
                    break 'families;
                }
                source_bytes += size;
                bytes += size;
                source_rows += 1;
                rows.push(value);
            }
        }
    }
    let result = json!({"kind":"ic_metrics_snapshot","phase":phase,"snapshot_id":snapshot_id,"metrics_level":level.name(),"server_generation":crate::memory_access_tracing::server_identity().generation,"clock":"host_monotonic","gather_start_ns":start.map(|n|n.to_string()),"gather_end_ns":stamp().map(|n|n.to_string()),"observation":"prometheus_counter_gauge_wall_histograms_not_component_cpu","precision":{"scalars_and_sums":"Observed f64 encoded as text; original integer precision cannot be restored","counts":"Exact u64 decimal strings"},"rows":rows,"coverage":{"truncated":truncated,"unavailable":unavailable,"unavailable_entries_omitted":unavailable_omitted,"available_registries":total_registries,"retained_registries":retained_registries,"source_family_count":source_family_count,"retained_family_count":family_count,"retained_row_count":rows.len(),"retained_row_bytes_upper_bound":bytes,"histogram_buckets":if level==MetricsLevel::Full{"bounded_raw_buckets"}else{"not_collected_at_standard_level"},"summary_quantiles":if level==MetricsLevel::Full{"bounded_raw_quantiles"}else{"not_collected_at_standard_level"},"atomic_across_registries":false,"external_adapter_scrapes":"not_collected"},"limits":{"max_families":MAX_FAMILIES,"max_rows":MAX_ROWS,"max_bytes":MAX_BYTES,"max_buckets":MAX_BUCKETS,"max_labels":MAX_LABELS},"note":"Cumulative source metrics, not per-request CPU. Standard retains scalar values and histogram count/sum; full additionally retains bounded buckets/quantiles. Differences require matching generations and reset checks. Gathering can materialize a registry internally; retained rows are bounded and instance locks are released first."});
    // One whole-snapshot serialization enforces the final hard bound. The row
    // estimate is conservative, so fallback is only a defensive metadata guard.
    let _serialization = ic_system_profiler::scope("ic.observability.metrics.serialize");
    if serde_json::to_vec(&result).map_or(true, |bytes| bytes.len() > MAX_BYTES) {
        json!({"kind":"ic_metrics_snapshot","phase":phase,"snapshot_id":snapshot_id,"metrics_level":level.name(),"rows":[],"coverage":{"truncated":true,"unavailable":[{"reason":"snapshot_hard_byte_limit"}]},"limits":{"max_bytes":MAX_BYTES}})
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
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
}
