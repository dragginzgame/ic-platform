use super::*;

fn capture() -> RoundCapture {
    RoundCapture::start(
        SubnetId::new(ic_types::PrincipalId::default()),
        ExecutionRound::from(1),
        "OrdinaryRound".to_string(),
        &SchedulerConfig::application_subnet(),
    )
}

fn worker(
    capture: &RoundCapture,
    index: u32,
    execution: u64,
    dirty_copy: u64,
    remaining: i64,
) -> RoundWorker {
    let heap_delta = 7 + u64::from(index);
    let mut worker = capture.worker(index, 100);
    worker.start();
    let mut pass = worker.pass(CanisterId::ic_00(), 100);
    pass.slice(
        execution,
        2,
        1,
        1,
        Some(9_000),
        RoundSliceNotes {
            dirty_page_copy_instructions: dirty_copy,
            kind: Some("callback".to_string()),
            ..Default::default()
        },
        1,
        2,
        heap_delta,
    );
    pass.overhead(8);
    worker.add_pass(pass, remaining);
    worker.finish(remaining, "all_canisters_processed", heap_delta)
}

#[test]
fn shared_budget_uses_maximum_worker_debit_not_sum() {
    let mut capture = capture();
    let first = worker(&capture, 0, 70, 10, 20);
    let second = worker(&capture, 1, 30, 5, 60);
    capture.iteration(100, 20, vec![first, second], 15);
    capture.finalize_iteration(12, 1, 8);
    let iteration = &capture.row.iterations[0];
    assert_eq!(iteration.max_worker_consumed, 80);
    assert_eq!(iteration.sum_worker_consumed, 120);
    assert_eq!(iteration.budget_after, 8);
    assert_eq!(iteration.finalization_instructions, 12);
    assert_eq!(capture.row.sum_worker_consumed, 120);
    assert_eq!(capture.row.execution_round_instructions, 100);
    assert_eq!(capture.row.guest_instructions, 85);
    assert_eq!(capture.row.dirty_page_copy_instructions, 15);
    assert_eq!(capture.row.execution_overhead_instructions, 4);
    assert_eq!(capture.row.canister_overhead_instructions, 16);
    assert_eq!(capture.row.heap_delta_bytes, Some(15));
    assert_eq!(iteration.heap_delta_bytes, Some(15));
    assert_eq!(iteration.workers[0].heap_delta_bytes, Some(7));
    assert_eq!(iteration.workers[0].canisters[0].heap_delta_bytes, Some(7));
    assert_eq!(
        iteration.workers[0].canisters[0].slices[0].heap_delta_bytes,
        Some(7)
    );
    // Completed-message accounting can include previous rounds. It cannot
    // replace this slice's measured execution debit in a resumed message.
    assert_eq!(
        iteration.workers[0].canisters[0].slices[0].completed_message_instructions,
        Some(9_000)
    );
    assert_eq!(
        iteration.workers[0].canisters[0].slices[0].guest_instructions,
        60
    );
}

#[test]
fn detail_limits_preserve_complete_aggregate_totals() {
    let mut capture = capture();
    capture.detail.passes.store(0, Ordering::Relaxed);
    capture.detail.slices.store(0, Ordering::Relaxed);
    let worker = worker(&capture, 0, 70, 10, 20);
    assert!(worker.canisters.is_empty());
    assert_eq!(worker.omitted_canister_passes, 1);
    assert_eq!(worker.guest_instructions, 60);
    capture
        .row
        .iterations
        .resize(ic_system_profiler::MAX_ROUND_ITERATIONS, Default::default());
    capture.iteration(100, 20, vec![worker], 7);
    capture.finalize_iteration(12, 1, 8);
    assert_eq!(capture.row.omitted_iterations, 1);
    assert_eq!(capture.row.sum_worker_consumed, 80);
    assert_eq!(capture.row.guest_instructions, 60);
    assert_eq!(capture.row.finalization_instructions, 12);
    assert_eq!(capture.row.heap_delta_bytes, Some(7));
}

#[test]
fn signed_remaining_preserves_last_slice_and_overhead_overshoot() {
    let mut capture = capture();
    let worker = worker(&capture, 0, 95, 0, -5);
    assert_eq!(worker.remaining_instructions, -5);
    assert_eq!(worker.canisters[0].remaining_instructions, -5);
    capture.iteration(100, -5, vec![worker], 7);
    capture.finalize_iteration(12, 1, -17);
    assert_eq!(capture.row.iterations[0].budget_after, -17);
    assert_eq!(capture.row.iterations[0].max_worker_consumed, 105);
}

#[test]
fn scheduler_dts_snapshot_keeps_current_slice_separate_from_completed_message() {
    use crate::scheduler::test_utilities::{SchedulerTestBuilder, ingress};
    use ic_interfaces::execution_environment::ExecutionRoundType;
    use ic_types::NumInstructions;
    let mut test = SchedulerTestBuilder::new()
        .with_scheduler_config(SchedulerConfig {
            scheduler_cores: 2,
            max_instructions_per_round: NumInstructions::new(40),
            max_instructions_per_message: NumInstructions::new(200),
            max_instructions_per_slice: NumInstructions::new(20),
            max_instructions_per_install_code_slice: NumInstructions::new(20),
            instruction_overhead_per_execution: NumInstructions::new(1),
            instruction_overhead_per_canister: NumInstructions::new(0),
            instruction_overhead_per_canister_for_finalization: NumInstructions::new(0),
            ..SchedulerConfig::application_subnet()
        })
        .build();
    let canister = test.create_canister();
    test.send_ingress(canister, ingress(41));
    let run = ic_system_profiler::start(ic_system_profiler::RunConfig {
        name: "scheduler DTS round snapshots".to_string(),
        max_events: 1,
        ..Default::default()
    })
    .unwrap();
    let id = run["run_id"].as_str().unwrap();
    for _ in 0..3 {
        test.execute_round(ExecutionRoundType::OrdinaryRound);
    }
    ic_system_profiler::stop(id, &Default::default()).unwrap();
    let result = ic_system_profiler::query(id, "rounds", &Default::default()).unwrap();
    ic_system_profiler::delete_run(id).unwrap();
    let rows = result["rounds"]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    macro_rules! number {
        ($value:expr) => {{
            let value = &$value;
            value
                .as_str()
                .map(|v| v.parse::<u64>().unwrap())
                .or_else(|| value.as_u64())
                .unwrap()
        }};
    }
    assert_eq!(
        rows.iter()
            .map(|row| number!(row["guest_instructions"]))
            .collect::<Vec<_>>(),
        vec![20, 20, 1]
    );
    assert_eq!(
        rows.iter()
            .map(|row| number!(row["slice_count"]))
            .sum::<u64>(),
        3
    );
    assert!(rows.iter().all(|row| row["accounting_reconciles"] == true));
    let completed = rows
        .iter()
        .flat_map(|row| row["iterations"].as_array().unwrap())
        .flat_map(|iteration| iteration["workers"].as_array().unwrap())
        .flat_map(|worker| worker["canisters"].as_array().unwrap())
        .flat_map(|pass| pass["slices"].as_array().unwrap())
        .filter_map(|slice| {
            (!slice["completed_message_instructions"].is_null())
                .then(|| number!(slice["completed_message_instructions"]))
        })
        .collect::<Vec<_>>();
    assert_eq!(completed, vec![41]);
}

#[test]
fn scheduler_heap_stop_sums_workers_and_prior_iterations() {
    use crate::scheduler::test_utilities::{
        SchedulerTestBuilder, ingress, on_response, other_side,
    };
    use ic_interfaces::execution_environment::ExecutionRoundType;
    use ic_types::NumBytes;
    let page = ic_sys::PAGE_SIZE as u64;
    let mut test = SchedulerTestBuilder::new()
        .with_scheduler_config(SchedulerConfig {
            scheduler_cores: 2,
            max_heap_delta_per_iteration: NumBytes::new(3 * page),
            ..SchedulerConfig::application_subnet()
        })
        .build();
    let a = test.create_canister();
    let b = test.create_canister();
    test.send_ingress(
        a,
        ingress(10)
            .dirty_pages(1)
            .call(other_side(b, 10).dirty_pages(1), on_response(1)),
    );
    test.send_ingress(
        b,
        ingress(10)
            .dirty_pages(1)
            .call(other_side(a, 10).dirty_pages(1), on_response(1)),
    );
    let run = ic_system_profiler::start(ic_system_profiler::RunConfig {
        name: "scheduler cumulative heap delta".to_string(),
        max_events: 1,
        ..Default::default()
    })
    .unwrap();
    let id = run["run_id"].as_str().unwrap();
    test.execute_round(ExecutionRoundType::OrdinaryRound);
    ic_system_profiler::stop(id, &Default::default()).unwrap();
    let result = ic_system_profiler::query(id, "rounds", &Default::default()).unwrap();
    ic_system_profiler::delete_run(id).unwrap();
    let rows = result["rounds"]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    macro_rules! number {
        ($value:expr) => {{
            let value = &$value;
            value
                .as_str()
                .map(|v| v.parse::<u64>().unwrap())
                .or_else(|| value.as_u64())
                .unwrap()
        }};
    }
    assert_eq!(number!(row["heap_delta_bytes"]), 4 * page);
    assert!(
        row["stop_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "iteration_heap_delta_limit")
    );
    let iterations = row["iterations"].as_array().unwrap();
    assert_eq!(iterations.len(), 2);
    for iteration in iterations {
        // Neither individual iteration reaches the three-page cap; their
        // round-cumulative sum does, and no third iteration is started.
        assert_eq!(number!(iteration["heap_delta_bytes"]), 2 * page);
        for worker in iteration["workers"].as_array().unwrap() {
            assert_eq!(number!(worker["heap_delta_bytes"]), page);
            let pass_sum = worker["canisters"]
                .as_array()
                .unwrap()
                .iter()
                .map(|pass| {
                    let slices = pass["slices"].as_array().unwrap();
                    assert_eq!(
                        slices
                            .iter()
                            .map(|slice| number!(slice["heap_delta_bytes"]))
                            .sum::<u64>(),
                        number!(pass["heap_delta_bytes"])
                    );
                    number!(pass["heap_delta_bytes"])
                })
                .sum::<u64>();
            assert_eq!(pass_sum, page);
        }
    }
}
