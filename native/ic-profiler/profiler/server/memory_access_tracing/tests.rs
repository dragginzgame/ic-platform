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
fn query_snapshot_selects_before_copying_and_keeps_export_and_cursor_unchanged() {
    use ic_interfaces::execution_environment::{RuntimeCostKind as Kind, RuntimeCostUnit as Unit};
    let recorder = MemoryAccessTraceRecorder::default();
    let canister = CanisterId::from_u64(1);
    recorder.start(canister);
    recorder.record(canister, execution("kept"));
    recorder.record(canister, execution("other"));
    recorder.record_runtime_cost(canister, runtime_event(Unit::Cycles, Kind::Debit, 7));
    let before = recorder.get(canister);
    let cursor = recorder.snapshot_cursor(canister);
    let query = recorder.query_snapshot(
        canister,
        &ExecutionFilter {
            function: Some("kept"),
            ..Default::default()
        },
    );
    assert_eq!(query.executions.len(), 1);
    assert_eq!(
        serde_json::to_value(&query.executions[0]).unwrap(),
        serde_json::to_value(&before.executions[0]).unwrap()
    );
    assert!(!before.runtime_costs.is_empty());
    assert!(query.runtime_costs.is_empty());
    assert!(query.subnet_runtime_costs.is_empty());
    assert_eq!(
        serde_json::to_value(&query.runtime_cost_totals).unwrap(),
        serde_json::to_value(&before.runtime_cost_totals).unwrap()
    );
    assert_eq!(query.trace_generation, cursor.generation);
    assert_eq!(recorder.snapshot_cursor(canister), cursor);
    assert_eq!(
        serde_json::to_value(recorder.get(canister)).unwrap(),
        serde_json::to_value(before).unwrap()
    );
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
    let evidence = recorder
        .snapshot_touched_pages(canister, trace.trace_generation, None, Some(0), None, None)
        .into_summary();
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
    let all = recorder
        .snapshot_touched_pages(
            canister,
            trace.trace_generation,
            None,
            Some(1),
            None,
            Some(logical),
        )
        .into_summary();
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
    let first = recorder
        .snapshot_touched_pages(
            canister,
            trace.trace_generation,
            None,
            Some(1),
            Some(1),
            None,
        )
        .into_summary();
    assert_eq!(first["execution_count"], "1");
    assert_eq!(
        first["known_touched_ranges"]["stable"],
        serde_json::json!([["0", "2"], ["7000000", "7000001"]])
    );
    assert_eq!(trace.executions[0].stable_memory_size, 8_000_000);
    recorder.clear(canister);
    assert_eq!(
        recorder
            .snapshot_touched_pages(canister, trace.trace_generation, None, Some(1), None, None)
            .into_summary()["available"],
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
    let evidence = recorder
        .snapshot_touched_pages(canister, trace.trace_generation, None, Some(0), None, None)
        .into_summary();
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
fn selected_unprofiled_execution_keeps_its_available_module_dictionary() {
    use ic_interfaces::execution_environment::{
        WasmInstructionModuleMetadata, WasmInstructionProfile,
    };
    let recorder = MemoryAccessTraceRecorder::default();
    let canister = CanisterId::from_u64(1);
    let mut profiled = execution("profiled");
    profiled.trace.instruction_profile = Some(WasmInstructionProfile {
        module: Some(WasmInstructionModuleMetadata {
            module_sha256: [0xab; 32],
            ..Default::default()
        }),
        ..Default::default()
    });
    recorder.record(canister, profiled);
    recorder.record(canister, execution("unprofiled"));
    let full = recorder.get(canister);
    let query = recorder.query_snapshot(
        canister,
        &ExecutionFilter {
            function: Some("unprofiled"),
            ..Default::default()
        },
    );
    assert_eq!(query.executions.len(), 1);
    assert!(query.executions[0].instruction_profile.is_none());
    assert_eq!(query.modules.len(), 1);
    assert_eq!(
        serde_json::to_value(&query.modules).unwrap(),
        serde_json::to_value(&full.modules).unwrap()
    );
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
        trace
            .executions
            .iter()
            .all(|e| e.instruction_profile.as_ref().unwrap().module.is_none())
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
    use ic_interfaces::execution_environment::{RuntimeCostKind as Kind, RuntimeCostUnit as Unit};
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
    assert!(!recorder.clear_if_unchanged(canister, trace.trace_generation, Some(0), Some(Some(1))));
    assert!(recorder.clear_if_unchanged(canister, trace.trace_generation, Some(0), Some(Some(2))));
    assert!(recorder.get(canister).runtime_costs.is_empty());
    assert!(recorder.get(canister).subnet_runtime_costs.is_empty());
    assert_eq!(recorder.traces.lock().unwrap().recorded_bytes, 0);
}

#[test]
fn clearing_shared_history_invalidates_every_visible_canister_snapshot() {
    use ic_interfaces::execution_environment::{RuntimeCostKind as Kind, RuntimeCostUnit as Unit};
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
    use ic_interfaces::execution_environment::{RuntimeCostKind as Kind, RuntimeCostUnit as Unit};
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
    use ic_interfaces::execution_environment::{RuntimeCostKind as Kind, RuntimeCostUnit as Unit};
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
    use ic_interfaces::execution_environment::{RuntimeCostKind as Kind, RuntimeCostUnit as Unit};
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
