use super::*;
use ic_interfaces::execution_environment::WasmMemoryAccessTrace;
use ic_types::methods::SystemMethod;
use std::sync::Mutex;

#[test]
fn profiling_after_install_preserves_ordinary_code_and_instruction_charges() {
    let logger = ic_logger::replica_logger::no_op_logger();
    let fd_factory: Arc<dyn PageAllocatorFileDescriptor> =
        Arc::new(ic_replicated_state::page_map::TestPageAllocatorFileDescriptorImpl);
    let executor = WasmExecutorImpl::new(
        WasmtimeEmbedder::new(
            ic_config::embedders::Config {
                page_overhead: NumInstructions::new(5_000),
                ..Default::default()
            },
            logger.clone(),
        ),
        &MetricsRegistry::new(),
        logger,
        Arc::clone(&fd_factory),
    );
    let binary = WasmBinary::new(CanisterModule::new(
        wat::parse_str(
            r#"(module (memory 1) (global (export "value") (mut i64) (i64.const 7))
            (func (export "canister_update test")
                (i32.store8 (i32.const 8) (i32.const 42))
                (drop (i32.load8_u (i32.const 8)))))"#,
        )
        .unwrap(),
    ));
    let compilation_cache = Arc::new(crate::CompilationCacheBuilder::new().build());
    let ordinary = executor
        .get_embedder_cache(&binary, Arc::clone(&compilation_cache))
        .unwrap();
    assert!(
        binary.profiling_embedder_cache.get().is_none(),
        "installation must not allocate a diagnostic cache"
    );
    let has_metadata = |cache: &EmbedderCache| {
        cache
            .downcast::<HypervisorResult<crate::wasmtime_embedder::WasmtimeInstancePre>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .profiling_metadata
            .is_some()
    };
    assert!(!has_metadata(&ordinary.cache));

    let diagnostic = executor
        .get_embedder_cache_for_profiling(&binary, Arc::clone(&compilation_cache), true)
        .unwrap();
    assert!(has_metadata(&diagnostic.cache));
    let after = executor
        .get_embedder_cache(&binary, Arc::clone(&compilation_cache))
        .unwrap();
    assert!(after.compilation_result.is_none());
    assert!(
        !has_metadata(&after.cache),
        "stopping a profile must select ordinary code immediately"
    );

    let run = |cache: &EmbedderCache, profile: u8| {
        let heap = Memory::new(PageMap::new(Arc::clone(&fd_factory)), NumWasmPages::from(1));
        let stable = Memory::new(PageMap::new(Arc::clone(&fd_factory)), NumWasmPages::from(0));
        let mut instance = executor
            .wasm_embedder
            .new_instance(
                CanisterId::ic_00(),
                cache,
                None,
                &heap,
                &stable,
                ModificationTracking::Track,
                None,
            )
            .map_err(|(error, _)| error)
            .unwrap();
        instance.set_instruction_counter(1_000_000);
        if profile == 1 {
            instance.enable_wasm_memory_access_tracing();
        } else if profile == 2 {
            instance.enable_instruction_profiling();
        }
        let result = instance
            .run(FuncRef::Method(WasmMethod::Update("test".to_string())))
            .unwrap();
        let pages: Vec<_> = compute_page_delta(
            &mut instance,
            &result.wasm_dirty_pages,
            CanisterMemoryType::Heap,
        )
        .into_iter()
        .map(|(index, bytes)| (index, bytes.to_vec()))
        .collect();
        let page_cost = instance
            .get_stats()
            .wasm_memory_access_trace
            .map(|trace| trace.page_access_instructions);
        (
            (
                instance.instruction_counter(),
                result.exported_globals,
                pages,
            ),
            page_cost,
        )
    };
    let ordinary_run = run(&ordinary.cache, 0);
    assert!(ordinary_run.1.is_none());
    let expected = ordinary_run.0;
    let page_only = run(&ordinary.cache, 1);
    assert_eq!(page_only.0, expected);
    assert!(
        page_only.1.unwrap() > 0,
        "page-only tracing must attach its debit observer lazily to ordinary code"
    );
    assert_eq!(
        run(&diagnostic.cache, 2).0,
        expected,
        "diagnostic hooks must not alter state or charges"
    );
    assert_eq!(run(&after.cache, 0).0, expected);

    // Both serialized variants also survive eviction of the local native cache.
    binary.clear_compilation_cache();
    let ordinary = executor
        .get_embedder_cache(&binary, Arc::clone(&compilation_cache))
        .unwrap();
    let diagnostic = executor
        .get_embedder_cache_for_profiling(&binary, compilation_cache, true)
        .unwrap();
    assert!(ordinary.compilation_result.is_none());
    assert!(diagnostic.compilation_result.is_none());
    assert!(!has_metadata(&ordinary.cache));
    assert!(has_metadata(&diagnostic.cache));
}

#[test]
fn nonsandbox_disk_cache_retains_exact_instrumented_origin_dictionary() {
    let logger = ic_logger::replica_logger::no_op_logger();
    let executor = WasmExecutorImpl::new(
        WasmtimeEmbedder::new(
            ic_config::embedders::Config {
                instruction_charge_profiling: true,
                ..Default::default()
            },
            logger.clone(),
        ),
        &MetricsRegistry::new(),
        logger,
        Arc::new(ic_replicated_state::page_map::TestPageAllocatorFileDescriptorImpl),
    );
    let binary=WasmBinary::new(CanisterModule::new(wat::parse_str("(module (memory 1) (func (export \"canister_update test\") (drop (memory.grow (i32.const 1)))))").unwrap()));
    let compilation_cache = Arc::new(crate::CompilationCacheBuilder::new().build());
    let first = executor
        .get_embedder_cache(&binary, Arc::clone(&compilation_cache))
        .unwrap();
    let expected = first
        .cache
        .downcast::<HypervisorResult<crate::wasmtime_embedder::WasmtimeInstancePre>>()
        .unwrap()
        .as_ref()
        .unwrap()
        .profiling_metadata
        .clone()
        .unwrap();
    assert!(!expected.instrumented_ranges.is_empty());
    binary.clear_compilation_cache();
    let restored = executor
        .get_embedder_cache(&binary, compilation_cache)
        .unwrap();
    assert!(
        restored.compilation_result.is_none(),
        "must exercise disk cache hit"
    );
    let actual = restored
        .cache
        .downcast::<HypervisorResult<crate::wasmtime_embedder::WasmtimeInstancePre>>()
        .unwrap()
        .as_ref()
        .unwrap()
        .profiling_metadata
        .as_ref()
        .unwrap();
    assert_eq!(expected.as_ref(), actual.as_ref());
}

#[derive(Default)]
struct Observations {
    costs: Mutex<Vec<TracedRuntimeCost>>,
    executions: Mutex<Vec<TracedWasmExecution>>,
}

impl WasmMemoryAccessTracer for Observations {
    fn is_tracing_enabled(&self, _: CanisterId) -> bool {
        true
    }

    fn record_runtime_cost(&self, _: CanisterId, cost: TracedRuntimeCost) {
        self.costs.lock().unwrap().push(cost);
    }

    fn record(&self, _: CanisterId, execution: TracedWasmExecution) {
        self.executions.lock().unwrap().push(execution);
    }
}

#[test]
fn physical_attempt_slices_are_bounded_and_abort_is_not_a_second_charge() {
    let observations = Arc::new(Observations::default());
    let tracer: Arc<dyn WasmMemoryAccessTracer> = observations.clone();
    let mut tracing = WasmMemoryAccessTracing::start(
        Some(&tracer),
        CanisterId::from_u64(1),
        &ApiType::init(
            Time::from_nanos_since_unix_epoch(1),
            vec![],
            CanisterId::from_u64(2).get(),
        ),
        &FuncRef::Method(WasmMethod::Update("test".into())),
        &CanisterModule::new(b"\0asm\x01\0\0\0".to_vec()),
    )
    .unwrap();
    let id = tracing.attempt_id();
    for i in 0..300 {
        tracing.observe_slice(
            &SliceExecutionOutput {
                executed_instructions: NumInstructions::new(i),
            },
            if i == 299 {
                "paused_dirty_page_copy"
            } else {
                "paused_instruction_budget"
            },
        );
    }
    let mut output = WasmExecutionOutput {
        wasm_result: Err(HypervisorError::Aborted),
        num_instructions_left: NumInstructions::new(500),
        allocated_bytes: NumBytes::new(0),
        allocated_guaranteed_response_message_bytes: NumBytes::new(0),
        new_memory_usage: None,
        new_message_memory_usage: None,
        instance_stats: InstanceStats {
            wasm_memory_access_trace: Some(Default::default()),
            ..Default::default()
        },
        system_api_call_counters: Default::default(),
    };
    tracing.finish(NumInstructions::new(1_000), &mut output);
    assert_eq!(output.instance_stats.trace_attempt_id, Some(id));
    let records = observations.executions.lock().unwrap();
    let record = &records[0];
    assert_eq!(record.attempt.attempt_id, id);
    assert_eq!(record.outcome, TracedWasmExecutionOutcome::Aborted);
    assert_eq!(record.attempt.status, "aborted");
    assert_eq!(record.attempt.slice_count, 300);
    assert_eq!(record.attempt.slices.len(), 256);
    assert_eq!(
        record.attempt.total_round_instructions,
        (0..300u128).sum::<u128>()
    );
    assert!(record.attempt.slice_details_truncated);
    assert!(!record.attempt.memory_details_available);
    assert!(record.attempt.instructions_available);
    assert_eq!(record.instructions_executed.get(), 500);
    drop(records);
    let tracing = WasmMemoryAccessTracing::start(
        Some(&tracer),
        CanisterId::from_u64(1),
        &ApiType::init(
            Time::from_nanos_since_unix_epoch(1),
            vec![],
            CanisterId::from_u64(2).get(),
        ),
        &FuncRef::Method(WasmMethod::Update("test".into())),
        &CanisterModule::new(b"\0asm\x01\0\0\0".to_vec()),
    )
    .unwrap();
    assert_ne!(tracing.attempt_id(), id);
    tracing.finish_unavailable_abort();
    let records = observations.executions.lock().unwrap();
    assert!(!records[1].attempt.instructions_available);
    assert!(!records[1].attempt.memory_details_available);
    assert!(records[1].trace.instruction_profile.is_none());
    drop(records);
    let tracing = WasmMemoryAccessTracing::start(
        Some(&tracer),
        CanisterId::from_u64(1),
        &ApiType::init(
            Time::from_nanos_since_unix_epoch(1),
            vec![],
            CanisterId::from_u64(2).get(),
        ),
        &FuncRef::Method(WasmMethod::Update("test".into())),
        &CanisterModule::new(b"\0asm\x01\0\0\0".to_vec()),
    )
    .unwrap();
    // Instantiation can fail before Wasm runs and before any snapshot exists.
    output.instance_stats.wasm_memory_access_trace = None;
    tracing.finish(NumInstructions::new(1_000), &mut output);
    assert_eq!(output.instance_stats.trace_attempt_id, None);
    assert_eq!(observations.executions.lock().unwrap().len(), 2);
}

#[derive(Default)]
struct HostOnlyObservations(Observations);

impl WasmMemoryAccessTracer for HostOnlyObservations {
    fn is_tracing_enabled(&self, _: CanisterId) -> bool {
        false
    }

    fn is_runtime_cost_tracing_enabled(&self, _: CanisterId) -> bool {
        true
    }

    fn take_instruction_profile(&self, _: CanisterId) -> bool {
        panic!("Host-only observation must not consume a detailed profile budget");
    }

    fn record_runtime_cost(&self, canister_id: CanisterId, cost: TracedRuntimeCost) {
        self.0.record_runtime_cost(canister_id, cost);
    }

    fn record(&self, canister_id: CanisterId, execution: TracedWasmExecution) {
        self.0.record(canister_id, execution);
    }
}

#[test]
fn host_only_tracing_keeps_dirty_copy_cost_after_detail_budget_stops() {
    let observations = Arc::new(HostOnlyObservations::default());
    let tracer: Arc<dyn WasmMemoryAccessTracer> = observations.clone();
    let time = Time::from_nanos_since_unix_epoch(123);
    let api_type = ApiType::system_task(
        SystemMethod::CanisterGlobalTimer,
        time,
        CallContextId::from(7),
    );
    let tracing = WasmMemoryAccessTracing::start(
        Some(&tracer),
        CanisterId::from_u64(1),
        &api_type,
        &FuncRef::Method(WasmMethod::System(SystemMethod::CanisterGlobalTimer)),
        &CanisterModule::new(b"\0asm\x01\0\0\0".to_vec()),
    )
    .expect("Host costs remain observable after detailed tracing stops");
    assert!(!tracing.memory_tracing_enabled);
    assert!(!tracing.instruction_profiling);
    let mut output = WasmExecutionOutput {
        wasm_result: Ok(None),
        num_instructions_left: NumInstructions::new(900),
        allocated_bytes: NumBytes::new(0),
        allocated_guaranteed_response_message_bytes: NumBytes::new(0),
        new_memory_usage: None,
        new_message_memory_usage: None,
        instance_stats: InstanceStats {
            dirty_page_copy_instructions: 48_000,
            ..InstanceStats::default()
        },
        system_api_call_counters: SystemApiCallCounters::default(),
    };

    tracing.finish(NumInstructions::new(1_000), &mut output);

    let costs = observations.0.costs.lock().unwrap();
    assert_eq!(costs.len(), 1);
    assert_eq!(costs[0].rule_id, "ic.scheduler.dirty_page_copy.v1");
    assert_eq!(costs[0].unit, RuntimeCostUnit::RoundInstructions);
    assert_eq!(costs[0].amount, 48_000);
    assert_eq!(costs[0].call_context_id, Some(CallContextId::from(7)));
    assert!(observations.0.executions.lock().unwrap().is_empty());
    assert!(output.instance_stats.wasm_memory_access_trace.is_none());
    assert_eq!(output.instance_stats.trace_attempt_id, None);
    assert_eq!(output.num_instructions_left.get(), 900);
}

#[test]
fn dirty_copy_round_cost_survives_ipc_without_changing_guest_instructions() {
    for dirty_page_copy_instructions in [0, 48_000] {
        let observations = Arc::new(Observations::default());
        let tracing = WasmMemoryAccessTracing {
            tracer: observations.clone(),
            canister_id: CanisterId::from_u64(1),
            api_type: "update",
            function: "canister_update dirty".to_string(),
            call_context_id: Some(CallContextId::from(7)),
            time: Time::from_nanos_since_unix_epoch(123),
            module_hash: "00".repeat(32),
            module_hash_kind: "installed_uncompressed_sha256",
            memory_tracing_enabled: true,
            instruction_profiling: false,
            read_mask: false,
            write_mask: false,
            attempt: WasmExecutionAttempt {
                attempt_id: 42,
                status: "completed".to_string(),
                slice_count: 0,
                total_round_instructions: 0,
                slices: Vec::new(),
                slice_details_truncated: false,
                memory_details_available: true,
                instructions_available: true,
            },
        };
        let stats = InstanceStats {
            wasm_memory_access_trace: Some(WasmMemoryAccessTrace::default()),
            dirty_page_copy_instructions,
            ..InstanceStats::default()
        };
        // Sandbox output transports the accounting observation in these
        // statistics. Exercise the actual serialization boundary as well.
        let stats = bincode::deserialize(&bincode::serialize(&stats).unwrap()).unwrap();
        let mut output = WasmExecutionOutput {
            wasm_result: Ok(None),
            num_instructions_left: NumInstructions::new(900),
            allocated_bytes: NumBytes::new(0),
            allocated_guaranteed_response_message_bytes: NumBytes::new(0),
            new_memory_usage: None,
            new_message_memory_usage: None,
            instance_stats: stats,
            system_api_call_counters: SystemApiCallCounters::default(),
        };

        tracing.finish(NumInstructions::new(1_000), &mut output);

        let costs = observations.costs.lock().unwrap();
        if dirty_page_copy_instructions == 0 {
            assert!(costs.is_empty());
        } else {
            assert_eq!(costs.len(), 1);
            assert_eq!(costs[0].unit, RuntimeCostUnit::RoundInstructions);
            assert_eq!(costs[0].amount, dirty_page_copy_instructions as u128);
            assert_eq!(costs[0].call_context_id, Some(CallContextId::from(7)));
        }
        let executions = observations.executions.lock().unwrap();
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].instructions_executed.get(), 100);
        assert_eq!(output.num_instructions_left.get(), 900);
    }
}
