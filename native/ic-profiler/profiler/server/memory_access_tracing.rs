//! Records the Wasm memory (heap) pages accessed by the message executions of
//! canisters for which memory access tracing was started.

use ic_crypto_sha2::Sha256;
use ic_interfaces::execution_environment::{
    TracedExecutionDisposition, TracedExecutionState, TracedLogicalExecution, TracedRuntimeCost,
    TracedWasmExecution, TracedWasmExecutionOutcome, WasmMemoryAccessTracer,
};
use ic_replicated_state::canister_state::WASM_PAGE_SIZE_IN_BYTES;
use ic_types::{CanisterId, SubnetId};
use pocket_ic::common::rest::{self, MemoryAccessTrace, TracedExecution, TracedExecutionOutcome};
use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

/// The budget covers retained trace data for one PocketIC instance. A full
/// recorder stops collecting detail visibly rather than evicting old traces.
const TRACE_MEMORY_LIMIT: usize = 64 * 1024 * 1024;
const TRACE_SUMMARY_RESERVE: usize = 64 * 1024;

pub(crate) struct ServerIdentity {
    pub generation: String,
    pub binary_hash: Option<String>,
}

/// Shared by all instances, including instances created after tracing starts.
/// A process restart must never look like a continuation of an old run.
pub(crate) fn server_identity() -> &'static ServerIdentity {
    static IDENTITY: OnceLock<ServerIdentity> = OnceLock::new();
    IDENTITY.get_or_init(|| {
        let binary_hash = std::env::current_exe().ok().and_then(|path| {
            let mut file = std::fs::File::open(path).ok()?;
            let mut hasher = Sha256::new();
            std::io::copy(&mut file, &mut hasher).ok()?;
            Some(hex::encode(hasher.finish()))
        });
        ServerIdentity {
            generation: uuid::Uuid::new_v4().to_string(),
            binary_hash,
        }
    })
}

/// Records heap accesses and selected charge profiles outside canister state.
pub(crate) struct MemoryAccessTraceRecorder {
    traces: Mutex<Traces>,
    memory_limit: usize,
    tracing_active: AtomicBool,
    costs_active: AtomicBool,
}

impl Default for MemoryAccessTraceRecorder {
    fn default() -> Self {
        Self::with_memory_limit(TRACE_MEMORY_LIMIT)
    }
}

#[derive(Default)]
struct Traces {
    next_sequence_number: u64,
    recorded_bytes: usize,
    canisters: BTreeMap<CanisterId, CanisterTrace>,
    modules: BTreeMap<String, StoredModule>,
    next_runtime_cost_sequence: u64,
    subnet_costs: Vec<rest::RuntimeCostEvent>,
    subnet_cost_totals: BTreeMap<(String, String, String), rest::RuntimeCostTotal>,
    subnet_costs_truncated: bool,
    subnet_cost_totals_incomplete: bool,
    subnet_cost_generation: u64,
    subnet_recorded_bytes: usize,
    last_subnet_cost_sequence: Option<u64>,
    runtime_configurations: BTreeMap<String, BTreeMap<String, String>>,
    next_logical_execution: u64,
}

struct StoredModule {
    metadata: rest::WasmInstructionModuleMetadata,
    bytes: usize,
}

#[derive(Default)]
struct CanisterTrace {
    tracing: bool,
    runtime_costs_recording: bool,
    profile_remaining: u64,
    read_mask_remaining: u64,
    write_mask_remaining: u64,
    generation: u64,
    recorded_bytes: usize,
    truncated: bool,
    dropped_executions: u64,
    dropped_instructions: u128,
    dropped_instructions_incomplete: bool,
    executions: Vec<TracedExecution>,
    runtime_costs: Vec<rest::RuntimeCostEvent>,
    runtime_cost_totals: BTreeMap<(String, String, String), rest::RuntimeCostTotal>,
    runtime_costs_truncated: bool,
    runtime_cost_totals_incomplete: bool,
    last_runtime_cost_sequence: Option<u64>,
    pending_attempts: BTreeMap<u64, PendingAttempt>,
    pending_dispositions: BTreeMap<u64, (rest::TracedExecutionDisposition, usize)>,
    logical_executions: BTreeMap<u64, StoredLogicalExecution>,
    logical_retry_slots: BTreeMap<(String, String), u64>,
    lifecycle_details_truncated: bool,
}

struct PendingAttempt {
    execution: TracedExecution,
    bytes: usize,
}

struct StoredLogicalExecution {
    context: rest::TracedLogicalExecution,
    bytes: usize,
}

const MAX_PENDING_ATTEMPTS_PER_CANISTER: usize = 256;

pub(crate) struct RuntimeCostSnapshot {
    pub runtime_costs_recording: bool,
    pub runtime_costs: Vec<rest::RuntimeCostEvent>,
    pub subnet_runtime_costs: Vec<rest::RuntimeCostEvent>,
    pub runtime_cost_totals: Vec<rest::RuntimeCostTotal>,
    pub subnet_runtime_cost_totals: Vec<rest::RuntimeCostTotal>,
    pub runtime_costs_truncated: bool,
    pub subnet_runtime_costs_truncated: bool,
    pub runtime_cost_totals_incomplete: bool,
    pub subnet_runtime_cost_totals_incomplete: bool,
    pub last_runtime_cost_sequence_number: Option<u64>,
    pub next_after: Option<u64>,
    pub more: bool,
    pub trace_generation: u64,
    pub runtime_configurations: BTreeMap<String, BTreeMap<String, String>>,
}

pub(crate) struct TraceSnapshot {
    pub trace_generation: u64,
    pub first_sequence_number: Option<u64>,
    pub last_sequence_number: Option<u64>,
    pub executions: Vec<TracedExecution>,
    pub modules: BTreeMap<String, rest::WasmInstructionModuleMetadata>,
    pub logical_executions: BTreeMap<String, rest::TracedLogicalExecution>,
    pub more: bool,
    pub truncated: bool,
    pub dropped_executions: u64,
    pub dropped_instructions: u128,
    pub dropped_instructions_incomplete: bool,
    pub recorded_bytes: usize,
}

#[derive(Default)]
pub(crate) struct ExecutionFilter<'a> {
    pub after: Option<u64>,
    pub before: Option<u64>,
    pub function: Option<&'a str>,
    pub outcome: Option<TracedExecutionOutcome>,
    pub module_hash: Option<&'a str>,
    pub attempt_id: Option<u64>,
}

impl ExecutionFilter<'_> {
    pub(crate) fn matches(&self, execution: &TracedExecution) -> bool {
        self.after
            .is_none_or(|after| execution.sequence_number > after)
            && self
                .before
                .is_none_or(|before| execution.sequence_number < before)
            && self
                .function
                .is_none_or(|function| execution.function.contains(function))
            && self
                .outcome
                .is_none_or(|outcome| execution.outcome == outcome)
            && self
                .module_hash
                .is_none_or(|hash| execution.module_hash.as_deref() == Some(hash))
            && self.attempt_id.is_none_or(|id| {
                execution
                    .attempt
                    .as_ref()
                    .is_some_and(|attempt| attempt.attempt_id == id)
            })
    }
}

impl MemoryAccessTraceRecorder {
    /// Sets the retained host-memory estimate budget for this instance. This
    /// does not alter canister memory fees or execution budgets.
    pub(crate) fn with_memory_limit(bytes: usize) -> Self {
        assert!(
            bytes >= 1024 * 1024,
            "Trace memory limit must be at least 1 MiB"
        );
        Self {
            traces: Mutex::default(),
            memory_limit: bytes,
            tracing_active: AtomicBool::new(false),
            costs_active: AtomicBool::new(false),
        }
    }

    /// Sparse positive coverage for a captured operation window. A range union
    /// never scans allocation-sized memory or infers touches from changed bytes.
    pub(crate) fn snapshot_touched_pages(
        &self,
        canister_id: CanisterId,
        trace_generation: u64,
        after: Option<u64>,
        through: Option<u64>,
        attempt_id: Option<u64>,
        logical_execution_id: Option<u64>,
    ) -> serde_json::Value {
        use serde_json::json;
        fn union(mut ranges: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
            ranges.retain(|&(start, end)| start < end);
            ranges.sort_unstable();
            let mut result: Vec<(u64, u64)> = Vec::new();
            for (start, end) in ranges {
                if let Some(last) = result.last_mut()
                    && start <= last.1
                {
                    last.1 = last.1.max(end);
                } else {
                    result.push((start, end));
                }
            }
            result
        }
        fn encoded(ranges: Vec<(u64, u64)>) -> serde_json::Value {
            json!(
                union(ranges)
                    .into_iter()
                    .map(|(start, end)| [start.to_string(), end.to_string()])
                    .collect::<Vec<_>>()
            )
        }
        let traces = self.traces.lock().unwrap();
        let Some(trace) = traces.canisters.get(&canister_id) else {
            return json!({"available": false, "reason": "No retained execution trace; enable tracing before the operation and both captures."});
        };
        if trace
            .generation
            .saturating_add(traces.subnet_cost_generation)
            != trace_generation
            || through.is_none()
            || after.zip(through).is_some_and(|(start, end)| start > end)
            || (attempt_id.is_some() && logical_execution_id.is_some())
        {
            return json!({"available": false, "reason": "Trace generation or cursor changed, no completed trace cursor is available, or selection IDs conflict."});
        }
        let mut heap = Vec::new();
        let mut heap_dirty = Vec::new();
        let mut stable = Vec::new();
        let mut stable_dirty = Vec::new();
        let mut count = 0_u64;
        let mut attempts = Vec::new();
        let mut heap_complete = !trace.truncated && trace.dropped_executions == 0;
        let mut stable_complete = heap_complete;
        let mut dirty_complete = heap_complete;
        let mut fee_complete = heap_complete;
        let mut heap_fees = 0_u128;
        let mut stable_fees = 0_u128;
        let mut selected_instructions = 0_u128;
        let mut instructions_complete = heap_complete;
        let mut read_ranges = Vec::new();
        let mut read_profiles = 0_u64;
        let mut observed_reads = 0_u64;
        let mut observed_read_bytes = 0_u64;
        let mut reads_truncated = !heap_complete;
        let mut read_counters_saturated = false;
        let mut read_coverage = std::collections::BTreeSet::new();
        const MAX_WINDOW_READ_RANGES: usize = 131_072;
        let mut write_ranges = Vec::new();
        let mut write_profiles = 0_u64;
        let mut observed_writes = 0_u64;
        let mut observed_write_bytes = 0_u64;
        let mut writes_truncated = !heap_complete;
        let mut write_counters_saturated = false;
        let mut write_coverage = std::collections::BTreeSet::new();
        const MAX_WINDOW_WRITE_RANGES: usize = 131_072;
        for execution in trace.executions.iter().filter(|execution| {
            after.is_none_or(|after| execution.sequence_number > after)
                && through.is_some_and(|end| execution.sequence_number <= end)
                && attempt_id.is_none_or(|id| {
                    execution
                        .attempt
                        .as_ref()
                        .is_some_and(|attempt| attempt.attempt_id == id)
                })
                && logical_execution_id.is_none_or(|id| {
                    execution
                        .disposition
                        .as_ref()
                        .is_some_and(|disposition| disposition.logical_execution_id == Some(id))
                })
        }) {
            count += 1;
            if let Some(reads) = &execution.read_profile {
                read_profiles += 1;
                read_counters_saturated |= reads.counters_saturated
                    || observed_reads.checked_add(reads.observed_reads).is_none()
                    || observed_read_bytes
                        .checked_add(reads.observed_bytes)
                        .is_none();
                observed_reads = observed_reads.saturating_add(reads.observed_reads);
                observed_read_bytes = observed_read_bytes.saturating_add(reads.observed_bytes);
                reads_truncated |= reads.truncated || execution.detail_truncated;
                read_coverage.extend(reads.coverage.iter().cloned());
                let remaining = MAX_WINDOW_READ_RANGES.saturating_sub(read_ranges.len());
                reads_truncated |= reads.ranges.len() > remaining;
                read_ranges.extend(
                    reads
                        .ranges
                        .iter()
                        .take(remaining)
                        .map(|range| json!(range)),
                );
                for range in &reads.ranges {
                    if range.start < range.end
                        && (range.heap_reads > 0
                            || range.stable_direct_reads > 0
                            || range.stable_native_reads > 0)
                    {
                        let pages = (range.start / 65_536, (range.end - 1) / 65_536 + 1);
                        match range.memory.as_str() {
                            "heap" => heap.push(pages),
                            "stable" => stable.push(pages),
                            _ => {}
                        }
                    }
                }
            }
            if let Some(writes) = &execution.write_profile {
                write_profiles += 1;
                write_counters_saturated |= writes.counters_saturated
                    || observed_writes
                        .checked_add(writes.observed_writes)
                        .is_none()
                    || observed_write_bytes
                        .checked_add(writes.observed_bytes)
                        .is_none();
                observed_writes = observed_writes.saturating_add(writes.observed_writes);
                observed_write_bytes = observed_write_bytes.saturating_add(writes.observed_bytes);
                writes_truncated |= writes.truncated || execution.detail_truncated;
                write_coverage.extend(writes.coverage.iter().cloned());
                let remaining = MAX_WINDOW_WRITE_RANGES.saturating_sub(write_ranges.len());
                writes_truncated |= writes.ranges.len() > remaining;
                write_ranges.extend(
                    writes
                        .ranges
                        .iter()
                        .take(remaining)
                        .map(|range| json!(range)),
                );
                for range in &writes.ranges {
                    if range.start < range.end && (range.heap_writes > 0 || range.stable_writes > 0)
                    {
                        let pages = (range.start / 65_536, (range.end - 1) / 65_536 + 1);
                        match range.memory.as_str() {
                            "heap" => heap.push(pages),
                            "stable" => stable.push(pages),
                            _ => {}
                        }
                    }
                }
            }
            let memory_available = !execution.detail_truncated
                && execution
                    .attempt
                    .as_ref()
                    .is_none_or(|attempt| attempt.memory_details_available);
            heap_complete &= memory_available;
            stable_complete &= memory_available && execution.stable_pages_tracked;
            dirty_complete &= memory_available && execution.dirty_pages_tracked;
            instructions_complete &= execution
                .attempt
                .as_ref()
                .is_none_or(|attempt| attempt.instructions_available);
            heap.extend_from_slice(&execution.accessed_pages);
            heap_dirty.extend_from_slice(&execution.dirty_pages);
            if execution.stable_pages_tracked {
                stable.extend_from_slice(&execution.stable_accessed_pages);
                stable_dirty.extend_from_slice(&execution.stable_dirty_pages);
            }
            heap_fees += u128::from(execution.page_access_instructions);
            selected_instructions += u128::from(execution.instructions);
            if let Some(profile) = &execution.instruction_profile {
                stable_fees += u128::from(profile.ledger.stable_page_instructions);
                fee_complete &= !profile.truncated;
            } else {
                fee_complete = false;
            }
            if attempts.len() < 128 {
                attempts.push(json!({
                    "sequence_number": execution.sequence_number.to_string(),
                    "attempt": execution.attempt,
                    "disposition": execution.disposition,
                    "function": execution.function,
                    "api_type": execution.api_type,
                    "outcome": execution.outcome,
                    "instructions": execution.instructions.to_string(),
                    "raw_heap_page_instructions": execution.page_access_instructions.to_string(),
                    "stable_page_instructions": execution.instruction_profile.as_ref().map(|profile| profile.ledger.stable_page_instructions.to_string()),
                    "memory_details_available": memory_available,
                    "stable_pages_tracked": execution.stable_pages_tracked,
                }));
            }
        }
        json!({
            "available": true,
            "window_after": after.map(|v| v.to_string()), "window_through": through.map(|v| v.to_string()),
            "attempt_id": attempt_id.map(|v| v.to_string()), "logical_execution_id": logical_execution_id.map(|v| v.to_string()),
            "execution_count": count.to_string(), "attempts": attempts, "attempt_list_truncated": count > 128,
            "known_touched_ranges": {"heap": encoded(heap), "stable": encoded(stable)},
            "known_dirty_ranges": {"heap": encoded(heap_dirty), "stable": encoded(stable_dirty)},
            "read_profile": {
                "available": read_profiles > 0,
                "ranges": read_ranges,
                "observed_reads": observed_reads.to_string(),
                "observed_bytes": observed_read_bytes.to_string(),
                "profiled_executions": read_profiles.to_string(),
                "missing_executions": count.saturating_sub(read_profiles).to_string(),
                "truncated": reads_truncated || read_profiles < count,
                "counters_saturated": read_counters_saturated,
                "coverage": read_coverage,
                "max_window_ranges": MAX_WINDOW_READ_RANGES.to_string(),
                "retained_coverage_complete": count > 0 && read_profiles == count && !reads_truncated && !read_counters_saturated,
                "semantics": "Successful logical byte reads from explicitly selected read-profile executions, independent of contents and native page fees. Counts measure read operations covering each byte; they are not CPU cache accesses. Missing executions or truncated ranges leave absence unknown."
            },
            "write_profile": {
                "available": write_profiles > 0,
                "ranges": write_ranges,
                "observed_writes": observed_writes.to_string(),
                "observed_bytes": observed_write_bytes.to_string(),
                "profiled_executions": write_profiles.to_string(),
                "missing_executions": count.saturating_sub(write_profiles).to_string(),
                "truncated": writes_truncated || write_profiles < count,
                "counters_saturated": write_counters_saturated,
                "coverage": write_coverage,
                "max_window_ranges": MAX_WINDOW_WRITE_RANGES.to_string(),
                "retained_coverage_complete": count > 0 && write_profiles == count && !writes_truncated && !write_counters_saturated,
                "semantics": "Completed logical byte writes from explicitly selected write-profile executions, independent of contents, net changes and dirty-page fees. Includes same-value writes and writes subsequently restored. Missing executions or truncated ranges leave absence unknown."
            },
            "retained_heap_coverage_complete": heap_complete && count > 0,
            "retained_stable_coverage_complete": stable_complete && count > 0,
            "retained_heap_dirty_coverage_complete": dirty_complete && count > 0,
            "continuous_recording_proven": false,
            "granularity_bytes": "65536",
            "instructions": selected_instructions.to_string(), "instructions_complete": instructions_complete,
            "raw_heap_page_instructions": heap_fees.to_string(),
            "heap_page_fee_total_complete": heap_complete && count > 0,
            "known_stable_page_instructions": stable_fees.to_string(), "stable_page_fee_total_complete": fee_complete && count > 0,
            "semantics": "Positive 64 KiB region coverage from native access tracking and opt-in logical byte-read/write profiles. Logical access ranges may include JIT-elided native accesses; actual page fees and native dirty ranges remain separate observations. Stable coverage includes direct first reads when stable_pages_tracked is true. Byte differences do not establish access history. Missing attempts and recording gaps keep absence unknown."
        })
    }

    fn logical_dictionary(
        trace: Option<&CanisterTrace>,
    ) -> BTreeMap<String, rest::TracedLogicalExecution> {
        trace
            .into_iter()
            .flat_map(|trace| &trace.logical_executions)
            .map(|(id, stored)| (id.to_string(), stored.context.clone()))
            .collect()
    }

    fn publish_attempt(
        traces: &mut Traces,
        canister_id: CanisterId,
        mut execution: TracedExecution,
    ) {
        execution.sequence_number = traces.next_sequence_number;
        traces.next_sequence_number = traces.next_sequence_number.saturating_add(1);
        traces
            .canisters
            .entry(canister_id)
            .or_default()
            .executions
            .push(execution);
    }

    fn collect_unreferenced_logical(trace: &mut CanisterTrace) -> usize {
        let referenced: std::collections::BTreeSet<u64> = trace
            .executions
            .iter()
            .chain(
                trace
                    .pending_attempts
                    .values()
                    .map(|pending| &pending.execution),
            )
            .filter_map(|execution| execution.disposition.as_ref()?.logical_execution_id)
            .chain(
                trace
                    .pending_dispositions
                    .values()
                    .filter_map(|(disposition, _)| disposition.logical_execution_id),
            )
            .chain(trace.logical_retry_slots.values().copied())
            .collect();
        let mut released = 0_usize;
        trace.logical_executions.retain(|id, stored| {
            if referenced.contains(id) {
                true
            } else {
                released = released.saturating_add(stored.bytes);
                false
            }
        });
        released
    }

    /// Starts tracing; retained records and budget state remain available.
    pub(crate) fn start(&self, canister_id: CanisterId) -> bool {
        let mut traces = self.traces.lock().unwrap();
        if traces.recorded_bytes < self.memory_limit - TRACE_SUMMARY_RESERVE {
            let trace = traces.canisters.entry(canister_id).or_default();
            trace.tracing = true;
            trace.runtime_costs_recording = true;
            self.tracing_active.store(true, Ordering::Release);
            self.costs_active.store(true, Ordering::Release);
            true
        } else {
            false
        }
    }

    /// Selects the next bounded window without changing IC execution budgets.
    #[cfg(test)]
    pub(crate) fn start_profiling(&self, canister_id: CanisterId, max_executions: u64) -> bool {
        self.start_profiling_with_reads(canister_id, max_executions, false)
    }

    #[cfg(test)]
    pub(crate) fn start_profiling_with_reads(
        &self,
        canister_id: CanisterId,
        max_executions: u64,
        read_mask: bool,
    ) -> bool {
        self.start_profiling_masks(canister_id, max_executions, read_mask, false)
    }

    pub(crate) fn start_profiling_masks(
        &self,
        canister_id: CanisterId,
        max_executions: u64,
        read_mask: bool,
        write_mask: bool,
    ) -> bool {
        let mut traces = self.traces.lock().unwrap();
        if traces.recorded_bytes >= self.memory_limit - TRACE_SUMMARY_RESERVE {
            return false;
        }
        let trace = traces.canisters.entry(canister_id).or_default();
        trace.tracing = true;
        trace.runtime_costs_recording = true;
        trace.profile_remaining = max_executions.clamp(1, 10_000);
        trace.read_mask_remaining = if read_mask {
            trace.profile_remaining
        } else {
            0
        };
        trace.write_mask_remaining = if write_mask {
            trace.profile_remaining
        } else {
            0
        };
        self.tracing_active.store(true, Ordering::Release);
        self.costs_active.store(true, Ordering::Release);
        true
    }

    pub(crate) fn stop(&self, canister_id: CanisterId) {
        let mut traces = self.traces.lock().unwrap();
        if let Some(trace) = traces.canisters.get_mut(&canister_id) {
            trace.tracing = false;
            trace.runtime_costs_recording = false;
            trace.profile_remaining = 0;
            trace.read_mask_remaining = 0;
            trace.write_mask_remaining = 0;
        }
        self.tracing_active.store(
            traces.canisters.values().any(|trace| trace.tracing),
            Ordering::Release,
        );
        self.costs_active.store(
            traces
                .canisters
                .values()
                .any(|trace| trace.runtime_costs_recording),
            Ordering::Release,
        );
    }

    /// Small immutable capture cursor; unlike `get`, this never clones trace
    /// events, profiles or module dictionaries.
    pub(crate) fn snapshot_cursor(
        &self,
        canister_id: CanisterId,
    ) -> (u64, Option<u64>, Option<u64>, bool) {
        let traces = self.traces.lock().unwrap();
        let trace = traces.canisters.get(&canister_id);
        (
            trace
                .map_or(0, |trace| trace.generation)
                .saturating_add(traces.subnet_cost_generation),
            trace
                .and_then(|trace| trace.executions.last())
                .map(|execution| execution.sequence_number),
            trace
                .and_then(|trace| trace.last_runtime_cost_sequence)
                .max(traces.last_subnet_cost_sequence),
            trace.is_some_and(|trace| trace.tracing),
        )
    }

    pub(crate) fn get(&self, canister_id: CanisterId) -> MemoryAccessTrace {
        let identity = server_identity();
        let traces = self.traces.lock().unwrap();
        let trace = traces.canisters.get(&canister_id);
        MemoryAccessTrace {
            format_version: 2,
            charge_rules: crate::instruction_charge_rules::charge_rules().clone(),
            server_generation: identity.generation.clone(),
            server_binary_hash: identity.binary_hash.clone(),
            trace_generation: trace
                .map_or(0, |t| t.generation)
                .saturating_add(traces.subnet_cost_generation),
            truncated: trace.is_some_and(|t| t.truncated),
            dropped_executions: trace.map_or(0, |t| t.dropped_executions),
            dropped_instructions: trace.map_or(0, |t| t.dropped_instructions),
            dropped_instructions_incomplete: trace
                .is_some_and(|t| t.dropped_instructions_incomplete),
            recorded_bytes: trace.map_or(0, |t| Self::retained_bytes(&traces, t) as u64),
            wasm_page_size: WASM_PAGE_SIZE_IN_BYTES as u64,
            executions: trace.map_or_else(Vec::new, |t| t.executions.clone()),
            runtime_costs_recording: trace.is_some_and(|t| t.runtime_costs_recording),
            runtime_costs: trace.map_or_else(Vec::new, |t| t.runtime_costs.clone()),
            subnet_runtime_costs: traces.subnet_costs.clone(),
            runtime_cost_totals: trace.map_or_else(Vec::new, |t| {
                t.runtime_cost_totals.values().cloned().collect()
            }),
            subnet_runtime_cost_totals: traces.subnet_cost_totals.values().cloned().collect(),
            runtime_costs_truncated: trace.is_some_and(|t| t.runtime_costs_truncated),
            subnet_runtime_costs_truncated: traces.subnet_costs_truncated,
            runtime_cost_totals_incomplete: trace.is_some_and(|t| t.runtime_cost_totals_incomplete),
            subnet_runtime_cost_totals_incomplete: traces.subnet_cost_totals_incomplete,
            last_runtime_cost_sequence_number: trace
                .and_then(|t| t.last_runtime_cost_sequence)
                .max(traces.last_subnet_cost_sequence),
            runtime_configurations: traces.runtime_configurations.clone(),
            modules: trace.map_or_else(BTreeMap::new, |t| {
                Self::referenced_modules(&traces, &t.executions)
            }),
            logical_executions: Self::logical_dictionary(trace),
        }
    }

    fn retained_bytes(traces: &Traces, trace: &CanisterTrace) -> usize {
        let hashes: std::collections::BTreeSet<_> = trace
            .executions
            .iter()
            .filter(|e| e.instruction_profile.is_some())
            .filter_map(|e| e.module_hash.as_ref())
            .collect();
        hashes
            .into_iter()
            .filter_map(|hash| traces.modules.get(hash))
            .fold(trace.recorded_bytes, |bytes, module| {
                bytes.saturating_add(module.bytes)
            })
    }

    fn referenced_modules(
        traces: &Traces,
        executions: &[TracedExecution],
    ) -> BTreeMap<String, rest::WasmInstructionModuleMetadata> {
        let hashes: std::collections::BTreeSet<_> = executions
            .iter()
            .filter(|e| e.instruction_profile.is_some())
            .filter_map(|e| e.module_hash.as_ref())
            .collect();
        hashes
            .into_iter()
            .filter_map(|hash| {
                traces
                    .modules
                    .get(hash)
                    .map(|module| (hash.clone(), module.metadata.clone()))
            })
            .collect()
    }

    fn clear_locked(traces: &mut Traces, canister_id: CanisterId) {
        if let Some(trace) = traces.canisters.get_mut(&canister_id) {
            let previous_bytes = trace.recorded_bytes;
            trace.executions = Vec::new();
            trace.runtime_costs = Vec::new();
            trace.runtime_cost_totals.clear();
            trace.runtime_costs_truncated = false;
            trace.runtime_cost_totals_incomplete = false;
            trace.last_runtime_cost_sequence = None;
            Self::collect_unreferenced_logical(trace);
            // In-flight executions can finish after a clear. Their retained
            // profiles, exact pending dispositions and active logical contexts
            // remain counted until they publish or the next explicit clear.
            trace.recorded_bytes = trace
                .pending_attempts
                .values()
                .map(|p| p.bytes)
                .sum::<usize>()
                .saturating_add(
                    trace
                        .pending_dispositions
                        .values()
                        .map(|(_, bytes)| bytes)
                        .sum::<usize>(),
                )
                .saturating_add(
                    trace
                        .logical_executions
                        .values()
                        .map(|s| s.bytes)
                        .sum::<usize>(),
                );
            traces.recorded_bytes = traces
                .recorded_bytes
                .saturating_sub(previous_bytes)
                .saturating_add(trace.recorded_bytes);
            trace.generation = trace.generation.saturating_add(1);
            trace.truncated = false;
            trace.dropped_executions = 0;
            trace.dropped_instructions = 0;
            trace.dropped_instructions_incomplete = false;
            trace.lifecycle_details_truncated = false;
        }
        if traces.canisters.values().all(|trace| {
            trace.executions.is_empty()
                && trace.runtime_costs.is_empty()
                && trace.runtime_cost_totals.is_empty()
        }) {
            traces.recorded_bytes = traces
                .recorded_bytes
                .saturating_sub(traces.subnet_recorded_bytes);
            if traces.last_subnet_cost_sequence.is_some() {
                traces.subnet_cost_generation = traces.subnet_cost_generation.saturating_add(1);
            }
            traces.subnet_recorded_bytes = 0;
            traces.subnet_costs = Vec::new();
            traces.subnet_cost_totals.clear();
            traces.subnet_costs_truncated = false;
            traces.subnet_cost_totals_incomplete = false;
            traces.last_subnet_cost_sequence = None;
        }
        Self::collect_unreferenced_modules(traces);
    }

    fn collect_unreferenced_modules(traces: &mut Traces) {
        let referenced: std::collections::BTreeSet<_> = traces
            .canisters
            .values()
            .flat_map(|trace| {
                trace.executions.iter().chain(
                    trace
                        .pending_attempts
                        .values()
                        .map(|pending| &pending.execution),
                )
            })
            .filter(|e| e.instruction_profile.is_some())
            .filter_map(|e| e.module_hash.as_ref())
            .collect();
        let mut released = 0_usize;
        traces.modules.retain(|hash, module| {
            let keep = referenced.contains(hash);
            if !keep {
                released = released.saturating_add(module.bytes);
            }
            keep
        });
        traces.recorded_bytes = traces.recorded_bytes.saturating_sub(released);
    }

    /// Clears completed records. Executions already in flight may finish later.
    pub(crate) fn clear(&self, canister_id: CanisterId) {
        Self::clear_locked(&mut self.traces.lock().unwrap(), canister_id);
    }

    pub(crate) fn clear_if_unchanged(
        &self,
        canister_id: CanisterId,
        expected_generation: u64,
        expected_last_sequence: Option<u64>,
        expected_last_runtime_cost_sequence: Option<Option<u64>>,
    ) -> bool {
        let mut traces = self.traces.lock().unwrap();
        let trace = traces.canisters.get(&canister_id);
        if trace
            .map_or(0, |t| t.generation)
            .saturating_add(traces.subnet_cost_generation)
            != expected_generation
            || trace
                .and_then(|t| t.executions.last())
                .map(|e| e.sequence_number)
                != expected_last_sequence
            || expected_last_runtime_cost_sequence.is_some_and(|expected| {
                trace
                    .and_then(|t| t.last_runtime_cost_sequence)
                    .max(traces.last_subnet_cost_sequence)
                    != expected
            })
        {
            return false;
        }
        Self::clear_locked(&mut traces, canister_id);
        true
    }

    pub(crate) fn canisters(&self) -> Vec<TracedCanister> {
        let traces = self.traces.lock().unwrap();
        traces
            .canisters
            .iter()
            .map(|(id, trace)| TracedCanister {
                canister_id: *id,
                tracing: trace.tracing,
                runtime_costs_recording: trace.runtime_costs_recording,
                executions: trace.executions.len(),
                first_sequence_number: trace.executions.first().map(|e| e.sequence_number),
                last_sequence_number: trace.executions.last().map(|e| e.sequence_number),
                trace_generation: trace
                    .generation
                    .saturating_add(traces.subnet_cost_generation),
                truncated: trace.truncated,
                profile_remaining: trace.profile_remaining,
            })
            .collect()
    }

    /// All pagination/reset metadata is captured under the same lock.
    pub(crate) fn executions_snapshot(
        &self,
        canister_id: CanisterId,
        limit: usize,
        filter: ExecutionFilter<'_>,
    ) -> TraceSnapshot {
        let traces = self.traces.lock().unwrap();
        let trace = traces.canisters.get(&canister_id);
        let mut snapshot = TraceSnapshot {
            trace_generation: trace
                .map_or(0, |t| t.generation)
                .saturating_add(traces.subnet_cost_generation),
            first_sequence_number: trace
                .and_then(|t| t.executions.first())
                .map(|e| e.sequence_number),
            last_sequence_number: trace
                .and_then(|t| t.executions.last())
                .map(|e| e.sequence_number),
            executions: vec![],
            modules: BTreeMap::new(),
            logical_executions: Self::logical_dictionary(trace),
            more: false,
            truncated: trace.is_some_and(|t| t.truncated),
            dropped_executions: trace.map_or(0, |t| t.dropped_executions),
            dropped_instructions: trace.map_or(0, |t| t.dropped_instructions),
            dropped_instructions_incomplete: trace
                .is_some_and(|t| t.dropped_instructions_incomplete),
            recorded_bytes: trace.map_or(0, |t| Self::retained_bytes(&traces, t)),
        };
        if let Some(trace) = trace {
            let first = filter.after.map_or(0, |n| {
                trace.executions.partition_point(|e| e.sequence_number <= n)
            });
            let mut matching = trace.executions[first..]
                .iter()
                .take_while(|e| {
                    filter
                        .before
                        .is_none_or(|before| e.sequence_number < before)
                })
                .filter(|e| filter.matches(e));
            snapshot.executions = matching
                .by_ref()
                .take(limit.clamp(1, 50_000))
                .cloned()
                .collect();
            snapshot.more = matching.next().is_some();
            snapshot.modules = Self::referenced_modules(&traces, &snapshot.executions);
            let ids: std::collections::BTreeSet<String> = snapshot
                .executions
                .iter()
                .filter_map(|execution| {
                    execution
                        .disposition
                        .as_ref()?
                        .logical_execution_id
                        .map(|id| id.to_string())
                })
                .collect();
            snapshot.logical_executions.retain(|id, _| ids.contains(id));
        }
        snapshot
    }

    pub(crate) fn runtime_costs_snapshot(
        &self,
        canister_id: CanisterId,
        after: Option<u64>,
        limit: usize,
    ) -> RuntimeCostSnapshot {
        let traces = self.traces.lock().unwrap();
        let trace = traces.canisters.get(&canister_id);
        let mut events: Vec<_> = trace
            .into_iter()
            .flat_map(|t| &t.runtime_costs)
            .chain(traces.subnet_costs.iter())
            .filter(|event| after.is_none_or(|after| event.sequence_number > after))
            .collect();
        events.sort_unstable_by_key(|event| event.sequence_number);
        let limit = limit.clamp(1, 50_000);
        let more = events.len() > limit;
        let next_after = events
            .iter()
            .take(limit)
            .last()
            .map(|event| event.sequence_number)
            .or(after);
        let (mut runtime_costs, mut subnet_runtime_costs) = (Vec::new(), Vec::new());
        for event in events.into_iter().take(limit) {
            if event.canister_id.is_some() {
                runtime_costs.push(event.clone());
            } else {
                subnet_runtime_costs.push(event.clone());
            }
        }
        RuntimeCostSnapshot {
            runtime_costs_recording: trace.is_some_and(|t| t.runtime_costs_recording),
            runtime_costs,
            subnet_runtime_costs,
            runtime_cost_totals: trace.map_or_else(Vec::new, |t| {
                t.runtime_cost_totals.values().cloned().collect()
            }),
            subnet_runtime_cost_totals: traces.subnet_cost_totals.values().cloned().collect(),
            runtime_costs_truncated: trace.is_some_and(|t| t.runtime_costs_truncated),
            subnet_runtime_costs_truncated: traces.subnet_costs_truncated,
            runtime_cost_totals_incomplete: trace.is_some_and(|t| t.runtime_cost_totals_incomplete),
            subnet_runtime_cost_totals_incomplete: traces.subnet_cost_totals_incomplete,
            last_runtime_cost_sequence_number: trace
                .and_then(|t| t.last_runtime_cost_sequence)
                .max(traces.last_subnet_cost_sequence),
            next_after,
            more,
            trace_generation: trace
                .map_or(0, |t| t.generation)
                .saturating_add(traces.subnet_cost_generation),
            runtime_configurations: traces.runtime_configurations.clone(),
        }
    }

    fn retain_runtime_cost(
        &self,
        canister_id: Option<CanisterId>,
        subnet_id: Option<SubnetId>,
        cost: TracedRuntimeCost,
    ) {
        if !self.costs_active.load(Ordering::Acquire) {
            return;
        }
        let mut traces = self.traces.lock().unwrap();
        let enabled = match canister_id {
            Some(id) => traces
                .canisters
                .get(&id)
                .is_some_and(|trace| trace.runtime_costs_recording),
            None => traces
                .canisters
                .values()
                .any(|trace| trace.runtime_costs_recording),
        };
        if !enabled {
            return;
        }
        let upstream_truncated = cost
            .details
            .get("detail_truncated")
            .is_some_and(|v| v == "true")
            || cost.details.contains_key("omitted_runtime_events");
        let upstream_totals_incomplete = cost
            .details
            .get("totals_incomplete")
            .is_some_and(|v| v == "true")
            || cost.details.contains_key("omitted_runtime_events");
        let event_count = cost
            .details
            .get("aggregated_event_count")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1);
        let sequence_number = traces.next_runtime_cost_sequence;
        traces.next_runtime_cost_sequence = traces.next_runtime_cost_sequence.saturating_add(1);
        let event = rest::RuntimeCostEvent {
            sequence_number,
            canister_id: canister_id.map(|id| id.to_string()),
            subnet_id: subnet_id
                .map(|id| id.to_string())
                .or_else(|| cost.details.get("subnet_id").cloned()),
            rule_id: cost.rule_id.to_string(),
            unit: cost.unit.as_str().to_string(),
            kind: cost.kind.as_str().to_string(),
            amount: cost.amount,
            nominal_cycles: cost.nominal_cycles,
            time_nanos: cost.time.map(|time| time.as_nanos_since_unix_epoch()),
            round: cost.round,
            call_context_id: cost.call_context_id.map(|id| id.get()),
            message_id: cost.message_id,
            callback_id: cost.callback_id,
            payload_bytes: cost.payload_bytes,
            instructions: cost.instructions,
            description: cost.description,
            details: cost.details,
        };
        let totals = match canister_id {
            Some(id) => &mut traces.canisters.entry(id).or_default().runtime_cost_totals,
            None => &mut traces.subnet_cost_totals,
        };
        let key = (
            event.rule_id.clone(),
            event.unit.clone(),
            event.kind.clone(),
        );
        let total = totals.entry(key).or_insert_with(|| rest::RuntimeCostTotal {
            rule_id: event.rule_id.clone(),
            unit: event.unit.clone(),
            kind: event.kind.clone(),
            events: 0,
            amount: 0,
            nominal_cycles: 0,
        });
        let overflowed = total.events.checked_add(event_count).is_none()
            || total.amount.checked_add(event.amount).is_none()
            || total
                .nominal_cycles
                .checked_add(event.nominal_cycles.unwrap_or_default())
                .is_none();
        total.events = total.events.saturating_add(event_count);
        total.amount = total.amount.saturating_add(event.amount);
        total.nominal_cycles = total
            .nominal_cycles
            .saturating_add(event.nominal_cycles.unwrap_or_default());
        let bytes = std::mem::size_of::<rest::RuntimeCostEvent>()
            .saturating_mul(2)
            .saturating_add(
                serde_json::to_vec(&event)
                    .map_or(self.memory_limit, |bytes| bytes.len().saturating_mul(4)),
            );
        let retained =
            traces.recorded_bytes.saturating_add(bytes) < self.memory_limit - TRACE_SUMMARY_RESERVE;
        if retained {
            traces.recorded_bytes = traces.recorded_bytes.saturating_add(bytes);
        }
        match canister_id {
            Some(id) => {
                let trace = traces.canisters.get_mut(&id).unwrap();
                trace.last_runtime_cost_sequence = Some(sequence_number);
                trace.runtime_costs_truncated |= !retained || overflowed || upstream_truncated;
                trace.runtime_cost_totals_incomplete |= overflowed || upstream_totals_incomplete;
                if retained {
                    trace.recorded_bytes = trace.recorded_bytes.saturating_add(bytes);
                    trace.runtime_costs.push(event);
                }
            }
            None => {
                traces.last_subnet_cost_sequence = Some(sequence_number);
                traces.subnet_costs_truncated |= !retained || overflowed || upstream_truncated;
                traces.subnet_cost_totals_incomplete |= overflowed || upstream_totals_incomplete;
                if retained {
                    traces.subnet_recorded_bytes =
                        traces.subnet_recorded_bytes.saturating_add(bytes);
                    traces.subnet_costs.push(event);
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn executions_after(
        &self,
        canister_id: CanisterId,
        after: Option<u64>,
        limit: usize,
        function: Option<&str>,
    ) -> (Vec<TracedExecution>, bool) {
        let snapshot = self.executions_snapshot(
            canister_id,
            limit,
            ExecutionFilter {
                after,
                function,
                ..Default::default()
            },
        );
        (snapshot.executions, snapshot.more)
    }
}

pub(crate) struct TracedCanister {
    pub canister_id: CanisterId,
    pub tracing: bool,
    pub runtime_costs_recording: bool,
    pub executions: usize,
    pub first_sequence_number: Option<u64>,
    pub last_sequence_number: Option<u64>,
    pub trace_generation: u64,
    pub truncated: bool,
    pub profile_remaining: u64,
}

impl WasmMemoryAccessTracer for MemoryAccessTraceRecorder {
    fn begin_logical_execution(
        &self,
        canister_id: CanisterId,
        context: TracedLogicalExecution,
    ) -> Option<u64> {
        if !self.tracing_active.load(Ordering::Acquire) {
            return None;
        }
        let mut traces = self.traces.lock().unwrap();
        let trace = traces.canisters.get(&canister_id)?;
        if !trace.tracing {
            return None;
        }
        let key = (context.kind.to_string(), context.identity.clone());
        if context.retry {
            if let Some(id) = trace.logical_retry_slots.get(&key) {
                return Some(*id);
            }
        }
        // Never shorten an identity and accidentally join different messages.
        if context.identity.len() > 1024
            || context
                .message_id
                .as_ref()
                .is_some_and(|id| id.len() > 1024)
            || trace.logical_retry_slots.len() >= MAX_PENDING_ATTEMPTS_PER_CANISTER
        {
            let trace = traces.canisters.get_mut(&canister_id).unwrap();
            trace.truncated = true;
            trace.lifecycle_details_truncated = true;
            return None;
        }
        let recorded = rest::TracedLogicalExecution {
            identity: context.identity,
            kind: context.kind.to_string(),
            retry: context.retry,
            message_id: context.message_id,
            callback_id: context.callback_id,
            parent_call_context_id: context.parent_call_context_id,
            time_nanos: context.time.as_nanos_since_unix_epoch(),
        };
        let bytes = serde_json::to_vec(&recorded)
            .ok()?
            .len()
            .saturating_mul(4)
            .saturating_add(512);
        if traces.recorded_bytes.saturating_add(bytes) > self.memory_limit - TRACE_SUMMARY_RESERVE {
            let trace = traces.canisters.get_mut(&canister_id).unwrap();
            trace.truncated = true;
            trace.lifecycle_details_truncated = true;
            return None;
        }
        let id = traces.next_logical_execution.checked_add(1)?;
        traces.next_logical_execution = id;
        traces.recorded_bytes = traces.recorded_bytes.saturating_add(bytes);
        let trace = traces.canisters.get_mut(&canister_id).unwrap();
        trace.recorded_bytes = trace.recorded_bytes.saturating_add(bytes);
        trace.logical_executions.insert(
            id,
            StoredLogicalExecution {
                context: recorded,
                bytes,
            },
        );
        trace.logical_retry_slots.insert(key, id);
        Some(id)
    }

    fn finish_logical_execution(
        &self,
        canister_id: CanisterId,
        logical_execution_id: u64,
        retry: bool,
    ) {
        let mut traces = self.traces.lock().unwrap();
        if let Some(trace) = traces.canisters.get_mut(&canister_id) {
            if !retry {
                trace
                    .logical_retry_slots
                    .retain(|_, id| *id != logical_execution_id);
            }
            // A final disposition can precede an asynchronous abort record.
            // Keep its dictionary until the paired attempt is retained/cleared.
            let released = Self::collect_unreferenced_logical(trace);
            trace.recorded_bytes = trace.recorded_bytes.saturating_sub(released);
            traces.recorded_bytes = traces.recorded_bytes.saturating_sub(released);
        }
    }

    fn record_execution_disposition(
        &self,
        canister_id: CanisterId,
        attempt_id: u64,
        disposition: TracedExecutionDisposition,
    ) {
        if attempt_id == 0 {
            return;
        }
        let mut reason = disposition.reason;
        reason.truncate(reason.floor_char_boundary(512));
        let recorded = rest::TracedExecutionDisposition {
            logical_execution_id: disposition.logical_execution_id,
            state: match disposition.state {
                TracedExecutionState::Pending => rest::TracedExecutionState::Pending,
                TracedExecutionState::Committed => rest::TracedExecutionState::Committed,
                TracedExecutionState::Discarded => rest::TracedExecutionState::Discarded,
                TracedExecutionState::NotApplicable => rest::TracedExecutionState::NotApplicable,
            },
            reason,
        };
        let mut traces = self.traces.lock().unwrap();
        let Some(trace) = traces.canisters.get_mut(&canister_id) else {
            return;
        };
        if trace.executions.iter().any(|execution| {
            execution
                .attempt
                .as_ref()
                .is_some_and(|attempt| attempt.attempt_id == attempt_id)
        }) {
            // Published rows are immutable; duplicate terminal notifications do
            // not rewrite history behind a pagination/conditional-clear cursor.
            return;
        }
        if let Some((_, bytes)) = trace.pending_dispositions.remove(&attempt_id) {
            trace.recorded_bytes = trace.recorded_bytes.saturating_sub(bytes);
            traces.recorded_bytes = traces.recorded_bytes.saturating_sub(bytes);
        }
        if recorded.state != rest::TracedExecutionState::Pending {
            if let Some(mut pending) = traces
                .canisters
                .get_mut(&canister_id)
                .unwrap()
                .pending_attempts
                .remove(&attempt_id)
            {
                pending.execution.disposition = Some(recorded);
                Self::publish_attempt(&mut traces, canister_id, pending.execution);
                return;
            }
        }
        let bytes = serde_json::to_vec(&recorded).map_or(self.memory_limit, |v| {
            v.len().saturating_mul(4).saturating_add(128)
        });
        let trace = traces.canisters.get(&canister_id).unwrap();
        if trace.pending_dispositions.len() >= MAX_PENDING_ATTEMPTS_PER_CANISTER
            || traces.recorded_bytes.saturating_add(bytes) > self.memory_limit
        {
            let trace = traces.canisters.get_mut(&canister_id).unwrap();
            trace.truncated = true;
            trace.lifecycle_details_truncated = true;
            return;
        }
        traces.recorded_bytes = traces.recorded_bytes.saturating_add(bytes);
        let trace = traces.canisters.get_mut(&canister_id).unwrap();
        trace.recorded_bytes = trace.recorded_bytes.saturating_add(bytes);
        trace
            .pending_dispositions
            .insert(attempt_id, (recorded, bytes));
    }

    fn is_tracing_enabled(&self, canister_id: CanisterId) -> bool {
        if !self.tracing_active.load(Ordering::Acquire) {
            return false;
        }
        self.traces
            .lock()
            .unwrap()
            .canisters
            .get(&canister_id)
            .is_some_and(|t| t.tracing)
    }

    fn is_runtime_cost_tracing_enabled(&self, canister_id: CanisterId) -> bool {
        if !self.costs_active.load(Ordering::Acquire) {
            return false;
        }
        self.traces
            .lock()
            .unwrap()
            .canisters
            .get(&canister_id)
            .is_some_and(|t| t.runtime_costs_recording)
    }

    fn take_instruction_profile(&self, canister_id: CanisterId) -> bool {
        self.take_instruction_profile_mode(canister_id).0
    }

    fn take_instruction_profile_mode(&self, canister_id: CanisterId) -> (bool, bool) {
        let (profile, reads, _) = self.take_memory_profile_mode(canister_id);
        (profile, reads)
    }

    fn take_memory_profile_mode(&self, canister_id: CanisterId) -> (bool, bool, bool) {
        if !self.tracing_active.load(Ordering::Acquire) {
            return (false, false, false);
        }
        let mut traces = self.traces.lock().unwrap();
        let Some(trace) = traces.canisters.get_mut(&canister_id) else {
            return (false, false, false);
        };
        if !trace.tracing || trace.profile_remaining == 0 {
            return (false, false, false);
        }
        trace.profile_remaining -= 1;
        let reads = trace.read_mask_remaining > 0;
        trace.read_mask_remaining = trace.read_mask_remaining.saturating_sub(1);
        let writes = trace.write_mask_remaining > 0;
        trace.write_mask_remaining = trace.write_mask_remaining.saturating_sub(1);
        (true, reads, writes)
    }

    fn record_runtime_cost(&self, canister_id: CanisterId, cost: TracedRuntimeCost) {
        self.retain_runtime_cost(Some(canister_id), None, cost);
    }

    fn record_subnet_runtime_cost(&self, subnet_id: SubnetId, cost: TracedRuntimeCost) {
        self.retain_runtime_cost(None, Some(subnet_id), cost);
    }

    fn record_runtime_configuration(
        &self,
        subnet_id: SubnetId,
        configuration: BTreeMap<String, String>,
    ) {
        self.traces
            .lock()
            .unwrap()
            .runtime_configurations
            .entry(subnet_id.to_string())
            .or_default()
            .extend(configuration);
    }

    fn record(&self, canister_id: CanisterId, execution: TracedWasmExecution) {
        let (outcome, message) = match execution.outcome {
            TracedWasmExecutionOutcome::Reply => (TracedExecutionOutcome::Reply, None),
            TracedWasmExecutionOutcome::Reject(message) => {
                (TracedExecutionOutcome::Reject, Some(message))
            }
            TracedWasmExecutionOutcome::NoResponse => (TracedExecutionOutcome::NoResponse, None),
            TracedWasmExecutionOutcome::Error(message) => {
                (TracedExecutionOutcome::Error, Some(message))
            }
            TracedWasmExecutionOutcome::Aborted => (TracedExecutionOutcome::Aborted, None),
        };
        let ranges = |ranges: Vec<Range<u64>>| -> Vec<(u64, u64)> {
            ranges.into_iter().map(|r| (r.start, r.end)).collect()
        };
        let mut traces = self.traces.lock().unwrap();
        let mut recorded = TracedExecution {
            sequence_number: 0, // Assigned only when the final disposition publishes this row.
            attempt: Some(rest::WasmExecutionAttempt {
                attempt_id: execution.attempt.attempt_id,
                status: execution.attempt.status,
                slice_count: execution.attempt.slice_count,
                total_round_instructions: execution.attempt.total_round_instructions,
                slices: execution
                    .attempt
                    .slices
                    .into_iter()
                    .map(|slice| rest::WasmExecutionSlice {
                        slice_index: slice.slice_index,
                        event: slice.event,
                        instructions: slice.instructions,
                        instruction_kind: slice.instruction_kind,
                    })
                    .collect(),
                slice_details_truncated: execution.attempt.slice_details_truncated,
                memory_details_available: execution.attempt.memory_details_available,
                instructions_available: execution.attempt.instructions_available,
            }),
            disposition: None,
            api_type: execution.api_type.to_string(),
            function: execution.function,
            call_context_id: execution.call_context_id.map(|id| id.get()),
            time_nanos: execution.time.as_nanos_since_unix_epoch(),
            instructions: execution.instructions_executed.get(),
            page_access_instructions: execution.trace.page_access_instructions,
            outcome,
            message,
            wasm_memory_size: execution.trace.wasm_memory_size,
            dirty_pages_tracked: execution.trace.dirty_pages_tracked,
            accessed_pages: ranges(execution.trace.accessed_pages),
            dirty_pages: ranges(execution.trace.dirty_pages),
            stable_memory_size: execution.trace.stable_memory_size,
            stable_pages_tracked: execution.trace.stable_pages_tracked,
            stable_accessed_pages: ranges(execution.trace.stable_accessed_pages),
            stable_dirty_pages: ranges(execution.trace.stable_dirty_pages),
            module_hash: Some(execution.module_hash),
            module_hash_kind: Some(execution.module_hash_kind.to_string()),
            instruction_profile: execution.trace.instruction_profile.map(profile_to_rest),
            read_profile: execution
                .trace
                .read_profile
                .map(|profile| rest::WasmReadProfile {
                    ranges: profile
                        .ranges
                        .into_iter()
                        .map(|range| rest::WasmReadRange {
                            memory: range.memory,
                            start: range.start,
                            end: range.end,
                            heap_reads: range.heap_reads,
                            stable_direct_reads: range.stable_direct_reads,
                            stable_native_reads: range.stable_native_reads,
                        })
                        .collect(),
                    observed_reads: profile.observed_reads,
                    observed_bytes: profile.observed_bytes,
                    truncated: profile.truncated,
                    counters_saturated: profile.counters_saturated,
                    max_unique_reads: profile.max_unique_reads,
                    max_export_ranges: profile.max_export_ranges,
                    coverage: profile.coverage,
                }),
            write_profile: execution
                .trace
                .write_profile
                .map(|profile| rest::WasmWriteProfile {
                    ranges: profile
                        .ranges
                        .into_iter()
                        .map(|range| rest::WasmWriteRange {
                            memory: range.memory,
                            start: range.start,
                            end: range.end,
                            heap_writes: range.heap_writes,
                            stable_writes: range.stable_writes,
                        })
                        .collect(),
                    observed_writes: profile.observed_writes,
                    observed_bytes: profile.observed_bytes,
                    truncated: profile.truncated,
                    counters_saturated: profile.counters_saturated,
                    max_unique_writes: profile.max_unique_writes,
                    max_export_ranges: profile.max_export_ranges,
                    coverage: profile.coverage,
                }),
            detail_truncated: false,
        };
        // Intern immutable origins once per installed module. Reject a mismatched
        // dictionary rather than attach approximate source locations.
        if let Some(profile) = &mut recorded.instruction_profile {
            if let Some(module) = profile.module.take() {
                profile.module_origin_map_truncated |= module.origin_map_truncated;
                let hash = hex::encode(module.module_sha256);
                if recorded.module_hash.as_ref() != Some(&hash) {
                    profile.module_origin_map_unavailable = true;
                    profile.truncated = true;
                    profile.limitations.push(
                        "Origin dictionary rejected: installed module hash does not match."
                            .to_string(),
                    );
                } else if !traces.modules.contains_key(&hash) {
                    let bytes =
                        std::mem::size_of_val(&module)
                            .saturating_add(module.functions.capacity().saturating_mul(
                                std::mem::size_of::<rest::WasmInstructionFunctionOrigin>(),
                            ))
                            .saturating_add(module.operators.capacity().saturating_mul(
                                std::mem::size_of::<rest::WasmInstructionOperator>(),
                            ))
                            .saturating_add(module.blocks.capacity().saturating_mul(
                                std::mem::size_of::<rest::WasmInstructionBlockDefinition>(),
                            ))
                            .saturating_add(module.instrumented_ranges.capacity().saturating_mul(
                                std::mem::size_of::<rest::WasmInstructionOriginRange>(),
                            ))
                            .saturating_add(
                                module
                                    .instrumented_ranges
                                    .iter()
                                    .fold(0_usize, |sum, range| {
                                        sum.saturating_add(range.kind.capacity())
                                    }),
                            )
                            .saturating_add(
                                module
                                    .blocks
                                    .iter()
                                    .map(|b| {
                                        b.origin_ids
                                            .capacity()
                                            .saturating_mul(std::mem::size_of::<u32>())
                                    })
                                    .sum::<usize>(),
                            )
                            .saturating_mul(2);
                    if traces.recorded_bytes.saturating_add(bytes)
                        < self.memory_limit - TRACE_SUMMARY_RESERVE
                    {
                        traces.recorded_bytes = traces.recorded_bytes.saturating_add(bytes);
                        traces.modules.insert(
                            hash,
                            StoredModule {
                                metadata: module,
                                bytes,
                            },
                        );
                    } else {
                        profile.module_origin_map_unavailable = true;
                        profile.truncated = true;
                        profile.limitations.push("Origin dictionary omitted because it exceeds the retained trace memory budget.".to_string());
                    }
                }
            }
            let retained = recorded
                .module_hash
                .as_ref()
                .and_then(|hash| traces.modules.get(hash));
            profile.module_origin_map_unavailable |= retained.is_none();
            profile.module_origin_map_truncated |=
                retained.is_some_and(|module| module.metadata.origin_map_truncated);
        }
        // A conservative retained-size estimate includes vector slack and the
        // heap behind serialized profile data. This is not an IC memory fee.
        let estimate = |record: &TracedExecution| -> usize {
            std::mem::size_of::<TracedExecution>() * 2
                + 4096
                + serde_json::to_vec(record)
                    .map_or(self.memory_limit, |v| v.len().saturating_mul(4))
        };
        let mut bytes = estimate(&recorded);
        let detail_limit = self.memory_limit - TRACE_SUMMARY_RESERVE;
        if traces.recorded_bytes.saturating_add(bytes) > detail_limit {
            // Preserve the authoritative execution total and signal omitted
            // page/profile detail; never fail or alter canister execution.
            recorded.accessed_pages = Vec::new();
            recorded.dirty_pages = Vec::new();
            recorded.stable_accessed_pages = Vec::new();
            recorded.stable_dirty_pages = Vec::new();
            recorded.stable_pages_tracked = false;
            if let Some(reads) = &mut recorded.read_profile {
                reads.ranges = Vec::new();
                reads.truncated = true;
                reads
                    .coverage
                    .push("Recorder memory budget reached; byte-read ranges omitted.".to_string());
            }
            if let Some(writes) = &mut recorded.write_profile {
                writes.ranges = Vec::new();
                writes.truncated = true;
                writes
                    .coverage
                    .push("Recorder memory budget reached; byte-write ranges omitted.".to_string());
            }
            if let Some(attempt) = &mut recorded.attempt {
                attempt.slices.clear();
                attempt.slice_details_truncated = true;
            }
            if let Some(profile) = &mut recorded.instruction_profile {
                profile.blocks = Vec::new();
                profile.dynamic_charges = Vec::new();
                profile.api_calls = Vec::new();
                profile.page_charges = Vec::new();
                profile.stacks = Vec::new();
                profile.stack_charges = Vec::new();
                profile.page_details_truncated = true;
                profile.stack_details_truncated = true;
                profile.truncated = true;
                profile.limitations.push("Recorder memory budget reached; charge detail and page sets omitted, ledger retained.".to_string());
            }
            recorded.detail_truncated = true;
            if let Some(message) = &mut recorded.message {
                message.truncate(message.floor_char_boundary(1024));
            }
            bytes = estimate(&recorded);
            for trace in traces.canisters.values_mut() {
                if trace.tracing {
                    trace.truncated = true;
                }
                trace.tracing = false;
                trace.profile_remaining = 0;
                trace.read_mask_remaining = 0;
                trace.write_mask_remaining = 0;
            }
            self.tracing_active.store(false, Ordering::Release);
            traces.canisters.entry(canister_id).or_default().truncated = true;
        }
        if traces.recorded_bytes.saturating_add(bytes) > self.memory_limit {
            let trace = traces.canisters.entry(canister_id).or_default();
            trace.truncated = true;
            trace.dropped_executions = trace.dropped_executions.saturating_add(1);
            trace.generation = trace.generation.saturating_add(1);
            trace.dropped_instructions_incomplete |= recorded
                .attempt
                .as_ref()
                .is_some_and(|attempt| !attempt.instructions_available)
                || trace
                    .dropped_instructions
                    .checked_add(recorded.instructions as u128)
                    .is_none();
            trace.dropped_instructions = trace
                .dropped_instructions
                .saturating_add(recorded.instructions as u128);
            Self::collect_unreferenced_modules(&mut traces);
            return;
        }
        traces.recorded_bytes = traces.recorded_bytes.saturating_add(bytes);
        let trace = traces.canisters.entry(canister_id).or_default();
        trace.recorded_bytes = trace.recorded_bytes.saturating_add(bytes);
        let attempt_id = recorded
            .attempt
            .as_ref()
            .map_or(0, |attempt| attempt.attempt_id);
        if let Some((disposition, pending_bytes)) = trace.pending_dispositions.remove(&attempt_id) {
            trace.recorded_bytes = trace.recorded_bytes.saturating_sub(pending_bytes);
            traces.recorded_bytes = traces.recorded_bytes.saturating_sub(pending_bytes);
            if disposition.state != rest::TracedExecutionState::Pending {
                recorded.disposition = Some(disposition);
                Self::publish_attempt(&mut traces, canister_id, recorded);
                return;
            }
            recorded.disposition = Some(disposition);
        }
        let trace = traces.canisters.get_mut(&canister_id).unwrap();
        if attempt_id == 0
            || trace.lifecycle_details_truncated
            || trace.pending_attempts.len() >= MAX_PENDING_ATTEMPTS_PER_CANISTER
        {
            if attempt_id != 0 {
                recorded.detail_truncated = true;
                trace.truncated = true;
            }
            Self::publish_attempt(&mut traces, canister_id, recorded);
        } else {
            trace.pending_attempts.insert(
                attempt_id,
                PendingAttempt {
                    execution: recorded,
                    bytes,
                },
            );
        }
    }
}

fn profile_to_rest(
    profile: ic_interfaces::execution_environment::WasmInstructionProfile,
) -> rest::WasmInstructionProfile {
    rest::WasmInstructionProfile {
        version: profile.version,
        page_charges: profile
            .page_charges
            .into_iter()
            .map(|page| rest::WasmInstructionPageCharge {
                memory: page.memory,
                region_index: page.region_index,
                kind: page.kind,
                instructions: page.instructions,
                native_pc: page.native_pc,
                module_offset: page.module_offset,
                origin_id: page.origin_id,
                origin_evidence: page.origin_evidence,
            })
            .collect(),
        page_details_truncated: profile.page_details_truncated,
        stacks: profile
            .stacks
            .into_iter()
            .map(|stack| rest::WasmInstructionStack {
                stack_id: stack.stack_id,
                frames: stack
                    .frames
                    .into_iter()
                    .map(|frame| rest::WasmInstructionStackFrame {
                        instrumented_function_index: frame.instrumented_function_index,
                        module_offset: frame.module_offset,
                        function_index: frame.function_index,
                        origin_id: frame.origin_id,
                        body_offset: frame.body_offset,
                        operator_ordinal: frame.operator_ordinal,
                        synthetic_kind: frame.synthetic_kind,
                    })
                    .collect(),
                truncated: stack.truncated,
            })
            .collect(),
        stack_charges: profile
            .stack_charges
            .into_iter()
            .map(|charge| rest::WasmInstructionStackCharge {
                stack_id: charge.stack_id,
                origin_id: charge.origin_id,
                category: charge.category,
                api: charge.api,
                events: charge.events,
                instructions: charge.instructions,
                origin_evidence: charge.origin_evidence,
            })
            .collect(),
        stack_details_truncated: profile.stack_details_truncated,
        module_origin_map_truncated: profile.module_origin_map_truncated,
        module_origin_map_unavailable: profile.module_origin_map_unavailable,
        rules: profile
            .rules
            .into_iter()
            .map(|rule| rest::WasmInstructionRuleCharge {
                rule_id: rule.rule_id,
                events: rule.events,
                instructions: rule.instructions,
            })
            .collect(),
        module: profile.module.map(module_to_rest),
        blocks: profile
            .blocks
            .into_iter()
            .map(|value| rest::WasmInstructionProfileBlock {
                block_id: value.block_id,
                function_index: value.function_index,
                body_offset: value.body_offset,
                end_body_offset: value.end_body_offset,
                operator_ordinal: value.operator_ordinal,
                weight: value.weight,
                entry_weight: value.entry_weight,
                hits: value.hits,
                instructions: value.instructions,
            })
            .collect(),
        dynamic_charges: profile
            .dynamic_charges
            .into_iter()
            .map(|value| rest::WasmInstructionDynamicCharge {
                function_index: value.function_index,
                body_offset: value.body_offset,
                operator_ordinal: value.operator_ordinal,
                category: value.category,
                events: value.events,
                instructions: value.instructions,
            })
            .collect(),
        api_calls: profile
            .api_calls
            .into_iter()
            .map(|value| rest::WasmInstructionApiCharge {
                api: value.api,
                calls: value.calls,
                base_instructions: value.base_instructions,
                byte_instructions: value.byte_instructions,
                direct_instructions: value.direct_instructions,
            })
            .collect(),
        budget_events: profile
            .budget_events
            .into_iter()
            .map(|event| rest::WasmInstructionBudgetEvent {
                before: event.before,
                after: event.after,
                cause: event.cause,
            })
            .collect(),
        ledger: rest::WasmInstructionChargeLedger {
            static_instructions: profile.ledger.static_instructions,
            dynamic_instructions: profile.ledger.dynamic_instructions,
            api_instructions: profile.ledger.api_instructions,
            stable_api_instructions: profile.ledger.stable_api_instructions,
            heap_page_instructions: profile.ledger.heap_page_instructions,
            stable_page_instructions: profile.ledger.stable_page_instructions,
            other_known_instructions: profile.ledger.other_known_instructions,
            raw_instructions: profile.ledger.raw_instructions,
            reported_instructions: profile.ledger.reported_instructions,
            adjustment: profile.ledger.adjustment,
            adjustment_reason: profile.ledger.adjustment_reason,
            unexplained_instructions: profile.ledger.unexplained_instructions,
            reconciled: profile.ledger.reconciled,
        },
        truncated: profile.truncated,
        limitations: profile.limitations,
    }
}

fn module_to_rest(
    module: ic_interfaces::execution_environment::WasmInstructionModuleMetadata,
) -> rest::WasmInstructionModuleMetadata {
    rest::WasmInstructionModuleMetadata {
        version: module.version,
        instrumentation_version: module.instrumentation_version,
        configuration_sha256: module.configuration_sha256,
        instrumentation_source_sha256: module.instrumentation_source_sha256,
        module_sha256: module.module_sha256,
        code_section_offset: module.code_section_offset,
        instrumented_code_section_offset: module.instrumented_code_section_offset,
        instrumented_ranges: module
            .instrumented_ranges
            .into_iter()
            .map(|range| rest::WasmInstructionOriginRange {
                instrumented_function_index: range.instrumented_function_index,
                start_offset: range.start_offset,
                end_offset: range.end_offset,
                origin_id: range.origin_id,
                kind: range.kind,
            })
            .collect(),
        origin_map_truncated: module.origin_map_truncated,
        functions: module
            .functions
            .into_iter()
            .map(|value| rest::WasmInstructionFunctionOrigin {
                function_index: value.function_index,
                body_start: value.body_start,
                body_end: value.body_end,
            })
            .collect(),
        operators: module
            .operators
            .into_iter()
            .map(|value| rest::WasmInstructionOperator {
                origin_id: value.origin_id,
                function_index: value.function_index,
                body_offset: value.body_offset,
                operator_ordinal: value.operator_ordinal,
                weight: value.weight,
            })
            .collect(),
        blocks: module
            .blocks
            .into_iter()
            .map(|value| rest::WasmInstructionBlockDefinition {
                block_id: value.block_id,
                function_index: value.function_index,
                body_offset: value.body_offset,
                end_body_offset: value.end_body_offset,
                operator_ordinal: value.operator_ordinal,
                weight: value.weight,
                entry_weight: value.entry_weight,
                origin_ids: value.origin_ids,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_interfaces::execution_environment::WasmMemoryAccessTrace;
    use ic_types::{NumInstructions, Time, messages::CallContextId};

    fn execution(function: &str) -> TracedWasmExecution {
        TracedWasmExecution {
            attempt: ic_interfaces::execution_environment::WasmExecutionAttempt {
                attempt_id: 0,
                status: "completed".to_string(),
                slice_count: 1,
                total_round_instructions: 0,
                slices: vec![],
                slice_details_truncated: false,
                memory_details_available: true,
                instructions_available: true,
            },
            api_type: "update",
            module_hash: "ab".repeat(32),
            module_hash_kind: "installed_uncompressed",
            function: function.to_string(),
            call_context_id: Some(CallContextId::new(7)),
            time: Time::from_nanos_since_unix_epoch(42),
            instructions_executed: NumInstructions::new(1_000),
            outcome: TracedWasmExecutionOutcome::Reject("no".to_string()),
            trace: WasmMemoryAccessTrace {
                accessed_pages: vec![0..2, 5..6],
                dirty_pages: vec![1..2, 5..6],
                dirty_pages_tracked: true,
                wasm_memory_size: 8,
                page_access_instructions: 640,
                ..WasmMemoryAccessTrace::default()
            },
        }
    }

    fn logical_context(retry: bool) -> TracedLogicalExecution {
        TracedLogicalExecution {
            identity: "ingress:exact-message-id".to_string(),
            kind: "ingress",
            retry,
            message_id: Some("exact-message-id".to_string()),
            callback_id: None,
            parent_call_context_id: Some(7),
            time: Time::from_nanos_since_unix_epoch(42),
        }
    }

    fn identified_attempt(id: u64) -> TracedWasmExecution {
        let mut attempt = execution("canister_update lifecycle");
        attempt.attempt.attempt_id = id;
        attempt.attempt.status = "completed".to_string();
        attempt
    }

    #[test]
    fn snapshot_touched_union_preserves_logical_mask_pages_without_native_fees() {
        use ic_interfaces::execution_environment::{
            WasmReadProfile, WasmReadRange, WasmWriteProfile, WasmWriteRange,
        };
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(99);
        recorder.start(canister);
        let mut observed = identified_attempt(7);
        observed.trace.accessed_pages.clear();
        observed.trace.dirty_pages.clear();
        observed.trace.page_access_instructions = 0;
        observed.trace.read_profile = Some(WasmReadProfile {
            ranges: vec![WasmReadRange {
                memory: "heap".into(),
                start: 42 * 65_536,
                end: 42 * 65_536 + 8,
                heap_reads: 1,
                ..Default::default()
            }],
            ..Default::default()
        });
        observed.trace.write_profile = Some(WasmWriteProfile {
            ranges: vec![WasmWriteRange {
                memory: "stable".into(),
                start: 129 * 65_536,
                end: 129 * 65_536 + 1,
                stable_writes: 2,
                ..Default::default()
            }],
            ..Default::default()
        });
        recorder.record(canister, observed);
        recorder.record_execution_disposition(
            canister,
            7,
            TracedExecutionDisposition {
                logical_execution_id: None,
                state: TracedExecutionState::Committed,
                reason: "applied".into(),
            },
        );
        let trace = recorder.get(canister);
        let evidence = recorder.snapshot_touched_pages(
            canister,
            trace.trace_generation,
            None,
            Some(0),
            None,
            None,
        );
        assert_eq!(
            evidence["known_touched_ranges"]["heap"],
            serde_json::json!([["42", "43"]])
        );
        assert_eq!(
            evidence["known_touched_ranges"]["stable"],
            serde_json::json!([["129", "130"]])
        );
        assert_eq!(
            evidence["known_dirty_ranges"]["stable"],
            serde_json::json!([])
        );
        assert_eq!(evidence["raw_heap_page_instructions"], "0");
        assert_eq!(evidence["write_profile"]["available"], true);
    }

    #[test]
    fn snapshot_touched_union_includes_sparse_stable_direct_reads_and_exact_attempt_filters() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(99);
        recorder.start(canister);
        let logical = recorder
            .begin_logical_execution(canister, logical_context(false))
            .unwrap();
        for (id, stable) in [(1, vec![0..2, 7_000_000..7_000_001]), (2, vec![1..4])] {
            let mut observed = identified_attempt(id);
            observed.trace.stable_memory_size = 8_000_000;
            observed.trace.stable_pages_tracked = true;
            observed.trace.stable_accessed_pages = stable;
            observed.trace.stable_dirty_pages = if id == 2 { vec![3..4] } else { vec![] };
            recorder.record(canister, observed);
            recorder.record_execution_disposition(
                canister,
                id,
                TracedExecutionDisposition {
                    logical_execution_id: Some(logical),
                    state: TracedExecutionState::Committed,
                    reason: "applied".to_string(),
                },
            );
        }
        let trace = recorder.get(canister);
        let all = recorder.snapshot_touched_pages(
            canister,
            trace.trace_generation,
            None,
            Some(1),
            None,
            Some(logical),
        );
        assert_eq!(
            all["known_touched_ranges"]["heap"],
            serde_json::json!([["0", "2"], ["5", "6"]])
        );
        assert_eq!(
            all["known_touched_ranges"]["stable"],
            serde_json::json!([["0", "4"], ["7000000", "7000001"]])
        );
        assert_eq!(
            all["known_dirty_ranges"]["stable"],
            serde_json::json!([["3", "4"]])
        );
        assert_eq!(all["retained_stable_coverage_complete"], true);
        assert_eq!(
            all["stable_page_fee_total_complete"], false,
            "coverage is not a page-fee estimate"
        );
        assert_eq!(all["known_stable_page_instructions"], "0");
        let first = recorder.snapshot_touched_pages(
            canister,
            trace.trace_generation,
            None,
            Some(1),
            Some(1),
            None,
        );
        assert_eq!(first["execution_count"], "1");
        assert_eq!(
            first["known_touched_ranges"]["stable"],
            serde_json::json!([["0", "2"], ["7000000", "7000001"]])
        );
        assert_eq!(trace.executions[0].stable_memory_size, 8_000_000);
        recorder.clear(canister);
        assert_eq!(
            recorder.snapshot_touched_pages(
                canister,
                trace.trace_generation,
                None,
                Some(1),
                None,
                None
            )["available"],
            false
        );
    }

    #[test]
    fn snapshot_touched_coverage_keeps_unavailable_stable_attempts_unknown() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(99);
        recorder.start(canister);
        let mut observed = execution("legacy or unavailable stable coverage");
        observed.trace.stable_accessed_pages = vec![100..101];
        observed.trace.stable_pages_tracked = false;
        recorder.record(canister, observed);
        let trace = recorder.get(canister);
        let evidence = recorder.snapshot_touched_pages(
            canister,
            trace.trace_generation,
            None,
            Some(0),
            None,
            None,
        );
        assert_eq!(evidence["retained_stable_coverage_complete"], false);
        assert_eq!(
            evidence["known_touched_ranges"]["stable"],
            serde_json::json!([])
        );
        assert_eq!(evidence["continuous_recording_proven"], false);
    }

    #[test]
    fn attempt_records_wait_for_actual_disposition_and_retries_share_exact_logical_id() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(99);
        recorder.start(canister);
        let logical = recorder
            .begin_logical_execution(canister, logical_context(false))
            .unwrap();
        recorder.record(canister, identified_attempt(1));
        assert!(recorder.get(canister).executions.is_empty());
        recorder.record_execution_disposition(
            canister,
            1,
            TracedExecutionDisposition {
                logical_execution_id: Some(logical),
                state: TracedExecutionState::Discarded,
                reason: "actual checkpoint abort".to_string(),
            },
        );
        recorder.finish_logical_execution(canister, logical, true);
        assert_eq!(
            recorder.begin_logical_execution(canister, logical_context(true)),
            Some(logical)
        );
        recorder.record(canister, identified_attempt(2));
        assert_eq!(recorder.get(canister).executions.len(), 1);
        recorder.record_execution_disposition(
            canister,
            2,
            TracedExecutionDisposition {
                logical_execution_id: Some(logical),
                state: TracedExecutionState::Committed,
                reason: "state changes applied".to_string(),
            },
        );
        recorder.finish_logical_execution(canister, logical, false);
        let exported = recorder.get(canister);
        assert_eq!(exported.executions.len(), 2);
        assert_eq!(exported.executions[0].sequence_number, 0);
        assert_eq!(exported.executions[1].sequence_number, 1);
        for attempt in &exported.executions {
            assert_eq!(
                attempt.disposition.as_ref().unwrap().logical_execution_id,
                Some(logical)
            );
        }
        assert_eq!(
            exported.executions[0].disposition.as_ref().unwrap().state,
            rest::TracedExecutionState::Discarded
        );
        assert_eq!(
            exported.executions[1].disposition.as_ref().unwrap().state,
            rest::TracedExecutionState::Committed
        );
        assert_eq!(exported.logical_executions.len(), 1);
        assert_eq!(
            exported.logical_executions[&logical.to_string()]
                .message_id
                .as_deref(),
            Some("exact-message-id")
        );
        // A duplicate terminal notification cannot mutate a previously exported row.
        recorder.record_execution_disposition(
            canister,
            2,
            TracedExecutionDisposition {
                logical_execution_id: None,
                state: TracedExecutionState::Discarded,
                reason: "duplicate".to_string(),
            },
        );
        assert_eq!(recorder.get(canister).executions, exported.executions);
        recorder.clear(canister);
        assert!(recorder.get(canister).logical_executions.is_empty());
        assert_eq!(recorder.traces.lock().unwrap().recorded_bytes, 0);
    }

    #[test]
    fn asynchronous_abort_pairs_with_earlier_disposition_across_clear_without_fake_memory() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(99);
        recorder.start(canister);
        let logical = recorder
            .begin_logical_execution(canister, logical_context(false))
            .unwrap();
        recorder.record_execution_disposition(
            canister,
            7,
            TracedExecutionDisposition {
                logical_execution_id: Some(logical),
                state: TracedExecutionState::Discarded,
                reason: "worker abort requested".to_string(),
            },
        );
        recorder.finish_logical_execution(canister, logical, false);
        recorder.clear(canister);
        let before = recorder.get(canister);
        assert!(before.executions.is_empty());
        assert!(before.recorded_bytes > 0);
        let mut aborted = identified_attempt(7);
        aborted.outcome = TracedWasmExecutionOutcome::Aborted;
        aborted.attempt.status = "aborted".to_string();
        aborted.attempt.memory_details_available = false;
        aborted.trace.accessed_pages.clear();
        aborted.trace.dirty_pages.clear();
        recorder.record(canister, aborted);
        let after = recorder.get(canister);
        assert_eq!(after.executions.len(), 1);
        assert_eq!(after.executions[0].outcome, TracedExecutionOutcome::Aborted);
        assert!(
            !after.executions[0]
                .attempt
                .as_ref()
                .unwrap()
                .memory_details_available
        );
        assert_eq!(after.executions[0].instructions, 1_000);
        assert!(after.logical_executions.contains_key(&logical.to_string()));
        assert!(!recorder.clear_if_unchanged(canister, before.trace_generation, None, None));
        recorder.clear(canister);
        assert_eq!(recorder.traces.lock().unwrap().recorded_bytes, 0);
    }

    #[test]
    fn pending_install_stages_publish_once_with_monotone_cursors_and_pending_budget_is_bounded() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(99);
        recorder.start(canister);
        for id in 1..=MAX_PENDING_ATTEMPTS_PER_CANISTER as u64 + 1 {
            recorder.record(canister, identified_attempt(id));
        }
        {
            let traces = recorder.traces.lock().unwrap();
            let trace = &traces.canisters[&canister];
            assert_eq!(
                trace.pending_attempts.len(),
                MAX_PENDING_ATTEMPTS_PER_CANISTER
            );
            assert_eq!(trace.executions.len(), 1);
            assert!(trace.executions[0].detail_truncated);
            assert!(traces.recorded_bytes < TRACE_MEMORY_LIMIT);
        }
        for id in (1..=MAX_PENDING_ATTEMPTS_PER_CANISTER as u64).rev() {
            recorder.record_execution_disposition(
                canister,
                id,
                TracedExecutionDisposition {
                    logical_execution_id: None,
                    state: TracedExecutionState::Discarded,
                    reason: "upgrade rolled back all earlier stages".to_string(),
                },
            );
        }
        let trace = recorder.get(canister);
        assert_eq!(
            trace.executions.len(),
            MAX_PENDING_ATTEMPTS_PER_CANISTER + 1
        );
        assert!(
            trace
                .executions
                .windows(2)
                .all(|rows| rows[0].sequence_number < rows[1].sequence_number)
        );
        assert!(
            recorder.traces.lock().unwrap().canisters[&canister]
                .pending_attempts
                .is_empty()
        );
        recorder.clear(canister);
        assert_eq!(recorder.traces.lock().unwrap().recorded_bytes, 0);
    }

    #[test]
    fn omitted_attempt_with_unknown_counter_marks_dropped_total_incomplete() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(99);
        recorder.start(canister);
        recorder.traces.lock().unwrap().recorded_bytes = TRACE_MEMORY_LIMIT;
        let mut unavailable = identified_attempt(42);
        unavailable.attempt.instructions_available = false;
        unavailable.instructions_executed = NumInstructions::new(0);
        recorder.record(canister, unavailable);
        let trace = recorder.get(canister);
        assert!(trace.executions.is_empty());
        assert_eq!(trace.dropped_executions, 1);
        assert_eq!(trace.dropped_instructions, 0);
        assert!(trace.dropped_instructions_incomplete);
    }

    #[test]
    fn records_only_traced_canisters_in_order() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister_1 = CanisterId::from_u64(1);
        let canister_2 = CanisterId::from_u64(2);
        assert!(!recorder.is_tracing_enabled(canister_1));

        recorder.start(canister_1);
        assert!(recorder.is_tracing_enabled(canister_1));
        assert!(!recorder.is_tracing_enabled(canister_2));

        recorder.record(canister_1, execution("canister_update a"));
        recorder.record(canister_2, execution("canister_update b"));
        recorder.record(canister_1, execution("canister_update c"));

        let trace = recorder.get(canister_1);
        assert_eq!(trace.wasm_page_size, 65536);
        assert_eq!(
            trace
                .executions
                .iter()
                .map(|e| (e.sequence_number, e.function.as_str()))
                .collect::<Vec<_>>(),
            vec![(0, "canister_update a"), (2, "canister_update c")]
        );
        let first = &trace.executions[0];
        assert_eq!(first.api_type, "update");
        assert_eq!(first.call_context_id, Some(7));
        assert_eq!(first.time_nanos, 42);
        assert_eq!(first.instructions, 1_000);
        assert_eq!(first.outcome, TracedExecutionOutcome::Reject);
        assert_eq!(first.message.as_deref(), Some("no"));
        assert_eq!(first.accessed_pages, vec![(0, 2), (5, 6)]);
        assert_eq!(first.dirty_pages, vec![(1, 2), (5, 6)]);
        assert!(first.dirty_pages_tracked);
        assert_eq!(first.wasm_memory_size, 8);
        assert_eq!(first.page_access_instructions, 640);

        // Incremental reads continue after a sequence number.
        let (after_0, more) = recorder.executions_after(canister_1, Some(0), 10, None);
        assert_eq!(
            after_0
                .iter()
                .map(|e| e.sequence_number)
                .collect::<Vec<_>>(),
            vec![2]
        );
        assert!(!more);
        let (first_page, more) = recorder.executions_after(canister_1, None, 1, None);
        assert_eq!(first_page.len(), 1);
        assert!(more);
        let (filtered, _) = recorder.executions_after(canister_1, None, 10, Some("update c"));
        assert_eq!(filtered.len(), 1);
        let canisters = recorder.canisters();
        assert_eq!(
            canisters
                .iter()
                .map(|c| (
                    c.canister_id,
                    c.tracing,
                    c.executions,
                    c.first_sequence_number,
                    c.last_sequence_number
                ))
                .collect::<Vec<_>>(),
            vec![
                (canister_1, true, 2, Some(0), Some(2)),
                (canister_2, false, 1, Some(1), Some(1))
            ]
        );

        // Stopping keeps the recorded executions, clearing deletes them.
        recorder.stop(canister_1);
        assert!(!recorder.is_tracing_enabled(canister_1));
        assert_eq!(recorder.get(canister_1).executions.len(), 2);
        recorder.clear(canister_1);
        assert!(recorder.get(canister_1).executions.is_empty());
        assert_eq!(recorder.get(canister_2).executions.len(), 1);
    }
    #[test]
    fn disabled_recorder_does_not_wait_for_history_lock() {
        let recorder = std::sync::Arc::new(MemoryAccessTraceRecorder::default());
        let canister = CanisterId::from_u64(1);
        // Exercise both a fresh recorder and retained history after stopping.
        for stopped in [false, true] {
            if stopped {
                assert!(recorder.start(canister));
                recorder.record(canister, execution("retained"));
                recorder.stop(canister);
                assert!(!recorder.get(canister).executions.is_empty());
            }
            let history = recorder.traces.lock().unwrap();
            let other = recorder.clone();
            let (send, receive) = std::sync::mpsc::channel();
            let worker = std::thread::spawn(move || {
                assert!(!other.is_tracing_enabled(canister));
                assert!(!other.is_runtime_cost_tracing_enabled(canister));
                assert_eq!(
                    other.take_memory_profile_mode(canister),
                    (false, false, false)
                );
                assert!(
                    other
                        .begin_logical_execution(canister, logical_context(false))
                        .is_none()
                );
                send.send(()).unwrap();
            });
            let completed_while_locked = receive.recv_timeout(std::time::Duration::from_secs(1));
            drop(history);
            worker.join().unwrap();
            completed_while_locked.expect("disabled observation must not acquire the history lock");
        }
    }

    #[test]
    fn stopping_one_canister_keeps_other_recorders_active() {
        let recorder = MemoryAccessTraceRecorder::default();
        let first = CanisterId::from_u64(1);
        let second = CanisterId::from_u64(2);
        assert!(recorder.start(first));
        assert!(recorder.start(second));
        recorder.stop(first);
        assert!(!recorder.is_tracing_enabled(first));
        assert!(recorder.is_tracing_enabled(second));
        assert!(recorder.is_runtime_cost_tracing_enabled(second));
        recorder.stop(second);
        assert!(!recorder.tracing_active.load(Ordering::Acquire));
        assert!(!recorder.costs_active.load(Ordering::Acquire));
    }

    #[test]
    fn profiling_is_explicit_bounded_and_stopped_with_tracing() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        recorder.start(canister);
        assert!(!recorder.take_instruction_profile(canister));
        recorder.start_profiling(canister, 2);
        assert!(recorder.take_instruction_profile(canister));
        assert!(recorder.take_instruction_profile(canister));
        assert!(!recorder.take_instruction_profile(canister));
        assert!(recorder.is_tracing_enabled(canister));
        recorder.start_profiling(canister, 2);
        recorder.stop(canister);
        assert!(!recorder.take_instruction_profile(canister));
        recorder.start(canister);
        assert!(!recorder.take_instruction_profile(canister));
    }

    #[test]
    fn write_mask_selection_is_independent_atomic_and_stopped() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        assert!(recorder.start_profiling_masks(canister, 1, false, true));
        assert_eq!(
            recorder.take_memory_profile_mode(canister),
            (true, false, true)
        );
        assert_eq!(
            recorder.take_memory_profile_mode(canister),
            (false, false, false)
        );
        assert!(recorder.start_profiling_masks(canister, 1, true, true));
        assert_eq!(
            recorder.take_memory_profile_mode(canister),
            (true, true, true)
        );
        assert!(recorder.start_profiling_with_reads(canister, 1, true));
        assert_eq!(
            recorder.take_memory_profile_mode(canister),
            (true, true, false)
        );
        assert!(recorder.start_profiling_masks(canister, 1, false, true));
        recorder.stop(canister);
        assert_eq!(
            recorder.take_memory_profile_mode(canister),
            (false, false, false)
        );
    }

    #[test]
    fn read_mask_window_is_explicit_atomic_and_stopped_with_tracing() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        assert_eq!(
            recorder.take_instruction_profile_mode(canister),
            (false, false)
        );
        assert!(recorder.start_profiling_with_reads(canister, 2, true));
        assert_eq!(
            recorder.take_instruction_profile_mode(canister),
            (true, true)
        );
        assert_eq!(
            recorder.take_instruction_profile_mode(canister),
            (true, true)
        );
        assert_eq!(
            recorder.take_instruction_profile_mode(canister),
            (false, false)
        );
        assert!(recorder.start_profiling(canister, 1));
        assert_eq!(
            recorder.take_instruction_profile_mode(canister),
            (true, false)
        );
        assert!(recorder.start_profiling_with_reads(canister, 2, true));
        recorder.stop(canister);
        assert_eq!(
            recorder.take_instruction_profile_mode(canister),
            (false, false)
        );
    }

    #[test]
    fn conditional_clear_never_deletes_new_executions_or_a_new_generation() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        recorder.start(canister);
        recorder.record(canister, execution("a"));
        let snapshot = recorder.executions_snapshot(canister, 100, ExecutionFilter::default());
        recorder.record(canister, execution("b"));
        assert!(!recorder.clear_if_unchanged(canister, snapshot.trace_generation, Some(0), None));
        assert_eq!(recorder.get(canister).executions.len(), 2);
        assert!(recorder.clear_if_unchanged(canister, snapshot.trace_generation, Some(1), None));
        let cleared = recorder.get(canister);
        assert_eq!(cleared.trace_generation, snapshot.trace_generation + 1);
        assert!(cleared.executions.is_empty());
        assert!(!recorder.clear_if_unchanged(canister, snapshot.trace_generation, None, None));
        recorder.record(canister, execution("c"));
        assert_eq!(recorder.get(canister).executions[0].sequence_number, 2);
    }

    #[test]
    fn retention_budget_preserves_last_execution_total_and_stops_detail() {
        use ic_interfaces::execution_environment as env;
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        recorder.start_profiling(canister, 20);
        recorder.traces.lock().unwrap().recorded_bytes = TRACE_MEMORY_LIMIT - TRACE_SUMMARY_RESERVE;
        let mut observed = execution("large");
        observed.trace.instruction_profile = Some(env::WasmInstructionProfile {
            page_charges: vec![env::WasmInstructionPageCharge::default(); 32],
            stacks: vec![env::WasmInstructionStack::default(); 32],
            stack_charges: vec![env::WasmInstructionStackCharge::default(); 32],
            ledger: env::WasmInstructionChargeLedger {
                raw_instructions: 1000,
                reported_instructions: 1000,
                reconciled: true,
                ..Default::default()
            },
            ..Default::default()
        });
        recorder.record(canister, observed);
        let trace = recorder.get(canister);
        assert!(trace.truncated);
        assert_eq!(trace.executions.len(), 1);
        assert_eq!(trace.executions[0].instructions, 1_000);
        assert!(trace.executions[0].detail_truncated);
        assert!(trace.executions[0].accessed_pages.is_empty());
        let profile = trace.executions[0].instruction_profile.as_ref().unwrap();
        assert!(
            profile.page_charges.is_empty()
                && profile.stacks.is_empty()
                && profile.stack_charges.is_empty()
        );
        assert!(profile.page_details_truncated && profile.stack_details_truncated);
        assert_eq!(profile.ledger.raw_instructions, 1000);
        assert!(!recorder.is_tracing_enabled(canister));
        assert!(!recorder.take_instruction_profile(canister));
        assert!(recorder.traces.lock().unwrap().recorded_bytes <= TRACE_MEMORY_LIMIT);
    }

    #[test]
    fn identity_is_shared_across_instances_and_module_identity_follows_execution() {
        let first = MemoryAccessTraceRecorder::default();
        let second = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        assert_eq!(
            first.get(canister).server_generation,
            second.get(canister).server_generation
        );
        let mut upgraded = execution("after upgrade");
        upgraded.module_hash = "cd".repeat(32);
        first.record(canister, execution("before upgrade"));
        first.record(canister, upgraded);
        let trace = first.get(canister);
        assert_ne!(
            trace.executions[0].module_hash,
            trace.executions[1].module_hash
        );
        assert_eq!(trace.format_version, 2);
    }
    #[test]
    fn module_dictionaries_are_interned_and_collected_when_unreferenced() {
        use ic_interfaces::execution_environment::{
            WasmInstructionModuleMetadata, WasmInstructionProfile,
        };
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        let mut observed = execution("profiled");
        observed.trace.instruction_profile = Some(WasmInstructionProfile {
            module: Some(WasmInstructionModuleMetadata {
                module_sha256: [0xab; 32],
                ..Default::default()
            }),
            ..Default::default()
        });
        recorder.record(canister, observed.clone());
        recorder.record(canister, observed);
        let trace = recorder.get(canister);
        assert_eq!(trace.modules.len(), 1);
        assert!(
            trace.executions.iter().all(|e| e
                .instruction_profile
                .as_ref()
                .unwrap()
                .module
                .is_none())
        );
        assert_eq!(recorder.traces.lock().unwrap().modules.len(), 1);
        recorder.clear(canister);
        let traces = recorder.traces.lock().unwrap();
        assert!(traces.modules.is_empty());
        assert_eq!(traces.recorded_bytes, 0);
    }

    #[test]
    fn mismatched_origin_dictionary_is_rejected_visibly() {
        use ic_interfaces::execution_environment::{
            WasmInstructionModuleMetadata, WasmInstructionProfile,
        };
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        let mut observed = execution("profiled");
        observed.trace.instruction_profile = Some(WasmInstructionProfile {
            module: Some(WasmInstructionModuleMetadata {
                module_sha256: [0xcd; 32],
                ..Default::default()
            }),
            ..Default::default()
        });
        recorder.record(canister, observed);
        let trace = recorder.get(canister);
        assert!(trace.modules.is_empty());
        let profile = trace.executions[0].instruction_profile.as_ref().unwrap();
        assert!(profile.truncated);
        assert!(profile.module_origin_map_unavailable);
        assert!(!profile.stack_details_truncated);
        assert!(
            profile
                .limitations
                .iter()
                .any(|reason| reason.contains("hash does not match"))
        );
    }

    #[test]
    fn module_retention_budget_and_origin_truncation_do_not_truncate_execution_stacks() {
        use ic_interfaces::execution_environment::{
            WasmInstructionModuleMetadata, WasmInstructionOperator, WasmInstructionProfile,
        };
        let canister = CanisterId::from_u64(1);
        let mut observed = execution("profiled");
        observed.trace.instruction_profile = Some(WasmInstructionProfile {
            module: Some(WasmInstructionModuleMetadata {
                module_sha256: [0xab; 32],
                // Enough immutable metadata to exceed the smaller host budget
                // without approaching the default 64 MiB budget.
                operators: vec![
                    WasmInstructionOperator {
                        origin_id: 0,
                        function_index: 0,
                        body_offset: 0,
                        operator_ordinal: 0,
                        weight: 1,
                    };
                    20_000
                ],
                origin_map_truncated: true,
                ..Default::default()
            }),
            module_origin_map_truncated: true,
            ..Default::default()
        });
        for (limit, retained) in [(1024 * 1024, false), (4 * 1024 * 1024, true)] {
            let recorder = MemoryAccessTraceRecorder::with_memory_limit(limit);
            recorder.record(canister, observed.clone());
            let trace = recorder.get(canister);
            assert_eq!(!trace.modules.is_empty(), retained);
            assert_eq!(trace.executions.len(), 1);
            let profile = trace.executions[0].instruction_profile.as_ref().unwrap();
            assert!(profile.module_origin_map_truncated);
            assert_eq!(profile.module_origin_map_unavailable, !retained);
            assert_eq!(profile.truncated, !retained);
            assert!(!profile.stack_details_truncated);
            assert!(recorder.traces.lock().unwrap().recorded_bytes <= limit);
        }
    }
    #[test]
    fn exhausted_summary_reserve_keeps_omitted_instruction_totals() {
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        recorder.traces.lock().unwrap().recorded_bytes = TRACE_MEMORY_LIMIT;
        recorder.record(canister, execution("in flight"));
        let trace = recorder.get(canister);
        assert!(trace.executions.is_empty());
        assert!(trace.truncated);
        assert_eq!(trace.dropped_executions, 1);
        assert_eq!(trace.dropped_instructions, 1000);
        assert!(!recorder.is_tracing_enabled(canister));
        assert!(!recorder.clear_if_unchanged(canister, 0, None, None));
    }
    fn runtime_event(
        unit: ic_interfaces::execution_environment::RuntimeCostUnit,
        kind: ic_interfaces::execution_environment::RuntimeCostKind,
        amount: u128,
    ) -> TracedRuntimeCost {
        TracedRuntimeCost {
            rule_id: "ic.cycles.execution.v1",
            unit,
            kind,
            amount,
            nominal_cycles: Some(amount),
            time: Some(Time::from_nanos_since_unix_epoch(42)),
            round: Some(7),
            call_context_id: None,
            message_id: None,
            callback_id: None,
            payload_bytes: None,
            instructions: None,
            description: "test accounting operation".into(),
            details: BTreeMap::new(),
        }
    }

    #[test]
    fn runtime_costs_preserve_units_scope_and_independent_pagination() {
        use ic_interfaces::execution_environment::{
            RuntimeCostKind as Kind, RuntimeCostUnit as Unit,
        };
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        let subnet = ic_types::SubnetId::from(ic_types::PrincipalId::new_subnet_test_id(1));
        recorder.start(canister);
        let large = u64::MAX as u128 + 2;
        recorder.record_runtime_cost(
            canister,
            runtime_event(Unit::Cycles, Kind::Prepayment, large),
        );
        recorder.record(canister, execution("run"));
        recorder.record_runtime_cost(
            canister,
            runtime_event(Unit::Cycles, Kind::Refund, large - 10),
        );
        let mut overhead = runtime_event(Unit::RoundInstructions, Kind::Debit, 12_000);
        overhead.rule_id = "ic.scheduler.finalization.v1";
        recorder.record_subnet_runtime_cost(subnet, overhead);
        let trace = recorder.get(canister);
        assert_eq!(trace.executions[0].sequence_number, 0);
        assert_eq!(trace.executions[0].instructions, 1_000);
        assert_eq!(trace.runtime_costs[0].amount, large);
        assert_eq!(trace.runtime_costs[1].amount, large - 10);
        assert_eq!(trace.subnet_runtime_costs[0].canister_id, None);
        assert_eq!(
            trace.subnet_runtime_costs[0].subnet_id.as_deref(),
            Some(subnet.to_string().as_str())
        );
        let first = recorder.runtime_costs_snapshot(canister, None, 2);
        assert!(first.more);
        assert_eq!(first.next_after, Some(1));
        let last = recorder.runtime_costs_snapshot(canister, first.next_after, 2);
        assert!(!last.more);
        assert_eq!(last.subnet_runtime_costs.len(), 1);
        assert_eq!(last.next_after, Some(2));
        assert!(!recorder.clear_if_unchanged(
            canister,
            trace.trace_generation,
            Some(0),
            Some(Some(1))
        ));
        assert!(recorder.clear_if_unchanged(
            canister,
            trace.trace_generation,
            Some(0),
            Some(Some(2))
        ));
        assert!(recorder.get(canister).runtime_costs.is_empty());
        assert!(recorder.get(canister).subnet_runtime_costs.is_empty());
        assert_eq!(recorder.traces.lock().unwrap().recorded_bytes, 0);
    }

    #[test]
    fn clearing_shared_history_invalidates_every_visible_canister_snapshot() {
        use ic_interfaces::execution_environment::{
            RuntimeCostKind as Kind, RuntimeCostUnit as Unit,
        };
        let recorder = MemoryAccessTraceRecorder::default();
        let first = CanisterId::from_u64(1);
        let other = CanisterId::from_u64(2);
        let unread = CanisterId::from_u64(3);
        recorder.start(first);
        recorder.start(other);
        let subnet = SubnetId::from(ic_types::PrincipalId::new_subnet_test_id(1));
        recorder.record_subnet_runtime_cost(
            subnet,
            runtime_event(Unit::RoundInstructions, Kind::Debit, 12_000),
        );
        let snapshot = recorder.get(other);
        let unread_generation = recorder.get(unread).trace_generation;
        recorder.clear(first);
        assert!(recorder.get(other).trace_generation > snapshot.trace_generation);
        assert!(recorder.get(unread).trace_generation > unread_generation);
        assert!(!recorder.clear_if_unchanged(other, snapshot.trace_generation, None, Some(None)));
        assert!(recorder.get(other).subnet_runtime_costs.is_empty());
        recorder.record_subnet_runtime_cost(
            subnet,
            runtime_event(Unit::RoundInstructions, Kind::Debit, 24_000),
        );
        let next = recorder.get(other);
        assert_eq!(next.subnet_runtime_cost_totals[0].amount, 24_000);
        assert_eq!(next.last_runtime_cost_sequence_number, Some(1));
    }

    #[test]
    fn automatic_detail_stop_preserves_host_refunds_and_committed_totals() {
        use ic_interfaces::execution_environment::{
            RuntimeCostKind as Kind, RuntimeCostUnit as Unit,
        };
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        recorder.start(canister);
        recorder.record_runtime_cost(canister, runtime_event(Unit::Cycles, Kind::Prepayment, 100));
        recorder.traces.lock().unwrap().recorded_bytes = TRACE_MEMORY_LIMIT - TRACE_SUMMARY_RESERVE;
        recorder.record(canister, execution("fills detail budget"));
        assert!(!recorder.is_tracing_enabled(canister));
        assert!(recorder.is_runtime_cost_tracing_enabled(canister));
        recorder.record_runtime_cost(canister, runtime_event(Unit::Cycles, Kind::Refund, 70));
        let trace = recorder.get(canister);
        assert!(trace.runtime_costs_recording);
        assert!(trace.runtime_costs_truncated);
        assert!(!trace.runtime_cost_totals_incomplete);
        let refund = trace
            .runtime_cost_totals
            .iter()
            .find(|row| row.kind == "refund")
            .unwrap();
        assert_eq!(refund.amount, 70);
        assert_eq!(trace.last_runtime_cost_sequence_number, Some(1));
        recorder.stop(canister);
        assert!(!recorder.is_runtime_cost_tracing_enabled(canister));
        recorder.record_runtime_cost(canister, runtime_event(Unit::Cycles, Kind::Debit, 1));
        assert_eq!(
            recorder.get(canister).last_runtime_cost_sequence_number,
            Some(1)
        );
    }

    #[test]
    fn upstream_aggregation_retains_event_counts_and_marks_missing_totals() {
        use ic_interfaces::execution_environment::{
            RuntimeCostKind as Kind, RuntimeCostUnit as Unit,
        };
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        recorder.start(canister);
        let mut aggregated = runtime_event(Unit::Cycles, Kind::Debit, 90);
        aggregated
            .details
            .insert("detail_truncated".into(), "true".into());
        aggregated
            .details
            .insert("aggregated_event_count".into(), "9".into());
        recorder.record_runtime_cost(canister, aggregated);
        let trace = recorder.get(canister);
        assert!(trace.runtime_costs_truncated);
        assert!(!trace.runtime_cost_totals_incomplete);
        assert_eq!(trace.runtime_cost_totals[0].events, 9);
        assert_eq!(trace.runtime_cost_totals[0].amount, 90);
        let mut missing = runtime_event(Unit::Cycles, Kind::Debit, 10);
        missing
            .details
            .insert("totals_incomplete".into(), "true".into());
        missing
            .details
            .insert("omitted_runtime_events".into(), "1".into());
        recorder.record_runtime_cost(canister, missing);
        assert!(recorder.get(canister).runtime_cost_totals_incomplete);
        recorder.clear(canister);
        assert!(!recorder.get(canister).runtime_cost_totals_incomplete);
    }

    #[test]
    fn runtime_detail_budget_keeps_exact_aggregates_and_clear_revision() {
        use ic_interfaces::execution_environment::{
            RuntimeCostKind as Kind, RuntimeCostUnit as Unit,
        };
        let recorder = MemoryAccessTraceRecorder::default();
        let canister = CanisterId::from_u64(1);
        recorder.start(canister);
        recorder.traces.lock().unwrap().recorded_bytes = TRACE_MEMORY_LIMIT;
        recorder.record_runtime_cost(
            canister,
            runtime_event(Unit::Cycles, Kind::Debit, u64::MAX as u128),
        );
        recorder.record_runtime_cost(
            canister,
            runtime_event(Unit::Cycles, Kind::Debit, u64::MAX as u128),
        );
        let trace = recorder.get(canister);
        assert!(trace.runtime_costs_truncated);
        assert!(!trace.runtime_cost_totals_incomplete);
        assert!(trace.runtime_costs.is_empty());
        assert_eq!(trace.runtime_cost_totals.len(), 1);
        assert_eq!(trace.runtime_cost_totals[0].events, 2);
        assert_eq!(trace.runtime_cost_totals[0].amount, 2 * u64::MAX as u128);
        assert_eq!(trace.last_runtime_cost_sequence_number, Some(1));
        assert!(!recorder.clear_if_unchanged(canister, trace.trace_generation, None, Some(None)));
        assert!(recorder.clear_if_unchanged(canister, trace.trace_generation, None, Some(Some(1))));
    }
}
