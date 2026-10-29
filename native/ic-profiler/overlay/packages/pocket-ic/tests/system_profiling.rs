//! Native system-profiler regressions: the observer must preserve execution,
//! and its run data must remain useful without per-instruction tracing.

use candid::Principal;
use pocket_ic::{PocketIc, PocketIcBuilder};
use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::time::Duration;

// Captures observe the whole shared server process, so independent test
// workloads must not run inside one another's recording windows.
static CAPTURE_TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn exclusive_capture() -> std::sync::MutexGuard<'static, ()> {
    CAPTURE_TEST
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

const WORK_WAT: &str = r#"
(module
  (import "ic0" "msg_reply" (func $reply))
  (import "ic0" "msg_reply_data_append" (func $append (param i32 i32)))
  (import "ic0" "performance_counter" (func $counter (param i32) (result i64)))
  (import "ic0" "stable64_grow" (func $grow (param i64) (result i64)))
  (import "ic0" "stable64_read" (func $read (param i64 i64 i64)))
  (import "ic0" "stable64_write" (func $write (param i64 i64 i64)))
  (memory (export "memory") 4)
  (func (export "canister_init") (drop (call $grow (i64.const 1))))
  (func (export "canister_update work") (local $n i32)
    (call $read (i64.const 131072) (i64.const 0) (i64.const 8))
    (call $read (i64.const 131072) (i64.const 0) (i64.const 8))
    (i64.store (i32.const 65536) (i64.const 42))
    (call $write (i64.const 0) (i64.const 65536) (i64.const 8))
    (local.set $n (i32.const 200000))
    (loop $again
      (local.set $n (i32.sub (local.get $n) (i32.const 1)))
      (br_if $again (local.get $n)))
    (i64.store (i32.const 0) (call $counter (i32.const 0)))
    (call $append (i32.const 0) (i32.const 8))
    (call $reply))
  (func (export "canister_query value")
    (call $append (i32.const 65536) (i32.const 8)) (call $reply))
  (func (export "canister_update fail")
    (i64.store (i32.const 65536) (i64.const 99)) unreachable))
"#;

const RELAY_WAT: &str = r#"
(module
  (import "ic0" "msg_arg_data_copy" (func $args (param i32 i32 i32)))
  (import "ic0" "msg_arg_data_size" (func $size (result i32)))
  (import "ic0" "msg_reply" (func $reply))
  (import "ic0" "call_new" (func $new (param i32 i32 i32 i32 i32 i32 i32 i32)))
  (import "ic0" "call_perform" (func $perform (result i32)))
  (memory (export "memory") 1)
  (global $target_len (mut i32) (i32.const 0))
  (table 2 funcref)
  (elem (i32.const 0) $done $rejected)
  (data (i32.const 128) "work")
  (func $done (param i32) (call $reply))
  (func $rejected (param i32) unreachable)
  (func (export "canister_update set_target")
    (global.set $target_len (call $size))
    (call $args (i32.const 0) (i32.const 0) (global.get $target_len))
    (call $reply))
  (func (export "canister_update relay")
    (call $new (i32.const 0) (global.get $target_len)
      (i32.const 128) (i32.const 4)
      (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 0))
    (if (call $perform) (then unreachable))))
"#;

fn client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .unwrap()
}

fn builder() -> PocketIcBuilder {
    let builder = PocketIcBuilder::new();
    if let Ok(server) = std::env::var("SYSTEM_PROFILE_TEST_SERVER_URL") {
        builder.with_server_url(server.parse().unwrap())
    } else {
        builder
    }
}

fn profile_url(pic: &PocketIc, suffix: &str) -> reqwest::Url {
    pic.get_server_url()
        .join(&format!("api/profiling/runs{suffix}"))
        .unwrap()
}

fn begin(pic: &PocketIc) -> String {
    let result: Value = client()
        .post(profile_url(pic, ""))
        .json(&json!({}))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    result["run_id"].as_str().unwrap().to_string()
}

fn view(pic: &PocketIc, id: &str, view: &str) -> Value {
    client()
        .get(profile_url(pic, &format!("/{id}/{view}")))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap()
}

fn finish(pic: &PocketIc, id: &str) -> Value {
    client()
        .post(profile_url(pic, &format!("/{id}/stop")))
        .json(&json!({}))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap()
}

fn install(pic: &PocketIc, subnet: Principal, wat: &str) -> Principal {
    let canister = pic.create_canister_on_subnet(None, None, subnet);
    pic.add_cycles(canister, 2_000_000_000_000);
    pic.install_canister(canister, wat::parse_str(wat).unwrap(), vec![], None);
    canister
}

#[test]
fn system_capture_preserves_instructions_cycles_and_rollback() {
    let _capture = exclusive_capture();
    let pic = builder().with_application_subnet().build();
    let canister = install(&pic, pic.topology().get_app_subnets()[0], WORK_WAT);
    let before = pic.cycle_balance(canister);
    let baseline = pic
        .update_call(canister, Principal::anonymous(), "work", vec![])
        .unwrap();
    let baseline_cost = before - pic.cycle_balance(canister);

    let id = begin(&pic);
    let before = pic.cycle_balance(canister);
    let profiled = pic
        .update_call(canister, Principal::anonymous(), "work", vec![])
        .unwrap();
    assert_eq!(
        profiled, baseline,
        "observation changed priced instructions"
    );
    assert_eq!(before - pic.cycle_balance(canister), baseline_cost);

    let expected_state = pic
        .query_call(canister, Principal::anonymous(), "value", vec![])
        .unwrap();
    assert!(
        pic.update_call(canister, Principal::anonymous(), "fail", vec![])
            .is_err()
    );
    assert_eq!(
        pic.query_call(canister, Principal::anonymous(), "value", vec![])
            .unwrap(),
        expected_state,
        "observing a trap changed rollback"
    );
    for _ in 0..8 {
        pic.update_call(canister, Principal::anonymous(), "work", vec![])
            .unwrap();
    }
    let stopped = finish(&pic, &id);
    assert_eq!(stopped["run_id"], id);
    let summary = view(&pic, &id, "summary");
    assert_eq!(summary["run_id"], id);
    assert!(summary.get("coverage").is_some());
    assert!(
        summary["summary"]["cpu_time_ns"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            > 0
    );
    assert!(
        summary.to_string().len() < 256 * 1024,
        "summary eagerly included raw traces"
    );
    let components = view(&pic, &id, "components");
    let text = components.to_string();
    assert!(
        text.contains("scheduler"),
        "missing scheduler component: {components}"
    );
    assert!(
        text.contains("wasm") || text.contains("execution"),
        "missing execution component"
    );
    let resources = view(&pic, &id, "resources");
    assert!(resources.get("coverage").is_some());
    let events = view(&pic, &id, "events");
    let root_pid = summary["scope"]["root_pid"].as_u64().unwrap();
    assert!(
        events["events"].as_array().unwrap().iter().any(|event| {
            event["pid"].as_u64().is_some_and(|pid| pid != root_pid)
                && event["exclusive_cpu_ns"]
                    .as_str()
                    .and_then(|v| v.parse::<u64>().ok())
                    .is_some_and(|v| v > 0)
        }),
        "child-process CPU scopes did not reach the capture: {events}"
    );
    assert!(
        pic.get_memory_access_trace(canister).executions.is_empty(),
        "system capture unexpectedly enabled canister instruction tracing"
    );
    let selected = pic.query_system_profile(
        &id,
        "summary",
        &pocket_ic::SystemProfileQuery {
            canister_id: Some(canister.to_text()),
            ..Default::default()
        },
    );
    assert_eq!(selected["summary"]["completed_root_requests"], "10");
    assert_eq!(
        selected["summary"]["latency_cohort"]["pending_or_right_censored"],
        "0"
    );
}

#[test]
fn system_capture_exposes_cross_subnet_calls_and_bounded_exports() {
    let _capture = exclusive_capture();
    let pic = builder()
        .with_application_subnet()
        .with_application_subnet()
        .build();
    let subnets = pic.topology().get_app_subnets();
    let caller = install(&pic, subnets[0], RELAY_WAT);
    let callee = install(&pic, subnets[1], WORK_WAT);
    pic.update_call(
        caller,
        Principal::anonymous(),
        "set_target",
        callee.as_slice().to_vec(),
    )
    .unwrap();
    let id = begin(&pic);
    let before = pic.cycle_balance(caller) + pic.cycle_balance(callee);
    for _ in 0..3 {
        pic.update_call(caller, Principal::anonymous(), "relay", vec![])
            .unwrap();
    }
    let consumed = before - pic.cycle_balance(caller) - pic.cycle_balance(callee);
    finish(&pic, &id);
    let prices = view(&pic, &id, "pricing");
    assert_eq!(
        prices["pricing"]["totals"]["cycles_net_burned"]
            .as_str()
            .unwrap()
            .parse::<u128>()
            .unwrap(),
        consumed,
        "cross-subnet/callback fees were lost or counted tentatively: {prices}"
    );
    let flows = view(&pic, &id, "flows");
    let text = flows.to_string();
    assert!(
        text.contains(&caller.to_text()),
        "missing caller flow identity"
    );
    assert!(
        text.contains(&callee.to_text()),
        "missing callee flow identity"
    );
    let edges = flows["flows"]["edges"].as_array().unwrap();
    assert!(edges.iter().all(|edge| edge["causal"] == true));
    assert_eq!(
        edges
            .iter()
            .filter(|edge| edge["kind"] == "root_request_lifecycle")
            .count(),
        3
    );
    for direction in ["request", "response"] {
        assert_eq!(
            edges
                .iter()
                .filter(
                    |edge| edge["kind"] == "stream_delivery" && edge["message_kind"] == direction
                )
                .count(),
            3,
            "missing confirmed directional XNet delivery: {flows}"
        );
    }
    assert!(
        edges
            .iter()
            .filter(|edge| edge["kind"] == "induction_to_execution")
            .count()
            >= 6
    );
    assert!(
        edges
            .iter()
            .all(|edge| edge["kind"] != "observed_identity_sequence"
                && edge["kind"] != "scope_parent")
    );
    let explain = view(&pic, &id, "explain");
    assert!(explain.to_string().contains("xnet") || explain.to_string().contains("XNet"));

    let svg = client()
        .get(profile_url(&pic, &format!("/{id}/render.svg")))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .text()
        .unwrap();
    assert!(svg.contains("<svg"));
    let bundle = client()
        .get(profile_url(&pic, &format!("/{id}/export")))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .bytes()
        .unwrap();
    assert!(bundle.len() > 100);
    assert!(
        bundle.len() < 32 * 1024 * 1024,
        "unbounded default capture export"
    );
    assert_eq!(
        client()
            .get(profile_url(&pic, "/does-not-exist/summary"))
            .send()
            .unwrap()
            .status(),
        reqwest::StatusCode::NOT_FOUND,
    );
}

#[test]
fn rust_client_and_agent_exports_share_the_same_evidence() {
    let _capture = exclusive_capture();
    use pocket_ic::{SystemProfileConfig, SystemProfileQuery};

    let pic = builder().with_application_subnet().build();
    let canister = install(&pic, pic.topology().get_app_subnets()[0], WORK_WAT);
    let started = pic.start_system_profile(SystemProfileConfig {
        name: "Rust client and agent evidence".to_string(),
        ..Default::default()
    });
    let id = started["run_id"].as_str().unwrap();
    pic.update_call(canister, Principal::anonymous(), "work", vec![])
        .unwrap();
    pic.stop_system_profile(id);
    let summary = pic.get_system_profile(id);
    assert_eq!(summary["scope"]["instance_id"], pic.instance_id());
    let components = pic.query_system_profile(id, "components", &SystemProfileQuery::default());
    let query_id = components["query_id"].as_str().unwrap();

    let metrics = pic.query_system_profile(id, "metrics", &SystemProfileQuery::default());
    let metric_rows = metrics["metrics"]["rows"].as_array().unwrap();
    let budget = metric_rows
        .iter()
        .find(|row| row["name"] == "scheduler_compute_utilization_per_core")
        .expect("existing scheduler metrics were not captured");
    assert_eq!(
        budget["interpretation"],
        "instruction_budget_utilization_not_measured_cpu"
    );
    assert_eq!(budget["unit"], "ratio");
    assert!(metric_rows.iter().any(|row| row["phase"] == "initial"));
    assert!(metric_rows.iter().any(|row| row["phase"] == "stop"));
    assert_eq!(metrics["metrics"]["snapshots"].as_array().unwrap().len(), 2);
    assert!(
        metric_rows
            .iter()
            .all(|row| row["phase"] != "foreground_end")
    );
    for row in metric_rows {
        if row["metric_type"] == "HISTOGRAM" {
            assert!(row["histogram"]["sample_count"].is_string());
            assert_eq!(row["histogram"]["buckets_collected"], false);
        }
    }
    assert!(
        metric_rows
            .iter()
            .all(|row| row["instance_id"] == pic.instance_id())
    );

    let resources = pic.query_system_profile(id, "resources", &SystemProfileQuery::default());
    let samples = resources["resources"]["samples"].as_array().unwrap();
    let timestamps: Vec<u64> = samples
        .iter()
        .map(|sample| sample["time_ns"].as_str().unwrap().parse().unwrap())
        .collect();
    assert!(timestamps.windows(2).all(|pair| pair[0] <= pair[1]));
    let end: u64 = resources["window"]["end_ns"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(timestamps.last(), Some(&end));
    assert_eq!(resources["quality"]["counter_resets"], "0");

    let mut metadata_url = profile_url(&pic, &format!("/{id}/render.json"));
    metadata_url
        .query_pairs_mut()
        .append_pair("query_id", query_id);
    let metadata: Value = client()
        .get(metadata_url)
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(metadata["query_id"], query_id);
    let mut image_url = profile_url(&pic, &format!("/{id}/render.png"));
    image_url
        .query_pairs_mut()
        .append_pair("query_id", query_id);
    let png = client()
        .get(image_url)
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .bytes()
        .unwrap();
    assert!(png.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
    let bundle = pic.export_system_profile(id);
    assert!(!bundle.is_empty());
    assert!(pic.list_system_profiles().to_string().contains(id));
}

#[test]
fn prices_reconcile_without_instruction_traces_even_when_event_details_drop() {
    let _capture = exclusive_capture();
    use pocket_ic::{SystemProfileConfig, SystemProfileQuery};

    let pic = builder().with_application_subnet().build();
    let canister = install(&pic, pic.topology().get_app_subnets()[0], WORK_WAT);
    let started = pic.start_system_profile(SystemProfileConfig {
        name: "Bounded details, independent price totals".into(),
        max_events: 1,
        max_retained_bytes: 1024 * 1024,
        ..Default::default()
    });
    let id = started["run_id"].as_str().unwrap();
    let balance_before = pic.cycle_balance(canister);
    for _ in 0..4 {
        pic.update_call(canister, Principal::anonymous(), "work", vec![])
            .unwrap();
    }
    let consumed = balance_before - pic.cycle_balance(canister);
    pic.stop_system_profile(id);
    let pricing = pic.query_system_profile(id, "pricing", &SystemProfileQuery::default());
    assert!(
        pricing["quality"]["dropped_events"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            > 0
    );
    assert_eq!(
        pricing["pricing"]["aggregates_survive_event_retention"],
        true
    );
    assert_eq!(
        pricing["pricing"]["totals"]["cycles_net_burned"]
            .as_str()
            .unwrap()
            .parse::<u128>()
            .unwrap(),
        consumed,
        "committed host-ledger bridge did not reconcile with actual cycle balance: {pricing}",
    );
    assert!(pic.get_memory_access_trace(canister).executions.is_empty());
}

#[test]
fn scheduler_rounds_preserve_dts_accounting_when_event_details_drop() {
    let _capture = exclusive_capture();
    use pocket_ic::{SystemProfileConfig, SystemProfileQuery};

    fn integer(value: &Value) -> i128 {
        value
            .as_str()
            .expect("exact decimal integer")
            .parse()
            .unwrap()
    }

    let pic = builder().with_application_subnet().build();
    // This loop crosses a production DTS slice. Its completed-message total
    // must not be mistaken for the work performed by the final slice alone.
    let wat = r#"(module
      (import "ic0" "performance_counter" (func $counter (param i32) (result i64)))
      (import "ic0" "msg_reply_data_append" (func $append (param i32 i32)))
      (import "ic0" "msg_reply" (func $reply))
      (memory (export "memory") 1)
      (func (export "canister_update burn") (local $n i32)
        (local.set $n (i32.const 700000000))
        (loop $again
          (local.tee $n (i32.sub (local.get $n) (i32.const 1)))
          (br_if $again))
        (i64.store (i32.const 0) (call $counter (i32.const 0)))
        (call $append (i32.const 0) (i32.const 8))
        (call $reply)))"#;
    let canister = install(&pic, pic.topology().get_app_subnets()[0], wat);
    let balance = pic.cycle_balance(canister);
    let baseline = pic
        .update_call(canister, Principal::anonymous(), "burn", vec![])
        .unwrap();
    let baseline_cost = balance - pic.cycle_balance(canister);

    let started = pic.start_system_profile(SystemProfileConfig {
        name: "DTS round budgets survive event truncation".into(),
        max_events: 1,
        max_retained_bytes: 16 * 1024 * 1024,
        metrics_level: "none".into(),
        memory_detail_interval_ms: 0,
        ..Default::default()
    });
    let id = started["run_id"].as_str().unwrap();
    let balance = pic.cycle_balance(canister);
    let profiled = pic
        .update_call(canister, Principal::anonymous(), "burn", vec![])
        .unwrap();
    assert_eq!(
        profiled, baseline,
        "round collection changed guest metering"
    );
    assert_eq!(balance - pic.cycle_balance(canister), baseline_cost);
    pic.stop_system_profile(id);

    let query = SystemProfileQuery {
        canister_id: Some(canister.to_text()),
        limit: 1000,
        ..Default::default()
    };
    let result = pic.query_system_profile(id, "rounds", &query);
    assert!(integer(&result["quality"]["dropped_events"]) > 0);
    assert_eq!(
        result["rounds"]["coverage"]["generic_event_loss_independent"],
        true
    );
    assert_eq!(result["rounds"]["coverage"]["status"], "complete");
    let rows = result["rounds"]["rows"].as_array().unwrap();
    assert!(rows.len() >= 2, "DTS must span multiple rounds: {result}");
    let configs = result["rounds"]["configurations"].as_object().unwrap();
    let mut slices = 0;
    let mut guest = 0;
    let mut completed = Vec::new();
    let mut reserve_observed = false;
    for row in rows {
        assert_eq!(row["accounting_reconciles"], true, "{row}");
        assert_eq!(row["details_complete"], true);
        let round_heap_delta = integer(&row["heap_delta_bytes"]);
        let mut observed_heap_delta = 0;
        let config = &configs[row["config_id"].as_str().unwrap()];
        let nominal = integer(&config["max_instructions_per_round"]);
        let slice_limit = integer(&config["max_instructions_per_slice"]);
        assert_eq!(nominal, 4_000_000_000);
        assert_eq!(slice_limit, 2_000_000_000);
        assert_eq!(
            integer(&config["max_instructions_per_message"]),
            40_000_000_000
        );
        assert_eq!(
            integer(&row["initial_instructions"]),
            nominal - slice_limit + 1
        );
        reserve_observed |= integer(&row["remaining_instructions"]) < 0;
        let mut global = 0;
        for iteration in row["iterations"].as_array().unwrap() {
            assert_eq!(iteration["accounting_reconciles"], true);
            let workers = iteration["workers"].as_array().unwrap();
            let max_worker = workers
                .iter()
                .map(|w| integer(&w["consumed_instructions"]))
                .max()
                .unwrap_or(0);
            let sum_worker: i128 = workers
                .iter()
                .map(|w| integer(&w["consumed_instructions"]))
                .sum();
            let iteration_heap_delta = integer(&iteration["heap_delta_bytes"]);
            assert_eq!(
                iteration_heap_delta,
                workers
                    .iter()
                    .map(|w| integer(&w["heap_delta_bytes"]))
                    .sum::<i128>()
            );
            observed_heap_delta += iteration_heap_delta;
            assert_eq!(max_worker, integer(&iteration["max_worker_consumed"]));
            assert_eq!(sum_worker, integer(&iteration["sum_worker_consumed"]));
            let iteration_used = max_worker + integer(&iteration["finalization_instructions"]);
            assert_eq!(
                integer(&iteration["budget_before"]) - integer(&iteration["budget_after"]),
                iteration_used
            );
            global += iteration_used;
            for worker in workers {
                assert_eq!(
                    integer(&worker["heap_delta_bytes"]),
                    worker["canisters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|pass| integer(&pass["heap_delta_bytes"]))
                        .sum::<i128>()
                );
                assert_eq!(
                    integer(&worker["consumed_instructions"]),
                    integer(&worker["execution_round_instructions"])
                        + integer(&worker["execution_overhead_instructions"])
                        + integer(&worker["canister_overhead_instructions"]),
                );
                for pass in worker["canisters"].as_array().unwrap() {
                    assert_eq!(
                        integer(&pass["heap_delta_bytes"]),
                        pass["slices"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|slice| integer(&slice["heap_delta_bytes"]))
                            .sum::<i128>()
                    );
                    for slice in pass["slices"].as_array().unwrap() {
                        slices += 1;
                        let used = integer(&slice["guest_instructions"]);
                        // A slice can cross its nominal counter at a metering
                        // block boundary (this short loop reports +1). Keep
                        // that observed debit rather than clamping it to 2B.
                        assert!(
                            used <= slice_limit + 64,
                            "whole-message total leaked into slice: {slice}"
                        );
                        guest += used;
                        assert_eq!(
                            integer(&slice["execution_round_instructions"]),
                            used + integer(&slice["dirty_page_copy_instructions"])
                        );
                        if let Some(total) = slice["completed_message_instructions"].as_str() {
                            completed.push(total.parse::<i128>().unwrap());
                        }
                    }
                }
            }
        }
        assert_eq!(global, integer(&row["global_budget_consumed"]));
        assert_eq!(observed_heap_delta, round_heap_delta);
        assert_eq!(row["heap_delta_limit_reached"], false);
    }
    assert!(slices >= 2);
    assert!(
        reserve_observed,
        "the final slice can legally use reserved headroom"
    );
    assert_eq!(completed.len(), 1, "one logical ingress completed");
    assert!(completed[0] > 2_000_000_000);
    assert_eq!(
        guest, completed[0],
        "DTS guest work must reconcile across rounds"
    );
    assert!(pic.get_memory_access_trace(canister).executions.is_empty());
    let svg = pic.render_system_profile(id, result["query_id"].as_str().unwrap(), "rounds", "svg");
    assert!(
        String::from_utf8(svg)
            .unwrap()
            .contains(result["query_id"].as_str().unwrap())
    );
}

#[test]
fn platform_client_boundary_completes_operations_and_preserves_trace_guards() {
    let _capture = exclusive_capture();
    let pic = builder().with_application_subnet().build();
    let canister = install(&pic, pic.topology().get_app_subnets()[0], WORK_WAT);
    let base = pic
        .get_server_url()
        .join(&format!(
            "api/profiling/instances/{}/canisters/{}/",
            pic.instance_id(),
            canister
        ))
        .unwrap();
    let call = |action: &str, body: Value| {
        client()
            .post(base.join(action).unwrap())
            .json(&body)
            .send()
            .unwrap()
    };
    assert_eq!(
        client()
            .post(base.join("start").unwrap())
            .header("Origin", "http://unrelated.invalid")
            .json(&json!({}))
            .send()
            .unwrap()
            .status()
            .as_u16(),
        403
    );
    // Zero synchronous wait forces operation-graph completion into Platform.
    call(
        "profile",
        json!({"max_executions":20,"read_mask":true,"write_mask":true}),
    )
    .error_for_status()
    .unwrap();
    pic.update_call(canister, Principal::anonymous(), "work", vec![])
        .unwrap();
    let snapshot: Value = call("trace", json!({}))
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    let native = serde_json::to_value(pic.get_memory_access_trace(canister)).unwrap();
    assert_eq!(snapshot, native);
    assert!(!snapshot["executions"].as_array().unwrap().is_empty());
    let image: Value = call("snapshot", json!({"label":"client-boundary"}))
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert!(image["snapshot_id"].is_string());
    assert_eq!(image["label"], "client-boundary");
    let mut guard = json!({"expected_server_generation":snapshot["server_generation"], "expected_trace_generation":snapshot["trace_generation"], "expected_last_sequence_number":snapshot["executions"].as_array().unwrap().last().unwrap()["sequence_number"], "expected_last_runtime_cost_sequence_number":snapshot["last_runtime_cost_sequence_number"]});
    guard["expected_server_generation"] = json!("different-server");
    assert_eq!(call("clear", guard.clone()).status().as_u16(), 409);
    let retained: Value = call("trace", json!({}))
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(retained["executions"], snapshot["executions"]);
    // Exercise the existing Python command against this actual native server,
    // including its durable export followed by independently guarded clear.
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("trace.json");
    let script = std::env::var_os("PROFILER_PYTHON_CLIENT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../rs/pocket_ic_server/profiling/viewer/memory_access_viewer.py"
            ))
        });
    let python = std::process::Command::new("python3")
        .arg(script)
        .args([
            "fetch",
            "--server",
            pic.get_server_url().as_str(),
            "--instance",
            &pic.instance_id().to_string(),
            "--canister",
            &canister.to_text(),
            "--clear",
            "-o",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        python.status.success(),
        "Python live client failed: {}",
        String::from_utf8_lossy(&python.stderr)
    );
    let exported: Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(exported["server_generation"], snapshot["server_generation"]);
    assert_eq!(
        exported["executions"].as_array().unwrap().len(),
        snapshot["executions"].as_array().unwrap().len()
    );
    let cleared: Value = call("trace", json!({}))
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert!(cleared["executions"].as_array().unwrap().is_empty());
    call("stop", json!({})).error_for_status().unwrap();
    assert_eq!(
        call("profile", json!({"max_executions":0}))
            .status()
            .as_u16(),
        400
    );
    let bad = pic
        .get_server_url()
        .join(&format!(
            "api/profiling/instances/{}/canisters/invalid-principal/trace",
            pic.instance_id()
        ))
        .unwrap();
    assert_eq!(
        client()
            .post(bad)
            .json(&json!({}))
            .send()
            .unwrap()
            .status()
            .as_u16(),
        400
    );
    let id = begin(&pic);
    let result = finish(&pic, &id);
    assert_eq!(
        result["scope"]["metadata"]["backend_description"]["availability"],
        "available"
    );
    let dictionary = view(&pic, &id, "explain");
    assert_eq!(
        dictionary["dictionary"]["backend_description"]["value_kind"],
        "compiled_source_description"
    );
}
