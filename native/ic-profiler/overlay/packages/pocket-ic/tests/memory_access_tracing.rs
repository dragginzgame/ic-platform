use candid::Principal;
use pocket_ic::PocketIc;
use pocket_ic::common::rest::{
    MemoryAccessTrace, RawClearMemoryAccessTrace, RuntimeCostEvent, RuntimeCostTotal,
    TracedExecutionOutcome,
};
use reqwest::blocking::Client;
use reqwest::header::{HOST, ORIGIN};
use serde_json::{Value, json};
use std::collections::BTreeMap;

// 2T cycles
const INIT_CYCLES: u128 = 2_000_000_000_000;

/// A Wasm64 canister with methods that access specific Wasm pages (64 KiB):
/// - `touch` and `peek` copy their arguments (two little-endian `u64` page
///   indices) to page 0, read the first page and write the second one;
/// - `call_self` calls `touch` on the canister itself (reading page 12 and
///   writing page 13) and its reply callback writes page 20;
/// - `set_timer` sets the global timer, which writes page 30;
/// - `trap` writes page 40 and traps.
const MEMORY_WAT: &str = r#"
(module
  (import "ic0" "msg_arg_data_copy" (func $msg_arg_data_copy (param i64 i64 i64)))
  (import "ic0" "msg_reply" (func $msg_reply))
  (import "ic0" "canister_self_size" (func $canister_self_size (result i64)))
  (import "ic0" "canister_self_copy" (func $canister_self_copy (param i64 i64 i64)))
  (import "ic0" "call_new" (func $call_new (param i64 i64 i64 i64 i64 i64 i64 i64)))
  (import "ic0" "call_data_append" (func $call_data_append (param i64 i64)))
  (import "ic0" "call_perform" (func $call_perform (result i32)))
  (import "ic0" "global_timer_set" (func $global_timer_set (param i64) (result i64)))
  (memory (export "memory") i64 64)
  (table 2 funcref)
  (elem (i32.const 0) func $on_reply $on_reject)
  (data (i64.const 1024) "touch")
  (data (i64.const 1040) "\0c\00\00\00\00\00\00\00\0d\00\00\00\00\00\00\00")

  (func $touch
    (call $msg_arg_data_copy (i64.const 0) (i64.const 0) (i64.const 16))
    (i64.store
      (i64.mul (i64.load (i64.const 8)) (i64.const 65536))
      (i64.load (i64.mul (i64.load (i64.const 0)) (i64.const 65536))))
    (call $msg_reply))

  (func $call_self
    (call $canister_self_copy (i64.const 2048) (i64.const 0) (call $canister_self_size))
    (call $call_new
      (i64.const 2048) (call $canister_self_size)
      (i64.const 1024) (i64.const 5)
      (i64.const 0) (i64.const 0)
      (i64.const 1) (i64.const 0))
    (call $call_data_append (i64.const 1040) (i64.const 16))
    (drop (call $call_perform)))

  (func $on_reply (param i64)
    (i64.store (i64.const 1310720) (i64.const 1))
    (call $msg_reply))

  (func $on_reject (param i64)
    (call $msg_reply))

  (func $set_timer
    (drop (call $global_timer_set (i64.const 1)))
    (call $msg_reply))

  (func $timer
    (i64.store (i64.const 1966080) (i64.const 1)))

  (func $trap
    (i64.store (i64.const 2621440) (i64.const 1))
    unreachable)

  (export "canister_update touch" (func $touch))
  (export "canister_query peek" (func $touch))
  (export "canister_update call_self" (func $call_self))
  (export "canister_update set_timer" (func $set_timer))
  (export "canister_update trap" (func $trap))
  (export "canister_global_timer" (func $timer))
)
"#;

fn touch_args(read_page: u64, write_page: u64) -> Vec<u8> {
    [read_page.to_le_bytes(), write_page.to_le_bytes()].concat()
}

fn install_memory_canister(pic: &PocketIc) -> Principal {
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.install_canister(
        canister_id,
        wat::parse_str(MEMORY_WAT).unwrap(),
        vec![],
        None,
    );
    canister_id
}

#[test]
fn memory_access_tracing() {
    let pic = PocketIc::new();
    let canister_id = install_memory_canister(&pic);
    let sender = Principal::anonymous();

    // Nothing is recorded before tracing is started.
    pic.update_call(canister_id, sender, "touch", touch_args(1, 2))
        .unwrap();
    assert!(
        pic.get_memory_access_trace(canister_id)
            .executions
            .is_empty()
    );

    pic.start_memory_access_tracing(canister_id);
    pic.update_call(canister_id, sender, "touch", touch_args(5, 9))
        .unwrap();
    pic.query_call(canister_id, sender, "peek", touch_args(5, 9))
        .unwrap();
    pic.update_call(canister_id, sender, "call_self", vec![])
        .unwrap();
    pic.update_call(canister_id, sender, "set_timer", vec![])
        .unwrap();
    pic.tick();
    pic.tick();
    pic.update_call(canister_id, sender, "trap", vec![])
        .unwrap_err();

    let trace = pic.get_memory_access_trace(canister_id);
    assert_eq!(trace.wasm_page_size, 65536);
    let e = &trace.executions;
    assert_eq!(
        e.iter()
            .map(|e| (e.api_type.as_str(), e.function.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("update", "canister_update touch"),
            ("non replicated query", "canister_query peek"),
            ("update", "canister_update call_self"),
            ("update", "canister_update touch"),
            ("reply callback", "callback (table index 0)"),
            ("update", "canister_update set_timer"),
            ("global timer", "canister_global_timer"),
            ("update", "canister_update trap"),
        ]
    );
    assert!(
        e.windows(2)
            .all(|w| w[0].sequence_number < w[1].sequence_number)
    );

    // The arguments are copied to page 0, page 5 is read, and page 9 is written.
    assert_eq!(e[0].accessed_pages, vec![(0, 1), (5, 6), (9, 10)]);
    assert_eq!(e[0].dirty_pages, vec![(0, 1), (9, 10)]);
    assert!(e[0].dirty_pages_tracked);
    assert_eq!(e[0].outcome, TracedExecutionOutcome::Reply);
    assert_eq!(e[0].message, None);
    assert_eq!(e[0].wasm_memory_size, 64);
    assert!(e[0].time_nanos > 0);
    // Every accessed and every written page is charged for each of its 16 OS
    // pages (with the default page overhead of 5000 instructions).
    assert_eq!(e[0].page_access_instructions, (3 + 2) * 16 * 5_000);
    assert!(e[0].instructions > e[0].page_access_instructions);

    // The writes of a query are not tracked.
    assert_eq!(e[1].accessed_pages, vec![(0, 1), (5, 6), (9, 10)]);
    assert!(e[1].dirty_pages.is_empty());
    assert!(!e[1].dirty_pages_tracked);

    // The self-call is a separate call, but its reply callback belongs to the
    // call context of `call_self`.
    assert_eq!(e[2].accessed_pages, vec![(0, 1)]);
    assert_eq!(e[3].accessed_pages, vec![(0, 1), (12, 14)]);
    assert_eq!(e[3].dirty_pages, vec![(0, 1), (13, 14)]);
    assert_eq!(e[4].accessed_pages, vec![(20, 21)]);
    assert_eq!(e[4].dirty_pages, vec![(20, 21)]);
    assert!(e[2].call_context_id.is_some());
    assert_eq!(e[4].call_context_id, e[2].call_context_id);
    assert_ne!(e[3].call_context_id, e[2].call_context_id);

    // The timer writes page 30.
    assert_eq!(e[6].dirty_pages, vec![(30, 31)]);
    assert_eq!(e[6].outcome, TracedExecutionOutcome::NoResponse);

    // The pages accessed by an execution that traps are recorded, too.
    assert_eq!(e[7].dirty_pages, vec![(40, 41)]);
    assert_eq!(e[7].outcome, TracedExecutionOutcome::Error);
    assert!(e[7].message.as_ref().unwrap().contains("trapped"));

    // Stopping keeps the recorded executions but records no new ones.
    pic.stop_memory_access_tracing(canister_id);
    pic.update_call(canister_id, sender, "touch", touch_args(1, 2))
        .unwrap();
    let mut expected_stopped_trace = trace;
    assert!(expected_stopped_trace.runtime_costs_recording);
    expected_stopped_trace.runtime_costs_recording = false;
    assert_eq!(
        pic.get_memory_access_trace(canister_id),
        expected_stopped_trace
    );

    // Clearing deletes the recorded executions.
    pic.clear_memory_access_trace(canister_id);
    assert!(
        pic.get_memory_access_trace(canister_id)
            .executions
            .is_empty()
    );
}

#[test]
fn memory_access_tracing_is_per_canister() {
    let pic = PocketIc::new();
    let traced = install_memory_canister(&pic);
    let other = install_memory_canister(&pic);
    let sender = Principal::anonymous();

    pic.start_memory_access_tracing(traced);
    pic.update_call(traced, sender, "touch", touch_args(3, 4))
        .unwrap();
    pic.update_call(other, sender, "touch", touch_args(3, 4))
        .unwrap();

    assert_eq!(pic.get_memory_access_trace(traced).executions.len(), 1);
    assert!(pic.get_memory_access_trace(other).executions.is_empty());
}

/// Compare actual charged work and persisted state with observation on and off.
/// The performance counter is read in the guest, so a collector accidentally
/// charging its callbacks changes the reply as well as the cycle balance.
#[test]
fn memory_access_tracing_preserves_execution_and_rollback() {
    assert_observation_preserves_execution_and_rollback(false);
}

#[test]
fn instruction_profiling_preserves_execution_and_rollback() {
    assert_observation_preserves_execution_and_rollback(true);
}

#[test]
fn instruction_profiling_window_and_upgrade_identity() {
    let pic = PocketIc::new();
    let canister = pic.create_canister();
    pic.add_cycles(canister, INIT_CYCLES);
    let module = |value| {
        wat::parse_str(format!(
            r#"(module
              (import "ic0" "msg_reply" (func $reply))
              (memory (export "memory") i64 1)
              (func (export "canister_update touch")
                (i64.store (i64.const 0) (i64.const {value}))
                (call $reply)))"#
        ))
        .unwrap()
    };
    let installed_hash = || {
        pic.canister_status(canister, None)
            .unwrap()
            .module_hash
            .unwrap()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let touch = || {
        pic.update_call(canister, Principal::anonymous(), "touch", vec![])
            .unwrap();
    };

    pic.install_canister(canister, module(1), vec![], None);
    let original_hash = installed_hash();
    pic.start_instruction_profiling(canister, 1);
    touch();
    touch();

    pic.upgrade_canister(canister, module(2), vec![], None)
        .unwrap();
    let upgraded_hash = installed_hash();
    assert_ne!(original_hash, upgraded_hash);
    pic.start_instruction_profiling(canister, 1);
    touch();

    let trace = pic.get_memory_access_trace(canister);
    let executions = trace
        .executions
        .iter()
        .filter(|execution| execution.function == "canister_update touch")
        .collect::<Vec<_>>();
    assert_eq!(executions.len(), 3);
    assert!(executions[0].instruction_profile.is_some());
    assert!(executions[1].instruction_profile.is_none());
    assert!(executions[2].instruction_profile.is_some());
    assert_eq!(executions[0].module_hash.as_ref(), Some(&original_hash));
    assert_eq!(executions[1].module_hash.as_ref(), Some(&original_hash));
    assert_eq!(executions[2].module_hash.as_ref(), Some(&upgraded_hash));
    assert_eq!(
        executions[2].module_hash_kind.as_deref(),
        Some("installed_uncompressed_sha256")
    );
}

#[test]
#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
fn instruction_profile_page_origins_follow_returns() {
    let pic = PocketIc::new();
    let canister = pic.create_canister();
    pic.add_cycles(canister, INIT_CYCLES);
    pic.install_canister(
        canister,
        wat::parse_str(
            r#"(module
          (import "ic0" "msg_reply" (func $reply))
          (memory (export "memory") i64 2)
          (func $callee
            (i64.store (i64.const 0) (i64.const 1)))
          (func (export "canister_update touch")
            (call $callee)
            (i64.store (i64.const 65536) (i64.const 2))
            (call $reply)))"#,
        )
        .unwrap(),
        vec![],
        None,
    );
    pic.start_instruction_profiling(canister, 1);
    pic.update_call(canister, Principal::anonymous(), "touch", vec![])
        .unwrap();
    let trace = pic.get_memory_access_trace(canister);
    let profile = trace.executions[0].instruction_profile.as_ref().unwrap();
    assert!(!profile.page_details_truncated);
    assert_eq!(profile.page_charges.len(), 4);
    assert_eq!(
        profile
            .page_charges
            .iter()
            .map(|charge| charge.instructions)
            .sum::<u64>(),
        profile.ledger.heap_page_instructions
    );
    let execution = &trace.executions[0];
    assert_eq!(
        execution.module_hash_kind.as_deref(),
        Some("installed_uncompressed_sha256")
    );
    let metadata = execution
        .module_hash
        .as_ref()
        .and_then(|hash| trace.modules.get(hash))
        .or(profile.module.as_ref())
        .expect("exact installed-module origin dictionary");
    assert_eq!(
        metadata
            .module_sha256
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        *execution.module_hash.as_ref().unwrap()
    );
    let function_for_page = |region| {
        let charge = profile
            .page_charges
            .iter()
            .find(|charge| charge.region_index == region && charge.kind == "dirty")
            .unwrap();
        assert_eq!(charge.memory, "heap");
        assert_eq!(charge.origin_evidence, "native_fault_pc");
        assert!(charge.native_pc.is_some());
        assert!(charge.module_offset.is_some());
        metadata
            .operators
            .iter()
            .find(|operator| Some(operator.origin_id) == charge.origin_id)
            .unwrap()
            .function_index
    };
    // A callee's last observed block must not be reused for the caller's store.
    assert_ne!(function_for_page(0), function_for_page(1));
}

fn assert_observation_preserves_execution_and_rollback(profile: bool) {
    const PARITY_WAT: &str = r#"
    (module
      (import "ic0" "msg_reply_data_append" (func $append (param i64 i64)))
      (import "ic0" "msg_reply" (func $reply))
      (import "ic0" "performance_counter" (func $counter (param i32) (result i64)))
      (import "ic0" "stable64_grow" (func $stable_grow (param i64) (result i64)))
      (import "ic0" "stable64_read" (func $stable_read (param i64 i64 i64)))
      (import "ic0" "stable64_write" (func $stable_write (param i64 i64 i64)))
      (memory (export "memory") i64 4)

      (func (export "canister_update exercise")
        (drop (call $stable_grow (i64.const 1)))
        (i64.store (i64.const 65536) (i64.const 42))
        (call $stable_write (i64.const 0) (i64.const 65536) (i64.const 8))
        (call $stable_read (i64.const 131072) (i64.const 0) (i64.const 8))
        (memory.fill (i64.const 196608) (i32.const 7) (i64.const 17))
        (memory.copy (i64.const 196640) (i64.const 196608) (i64.const 17))
        (drop (memory.grow (i64.const 1)))
        (memory.fill (i64.const 262144) (i32.const 1) (i64.const 0))
        (memory.copy (i64.const 262144) (i64.const 0) (i64.const 0))
        (call $stable_write (i64.const 0) (i64.const 0) (i64.const 0))
        (call $stable_read (i64.const 0) (i64.const 0) (i64.const 0))
        (i64.store (i64.const 0) (call $counter (i32.const 0)))
        (call $append (i64.const 0) (i64.const 8))
        (call $reply))

      (func (export "canister_query state")
        (i64.store (i64.const 0) (i64.load (i64.const 65536)))
        (i64.store (i64.const 8) (i64.load (i64.const 131072)))
        (i64.store (i64.const 16) (i64.load (i64.const 196608)))
        (i64.store (i64.const 24) (i64.load (i64.const 196640)))
        (i64.store (i64.const 32) (memory.size))
        (call $append (i64.const 0) (i64.const 40))
        (call $reply))

      (func (export "canister_update fail")
        (i64.store (i64.const 65536) (i64.const 99))
        (call $stable_write (i64.const 0) (i64.const 65536) (i64.const 8))
        unreachable))
    "#;

    let sender = Principal::anonymous();
    let baseline = PocketIc::new();
    let observed = PocketIc::new();
    let install = |pic: &PocketIc| {
        let canister = pic.create_canister();
        pic.add_cycles(canister, INIT_CYCLES);
        pic.install_canister(canister, wat::parse_str(PARITY_WAT).unwrap(), vec![], None);
        canister
    };
    let baseline_canister = install(&baseline);
    let observed_canister = install(&observed);
    observed.start_memory_access_tracing(observed_canister);
    if profile {
        observed.start_instruction_profiling(observed_canister, 20);
    }

    let exercise = |pic: &PocketIc, canister| {
        pic.update_call(canister, sender, "exercise", vec![])
            .unwrap()
    };
    let baseline_counter = exercise(&baseline, baseline_canister);
    assert_eq!(exercise(&observed, observed_canister), baseline_counter);
    assert_eq!(baseline_counter.len(), 8);
    assert!(u64::from_le_bytes(baseline_counter.try_into().unwrap()) > 0);
    assert_eq!(
        observed.cycle_balance(observed_canister),
        baseline.cycle_balance(baseline_canister)
    );

    let state = |pic: &PocketIc, canister, nonce| {
        pic.query_call(canister, sender, "state", vec![nonce])
            .unwrap()
    };
    let heap_before_trap = state(&baseline, baseline_canister, 0);
    assert_eq!(state(&observed, observed_canister, 0), heap_before_trap);
    let stable_before_trap = baseline.get_stable_memory(baseline_canister);
    assert_eq!(
        observed.get_stable_memory(observed_canister),
        stable_before_trap
    );
    assert_eq!(&stable_before_trap[..8], &42_u64.to_le_bytes());
    assert_eq!(
        u64::from_le_bytes(heap_before_trap[32..40].try_into().unwrap()),
        5
    );

    let fail = |pic: &PocketIc, canister| {
        pic.update_call(canister, sender, "fail", vec![])
            .unwrap_err()
    };
    assert_eq!(
        fail(&observed, observed_canister),
        fail(&baseline, baseline_canister)
    );
    assert_eq!(
        observed.cycle_balance(observed_canister),
        baseline.cycle_balance(baseline_canister)
    );
    // Both heap and stable writes in a failed update must be rolled back.
    assert_eq!(state(&baseline, baseline_canister, 1), heap_before_trap);
    assert_eq!(state(&observed, observed_canister, 1), heap_before_trap);
    assert_eq!(
        baseline.get_stable_memory(baseline_canister),
        stable_before_trap
    );
    assert_eq!(
        observed.get_stable_memory(observed_canister),
        stable_before_trap
    );

    let trace = observed.get_memory_access_trace(observed_canister);
    let exercise = &trace.executions[0];
    assert_eq!(exercise.function, "canister_update exercise");
    assert_eq!(exercise.outcome, TracedExecutionOutcome::Reply);
    assert_eq!(exercise.wasm_memory_size, 5);
    assert_eq!(exercise.accessed_pages, vec![(0, 4)]);
    assert_eq!(exercise.dirty_pages, vec![(0, 4)]);
    if profile {
        let profile = exercise.instruction_profile.as_ref().unwrap();
        assert!(!profile.truncated);
        assert!(!profile.blocks.is_empty());
        assert!(profile.ledger.static_instructions > 0);
        assert!(profile.ledger.dynamic_instructions > 0);
        assert!(profile.ledger.api_instructions > 0);
        assert!(profile.ledger.stable_api_instructions > 0);
        assert_eq!(profile.ledger.reported_instructions, exercise.instructions);
        assert_eq!(
            profile.ledger.heap_page_instructions,
            exercise.page_access_instructions
        );
        assert_eq!(profile.ledger.unexplained_instructions, 0);
        assert!(profile.ledger.reconciled, "{:?}", profile.ledger);
        assert_eq!(
            profile
                .blocks
                .iter()
                .map(|block| block.instructions)
                .sum::<u64>(),
            profile.ledger.static_instructions
        );
    } else {
        assert!(exercise.instruction_profile.is_none());
    }
    let failed = trace
        .executions
        .iter()
        .find(|execution| execution.function == "canister_update fail")
        .unwrap();
    assert_eq!(failed.outcome, TracedExecutionOutcome::Error);
    assert_eq!(failed.dirty_pages, vec![(1, 2)]);
    assert!(
        baseline
            .get_memory_access_trace(baseline_canister)
            .executions
            .is_empty()
    );
}

#[test]
#[should_panic(expected = "CanisterNotFound")]
fn memory_access_tracing_of_unknown_canister_fails() {
    let pic = PocketIc::new();
    pic.start_memory_access_tracing(Principal::from_slice(&[0xff; 10]));
}

#[test]
fn memory_access_viewer() {
    let pic = PocketIc::new();
    let canister_id = install_memory_canister(&pic);
    let sender = Principal::anonymous();
    let viewer = pic.start_memory_access_viewer(None);
    // The viewer is served once per PocketIC server.
    assert_eq!(pic.start_memory_access_viewer(None), viewer);

    let client = Client::new();
    let url = |path: &str| viewer.join(path).unwrap();
    let get = |path: &str| -> Value {
        let response = client.get(url(path)).send().unwrap();
        assert!(response.status().is_success(), "GET {path}: {response:?}");
        response.json().unwrap()
    };
    let post = |path: &str| client.post(url(path)).json(&json!({})).send().unwrap();
    let canister = format!(
        "api/instances/{}/canisters/{canister_id}",
        pic.instance_id()
    );

    // The web app loads the traces from the API, which is described for agents.
    let page = client.get(viewer.clone()).send().unwrap().text().unwrap();
    let configuration = page
        .split_once(r#"<script type="application/json" id="trace-data">"#)
        .unwrap()
        .1
        .split_once("</script>")
        .unwrap()
        .0;
    let configuration: Value = serde_json::from_str(configuration).unwrap();
    assert_eq!(configuration["live"]["api"], "api");
    assert_eq!(configuration["live"]["system_profiling"], true);
    let guide = client.get(url("llms.txt")).send().unwrap().text().unwrap();
    assert!(guide.contains(&format!("{viewer}mcp")));

    // Tracing can be started in the viewer.
    assert_eq!(post(&format!("{canister}/start")).status(), 200);
    pic.update_call(canister_id, sender, "touch", touch_args(5, 9))
        .unwrap();
    pic.update_call(canister_id, sender, "call_self", vec![])
        .unwrap();

    let traces = get("api/traces");
    assert_eq!(traces["format_version"], 2);
    assert_eq!(traces["integer_encoding"], "decimal");
    assert!(traces["server_generation"].as_str().unwrap().len() > 20);
    let instance = traces["instances"]
        .as_array()
        .unwrap()
        .iter()
        .find(|instance| instance["instance_id"] == pic.instance_id())
        .unwrap();
    assert_eq!(instance["deleted"], false);
    let traced = &instance["canisters"][0];
    assert_eq!(traced["canister_id"], canister_id.to_text());
    assert_eq!(traced["tracing"], true);
    assert_eq!(traced["executions"], "4");

    // Downloads continue after the last execution seen.
    let first = get(&format!("{canister}/executions?limit=1"));
    assert_eq!(first["more"], true);
    let first_sequence_number = first["executions"][0]["sequence_number"].as_str().unwrap();
    assert_eq!(first["first_sequence_number"], first_sequence_number);
    let rest = get(&format!(
        "{canister}/executions?after={first_sequence_number}"
    ));
    assert_eq!(rest["more"], false);
    assert_eq!(rest["executions"].as_array().unwrap().len(), 3);
    let touches = get(&format!("{canister}/executions?function=touch"));
    assert_eq!(touches["executions"].as_array().unwrap().len(), 2);

    // The summary attributes instructions to functions and to the heap.
    let summary = get(&format!("{canister}/summary?top=5"));
    assert_eq!(summary["executions"], "4");
    assert_eq!(summary["instructions"]["per_page_access"], 16.0 * 5_000.0);
    let functions: Vec<_> = summary["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|function| function["function"].as_str().unwrap())
        .collect();
    assert_eq!(functions.len(), 3);
    assert!(functions.contains(&"callback (table index 0)"));
    // Attempt-aware labels preserve the actual callback function. Its runtime
    // context supplies correlations without inferring a caller from proximity.
    let callback = rest["executions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|execution| execution["api_type"] == "reply callback")
        .unwrap();
    assert_eq!(callback["function"], "callback (table index 0)");
    assert_eq!(callback["attempt"]["status"], "completed");
    assert!(callback["attempt"]["attempt_id"].as_str().is_some());
    assert_eq!(callback["disposition"]["state"], "committed");
    let logical_id = callback["disposition"]["logical_execution_id"]
        .as_str()
        .unwrap();
    let context = &rest["logical_executions"][logical_id];
    assert_eq!(context["kind"], "response");
    assert!(context["callback_id"].as_str().is_some());
    assert_eq!(
        context["parent_call_context_id"],
        callback["call_context_id"]
    );
    assert_eq!(summary["logical_executions"][logical_id], *context);
    // Page 0 holds the arguments of all calls (but not of the callback).
    assert_eq!(summary["hot_pages"][0]["first_page"], "0");
    assert_eq!(summary["hot_pages"][0]["accessed_by"], "3");
    assert_eq!(
        summary["urls"]["viewer"],
        format!("{viewer}#{}:{canister_id}", pic.instance_id())
    );

    // The history of page 13, which is written by the self-call.
    let history = get(&format!("{canister}/pages/13"));
    assert_eq!(history["accessed_by"], "1");
    assert_eq!(history["written_by"], "1");
    assert_eq!(
        history["executions"][0]["function"],
        "canister_update touch"
    );
    let history_page = get(&format!("{canister}/pages/0?limit=1"));
    assert_eq!(history_page["more"], true);
    let after = history_page["next_after"].as_str().unwrap();
    let history_rest = get(&format!("{canister}/pages/0?after={after}"));
    assert_eq!(history_rest["more"], false);
    assert_eq!(history_rest["executions"].as_array().unwrap().len(), 2);

    // The MCP server offers the same.
    let mcp = |message: Value| -> Value {
        client
            .post(url("mcp"))
            .header("Accept", "application/json, text/event-stream")
            .json(&message)
            .send()
            .unwrap()
            .json()
            .unwrap()
    };
    let initialize = mcp(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" },
        },
    }));
    assert_eq!(initialize["id"], 1);
    assert_eq!(initialize["result"]["protocolVersion"], "2025-06-18");
    assert!(initialize["result"]["capabilities"]["tools"].is_object());
    let notification = client
        .post(url("mcp"))
        .json(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }))
        .send()
        .unwrap();
    assert_eq!(notification.status(), 202);
    let tools = mcp(json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }));
    let tools: Vec<_> = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap().to_string())
        .collect();
    assert!(tools.contains(&"get_trace_summary".to_string()));
    let call = |id: u64, name: &str, arguments: Value| -> Value {
        let response = mcp(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments },
        }));
        response["result"].clone()
    };
    let arguments =
        json!({ "instance_id": pic.instance_id(), "canister_id": canister_id.to_text() });
    let result = call(3, "get_trace_summary", arguments.clone());
    assert_eq!(result["isError"], false);
    let summary: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(summary["executions"], "4");
    let result = call(
        4,
        "get_executions",
        json!({ "instance_id": 999, "canister_id": canister_id.to_text() }),
    );
    assert_eq!(result["isError"], true);
    let unknown = mcp(json!({ "jsonrpc": "2.0", "id": 5, "method": "unknown" }));
    assert_eq!(unknown["error"]["code"], -32601);
    let empty_batch = mcp(json!([]));
    assert_eq!(empty_batch["error"]["code"], -32600);
    let malformed = mcp(json!({ "method": "ping", "id": 1 }));
    assert_eq!(malformed["error"]["code"], -32600);

    // Reads and writes reject other websites and rebound host names.
    let rebound_read = client
        .get(url("api/traces"))
        .header(HOST, "example.com")
        .send()
        .unwrap();
    assert_eq!(rebound_read.status(), 403);
    let foreign_read = client
        .get(url("api/traces"))
        .header(ORIGIN, "http://example.com")
        .send()
        .unwrap();
    assert_eq!(foreign_read.status(), 403);
    let stop = url(&format!("{canister}/stop"));
    let from_website = client
        .post(stop.clone())
        .header(ORIGIN, "http://example.com")
        .json(&json!({}))
        .send()
        .unwrap();
    assert_eq!(from_website.status(), 403);
    let rebound = client
        .post(stop.clone())
        .header(HOST, "example.com")
        .json(&json!({}))
        .send()
        .unwrap();
    assert_eq!(rebound.status(), 403);
    assert!(get("api/traces").to_string().contains(r#""tracing":true"#));
    // The web app itself sends its origin.
    let from_viewer = client
        .post(stop)
        .header(ORIGIN, viewer.origin().ascii_serialization())
        .json(&json!({}))
        .send()
        .unwrap();
    assert_eq!(from_viewer.status(), 200);
    pic.update_call(canister_id, sender, "touch", touch_args(1, 2))
        .unwrap();
    assert_eq!(pic.get_memory_access_trace(canister_id).executions.len(), 4);

    // Errors.
    let unknown_instance = client
        .get(url(&format!(
            "api/instances/999/canisters/{canister_id}/summary"
        )))
        .send()
        .unwrap();
    assert_eq!(unknown_instance.status(), 404);
    let invalid_canister = client
        .get(url(&format!(
            "api/instances/{}/canisters/invalid/summary",
            pic.instance_id()
        )))
        .send()
        .unwrap();
    assert_eq!(invalid_canister.status(), 400);

    // The traces of deleted instances can still be viewed and cleared.
    pic.start_memory_access_tracing(canister_id);
    let instance_id = pic.instance_id();
    drop(pic);
    let traces = get("api/traces");
    let instance = traces["instances"]
        .as_array()
        .unwrap()
        .iter()
        .find(|instance| instance["instance_id"] == instance_id)
        .unwrap();
    assert_eq!(instance["deleted"], true);
    assert_eq!(instance["canisters"][0]["executions"], "4");
    assert_eq!(instance["canisters"][0]["tracing"], false);
    assert_eq!(post(&format!("{canister}/start")).status(), 400);
    assert_eq!(post(&format!("{canister}/clear")).status(), 200);
    assert_eq!(
        get(&format!("{canister}/executions"))["executions"],
        json!([])
    );
}

#[test]
fn memory_access_viewer_profiles_filters_and_conditional_clear() {
    let pic = PocketIc::new();
    let canister_id = install_memory_canister(&pic);
    let viewer = pic.start_memory_access_viewer(None);
    let client = Client::new();
    let base = format!(
        "api/instances/{}/canisters/{canister_id}",
        pic.instance_id()
    );
    let get = |suffix: &str| -> Value {
        let response = client
            .get(viewer.join(&format!("{base}/{suffix}")).unwrap())
            .send()
            .unwrap();
        assert!(response.status().is_success(), "{response:?}");
        response.json().unwrap()
    };
    let post = |suffix: &str, body: Value| {
        client
            .post(viewer.join(&format!("{base}/{suffix}")).unwrap())
            .json(&body)
            .send()
            .unwrap()
    };
    assert_eq!(
        post("profile", json!({ "max_executions": 2 })).status(),
        200
    );
    for page in [5, 6] {
        pic.update_call(
            canister_id,
            Principal::anonymous(),
            "touch",
            touch_args(page, page + 1),
        )
        .unwrap();
    }
    pic.update_call(canister_id, Principal::anonymous(), "trap", vec![])
        .unwrap_err();
    let profile = get("profiles?top=5");
    assert_eq!(profile["profiled_executions"], "2");
    assert_eq!(profile["unprofiled_executions"], "1");
    assert_eq!(profile["ledger"]["reconciled"], true);
    assert_eq!(profile["rules_cover_raw_instructions"], true);
    assert_eq!(profile["raw_minus_rule_instructions"], "0");
    assert!(!profile["rules"].as_array().unwrap().is_empty());
    let definitions: Value = client
        .get(viewer.join("api/charge-rules").unwrap())
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(
        definitions["server_generation"],
        profile["server_generation"]
    );
    for rule in profile["rules"].as_array().unwrap() {
        let rule_id = rule["rule_id"].as_str().unwrap();
        let definition = &definitions["charge_rules"][rule_id];
        assert_eq!(definition["id"], rule_id);
        assert!(!definition["tip"].as_str().unwrap().is_empty());
        assert!(!definition["condition"].as_str().unwrap().is_empty());
        let source = &definition["source"][0];
        assert!(!source["code"].as_str().unwrap().is_empty());
        assert_eq!(source["sha256"].as_str().unwrap().len(), 64);
    }
    assert!(!profile["functions"].as_array().unwrap().is_empty());
    assert_eq!(profile["source_coverage"], Value::Null);
    let snapshot = get("executions");
    assert_eq!(snapshot["dropped_instructions"], "0");
    let executions = snapshot["executions"].as_array().unwrap();
    assert_eq!(executions.len(), 3);
    assert!(executions[0]["instruction_profile"].is_object());
    assert!(executions[2].get("instruction_profile").is_none());
    let last = executions[2]["sequence_number"].as_str().unwrap();
    let selected = get(&format!("executions?before={last}&outcome=reply"));
    assert_eq!(selected["executions"].as_array().unwrap().len(), 2);
    assert_eq!(get("summary?outcome=error")["executions"], "1");
    assert_eq!(get("profiles?outcome=error")["profiled_executions"], "0");
    let hash = executions[0]["module_hash"].as_str().unwrap();
    assert!(snapshot["modules"][hash]["operators"].is_array());
    assert!(snapshot["modules"][hash]["configuration_sha256"][0].is_number());
    assert_eq!(
        get(&format!("summary?module_hash={hash}"))["executions"],
        "3"
    );
    assert_eq!(get("summary?module_hash=wrong")["executions"], "0");

    // A downloaded snapshot cannot clear executions recorded after it.
    let condition = json!({
        "expected_server_generation": snapshot["server_generation"],
        "trace_generation": snapshot["trace_generation"],
        "last_sequence_number": snapshot["last_sequence_number"],
    });
    pic.update_call(
        canister_id,
        Principal::anonymous(),
        "touch",
        touch_args(7, 8),
    )
    .unwrap();
    assert_eq!(post("clear", condition).status(), 409);
    assert_eq!(get("executions")["executions"].as_array().unwrap().len(), 4);
    let current = get("executions");
    assert_eq!(
        post(
            "clear",
            json!({
                "expected_server_generation": current["server_generation"],
                "trace_generation": current["trace_generation"],
                "last_sequence_number": current["last_sequence_number"],
            })
        )
        .status(),
        200
    );
    let cleared = get("executions");
    assert_ne!(cleared["trace_generation"], current["trace_generation"]);
    assert_eq!(cleared["executions"], json!([]));
    assert_eq!(
        post(
            "clear",
            json!({ "expected_server_generation": "a-previous-server" })
        )
        .status(),
        409
    );
    let invalid_page = client
        .get(
            viewer
                .join(&format!("{base}/pages/18446744073709551615"))
                .unwrap(),
        )
        .send()
        .unwrap();
    assert_eq!(invalid_page.status(), 400);
}

// Exercises real message delivery and timer dispatch, rather than synthesizing
// collector events. A query can read the state without adding an ingress bill.
const HOST_COST_WAT: &str = r#"
(module
  (import "ic0" "msg_arg_data_size" (func $size (result i64)))
  (import "ic0" "msg_arg_data_copy" (func $copy (param i64 i64 i64)))
  (import "ic0" "msg_reply_data_append" (func $append (param i64 i64)))
  (import "ic0" "msg_reply" (func $reply))
  (import "ic0" "canister_self_size" (func $self_size (result i64)))
  (import "ic0" "canister_self_copy" (func $self_copy (param i64 i64 i64)))
  (import "ic0" "call_new" (func $call_new (param i64 i64 i64 i64 i64 i64 i64 i64)))
  (import "ic0" "call_data_append" (func $call_data (param i64 i64)))
  (import "ic0" "call_perform" (func $perform (result i32)))
  (import "ic0" "global_timer_set" (func $timer_set (param i64) (result i64)))
  (memory (export "memory") i64 2)
  (table 2 funcref)
  (elem (i32.const 0) func $on_reply $on_reject)
  (data (i64.const 1000) "echo")
  (data (i64.const 1100) "transport")
  (data (i64.const 1200) "query_leaf")

  (func $echo
    (call $copy (i64.const 0) (i64.const 0) (call $size))
    (i64.store (i64.const 65536)
      (i64.add (i64.load (i64.const 65536)) (i64.const 1)))
    (call $append (i64.const 0) (call $size))
    (call $reply))
  (func $call_self
    (call $self_copy (i64.const 1024) (i64.const 0) (call $self_size))
    (call $call_new
      (i64.const 1024) (call $self_size)
      (i64.const 1000) (i64.const 4)
      (i64.const 0) (i64.const 0) (i64.const 1) (i64.const 0))
    (call $call_data (i64.const 1100) (i64.const 9))
    (drop (call $perform)))
  (func $query_self
    (call $self_copy (i64.const 1024) (i64.const 0) (call $self_size))
    (call $call_new
      (i64.const 1024) (call $self_size)
      (i64.const 1200) (i64.const 10)
      (i64.const 0) (i64.const 0) (i64.const 1) (i64.const 0))
    (call $call_data (i64.const 1100) (i64.const 9))
    (drop (call $perform)))
  (func $on_reply (param i64)
    (i64.store (i64.const 65536)
      (i64.add (i64.load (i64.const 65536)) (i64.const 2)))
    (call $copy (i64.const 0) (i64.const 0) (call $size))
    (call $append (i64.const 0) (call $size))
    (call $reply))
  (func $on_reject (param i64) (call $reply))
  (func (export "canister_update arm_timer")
    (drop (call $timer_set (i64.const 1)))
    (call $reply))
  (func (export "canister_global_timer")
    (i64.store (i64.const 65536)
      (i64.add (i64.load (i64.const 65536)) (i64.const 10))))
  (func (export "canister_query state")
    (call $append (i64.const 65536) (i64.const 8))
    (call $reply))
  (export "canister_update echo" (func $echo))
  (export "canister_query query_leaf" (func $echo))
  (export "canister_update call_self" (func $call_self))
  (export "canister_composite_query query_self" (func $query_self))
)
"#;

fn install_host_cost_canister(pic: &PocketIc) -> Principal {
    let canister = pic.create_canister();
    pic.add_cycles(canister, INIT_CYCLES);
    pic.install_canister(
        canister,
        wat::parse_str(HOST_COST_WAT).unwrap(),
        vec![],
        None,
    );
    canister
}

fn runtime_config<'a>(
    pic: &PocketIc,
    canister: Principal,
    trace: &'a MemoryAccessTrace,
) -> &'a BTreeMap<String, String> {
    &trace.runtime_configurations[&pic.get_subnet(canister).unwrap().to_text()]
}

fn config_amount(configuration: &BTreeMap<String, String>, name: &str) -> u128 {
    configuration
        .get(name)
        .unwrap_or_else(|| panic!("missing actual configuration {name}"))
        .parse()
        .unwrap()
}

fn cycle_net(events: &[RuntimeCostEvent], rule_id: Option<&str>) -> u128 {
    let (mut charged, mut refunded) = (0, 0);
    for event in events
        .iter()
        .filter(|event| event.unit == "cycles" && rule_id.is_none_or(|rule| event.rule_id == rule))
    {
        match event.kind.as_str() {
            "debit" | "prepayment" => charged += event.amount,
            "refund" => refunded += event.amount,
            kind => assert_eq!(
                event.amount, 0,
                "unexpected balance movement {kind}: {event:?}"
            ),
        }
    }
    charged
        .checked_sub(refunded)
        .expect("refund exceeds observed prepayment")
}

fn assert_runtime_totals(events: &[RuntimeCostEvent], totals: &[RuntimeCostTotal]) {
    let mut expected = BTreeMap::new();
    for event in events {
        let total = expected
            .entry((
                event.rule_id.clone(),
                event.unit.clone(),
                event.kind.clone(),
            ))
            .or_insert((0_u64, 0_u128, 0_u128));
        total.0 += 1;
        total.1 += event.amount;
        total.2 += event.nominal_cycles.unwrap_or(0);
    }
    let actual: BTreeMap<_, _> = totals
        .iter()
        .map(|total| {
            (
                (
                    total.rule_id.clone(),
                    total.unit.clone(),
                    total.kind.clone(),
                ),
                (total.events, total.amount, total.nominal_cycles),
            )
        })
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn runtime_costs_preserve_balances_replies_state_and_timer_charges() {
    let baseline = PocketIc::new();
    let observed = PocketIc::new();
    let baseline_canister = install_host_cost_canister(&baseline);
    let canister = install_host_cost_canister(&observed);
    let before = observed.cycle_balance(canister);
    assert_eq!(baseline.cycle_balance(baseline_canister), before);
    observed.start_instruction_profiling(canister, 20);
    for (method, payload, reply) in [
        ("echo", b"first payload".to_vec(), b"first payload".to_vec()),
        ("call_self", vec![], b"transport".to_vec()),
        ("arm_timer", vec![], vec![]),
    ] {
        assert_eq!(
            baseline
                .update_call(
                    baseline_canister,
                    Principal::anonymous(),
                    method,
                    payload.clone()
                )
                .unwrap(),
            reply
        );
        assert_eq!(
            observed
                .update_call(canister, Principal::anonymous(), method, payload)
                .unwrap(),
            reply
        );
        assert_eq!(
            observed.cycle_balance(canister),
            baseline.cycle_balance(baseline_canister)
        );
    }
    for _ in 0..2 {
        baseline.tick();
        observed.tick();
    }
    let read_state = |pic: &PocketIc, canister| {
        pic.query_call(canister, Principal::anonymous(), "state", vec![])
            .unwrap()
    };
    let state = read_state(&observed, canister);
    assert_eq!(state, read_state(&baseline, baseline_canister));
    assert_eq!(u64::from_le_bytes(state.try_into().unwrap()), 14);
    let after = observed.cycle_balance(canister);
    assert_eq!(after, baseline.cycle_balance(baseline_canister));
    let trace = observed.get_memory_access_trace(canister);
    assert!(trace.runtime_costs_recording);
    assert!(!trace.runtime_costs_truncated);
    assert!(!trace.subnet_runtime_costs_truncated);
    assert!(!trace.runtime_cost_totals_incomplete);
    assert!(!trace.subnet_runtime_cost_totals_incomplete);
    assert_eq!(cycle_net(&trace.runtime_costs, None), before - after);
    assert_runtime_totals(&trace.runtime_costs, &trace.runtime_cost_totals);
    assert_runtime_totals(
        &trace.subnet_runtime_costs,
        &trace.subnet_runtime_cost_totals,
    );
    let configuration = runtime_config(&observed, canister, &trace);
    let prepayment = trace
        .runtime_costs
        .iter()
        .find(|event| event.rule_id == "ic.cycles.execution.v1" && event.kind == "prepayment")
        .unwrap();
    let subnet_size: u128 = prepayment.details["subnet_size"].parse().unwrap();
    let reference_size: u128 = prepayment.details["reference_subnet_size"].parse().unwrap();
    assert_eq!(
        subnet_size, reference_size,
        "this application subnet uses the reference price"
    );
    let scale = |fee: u128| fee * subnet_size / reference_size;
    let base = config_amount(configuration, "cycles.update_message_execution_fee");
    let instruction_fee = config_amount(
        configuration,
        "cycles.ten_update_instructions_execution_fee_wasm64",
    );
    let replicated: Vec<_> = trace
        .executions
        .iter()
        .filter(|execution| execution.api_type != "non replicated query")
        .collect();
    assert_eq!(
        replicated.len(),
        6,
        "caller, callee, callback and timer are distinct executions"
    );
    assert!(
        replicated
            .iter()
            .any(|execution| execution.api_type == "global timer")
    );
    let expected_execution_cycles: u128 = replicated
        .iter()
        .map(|execution| scale(base + instruction_fee * execution.instructions as u128 / 10))
        .sum();
    assert_eq!(
        cycle_net(&trace.runtime_costs, Some("ic.cycles.execution.v1")),
        expected_execution_cycles
    );
    assert!(
        trace
            .runtime_costs
            .iter()
            .any(|event| event.kind == "refund"
                && event.rule_id == "ic.cycles.execution.v1"
                && event.amount > 0)
    );
    assert!(
        trace
            .runtime_costs
            .iter()
            .any(|event| event.rule_id == "ic.cycles.execution.v1"
                && event.kind == "prepayment"
                && event.description.contains("callback"))
    );

    // A self call still pays the transport base, method bytes, request bytes,
    // response bytes, and its separate callback execution base.
    let request = trace
        .runtime_costs
        .iter()
        .find(|event| event.rule_id == "ic.cycles.transmission.v1" && event.kind == "debit")
        .unwrap();
    let byte_fee = config_amount(configuration, "cycles.xnet_byte_transmission_fee");
    assert_eq!(request.payload_bytes, Some(4 + 9));
    assert_eq!(request.details["receiver"], canister.to_text());
    assert_eq!(
        request.amount,
        scale(config_amount(configuration, "cycles.xnet_call_fee") + byte_fee * (4 + 9))
    );
    assert_eq!(
        cycle_net(
            &trace.runtime_costs,
            Some("ic.cycles.response_transmission.v1")
        ),
        scale(byte_fee * 9)
    );
    assert!(trace.runtime_costs.iter().any(|event| event.rule_id
        == "ic.cycles.response_transmission.v1"
        && event.kind == "refund"
        && event.payload_bytes == Some(9)
        && event.callback_id.is_some()));
    for event in &trace.runtime_costs {
        assert_eq!(
            event.canister_id.as_deref(),
            Some(canister.to_text().as_str())
        );
        assert!(
            trace.charge_rules.contains_key(&event.rule_id),
            "missing explanation for {}",
            event.rule_id
        );
        if event.unit == "round_instructions" {
            assert_eq!(event.nominal_cycles, None);
        }
    }
    let slice_overhead = config_amount(
        configuration,
        "scheduler.instruction_overhead_per_execution",
    );
    assert!(
        trace
            .runtime_costs
            .iter()
            .any(|event| event.rule_id == "ic.scheduler.slice.v1"
                && event.amount == slice_overhead
                && event.description.to_ascii_lowercase().contains("global"))
    );
    assert!(
        trace
            .subnet_runtime_costs
            .iter()
            .all(|event| event.canister_id.is_none()
                && event.subnet_id.is_some()
                && event.unit == "round_instructions")
    );
    assert!(!trace.subnet_runtime_costs.is_empty());
}

#[test]
fn runtime_ingress_charge_uses_complete_signed_envelope_bytes() {
    use serde_cbor::Value as Cbor;
    let pic = PocketIc::new();
    let canister = install_host_cost_canister(&pic);
    pic.start_memory_access_tracing(canister);
    let payload = b"envelope probe";
    let content = Cbor::Map(BTreeMap::from([
        (Cbor::Text("request_type".into()), Cbor::Text("call".into())),
        (
            Cbor::Text("canister_id".into()),
            Cbor::Bytes(canister.as_slice().to_vec()),
        ),
        (Cbor::Text("method_name".into()), Cbor::Text("echo".into())),
        (Cbor::Text("arg".into()), Cbor::Bytes(payload.to_vec())),
        (
            Cbor::Text("sender".into()),
            Cbor::Bytes(Principal::anonymous().as_slice().to_vec()),
        ),
        (
            Cbor::Text("ingress_expiry".into()),
            Cbor::Integer((pic.get_time().as_nanos_since_unix_epoch() + 120_000_000_000).into()),
        ),
        (
            Cbor::Text("nonce".into()),
            Cbor::Bytes(b"envelope".to_vec()),
        ),
    ]));
    // SignedIngress reconstruction uses the self-described CBOR tag and an
    // indefinite map for the flattened call content. Submit that representation
    // so its complete byte length is the actual induction accounting input.
    let mut content = serde_cbor::to_vec(&content).unwrap();
    assert_eq!(content[0], 0xa7);
    content[0] = 0xbf;
    content.push(0xff);
    let mut envelope = vec![0xd9, 0xd9, 0xf7, 0xa1];
    envelope.extend(serde_cbor::to_vec(&Cbor::Text("content".into())).unwrap());
    envelope.extend(content);
    let before = pic.cycle_balance(canister);
    let response = Client::new()
        .post(
            pic.get_server_url()
                .join(&format!(
                    "instances/{}/api/v2/canister/{canister}/call",
                    pic.instance_id()
                ))
                .unwrap(),
        )
        .header("Content-Type", "application/cbor")
        .body(envelope.clone())
        .send()
        .unwrap();
    assert_eq!(response.status(), 202, "{}", response.text().unwrap());
    for _ in 0..10 {
        pic.tick();
        if !pic.get_memory_access_trace(canister).executions.is_empty() {
            break;
        }
    }
    let trace = pic.get_memory_access_trace(canister);
    assert_eq!(trace.executions.len(), 1);
    let ingress: Vec<_> = trace
        .runtime_costs
        .iter()
        .filter(|event| event.rule_id == "ic.cycles.ingress.v1" && event.kind == "debit")
        .collect();
    assert_eq!(ingress.len(), 1);
    let prepayment = trace
        .runtime_costs
        .iter()
        .find(|event| event.rule_id == "ic.cycles.execution.v1" && event.kind == "prepayment")
        .unwrap();
    let subnet_size: u128 = prepayment.details["subnet_size"].parse().unwrap();
    let reference_size: u128 = prepayment.details["reference_subnet_size"].parse().unwrap();
    let configuration = runtime_config(&pic, canister, &trace);
    let base = config_amount(configuration, "cycles.ingress_message_reception_fee") * subnet_size
        / reference_size;
    let per_byte = config_amount(configuration, "cycles.ingress_byte_reception_fee") * subnet_size
        / reference_size;
    assert!(envelope.len() > payload.len() + "echo".len());
    assert_eq!(ingress[0].amount, base + per_byte * envelope.len() as u128);
    assert_ne!(
        ingress[0].amount,
        base + per_byte * (payload.len() + "echo".len()) as u128
    );
    assert_eq!(
        cycle_net(&trace.runtime_costs, None),
        before - pic.cycle_balance(canister)
    );
}

#[test]
fn runtime_cost_cursor_paginates_native_work_and_guards_clear() {
    let pic = PocketIc::new();
    let canister = install_host_cost_canister(&pic);
    pic.start_memory_access_tracing(canister);
    pic.update_call(canister, Principal::anonymous(), "echo", b"cursor".to_vec())
        .unwrap();
    let viewer = pic.start_memory_access_viewer(None);
    let client = Client::new();
    let base = format!(
        "api/instances/{}/canisters/{canister}/runtime-costs",
        pic.instance_id()
    );
    let fetch = |after: Option<u64>| -> Value {
        let suffix = after.map_or_else(
            || "?limit=1".to_string(),
            |after| format!("?limit=1&after={after}"),
        );
        client
            .get(viewer.join(&format!("{base}{suffix}")).unwrap())
            .send()
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .unwrap()
    };
    let snapshot = pic.get_memory_access_trace(canister);
    let mut cursor = None;
    let mut received = Vec::new();
    for _ in 0..256 {
        let page = fetch(cursor);
        let events: Vec<_> = page["runtime_costs"]
            .as_array()
            .unwrap()
            .iter()
            .chain(page["subnet_runtime_costs"].as_array().unwrap())
            .collect();
        assert!(events.len() <= 1, "one shared limit covers both streams");
        for event in events {
            let sequence: u64 = event["sequence_number"].as_str().unwrap().parse().unwrap();
            assert!(cursor.is_none_or(|after| sequence > after));
            received.push(sequence);
        }
        let next: Option<u64> = page["next_after"]
            .as_str()
            .map(|value| value.parse().unwrap());
        if page["more"] == false {
            cursor = next;
            break;
        }
        assert_ne!(next, cursor);
        cursor = next;
    }
    let mut expected: Vec<_> = snapshot
        .runtime_costs
        .iter()
        .chain(&snapshot.subnet_runtime_costs)
        .map(|event| event.sequence_number)
        .collect();
    expected.sort_unstable();
    assert_eq!(received, expected);
    assert_eq!(cursor, snapshot.last_runtime_cost_sequence_number);
    assert!(received.len() > snapshot.executions.len());
    let old_wasm_sequence = snapshot
        .executions
        .last()
        .map(|execution| execution.sequence_number);
    pic.upload_chunk(canister, None, b"native-only work".to_vec())
        .unwrap();
    let current = pic.get_memory_access_trace(canister);
    assert_eq!(current.trace_generation, snapshot.trace_generation);
    assert_eq!(
        current
            .executions
            .last()
            .map(|execution| execution.sequence_number),
        old_wasm_sequence
    );
    assert_ne!(
        current.last_runtime_cost_sequence_number,
        snapshot.last_runtime_cost_sequence_number
    );
    let next = fetch(cursor);
    assert!(
        next["runtime_costs"]
            .as_array()
            .unwrap()
            .iter()
            .chain(next["subnet_runtime_costs"].as_array().unwrap())
            .all(|event| event["sequence_number"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
                > cursor.unwrap())
    );
    let clear_endpoint = pic
        .get_server_url()
        .join(&format!(
            "instances/{}/update/clear_memory_access_trace",
            pic.instance_id()
        ))
        .unwrap();
    let condition = |trace: &MemoryAccessTrace| RawClearMemoryAccessTrace {
        canister_id: canister.as_slice().to_vec(),
        expected_server_generation: Some(trace.server_generation.clone()),
        expected_trace_generation: Some(trace.trace_generation),
        expected_last_sequence_number: trace
            .executions
            .last()
            .map(|execution| execution.sequence_number),
        expected_last_runtime_cost_sequence_number: Some(trace.last_runtime_cost_sequence_number),
    };
    assert_eq!(
        client
            .post(clear_endpoint.clone())
            .json(&condition(&snapshot))
            .send()
            .unwrap()
            .status(),
        409
    );
    assert_eq!(
        pic.get_memory_access_trace(canister).runtime_cost_totals,
        current.runtime_cost_totals
    );
    assert_eq!(
        client
            .post(clear_endpoint)
            .json(&condition(&current))
            .send()
            .unwrap()
            .status(),
        200
    );
    let cleared = pic.get_memory_access_trace(canister);
    assert!(cleared.executions.is_empty());
    assert!(cleared.runtime_costs.is_empty());
    assert!(cleared.runtime_cost_totals.is_empty());
}

#[test]
fn runtime_costs_record_compilation_and_charge_duplicate_chunk_uploads() {
    let pic = PocketIc::new();
    let canister = pic.create_canister();
    pic.add_cycles(canister, INIT_CYCLES);
    pic.start_instruction_profiling(canister, 20);
    pic.install_canister(
        canister,
        wat::parse_str(HOST_COST_WAT).unwrap(),
        vec![],
        None,
    );
    let trace = pic.get_memory_access_trace(canister);
    let configuration = runtime_config(&pic, canister, &trace);
    let compilation = trace
        .runtime_costs
        .iter()
        .find(|event| event.rule_id == "ic.management.compilation.v1")
        .unwrap();
    assert_eq!(compilation.unit, "execution_instructions");
    assert!(
        compilation.amount
            >= config_amount(configuration, "embedder.create_execution_state_base_cost")
    );
    assert_eq!(
        compilation.instructions,
        Some(compilation.amount.try_into().unwrap())
    );
    assert!(
        trace.executions.is_empty(),
        "a module without init performs native compilation without a Wasm execution"
    );
    let upload_instructions =
        config_amount(configuration, "scheduler.upload_wasm_chunk_instructions");
    pic.clear_memory_access_trace(canister);
    let before = pic.cycle_balance(canister);
    let chunk = b"duplicate chunk must still be hashed and charged".to_vec();
    let hash = pic.upload_chunk(canister, None, chunk.clone()).unwrap();
    assert_eq!(pic.upload_chunk(canister, None, chunk).unwrap(), hash);
    let trace = pic.get_memory_access_trace(canister);
    let uploads: Vec<_> = trace
        .runtime_costs
        .iter()
        .filter(|event| event.rule_id == "ic.management.upload_chunk.v1")
        .collect();
    assert_eq!(uploads.len(), 2);
    assert!(
        uploads
            .iter()
            .all(|event| event.unit == "execution_instructions"
                && event.kind == "debit"
                && event.amount == upload_instructions
                && event.instructions == Some(upload_instructions.try_into().unwrap()))
    );
    assert!(trace.executions.is_empty());
    assert_eq!(
        cycle_net(&trace.runtime_costs, None),
        before - pic.cycle_balance(canister)
    );
    assert_runtime_totals(&trace.runtime_costs, &trace.runtime_cost_totals);
}

#[test]
fn runtime_query_call_budget_is_separate_from_cycles_and_round_work() {
    let baseline = PocketIc::new();
    let observed = PocketIc::new();
    let baseline_canister = install_host_cost_canister(&baseline);
    let canister = install_host_cost_canister(&observed);
    observed.start_instruction_profiling(canister, 20);
    let before = observed.cycle_balance(canister);
    let query = |pic: &PocketIc, canister| {
        pic.query_call(canister, Principal::anonymous(), "query_self", vec![])
            .unwrap()
    };
    assert_eq!(query(&observed, canister), b"transport");
    assert_eq!(query(&baseline, baseline_canister), b"transport");
    assert_eq!(observed.cycle_balance(canister), before);
    assert_eq!(
        observed.cycle_balance(canister),
        baseline.cycle_balance(baseline_canister)
    );
    let state = observed
        .query_call(canister, Principal::anonymous(), "state", vec![])
        .unwrap();
    assert_eq!(
        state,
        0_u64.to_le_bytes(),
        "composite-query writes stay uncommitted"
    );
    let trace = observed.get_memory_access_trace(canister);
    let configuration = runtime_config(&observed, canister, &trace);
    let calls: Vec<_> = trace
        .runtime_costs
        .iter()
        .filter(|event| event.rule_id == "ic.query.call_overhead.v1")
        .collect();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].unit, "query_graph_instructions");
    assert_eq!(calls[0].kind, "debit");
    assert_eq!(
        calls[0].amount,
        config_amount(configuration, "query.instruction_overhead_per_call")
    );
    assert_eq!(calls[0].nominal_cycles, None);
    assert_eq!(calls[0].details["receiver"], canister.to_text());
    assert!(calls[0].callback_id.is_some());
    assert_eq!(cycle_net(&trace.runtime_costs, None), 0);
    assert_runtime_totals(&trace.runtime_costs, &trace.runtime_cost_totals);
}

#[test]
fn runtime_costs_retain_discarded_balance_after_canister_deletion() {
    let baseline = PocketIc::new();
    let observed = PocketIc::new();
    let create = |pic: &PocketIc| {
        let canister = pic.create_canister();
        pic.add_cycles(canister, INIT_CYCLES);
        canister
    };
    let baseline_canister = create(&baseline);
    let canister = create(&observed);
    observed.start_memory_access_tracing(canister);
    baseline.stop_canister(baseline_canister, None).unwrap();
    observed.stop_canister(canister, None).unwrap();
    let remaining = observed.cycle_balance(canister);
    assert_eq!(remaining, baseline.cycle_balance(baseline_canister));
    // Exclude the stop operation so the remaining balance must be explained by
    // the delete ingress and the actual balance discarded at committed removal.
    observed.clear_memory_access_trace(canister);
    baseline.delete_canister(baseline_canister, None).unwrap();
    observed.delete_canister(canister, None).unwrap();
    assert!(baseline.canister_status(baseline_canister, None).is_err());
    assert!(observed.canister_status(canister, None).is_err());
    let trace = observed.get_memory_access_trace(canister);
    let discarded: Vec<_> = trace
        .runtime_costs
        .iter()
        .filter(|event| event.rule_id == "ic.cycles.deleted_canister.v1")
        .collect();
    assert_eq!(discarded.len(), 1);
    assert_eq!(discarded[0].unit, "cycles");
    assert_eq!(discarded[0].kind, "debit");
    assert!(discarded[0].amount > 0);
    assert_eq!(discarded[0].nominal_cycles, Some(discarded[0].amount));
    let ingress = cycle_net(&trace.runtime_costs, Some("ic.cycles.ingress.v1"));
    assert_eq!(discarded[0].amount, remaining - ingress);
    assert_eq!(cycle_net(&trace.runtime_costs, None), remaining);
    assert!(
        trace.executions.is_empty(),
        "deletion does not execute guest Wasm"
    );
    assert!(
        trace
            .charge_rules
            .contains_key("ic.cycles.deleted_canister.v1")
    );
    assert_runtime_totals(&trace.runtime_costs, &trace.runtime_cost_totals);
}
