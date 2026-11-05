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
    #[expect(
        clippy::too_many_arguments,
        reason = "Retain the scheduler slice counters and timing inputs as one observation."
    )]
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
mod tests;
