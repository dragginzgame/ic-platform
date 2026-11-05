use super::*;
use ic_embedders::{InstanceRunResult, WasmtimeEmbedder};
use ic_interfaces::execution_environment::{HypervisorResult, InstanceStats};
use ic_logger::replica_logger::no_op_logger;
use ic_wasm_types::BinaryEncodedWasm;

fn run_read_mask(
    wat: &str,
    instrumented: bool,
    collecting: bool,
) -> (HypervisorResult<InstanceRunResult>, u64, InstanceStats) {
    let mut instance = WasmtimeInstanceBuilder::new()
        .with_config(Config {
            instruction_charge_profiling: instrumented,
            ..Config::default()
        })
        .with_wat(wat)
        .with_num_instructions(NumInstructions::new(1_000_000))
        .with_page_overhead(NumInstructions::new(10))
        .build();
    instance.enable_wasm_memory_access_tracing();
    if instrumented {
        instance.enable_instruction_profiling();
    }
    if collecting {
        instance.enable_read_mask();
    }
    let result = instance.run(FuncRef::Method(WasmMethod::Update("test".into())));
    let counter = instance.instruction_counter();
    let instructions = instance
        .store_data()
        .system_api()
        .unwrap()
        .slice_instructions_executed(counter)
        .get();
    (result, instructions, instance.get_stats())
}

fn heap_reads(
    profile: &ic_interfaces::execution_environment::WasmReadProfile,
    address: u64,
) -> u64 {
    profile
        .ranges
        .iter()
        .find(|range| range.memory == "heap" && range.start <= address && address < range.end)
        .map_or(0, |range| range.heap_reads)
}

fn run_write_mask(
    wat: &str,
    instrumented: bool,
    collecting: bool,
) -> (HypervisorResult<InstanceRunResult>, u64, InstanceStats) {
    let mut instance = WasmtimeInstanceBuilder::new()
        .with_config(Config {
            instruction_charge_profiling: instrumented,
            ..Config::default()
        })
        .with_wat(wat)
        .with_api_type(ApiType::init(
            UNIX_EPOCH,
            vec![9, 8, 7],
            user_test_id(24).get(),
        ))
        .with_num_instructions(NumInstructions::new(1_000_000))
        .with_page_overhead(NumInstructions::new(10))
        .build();
    instance.enable_wasm_memory_access_tracing();
    if instrumented {
        instance.enable_instruction_profiling();
    }
    if collecting {
        instance.enable_write_mask();
    }
    let result = instance.run(FuncRef::Method(WasmMethod::Update("test".into())));
    let counter = instance.instruction_counter();
    let instructions = instance
        .store_data()
        .system_api()
        .unwrap()
        .slice_instructions_executed(counter)
        .get();
    (result, instructions, instance.get_stats())
}

fn writes_at(
    profile: &ic_interfaces::execution_environment::WasmWriteProfile,
    memory: &str,
    address: u64,
) -> u64 {
    profile
        .ranges
        .iter()
        .find(|range| range.memory == memory && range.start <= address && address < range.end)
        .map_or(0, |range| {
            if memory == "heap" {
                range.heap_writes
            } else {
                range.stable_writes
            }
        })
}

#[test]
fn write_mask_scalar_simd_bulk_host_and_same_value_writes_preserve_billing() {
    for (memory, address_type) in [("", "i32"), ("i64", "i64")] {
        let wat = r#"(module
            (import "ic0" "msg_arg_data_copy" (func $args (param ADDR ADDR ADDR)))
            (memory MEMORY 2)
            (data "\01\02\03\04")
            (global $value (export "value") (mut i32) (i32.const 0))
            (func (export "canister_update test")
                (i32.store8 offset=3 (ADDR.const 10) (i32.const 0))
                (i32.store8 offset=3 (ADDR.const 10) (i32.const 7))
                (i32.store8 offset=3 (ADDR.const 10) (i32.const 0))
                (i32.store16 (ADDR.const 20) (i32.const 123))
                (i64.store32 (ADDR.const 24) (i64.const 0))
                (f32.store (ADDR.const 32) (f32.const 1.5))
                (f64.store (ADDR.const 40) (f64.const 2.5))
                (v128.store (ADDR.const 56) (v128.const i32x4 0 0 0 0))
                (v128.store8_lane 0 (ADDR.const 80) (v128.const i32x4 0 0 0 0))
                (v128.store16_lane 0 (ADDR.const 84) (v128.const i32x4 0 0 0 0))
                (v128.store32_lane 0 (ADDR.const 88) (v128.const i32x4 0 0 0 0))
                (v128.store64_lane 0 (ADDR.const 96) (v128.const i32x4 0 0 0 0))
                (i32.store (ADDR.const 112) (i32.const 0))
                (i64.store (ADDR.const 120) (i64.const 0))
                (i64.store8 (ADDR.const 132) (i64.const 0))
                (i64.store16 (ADDR.const 136) (i64.const 0))
                (memory.copy (ADDR.const 200) (ADDR.const 300) (ADDR.const 5))
                (memory.fill (ADDR.const 220) (i32.const 0) (ADDR.const 7))
                (memory.init 0 (ADDR.const 240) (i32.const 1) (i32.const 2))
                (call $args (ADDR.const 260) (ADDR.const 0) (ADDR.const 3))
                (memory.copy (ADDR.const 0) (ADDR.const 0) (ADDR.const 0))
                (memory.fill (ADDR.const 0) (i32.const 0) (ADDR.const 0))
                (memory.init 0 (ADDR.const 0) (i32.const 0) (i32.const 0))
                (global.set $value (i32.load8_u (ADDR.const 13)))))"#
            .replace("MEMORY", memory)
            .replace("ADDR", address_type);
        let (control, control_instructions, control_stats) = run_write_mask(&wat, false, false);
        let (observed, observed_instructions, observed_stats) = run_write_mask(&wat, true, true);
        let control = control.unwrap();
        let observed = observed.unwrap();
        assert_eq!(control_instructions, observed_instructions);
        assert_eq!(control.exported_globals, observed.exported_globals);
        assert_eq!(
            observed.exported_globals[0],
            Global::I32(0),
            "write-and-restore has no net data change"
        );
        assert_eq!(control.wasm_dirty_pages, observed.wasm_dirty_pages);
        let control_trace = control_stats.wasm_memory_access_trace.unwrap();
        let observed_trace = observed_stats.wasm_memory_access_trace.unwrap();
        assert_eq!(control_trace.accessed_pages, observed_trace.accessed_pages);
        assert_eq!(control_trace.dirty_pages, observed_trace.dirty_pages);
        assert!(control_trace.write_profile.is_none());
        assert!(
            observed_trace.read_profile.is_none(),
            "write collection is independent of reads"
        );
        let writes = observed_trace.write_profile.unwrap();
        assert!(!writes.truncated);
        assert_eq!(writes.observed_writes, 20);
        assert_eq!(
            writes_at(&writes, "heap", 13),
            3,
            "same-value writes and restored bytes remain visible"
        );
        assert_eq!(writes_at(&writes, "heap", 12), 0);
        for (start, length) in [
            (20, 2),
            (24, 4),
            (32, 4),
            (40, 8),
            (56, 16),
            (80, 1),
            (84, 2),
            (88, 4),
            (96, 8),
            (112, 4),
            (120, 8),
            (132, 1),
            (136, 2),
            (200, 5),
            (220, 7),
            (240, 2),
            (260, 3),
        ] {
            for byte in start..start + length {
                assert_eq!(writes_at(&writes, "heap", byte), 1, "byte {byte}");
            }
            assert_eq!(writes_at(&writes, "heap", start + length), 0);
        }
        assert_eq!(
            writes_at(&writes, "heap", 300),
            0,
            "copy source is not a write"
        );
        assert_eq!(
            writes_at(&writes, "heap", 0),
            0,
            "zero-length bulk operations are not writes"
        );
        let (_, _, gated_stats) = run_write_mask(&wat, true, false);
        assert!(
            gated_stats
                .wasm_memory_access_trace
                .unwrap()
                .write_profile
                .is_none()
        );

        let wasm = BinaryEncodedWasm::new(wat::parse_str(&wat).unwrap());
        let mut compilation_costs = Vec::new();
        for profiling in [false, true] {
            let (_, output) = ic_embedders::wasm_utils::validate_and_instrument_for_testing(
                &WasmtimeEmbedder::new(
                    Config {
                        instruction_charge_profiling: profiling,
                        ..Config::default()
                    },
                    no_op_logger(),
                ),
                &wasm,
            )
            .unwrap();
            compilation_costs.push(output.compilation_cost);
        }
        assert_eq!(compilation_costs[0], compilation_costs[1]);
    }
}

#[test]
fn write_mask_stable32_stable64_and_direct_native_heap_destinations_are_exact() {
    let wat = r#"(module
        (import "ic0" "stable_grow" (func $grow (param i32) (result i32)))
        (import "ic0" "stable64_read" (func $read (param i64 i64 i64)))
        (import "ic0" "stable_read" (func $read32 (param i32 i32 i32)))
        (import "ic0" "stable64_write" (func $write (param i64 i64 i64)))
        (import "ic0" "stable_write" (func $write32 (param i32 i32 i32)))
        (memory 1)
        (func (export "canister_update test")
            (drop (call $grow (i32.const 2)))
            (call $read (i64.const 0) (i64.const 65536) (i64.const 4))
            (call $read32 (i32.const 8) (i32.const 65536) (i32.const 4))
            (call $write (i64.const 10) (i64.const 32) (i64.const 4))
            (call $write32 (i32.const 12) (i32.const 40) (i32.const 2))
            (call $write (i64.const 10) (i64.const 32) (i64.const 4))
            (call $read (i64.const 16) (i64.const 10) (i64.const 4))
            (call $write (i64.const -1) (i64.const -1) (i64.const 0))))"#;
    let (control, control_instructions, control_stats) = run_write_mask(wat, false, false);
    let (observed, observed_instructions, observed_stats) = run_write_mask(wat, true, true);
    let control = control.unwrap();
    let observed = observed.unwrap();
    assert_eq!(control_instructions, observed_instructions);
    assert_eq!(control.wasm_dirty_pages, observed.wasm_dirty_pages);
    assert_eq!(
        control.stable_memory_dirty_pages,
        observed.stable_memory_dirty_pages
    );
    let control_trace = control_stats.wasm_memory_access_trace.unwrap();
    let observed_trace = observed_stats.wasm_memory_access_trace.unwrap();
    assert_eq!(
        control_trace.stable_accessed_pages,
        observed_trace.stable_accessed_pages
    );
    let writes = observed_trace.write_profile.unwrap();
    assert_eq!(writes.observed_writes, 6);
    for start in [0, 8, 16] {
        for byte in start..start + 4 {
            assert_eq!(
                writes_at(&writes, "heap", byte),
                1,
                "stable read destination counted once"
            );
        }
    }
    assert_eq!(
        writes_at(&writes, "heap", 32),
        0,
        "stable write heap source is a read"
    );
    assert_eq!(writes_at(&writes, "stable", 10), 2);
    assert_eq!(writes_at(&writes, "stable", 12), 3);
    assert_eq!(writes_at(&writes, "stable", 14), 0);
    assert_eq!(
        writes_at(&writes, "stable", 65536),
        0,
        "stable read source is not a write"
    );
}

#[test]
fn write_mask_rejected_store_bulk_and_host_source_do_not_report_completed_writes() {
    for failing_operation in [
        "(i64.store (i32.const 65532) (i64.const 0))",
        "(memory.copy (i32.const 65532) (i32.const 100) (i32.const 8))",
        "(call $args (i32.const 100) (i32.const 1) (i32.const 3))",
    ] {
        let wat = format!(
            r#"(module
            (import "ic0" "msg_arg_data_copy" (func $args (param i32 i32 i32)))
            (memory 1)
            (func (export "canister_update test")
                (i32.store8 (i32.const 3) (i32.const 0))
                {failing_operation}))"#
        );
        let (control, control_instructions, _) = run_write_mask(&wat, false, false);
        let (observed, observed_instructions, stats) = run_write_mask(&wat, true, true);
        assert!(control.is_err());
        assert!(observed.is_err());
        assert_eq!(control_instructions, observed_instructions);
        let writes = stats
            .wasm_memory_access_trace
            .unwrap()
            .write_profile
            .unwrap();
        assert_eq!(writes.observed_writes, 1);
        assert_eq!(writes.observed_bytes, 1);
        assert_eq!(writes_at(&writes, "heap", 3), 1);
        assert_eq!(
            writes_at(&writes, "heap", 100),
            0,
            "valid destination with rejected source is not a write"
        );
    }
}

#[test]
fn read_mask_scalar_simd_bulk_and_host_reads_preserve_execution_and_billing() {
    for (memory, address_type) in [("", "i32"), ("i64", "i64")] {
        let wat = r#"(module
            (import "ic0" "certified_data_set" (func $cert (param ADDR ADDR)))
            (memory MEMORY 2)
            (global $value (export "value") (mut i32) (i32.const 0))
            (func (export "canister_update test")
                (drop (i32.load8_u offset=3 (ADDR.const 10)))
                (drop (i32.load8_u offset=3 (ADDR.const 10)))
                (drop (i32.load16_u (ADDR.const 20)))
                (drop (i64.load32_u (ADDR.const 24)))
                (drop (f64.load (ADDR.const 32)))
                (drop (v128.load (ADDR.const 48)))
                (drop (v128.load8x8_u (ADDR.const 64)))
                (drop (v128.load8_lane 0 (ADDR.const 80) (v128.const i32x4 0 0 0 0)))
                (memory.copy (ADDR.const 200) (ADDR.const 96) (ADDR.const 5))
                (call $cert (ADDR.const 120) (ADDR.const 3))
                (global.set $value (i32.load (ADDR.const 0)))))"#
            .replace("MEMORY", memory)
            .replace("ADDR", address_type);
        let (control, control_instructions, control_stats) = run_read_mask(&wat, false, false);
        let (observed, observed_instructions, observed_stats) = run_read_mask(&wat, true, true);
        let control = control.unwrap();
        let observed = observed.unwrap();
        assert_eq!(control_instructions, observed_instructions);
        assert_eq!(control.exported_globals, observed.exported_globals);
        assert_eq!(control.wasm_dirty_pages, observed.wasm_dirty_pages);
        let control_trace = control_stats.wasm_memory_access_trace.unwrap();
        let observed_trace = observed_stats.wasm_memory_access_trace.unwrap();
        assert_eq!(control_trace.accessed_pages, observed_trace.accessed_pages);
        assert_eq!(control_trace.dirty_pages, observed_trace.dirty_pages);
        assert!(control_trace.read_profile.is_none());
        let reads = observed_trace.read_profile.unwrap();
        assert!(!reads.truncated);
        assert_eq!(reads.observed_reads, 11);
        assert_eq!(heap_reads(&reads, 13), 2);
        assert_eq!(heap_reads(&reads, 12), 0);
        for (start, length) in [
            (0, 4),
            (20, 2),
            (24, 4),
            (32, 8),
            (48, 16),
            (64, 8),
            (80, 1),
            (96, 5),
            (120, 3),
        ] {
            for byte in start..start + length {
                assert_eq!(heap_reads(&reads, byte), 1, "byte {byte}");
            }
            if !matches!(start + length, 64) {
                assert_eq!(heap_reads(&reads, start + length), 0);
            }
        }
        assert_eq!(heap_reads(&reads, 200), 0, "copy destination is not a read");
        assert!(reads.ranges.iter().all(|range| range.memory == "heap"));
        let (_, _, gated_stats) = run_read_mask(&wat, true, false);
        assert!(
            gated_stats
                .wasm_memory_access_trace
                .unwrap()
                .read_profile
                .is_none()
        );

        // Diagnostics add neither compilation nor execution instructions.
        let wasm = BinaryEncodedWasm::new(wat::parse_str(&wat).unwrap());
        let mut compilation_costs = Vec::new();
        for profiling in [false, true] {
            let (_, output) = ic_embedders::wasm_utils::validate_and_instrument_for_testing(
                &WasmtimeEmbedder::new(
                    Config {
                        instruction_charge_profiling: profiling,
                        ..Config::default()
                    },
                    no_op_logger(),
                ),
                &wasm,
            )
            .unwrap();
            compilation_costs.push(output.compilation_cost);
        }
        assert_eq!(compilation_costs[0], compilation_costs[1]);
    }
}

#[test]
fn read_mask_stable_direct_native_repeats_and_disjoint_reads_are_distinct() {
    let wat = r#"(module
        (import "ic0" "stable_grow" (func $grow (param i32) (result i32)))
        (import "ic0" "stable64_read" (func $read (param i64 i64 i64)))
        (import "ic0" "stable_read" (func $read32 (param i32 i32 i32)))
        (import "ic0" "stable64_write" (func $write (param i64 i64 i64)))
        (memory 1)
        (func (export "canister_update test")
            (drop (call $grow (i32.const 2)))
            (call $read (i64.const 0) (i64.const 0) (i64.const 4))
            (call $read (i64.const 8) (i64.const 0) (i64.const 4))
            (call $read32 (i32.const 16) (i32.const 0) (i32.const 4))
            (call $read (i64.const 24) (i64.const 4096) (i64.const 4))
            (call $read (i64.const 32) (i64.const 65535) (i64.const 2))
            (call $write (i64.const 100) (i64.const 64) (i64.const 4))
            (call $read (i64.const -1) (i64.const -1) (i64.const 0))))"#;
    let (control, control_instructions, control_stats) = run_read_mask(wat, false, false);
    let (observed, observed_instructions, observed_stats) = run_read_mask(wat, true, true);
    let control = control.unwrap();
    let observed = observed.unwrap();
    assert_eq!(control_instructions, observed_instructions);
    assert_eq!(control.wasm_dirty_pages, observed.wasm_dirty_pages);
    assert_eq!(
        control.stable_memory_dirty_pages,
        observed.stable_memory_dirty_pages
    );
    let observed_trace = observed_stats.wasm_memory_access_trace.unwrap();
    let control_trace = control_stats.wasm_memory_access_trace.unwrap();
    assert_eq!(
        control_trace.stable_accessed_pages,
        observed_trace.stable_accessed_pages
    );
    let profile = observed_trace.instruction_profile.unwrap();
    assert_eq!(
        profile.ledger.stable_page_instructions, 320,
        "one native access and one dirty debit; repeats do not repeat the page tax"
    );
    let reads = observed_trace.read_profile.unwrap();
    assert_eq!(reads.observed_reads, 6);
    assert_eq!(
        heap_reads(&reads, 64),
        1,
        "stable write consumes heap source bytes"
    );
    assert_eq!(
        heap_reads(&reads, 0),
        0,
        "stable read destination is a write"
    );
    let stable: Vec<_> = reads
        .ranges
        .iter()
        .filter(|range| range.memory == "stable")
        .map(|range| {
            (
                range.start,
                range.end,
                range.stable_direct_reads,
                range.stable_native_reads,
            )
        })
        .collect();
    assert_eq!(
        stable,
        vec![(0, 4, 1, 2), (4096, 4100, 1, 0), (65535, 65537, 1, 0)]
    );
}

#[test]
fn read_mask_trapping_load_is_not_reported_as_a_completed_read() {
    let wat = r#"(module (memory 1)
        (func (export "canister_update test")
            (drop (i32.load8_u (i32.const 3)))
            (drop (i64.load (i32.const 65532)))))"#;
    let (control, control_instructions, _) = run_read_mask(wat, false, false);
    let (observed, observed_instructions, stats) = run_read_mask(wat, true, true);
    assert!(control.is_err());
    assert!(observed.is_err());
    assert_eq!(control_instructions, observed_instructions);
    let reads = stats
        .wasm_memory_access_trace
        .unwrap()
        .read_profile
        .unwrap();
    assert_eq!(reads.observed_reads, 1);
    assert_eq!(reads.observed_bytes, 1);
    assert_eq!(heap_reads(&reads, 3), 1);
    assert_eq!(heap_reads(&reads, 65532), 0);
}

#[test]
fn instruction_profile_retains_host_debits_when_a_paused_guest_is_aborted() {
    struct Abort;
    impl ic_interfaces::execution_environment::OutOfInstructionsHandler for Abort {
        fn out_of_instructions(&self, _: i64) -> HypervisorResult<i64> {
            Err(HypervisorError::Aborted)
        }
        fn yield_for_dirty_memory_copy(&self) -> HypervisorResult<i64> {
            Err(HypervisorError::Aborted)
        }
    }
    let wat = r#"(module (memory 1)
        (func (export "canister_update test")
            (i32.store (i32.const 0) (i32.const 7))
            (loop $again (br $again))))"#;
    let mut counters = Vec::new();
    for profiling in [false, true] {
        let mut instance = WasmtimeInstanceBuilder::new()
            .with_config(Config {
                instruction_charge_profiling: true,
                ..Config::default()
            })
            .with_wat(wat)
            .with_num_instructions(NumInstructions::new(1_000))
            .with_page_overhead(NumInstructions::new(10))
            .with_out_of_instructions_handler(std::rc::Rc::new(Abort))
            .build();
        instance.enable_wasm_memory_access_tracing();
        if profiling {
            instance.enable_instruction_profiling();
        }
        assert!(matches!(
            instance.run(FuncRef::Method(WasmMethod::Update("test".into()))),
            Err(HypervisorError::Aborted)
        ));
        counters.push(instance.instruction_counter());
        let trace = instance.get_stats().wasm_memory_access_trace.unwrap();
        assert!(!trace.stable_pages_tracked);
        assert!(trace.accessed_pages.is_empty()); // unavailable after abort, not a complete page set
        assert_eq!(trace.page_access_instructions, 320);
        if profiling {
            let profile = trace.instruction_profile.unwrap();
            assert_eq!(profile.ledger.heap_page_instructions, 320);
            assert!(profile.ledger.static_instructions > 0);
            assert!(profile.ledger.reconciled);
            assert_eq!(
                profile
                    .page_charges
                    .iter()
                    .map(|charge| charge.instructions)
                    .sum::<u64>(),
                320
            );
            assert!(profile.module.is_some());
        } else {
            assert!(trace.instruction_profile.is_none());
        }
    }
    assert_eq!(counters[0], counters[1]);
}

fn run(
    wat: &str,
    profiling: bool,
    limit: u64,
    payload: Vec<u8>,
) -> (HypervisorResult<InstanceRunResult>, u64, InstanceStats) {
    run_with_gate(wat, profiling, profiling, limit, payload)
}

fn run_with_gate(
    wat: &str,
    instrumented: bool,
    collecting: bool,
    limit: u64,
    payload: Vec<u8>,
) -> (HypervisorResult<InstanceRunResult>, u64, InstanceStats) {
    let mut instance = WasmtimeInstanceBuilder::new()
        .with_config(Config {
            instruction_charge_profiling: instrumented,
            ..Config::default()
        })
        .with_wat(wat)
        .with_api_type(ApiType::init(UNIX_EPOCH, payload, user_test_id(24).get()))
        .with_num_instructions(NumInstructions::new(limit))
        .with_page_overhead(NumInstructions::new(10))
        .build();
    instance.enable_wasm_memory_access_tracing();
    if collecting {
        instance.enable_instruction_profiling();
    }
    let result = instance.run(FuncRef::Method(WasmMethod::Update("test".into())));
    let counter = instance.instruction_counter();
    let instructions = instance
        .store_data()
        .system_api()
        .unwrap()
        .slice_instructions_executed(counter)
        .get();
    (result, instructions, instance.get_stats())
}

#[test]
fn instruction_profile_preserves_bulk_api_heap_state_and_debits() {
    let wat = r#"
        (module
            (import "ic0" "msg_arg_data_copy" (func $copy (param i32 i32 i32)))
            (import "ic0" "time" (func $time (result i64)))
            (memory 2)
            (global $value (export "value") (mut i32) (i32.const 0))
            (func (export "canister_update test")
                (call $copy (i32.const 0) (i32.const 0) (i32.const 4))
                (memory.copy (i32.const 65536) (i32.const 0) (i32.const 4))
                (memory.fill (i32.const 65540) (i32.const 7) (i32.const 4))
                (global.set $value (i32.load (i32.const 65536)))
                (drop (call $time))))
    "#;
    let (control, control_instructions, control_stats) =
        run(wat, false, 1_000_000, vec![42, 0, 0, 0]);
    let (observed, observed_instructions, observed_stats) =
        run(wat, true, 1_000_000, vec![42, 0, 0, 0]);
    let control = control.unwrap();
    let observed = observed.unwrap();
    assert_eq!(control_instructions, observed_instructions);
    assert_eq!(control.exported_globals, observed.exported_globals);
    assert_eq!(observed.exported_globals[0], Global::I32(42));
    // The diagnostic module with its guest observer flag disabled must
    // preserve the same debit boundaries as the production instrumenter.
    let (gated, gated_instructions, gated_stats) =
        run_with_gate(wat, true, false, 1_000_000, vec![42, 0, 0, 0]);
    let gated = gated.unwrap();
    assert_eq!(gated_instructions, control_instructions);
    assert_eq!(gated.exported_globals, control.exported_globals);
    assert_eq!(gated.wasm_dirty_pages, control.wasm_dirty_pages);
    assert_eq!(
        gated_stats
            .wasm_memory_access_trace
            .unwrap()
            .instruction_profile,
        None,
    );
    assert_eq!(control.wasm_dirty_pages, observed.wasm_dirty_pages);
    assert_eq!(
        control.stable_memory_dirty_pages,
        observed.stable_memory_dirty_pages
    );
    let control_trace = control_stats.wasm_memory_access_trace.unwrap();
    let observed_trace = observed_stats.wasm_memory_access_trace.unwrap();
    assert_eq!(control_trace.accessed_pages, observed_trace.accessed_pages);
    assert_eq!(control_trace.dirty_pages, observed_trace.dirty_pages);
    assert_eq!(control_trace.instruction_profile, None);
    let profile = observed_trace.instruction_profile.unwrap();
    assert!(!profile.truncated);
    assert!(!profile.blocks.is_empty());
    assert_eq!(profile.ledger.raw_instructions, observed_instructions);
    assert_eq!(profile.ledger.dynamic_instructions, 8);
    assert_eq!(profile.ledger.api_instructions, 1004);
    assert_eq!(profile.ledger.heap_page_instructions, 640);
    assert_eq!(profile.ledger.stable_page_instructions, 0);
    let copy = profile
        .api_calls
        .iter()
        .find(|call| call.api == "msg_arg_data_copy")
        .unwrap();
    assert_eq!(copy.calls, 1);
    assert_eq!(copy.base_instructions, 500);
    assert_eq!(copy.byte_instructions, 4);
    assert_eq!(copy.direct_instructions, 0);
    assert!(profile.dynamic_charges.iter().all(|charge| {
        charge.category == "bulk_length"
            && charge.function_index.is_some()
            && charge.body_offset.is_some()
    }));
}

#[test]
fn instruction_profile_preserves_wasm64_bulk_operands() {
    let wat = r#"
        (module
            (import "ic0" "msg_arg_data_copy" (func $copy (param i64 i64 i64)))
            (memory i64 2)
            (global $value (export "value") (mut i32) (i32.const 0))
            (func (export "canister_update test")
                (call $copy (i64.const 0) (i64.const 0) (i64.const 4))
                (memory.copy (i64.const 65536) (i64.const 0) (i64.const 4))
                (memory.fill (i64.const 65540) (i32.const 7) (i64.const 4))
                (global.set $value (i32.load (i64.const 65536)))))
    "#;
    let (control, control_instructions, control_stats) =
        run(wat, false, 1_000_000, vec![42, 0, 0, 0]);
    let (observed, observed_instructions, observed_stats) =
        run(wat, true, 1_000_000, vec![42, 0, 0, 0]);
    let control = control.unwrap();
    let observed = observed.unwrap();
    assert_eq!(control_instructions, observed_instructions);
    assert_eq!(control.exported_globals, observed.exported_globals);
    assert_eq!(observed.exported_globals[0], Global::I32(42));
    assert_eq!(control.wasm_dirty_pages, observed.wasm_dirty_pages);
    let control_trace = control_stats.wasm_memory_access_trace.unwrap();
    let observed_trace = observed_stats.wasm_memory_access_trace.unwrap();
    assert_eq!(control_trace.accessed_pages, observed_trace.accessed_pages);
    assert_eq!(control_trace.dirty_pages, observed_trace.dirty_pages);
    let profile = observed_trace.instruction_profile.unwrap();
    assert_eq!(profile.ledger.raw_instructions, observed_instructions);
    assert_eq!(profile.ledger.dynamic_instructions, 8);
    assert_eq!(profile.ledger.api_instructions, 504);
    assert_eq!(profile.ledger.heap_page_instructions, 640);
}

#[test]
fn instruction_profile_observes_replacement_fees_without_counting_generated_copies() {
    let wat = r#"
        (module
            (import "ic0" "stable_grow" (func $grow (param i32) (result i32)))
            (import "ic0" "stable_read" (func $read32 (param i32 i32 i32)))
            (import "ic0" "stable_write" (func $write32 (param i32 i32 i32)))
            (import "ic0" "stable64_read" (func $read (param i64 i64 i64)))
            (import "ic0" "stable64_write" (func $write (param i64 i64 i64)))
            (memory 1)
            (global $value (export "value") (mut i32) (i32.const 0))
            (func (export "canister_update test")
                (drop (call $grow (i32.const 1)))
                (i32.store (i32.const 0) (i32.const 123))
                (call $write (i64.const 0) (i64.const 0) (i64.const 4))
                (call $read (i64.const 8) (i64.const 0) (i64.const 4))
                (call $read (i64.const 12) (i64.const 0) (i64.const 4))
                (call $read (i64.const -1) (i64.const -1) (i64.const 0))
                (call $write (i64.const -1) (i64.const -1) (i64.const 0))
                (call $read32 (i32.const 0) (i32.const 0) (i32.const 0))
                (call $write32 (i32.const 0) (i32.const 0) (i32.const 0))
                (global.set $value (i32.load (i32.const 12)))))
    "#;
    let (control, control_instructions, control_stats) = run(wat, false, 1_000_000, vec![]);
    let (observed, observed_instructions, observed_stats) = run(wat, true, 1_000_000, vec![]);
    let control = control.unwrap();
    let observed = observed.unwrap();
    assert_eq!(control_instructions, observed_instructions);
    assert_eq!(control.exported_globals, observed.exported_globals);
    assert_eq!(observed.exported_globals[0], Global::I32(123));
    assert_eq!(control.wasm_dirty_pages, observed.wasm_dirty_pages);
    assert_eq!(
        control.stable_memory_dirty_pages,
        observed.stable_memory_dirty_pages
    );
    let control_trace = control_stats.wasm_memory_access_trace.unwrap();
    let observed_trace = observed_stats.wasm_memory_access_trace.unwrap();
    assert_eq!(control_trace.accessed_pages, observed_trace.accessed_pages);
    assert_eq!(control_trace.dirty_pages, observed_trace.dirty_pages);
    let profile = observed_trace.instruction_profile.unwrap();
    assert_eq!(profile.ledger.raw_instructions, observed_instructions);
    assert_eq!(profile.ledger.api_instructions, 100);
    // Nonempty stable64 operations charge 20 + length. The 32-bit
    // replacements charge their base fee even for length zero, whereas
    // the 64-bit replacements return before charging when length is zero.
    assert_eq!(
        profile.ledger.stable_api_instructions,
        3 * (20 + 4) + 2 * 20
    );
    assert_eq!(profile.ledger.dynamic_instructions, 0);
    assert!(profile.ledger.stable_page_instructions > 0);
    let replacement = profile
        .dynamic_charges
        .iter()
        .find(|charge| charge.category == "stable_api")
        .unwrap();
    assert_eq!(replacement.events, 5);
    assert_eq!(replacement.function_index, None);
    assert_eq!(replacement.body_offset, None);
    assert_eq!(replacement.operator_ordinal, None);
    assert_eq!(rule(&profile, "ic.stable.read.direct.v1"), (0, 0));
    assert_eq!(rule(&profile, "ic.stable.read.native.v1"), (2, 0));
    assert_eq!(rule(&profile, "ic.stable64.read.zero_length.v1"), (1, 0));
    assert_eq!(rule(&profile, "ic.stable64.write.zero_length.v1"), (1, 0));
    assert_eq!(
        profile
            .rules
            .iter()
            .map(|rule| rule.instructions)
            .sum::<u64>(),
        profile.ledger.raw_instructions,
    );
}

fn rule(
    profile: &ic_interfaces::execution_environment::WasmInstructionProfile,
    rule_id: &str,
) -> (u64, u64) {
    profile
        .rules
        .iter()
        .find(|rule| rule.rule_id == rule_id)
        .map_or((0, 0), |rule| (rule.events, rule.instructions))
}

#[test]
fn instruction_profile_records_direct_reread_overlap_and_zero_length_decisions() {
    let first_read = "(call $read (i64.const 0) (i64.const 0) (i64.const 4))";
    let repeated_read =
        format!("{first_read} (call $read (i64.const 4) (i64.const 0) (i64.const 4))");
    // The second read spans a previously read 4 KiB granule and a fresh
    // one, so it must select the native path rather than the direct path.
    let overlapping_read =
        format!("{first_read} (call $read (i64.const 4) (i64.const 4094) (i64.const 4))");
    for (reads, read_count, native_count, stable_page_instructions) in [
        (first_read, 1, 0, 0),
        (repeated_read.as_str(), 2, 1, 160),
        (overlapping_read.as_str(), 2, 1, 160),
    ] {
        let wat = format!(
            r#"
            (module
                (import "ic0" "stable_grow" (func $grow (param i32) (result i32)))
                (import "ic0" "stable64_read" (func $read (param i64 i64 i64)))
                (import "ic0" "stable64_write" (func $write (param i64 i64 i64)))
                (memory 1)
                (func (export "canister_update test")
                    (drop (call $grow (i32.const 1)))
                    {reads}
                    (call $read (i64.const -1) (i64.const -1) (i64.const 0))
                    (call $write (i64.const -1) (i64.const -1) (i64.const 0))))
        "#
        );
        let (control, control_instructions, control_stats) = run(&wat, false, 1_000_000, vec![]);
        let (observed, observed_instructions, observed_stats) = run(&wat, true, 1_000_000, vec![]);
        let control = control.unwrap();
        let observed = observed.unwrap();
        assert_eq!(control_instructions, observed_instructions);
        assert_eq!(control.exported_globals, observed.exported_globals);
        assert_eq!(control.wasm_dirty_pages, observed.wasm_dirty_pages);
        assert_eq!(
            control.stable_memory_dirty_pages,
            observed.stable_memory_dirty_pages
        );
        let control_trace = control_stats.wasm_memory_access_trace.unwrap();
        let observed_trace = observed_stats.wasm_memory_access_trace.unwrap();
        assert!(observed_trace.stable_pages_tracked);
        assert_eq!(observed_trace.stable_memory_size, 1);
        assert_eq!(observed_trace.stable_accessed_pages, vec![0..1]);
        assert!(observed_trace.stable_dirty_pages.is_empty());
        assert_eq!(control_trace.accessed_pages, observed_trace.accessed_pages);
        assert_eq!(control_trace.dirty_pages, observed_trace.dirty_pages);
        let profile = observed_trace.instruction_profile.unwrap();
        assert_eq!(profile.ledger.raw_instructions, observed_instructions);
        assert_eq!(profile.ledger.stable_api_instructions, read_count * 24);
        assert_eq!(
            profile.ledger.stable_page_instructions,
            stable_page_instructions
        );
        assert_eq!(profile.ledger.dynamic_instructions, 0);
        assert_eq!(rule(&profile, "ic.stable.read.direct.v1"), (1, 0));
        assert_eq!(
            rule(&profile, "ic.stable.read.native.v1"),
            (native_count, 0)
        );
        assert_eq!(rule(&profile, "ic.stable64.read.zero_length.v1"), (1, 0));
        assert_eq!(rule(&profile, "ic.stable64.write.zero_length.v1"), (1, 0));
        assert_eq!(profile.dynamic_charges.len(), 1);
        assert_eq!(profile.dynamic_charges[0].category, "stable_api");
        assert_eq!(profile.dynamic_charges[0].events, read_count);
        // Decisions add no instructions. Their associated API, memory,
        // and static charges remain present in the additive rule ledger.
        assert_eq!(
            profile
                .rules
                .iter()
                .map(|rule| rule.instructions)
                .sum::<u64>(),
            profile.ledger.raw_instructions
        );
    }
}

#[test]
fn stable_touch_coverage_includes_direct_reads_sparse_writes_and_query_attempts() {
    use ic_embedders::wasmtime_embedder::system_api::ModificationTracking;
    let wat = r#"(module
        (import "ic0" "stable_grow" (func $grow (param i32) (result i32)))
        (import "ic0" "stable64_read" (func $read (param i64 i64 i64)))
        (import "ic0" "stable64_write" (func $write (param i64 i64 i64)))
        (memory 1)
        (func (export "canister_update test")
            (drop (call $grow (i32.const 5)))
            (call $read (i64.const 0) (i64.const 65535) (i64.const 2))
            (call $read (i64.const 4) (i64.const 196611) (i64.const 1))
            (call $write (i64.const 262145) (i64.const 0) (i64.const 1))
            (call $read (i64.const -1) (i64.const 131072) (i64.const 0))
            (call $write (i64.const 131072) (i64.const -1) (i64.const 0))))"#;
    for query in [false, true] {
        let api_type = if query {
            ApiType::replicated_query(UNIX_EPOCH, vec![], user_test_id(24).get(), 0.into(), None)
        } else {
            ApiType::init(UNIX_EPOCH, vec![], user_test_id(24).get())
        };
        let mut counters = Vec::new();
        for profiling in [false, true] {
            let mut instance = WasmtimeInstanceBuilder::new()
                .with_config(Config {
                    instruction_charge_profiling: true,
                    ..Config::default()
                })
                .with_wat(wat)
                .with_api_type(api_type.clone())
                .with_modification_tracking(if query {
                    ModificationTracking::Ignore
                } else {
                    ModificationTracking::Track
                })
                .with_page_overhead(NumInstructions::new(10))
                .build();
            instance.enable_wasm_memory_access_tracing();
            if profiling {
                instance.enable_instruction_profiling();
            }
            instance
                .run(FuncRef::Method(WasmMethod::Update("test".into())))
                .unwrap();
            counters.push(instance.instruction_counter());
            let trace = instance.get_stats().wasm_memory_access_trace.unwrap();
            assert!(trace.stable_pages_tracked);
            assert_eq!(trace.stable_memory_size, 5);
            assert_eq!(trace.stable_accessed_pages, vec![0..2, 3..5]);
            assert_eq!(trace.stable_dirty_pages, vec![4..5]);
            if profiling {
                let profile = trace.instruction_profile.unwrap();
                assert_eq!(rule(&profile, "ic.stable.read.direct.v1"), (2, 0));
                // Direct-read regions0,1,3 have touch coverage even though
                // only the write to stable region4 causes native page fees.
                assert!(
                    profile
                        .page_charges
                        .iter()
                        .filter(|charge| charge.memory == "stable")
                        .all(|charge| charge.region_index == 4)
                );
                assert!(profile.ledger.reconciled);
            }
        }
        assert_eq!(counters[0], counters[1]);
    }
}

#[test]
fn instruction_profile_keeps_api_debit_and_budget_failure_without_changing_limit() {
    let wat = r#"
        (module
            (import "ic0" "msg_arg_data_copy" (func $copy (param i32 i32 i32)))
            (memory 1)
            (func (export "canister_update test")
                (call $copy (i32.const 0) (i32.const 0) (i32.const 1024))))
    "#;
    let (control, control_instructions, control_stats) = run(wat, false, 1000, vec![0; 1024]);
    let (observed, observed_instructions, observed_stats) = run(wat, true, 1000, vec![0; 1024]);
    let expected = HypervisorError::InstructionLimitExceeded(NumInstructions::new(1000));
    assert_eq!(control.unwrap_err(), expected);
    assert_eq!(observed.unwrap_err(), expected);
    assert_eq!(control_instructions, observed_instructions);
    assert_eq!(observed_instructions, 1000);
    let control_trace = control_stats.wasm_memory_access_trace.unwrap();
    let observed_trace = observed_stats.wasm_memory_access_trace.unwrap();
    assert_eq!(control_trace.accessed_pages, observed_trace.accessed_pages);
    assert_eq!(control_trace.dirty_pages, observed_trace.dirty_pages);
    assert!(observed_trace.accessed_pages.is_empty());
    let profile = observed_trace.instruction_profile.unwrap();
    // Record the debit even though it immediately exceeds the message
    // budget. The final reporting clamp must not erase its raw charge.
    assert_eq!(profile.ledger.api_instructions, 1524);
    assert!(profile.ledger.raw_instructions > observed_instructions);
    assert_eq!(profile.budget_events.len(), 1);
    let event = &profile.budget_events[0];
    assert!(event.before < 0);
    assert_eq!(event.after, None);
    assert_eq!(event.cause, "system_api_post_debit");
    assert_eq!(
        profile.ledger.adjustment_reason,
        "message_instruction_limit_clamp"
    );
    assert!(profile.ledger.adjustment < 0);
    assert_eq!(profile.ledger.unexplained_instructions, 0);
    assert!(profile.ledger.reconciled);
}
#[test]
fn instruction_profile_private_symbols_remain_user_globals_in_normal_mode() {
    let wat = r#"(module
        (global $value (export "__ic_instruction_profile_enabled") (mut i32) (i32.const 0))
        (func (export "canister_update test") (global.set $value (i32.const 42))))"#;
    let (result, _, _) = run(wat, false, 1_000_000, vec![]);
    assert_eq!(result.unwrap().exported_globals[0], Global::I32(42));
}
#[test]
fn instruction_profile_captures_exact_api_replacement_and_indirect_call_origins() {
    let wat = r#"(module
        (type $void (func))
        (import "ic0" "time" (func $time (result i64)))
        (import "ic0" "stable64_grow" (func $grow (param i64) (result i64)))
        (import "ic0" "stable64_read" (func $read (param i64 i64 i64)))
        (memory i64 1)
        (table 1 funcref) (elem (i32.const 0) $leaf)
        (func $leaf (drop (call $time)))
        (func $middle (call $leaf))
        (func (export "canister_update test")
            (drop (memory.grow (i64.const 1)))
            (drop (call $grow (i64.const 1)))
            (call $middle)
            (call_indirect (type $void) (i32.const 0))
            (call $read (i64.const 0) (i64.const 0) (i64.const 4))
            (call $read (i64.const 4) (i64.const 0) (i64.const 4))
            (memory.fill (i64.const 65536) (i32.const 1) (i64.const 4))))"#;
    let (control, control_instructions, control_stats) = run(wat, false, 1_000_000, vec![]);
    let (observed, instructions, stats) = run(wat, true, 1_000_000, vec![]);
    assert_eq!(
        control.unwrap().exported_globals,
        observed.unwrap().exported_globals
    );
    assert_eq!(control_instructions, instructions);
    let trace = stats.wasm_memory_access_trace.unwrap();
    assert_eq!(
        trace.accessed_pages,
        control_stats
            .wasm_memory_access_trace
            .unwrap()
            .accessed_pages
    );
    let profile = trace.instruction_profile.unwrap();
    let metadata = profile.module.as_ref().unwrap();
    assert!(!metadata.origin_map_truncated);
    assert!(!profile.stacks.is_empty());
    assert!(!profile.stack_details_truncated);
    let time: Vec<_> = profile
        .stack_charges
        .iter()
        .filter(|charge| charge.api.as_deref() == Some("time"))
        .collect();
    assert_eq!(
        time.len(),
        2,
        "different physical callers retain separate charge stacks"
    );
    for charge in time {
        assert_eq!(charge.origin_evidence, "exact_operation");
        let origin = &metadata.operators[charge.origin_id.unwrap() as usize];
        assert_eq!(origin.function_index, 3, "time is called by leaf");
        let stack = &profile.stacks[charge.stack_id.unwrap() as usize];
        assert_eq!(stack.frames[0].function_index, Some(3));
        assert_eq!(stack.frames.last().unwrap().function_index, Some(5));
        assert_eq!(charge.instructions, 500);
    }
    let replacements: Vec<_> = profile
        .stack_charges
        .iter()
        .filter(|charge| charge.category == "stable_api")
        .collect();
    assert_eq!(replacements.len(), 2);
    let origins: std::collections::BTreeSet<_> = replacements
        .iter()
        .map(|charge| {
            assert_eq!(charge.origin_evidence, "exact_operation");
            let origin = &metadata.operators[charge.origin_id.unwrap() as usize];
            assert_eq!(origin.function_index, 5);
            assert_eq!(charge.instructions, 24);
            origin.origin_id
        })
        .collect();
    assert_eq!(
        origins.len(),
        2,
        "two exact stable read callsites must remain distinct"
    );
    assert!(!profile.page_details_truncated);
    assert!(!profile.page_charges.is_empty());
    assert_eq!(
        profile
            .page_charges
            .iter()
            .map(|charge| charge.instructions)
            .sum::<u64>(),
        profile.ledger.heap_page_instructions + profile.ledger.stable_page_instructions
    );
    for charge in &profile.page_charges {
        assert!(charge.native_pc.is_some(), "actual Linux fault PC retained");
        let origin = &metadata.operators[charge
            .origin_id
            .expect("actual host,bulk,stable-copy page origin must resolve")
            as usize];
        assert_eq!(origin.function_index, 5);
        if charge.memory == "stable" {
            assert!(
                origins.contains(&origin.origin_id),
                "native stable copy belongs to proven stable read callsite"
            );
        }
    }
    assert_eq!(
        profile
            .stack_charges
            .iter()
            .map(|charge| charge.instructions)
            .sum::<u64>(),
        profile.ledger.raw_instructions
    );
}

#[test]
fn instruction_profile_bounds_physical_stack_capture_visibly() {
    let mut wat = String::from("(module (import \"ic0\" \"time\" (func $time (result i64)))");
    wat.push_str("(func $f0 (drop (call $time)))");
    for index in 1..30 {
        wat.push_str(&format!("(func $f{index} (call $f{}))", index - 1));
    }
    wat.push_str("(func (export \"canister_update test\") (call $f29)))");
    let (result, _, stats) = run(&wat, true, 1_000_000, vec![]);
    result.unwrap();
    let profile = stats
        .wasm_memory_access_trace
        .unwrap()
        .instruction_profile
        .unwrap();
    assert!(profile.stacks.iter().any(|stack| stack.truncated));
    assert!(profile.stacks.iter().all(|stack| stack.frames.len() <= 20));
}
