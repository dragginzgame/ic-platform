//! Observation-only scheduler budget snapshots. No value here controls execution.

use ic_config::subnet_config::SchedulerConfig;
use ic_system_profiler::{
    CanisterRoundPass, RoundBudgetConfig, RoundIteration, RoundSlice, RoundSliceNotes,
    RoundSnapshot, RoundWorker,
};
use ic_types::{CanisterId, ExecutionRound, SubnetId};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct DetailBudget {
    passes: AtomicUsize,
    slices: AtomicUsize,
}

fn take(remaining: &AtomicUsize) -> bool {
    remaining
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |left| {
            left.checked_sub(1)
        })
        .is_ok()
}

pub(super) struct RoundCapture {
    row: RoundSnapshot,
    detail: Arc<DetailBudget>,
    iteration: u32,
    retained_iteration: bool,
}

impl RoundCapture {
    pub(super) fn start(
        subnet: SubnetId,
        round: ExecutionRound,
        kind: String,
        config: &SchedulerConfig,
    ) -> Self {
        let reserve = config
            .max_instructions_per_slice
            .get()
            .max(config.max_instructions_per_install_code_slice.get());
        let initial = config.max_instructions_per_round.get() as i64 - reserve as i64 + 1;
        Self {
            row: RoundSnapshot {
                subnet_id: subnet.to_string(), round: round.get(), round_kind: kind,
                start_ns: ic_system_profiler::round_clock_ns(),
                config: RoundBudgetConfig {
                    scheduler_cores: config.scheduler_cores as u32,
                    max_instructions_per_round: config.max_instructions_per_round.get(),
                    max_instructions_per_message: config.max_instructions_per_message.get(),
                    max_instructions_per_query_message: config.max_instructions_per_query_message.get(),
                    max_instructions_per_slice: config.max_instructions_per_slice.get(),
                    max_instructions_per_install_code: config.max_instructions_per_install_code.get(),
                    max_instructions_per_install_code_slice: config.max_instructions_per_install_code_slice.get(),
                    instruction_overhead_per_execution: config.instruction_overhead_per_execution.get(),
                    instruction_overhead_per_canister: config.instruction_overhead_per_canister.get(),
                    instruction_overhead_per_canister_for_finalization: config.instruction_overhead_per_canister_for_finalization.get(),
                    max_heap_delta_per_iteration_bytes: config.max_heap_delta_per_iteration.get(),
                    deterministic_time_slicing: ic_embedders::wasmtime_embedder::system_api::InstructionLimits::new(config.max_instructions_per_message, config.max_instructions_per_slice).slicing_enabled(),
                    source: "SchedulerConfig; initial=round-max(update_slice,install_slice)+1; management=round/16; iteration debit=max(worker debits)+finalization".to_string(),
                },
                initial_instructions: initial,
                heap_delta_bytes: Some(0),
                management_initial_instructions: (config.max_instructions_per_round.get() / super::SUBNET_MESSAGES_LIMIT_FRACTION) as i64,
                ..Default::default()
            },
            detail: Arc::new(DetailBudget {
                passes: AtomicUsize::new(ic_system_profiler::MAX_ROUND_PASSES),
                slices: AtomicUsize::new(ic_system_profiler::MAX_ROUND_SLICES),
            }),
            iteration: 0, retained_iteration: false,
        }
    }

    pub(super) fn worker(&self, index: u32, initial: i64) -> WorkerCapture {
        WorkerCapture {
            row: RoundWorker {
                worker: index,
                initial_instructions: initial,
                heap_delta_bytes: Some(0),
                ..Default::default()
            },
            detail: Arc::clone(&self.detail),
            retain: self.row.iterations.len() < ic_system_profiler::MAX_ROUND_ITERATIONS,
        }
    }

    pub(super) fn iteration(
        &mut self,
        before: i64,
        after: i64,
        workers: Vec<RoundWorker>,
        heap_delta_bytes: u64,
    ) {
        self.iteration = self.iteration.saturating_add(1);
        self.row.heap_delta_bytes = Some(
            self.row
                .heap_delta_bytes
                .unwrap_or(0)
                .saturating_add(heap_delta_bytes),
        );
        for worker in &workers {
            self.row.sum_worker_consumed = self.row.sum_worker_consumed.saturating_add(
                worker
                    .initial_instructions
                    .saturating_sub(worker.remaining_instructions)
                    .max(0) as u64,
            );
            self.row.guest_instructions = self
                .row
                .guest_instructions
                .saturating_add(worker.guest_instructions);
            self.row.dirty_page_copy_instructions = self
                .row
                .dirty_page_copy_instructions
                .saturating_add(worker.dirty_page_copy_instructions);
            self.row.execution_round_instructions = self
                .row
                .execution_round_instructions
                .saturating_add(worker.execution_round_instructions);
            self.row.execution_overhead_instructions = self
                .row
                .execution_overhead_instructions
                .saturating_add(worker.execution_overhead_instructions);
            self.row.canister_overhead_instructions = self
                .row
                .canister_overhead_instructions
                .saturating_add(worker.canister_overhead_instructions);
            self.row.slice_count = self.row.slice_count.saturating_add(worker.slice_count);
            self.row.completed_messages = self
                .row
                .completed_messages
                .saturating_add(worker.completed_messages);
        }
        self.retained_iteration =
            self.row.iterations.len() < ic_system_profiler::MAX_ROUND_ITERATIONS;
        if self.retained_iteration {
            self.row.iterations.push(RoundIteration {
                iteration: self.iteration,
                heap_delta_bytes: Some(heap_delta_bytes),
                budget_before: before,
                budget_after: after,
                max_worker_consumed: before.saturating_sub(after).max(0) as u64,
                sum_worker_consumed: workers
                    .iter()
                    .map(|worker| {
                        worker
                            .initial_instructions
                            .saturating_sub(worker.remaining_instructions)
                            .max(0) as u64
                    })
                    .fold(0_u64, u64::saturating_add),
                workers,
                ..Default::default()
            });
        } else {
            self.row.omitted_iterations = self.row.omitted_iterations.saturating_add(1);
        }
    }

    pub(super) fn finalize_iteration(&mut self, instructions: u64, canisters: u64, remaining: i64) {
        self.row.finalization_instructions = self
            .row
            .finalization_instructions
            .saturating_add(instructions);
        self.row.finalized_canisters = self.row.finalized_canisters.saturating_add(canisters);
        if self.retained_iteration {
            if let Some(iteration) = self.row.iterations.last_mut() {
                iteration.finalization_instructions = instructions;
                iteration.budget_after = remaining;
            }
        }
    }

    pub(super) fn stop(&mut self, reason: &str) {
        if !self.row.stop_reasons.iter().any(|value| value == reason) {
            self.row.stop_reasons.push(reason.to_string());
        }
    }

    pub(super) fn finish(mut self, remaining: i64, management_remaining: i64) {
        self.row.remaining_instructions = remaining;
        self.row.management_remaining_instructions = management_remaining;
        if management_remaining <= 0 {
            self.stop("management_instruction_limit");
        }
        self.row.end_ns = ic_system_profiler::round_clock_ns();
        ic_system_profiler::record_round(self.row);
    }
}

pub(super) struct WorkerCapture {
    row: RoundWorker,
    detail: Arc<DetailBudget>,
    retain: bool,
}

impl WorkerCapture {
    pub(super) fn start(&mut self) {
        self.row.start_ns = ic_system_profiler::round_clock_ns();
    }

    pub(super) fn pass(&self, canister: CanisterId, initial: i64) -> PassCapture {
        PassCapture {
            row: CanisterRoundPass {
                canister_id: canister.to_string(),
                pass: self.row.canisters.len() as u32 + self.row.omitted_canister_passes as u32,
                initial_instructions: initial,
                heap_delta_bytes: Some(0),
                start_ns: ic_system_profiler::round_clock_ns(),
                ..Default::default()
            },
            detail: Arc::clone(&self.detail),
            retain: self.retain && take(&self.detail.passes),
        }
    }

    pub(super) fn add_pass(&mut self, mut pass: PassCapture, remaining: i64) {
        pass.row.remaining_instructions = remaining;
        pass.row.end_ns = ic_system_profiler::round_clock_ns();
        self.row.guest_instructions = self
            .row
            .guest_instructions
            .saturating_add(pass.row.guest_instructions);
        self.row.dirty_page_copy_instructions = self
            .row
            .dirty_page_copy_instructions
            .saturating_add(pass.row.dirty_page_copy_instructions);
        self.row.execution_round_instructions = self
            .row
            .execution_round_instructions
            .saturating_add(pass.row.execution_round_instructions);
        self.row.execution_overhead_instructions = self
            .row
            .execution_overhead_instructions
            .saturating_add(pass.row.execution_overhead_instructions);
        self.row.canister_overhead_instructions = self
            .row
            .canister_overhead_instructions
            .saturating_add(pass.row.canister_overhead_instructions);
        self.row.slice_count = self.row.slice_count.saturating_add(pass.row.slice_count);
        self.row.completed_messages = self
            .row
            .completed_messages
            .saturating_add(pass.row.completed_messages);
        if pass.retain {
            self.row.canisters.push(pass.row);
        } else {
            self.row.omitted_canister_passes = self.row.omitted_canister_passes.saturating_add(1);
        }
    }

    pub(super) fn finish(
        mut self,
        remaining: i64,
        reason: &str,
        heap_delta_bytes: u64,
    ) -> RoundWorker {
        self.row.remaining_instructions = remaining;
        self.row.heap_delta_bytes = Some(heap_delta_bytes);
        self.row.end_ns = ic_system_profiler::round_clock_ns();
        self.row.stop_reasons.push(reason.to_string());
        self.row
    }
}

pub(super) struct PassCapture {
    row: CanisterRoundPass,
    detail: Arc<DetailBudget>,
    retain: bool,
}

impl PassCapture {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn slice(
        &mut self,
        instructions: u64,
        overhead: u64,
        slices: u64,
        messages: u64,
        completed_instructions: Option<u64>,
        notes: RoundSliceNotes,
        start_ns: u64,
        end_ns: u64,
        heap_delta_bytes: u64,
    ) {
        let dirty_copy = notes.dirty_page_copy_instructions;
        let guest = instructions.saturating_sub(dirty_copy);
        self.row.guest_instructions = self.row.guest_instructions.saturating_add(guest);
        self.row.dirty_page_copy_instructions = self
            .row
            .dirty_page_copy_instructions
            .saturating_add(dirty_copy);
        self.row.execution_round_instructions = self
            .row
            .execution_round_instructions
            .saturating_add(instructions);
        self.row.execution_overhead_instructions = self
            .row
            .execution_overhead_instructions
            .saturating_add(overhead);
        self.row.completed_message_instructions = self
            .row
            .completed_message_instructions
            .saturating_add(completed_instructions.unwrap_or(0));
        self.row.slice_count = self.row.slice_count.saturating_add(slices);
        self.row.completed_messages = self.row.completed_messages.saturating_add(messages);
        self.row.heap_delta_bytes = Some(
            self.row
                .heap_delta_bytes
                .unwrap_or(0)
                .saturating_add(heap_delta_bytes),
        );
        if dirty_copy > instructions {
            self.stop("dirty_copy_exceeds_execution_debit");
        }
        if self.retain && take(&self.detail.slices) {
            self.row.slices.push(RoundSlice {
                slice: (self.row.slices.len() as u64 + self.row.omitted_slices) as u32,
                kind: notes.kind,
                method: notes.method,
                logical_execution_id: notes.logical_execution_id,
                start_ns,
                end_ns,
                guest_instructions: guest,
                dirty_page_copy_instructions: dirty_copy,
                execution_round_instructions: instructions,
                execution_overhead_instructions: overhead,
                completed_message_instructions: completed_instructions,
                completed: completed_instructions.is_some(),
                heap_delta_bytes: Some(heap_delta_bytes),
            });
        } else {
            self.row.omitted_slices = self.row.omitted_slices.saturating_add(1);
        }
    }

    pub(super) fn overhead(&mut self, instructions: u64) {
        self.row.canister_overhead_instructions = instructions;
    }
    pub(super) fn stop(&mut self, reason: &str) {
        self.row.stop_reasons.push(reason.to_string());
    }
}

#[cfg(test)]
mod tests {
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
}
