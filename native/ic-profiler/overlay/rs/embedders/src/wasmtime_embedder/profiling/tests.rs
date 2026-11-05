use super::*;

#[test]
fn proven_host_operation_precedes_a_synthetic_native_helper() {
    use ic_interfaces::execution_environment::{
        WasmInstructionModuleMetadata, WasmInstructionOriginRange,
    };
    let metadata = WasmInstructionModuleMetadata {
        instrumented_ranges: vec![
            WasmInstructionOriginRange {
                instrumented_function_index: 1,
                start_offset: 10,
                end_offset: 20,
                origin_id: None,
                kind: "stable_helper".into(),
            },
            WasmInstructionOriginRange {
                instrumented_function_index: 2,
                start_offset: 30,
                end_offset: 40,
                origin_id: Some(7),
                kind: "original_call".into(),
            },
            WasmInstructionOriginRange {
                instrumented_function_index: 3,
                start_offset: 50,
                end_offset: 60,
                origin_id: Some(8),
                kind: "original".into(),
            },
        ],
        ..Default::default()
    };
    assert_eq!(
        select_page_origin(Some(12), Some(32), Some(&metadata)),
        (Some(32), true)
    );
    assert_eq!(
        select_page_origin(Some(52), Some(32), Some(&metadata)),
        (Some(52), false)
    );
    assert_eq!(
        select_page_origin(Some(12), Some(42), Some(&metadata)),
        (Some(12), false)
    );
    assert_eq!(
        select_page_origin(None, Some(32), Some(&metadata)),
        (Some(32), true)
    );
    assert_eq!(
        select_page_origin(None, Some(42), Some(&metadata)),
        (None, false)
    );
    assert_eq!(
        select_page_origin(Some(12), Some(32), None),
        (Some(12), false)
    );
}

#[test]
fn native_pc_mapping_preserves_unknown_intervals_and_text_bounds() {
    let address_map = [(0, Some(42)), (4, None), (8, Some(77))];
    assert_eq!(
        module_offset_from_native_pc(1001, 1000..1012, &address_map),
        Some(42)
    );
    assert_eq!(
        module_offset_from_native_pc(1005, 1000..1012, &address_map),
        None
    );
    assert_eq!(
        module_offset_from_native_pc(1008, 1000..1012, &address_map),
        Some(77)
    );
    assert_eq!(
        module_offset_from_native_pc(999, 1000..1012, &address_map),
        None
    );
    assert_eq!(
        module_offset_from_native_pc(1012, 1000..1012, &address_map),
        None
    );
}

#[test]
fn host_operation_tags_are_scoped_and_unavailable_nested_calls_clear_them() {
    let charges = Arc::new(InstructionPageCharges::default());
    charges.enabled.store(true, Ordering::Relaxed);
    charges.enable_page_details();
    let observe = || {
        charges.observe_page_rule(
            CanisterMemoryType::Heap,
            memory_tracker::PageDebitKind::Access,
            0,
            80_000,
        )
    };
    {
        let _outer = charges.enter_host_operation(Some(42));
        observe();
        {
            let _unknown = charges.enter_host_operation(None);
            observe();
        }
        {
            let _inner = charges.enter_host_operation(Some(77));
            observe();
        }
        observe();
    }
    observe();
    let records = charges.page_records.get().unwrap();
    let tags: Vec<_> = records
        .iter()
        .take(5)
        .map(|record| {
            record
                .host_operation_active
                .load(Ordering::Relaxed)
                .then(|| record.host_module_offset.load(Ordering::Relaxed))
        })
        .collect();
    assert_eq!(tags, vec![Some(42), None, Some(77), Some(42), None]);
}

#[test]
fn page_details_are_bounded_and_restore_nested_fault_context() {
    let charges = InstructionPageCharges::default();
    charges.enabled.store(true, Ordering::Relaxed);
    charges.enable_page_details();
    {
        let _outer = charges.enter_fault(Some(0x1234));
        charges.observe_page_rule(
            CanisterMemoryType::Heap,
            memory_tracker::PageDebitKind::Access,
            7,
            80_000,
        );
        {
            let _inner = charges.enter_fault(Some(0x5678));
            charges.observe_page_rule(
                CanisterMemoryType::Stable,
                memory_tracker::PageDebitKind::Dirty,
                8,
                80_000,
            );
        }
        charges.observe_page_rule(
            CanisterMemoryType::Heap,
            memory_tracker::PageDebitKind::Dirty,
            7,
            80_000,
        );
    }
    charges.observe_page_rule(
        CanisterMemoryType::Heap,
        memory_tracker::PageDebitKind::Access,
        9,
        80_000,
    );
    let records = charges.page_records.get().unwrap();
    assert_eq!(records[0].native_pc.load(Ordering::Relaxed), 0x1234);
    assert_eq!(records[1].native_pc.load(Ordering::Relaxed), 0x5678);
    assert_eq!(records[2].native_pc.load(Ordering::Relaxed), 0x1234);
    assert_eq!(records[3].native_pc.load(Ordering::Relaxed), 0);
    assert_eq!(records[0].region_index.load(Ordering::Relaxed), 7);
    assert_eq!(records[1].memory.load(Ordering::Relaxed), 1);
    assert_eq!(records[1].kind.load(Ordering::Relaxed), 1);
    for region in 4..=MAX_PAGE_DETAILS {
        charges.observe_page_rule(
            CanisterMemoryType::Heap,
            memory_tracker::PageDebitKind::Access,
            region as u64,
            80_000,
        );
    }
    assert_eq!(records.len(), MAX_PAGE_DETAILS);
    assert!(charges.page_details_truncated.load(Ordering::Relaxed));
    assert!(
        records
            .iter()
            .all(|record| record.ready.load(Ordering::Acquire))
    );
    // Dropping detail does not drop actual debit aggregates.
    assert_eq!(
        charges.rule_events[0].load(Ordering::Relaxed),
        MAX_PAGE_DETAILS as u64 - 1
    );
    assert_eq!(
        charges.rule_instructions[0].load(Ordering::Relaxed),
        (MAX_PAGE_DETAILS as u64 - 1) * 80_000
    );
}

#[test]
fn module_origin_map_availability_does_not_truncate_one_shallow_stack() {
    use ic_interfaces::execution_environment::{
        WasmInstructionModuleMetadata, WasmInstructionOperator, WasmInstructionOriginRange,
    };

    for state in ["complete", "truncated", "unavailable"] {
        let module = (state != "unavailable").then(|| {
            Arc::new(WasmInstructionModuleMetadata {
                operators: vec![WasmInstructionOperator {
                    origin_id: 0,
                    function_index: 1,
                    body_offset: 9,
                    operator_ordinal: 0,
                    weight: 1,
                }],
                instrumented_ranges: vec![WasmInstructionOriginRange {
                    instrumented_function_index: 1,
                    start_offset: 10,
                    end_offset: 20,
                    origin_id: Some(0),
                    kind: "original_call".into(),
                }],
                origin_map_truncated: state == "truncated",
                ..Default::default()
            })
        });
        let mut collector = InstructionChargeProfileCollector::new(module, None, None);
        collector.observe_dynamic(7, 1, 9, 0, 0);
        // For the truncated map, use a coordinate beyond its retained
        // prefix. The physical frame is complete; its original operator
        // must remain unavailable instead of truncating the stack.
        collector.record_stack_charge(
            Some(vec![CapturedFrame {
                function: 1,
                offset: Some(if state == "truncated" { 25 } else { 12 }),
                matching_module: true,
            }]),
            "bulk_length",
            None,
            7,
            None,
            "exact_operation",
        );
        let profile = collector.finish(80_000, 160_000);
        assert_eq!(profile.module_origin_map_truncated, state == "truncated");
        assert_eq!(
            profile.module_origin_map_unavailable,
            state == "unavailable"
        );
        assert!(!profile.stack_details_truncated, "{state}");
        assert!(!profile.truncated, "{state}");
        assert_eq!(profile.stacks.len(), 1);
        assert!(!profile.stacks[0].truncated);
        assert_eq!(profile.stacks[0].frames.len(), 1);
        let expected_origin = (state == "complete").then_some(0);
        assert_eq!(profile.stacks[0].frames[0].origin_id, expected_origin);
        assert_eq!(profile.stack_charges[0].origin_id, expected_origin);
        assert_eq!(profile.stack_charges[0].instructions, 7);
        // Map availability never changes existing instruction/page debits.
        assert_eq!(profile.ledger.dynamic_instructions, 7);
        assert_eq!(profile.ledger.heap_page_instructions, 80_000);
        assert_eq!(profile.ledger.stable_page_instructions, 160_000);
        assert_eq!(profile.ledger.raw_instructions, 240_007);
    }
}

#[test]
fn stack_sampling_and_dictionary_caps_keep_full_charge_totals() {
    use ic_interfaces::execution_environment::{
        WasmInstructionModuleMetadata, WasmInstructionOperator,
    };
    let count = MAX_STACK_OBSERVATIONS + 32;
    let module = Arc::new(WasmInstructionModuleMetadata {
        operators: (0..count)
            .map(|i| WasmInstructionOperator {
                origin_id: i as u32,
                function_index: 0,
                body_offset: i as u64 + 1,
                operator_ordinal: i as u32,
                weight: 1,
            })
            .collect(),
        ..Default::default()
    });
    let mut collector = InstructionChargeProfileCollector::new(Some(module), None, None);
    for i in 0..count {
        collector.observe_dynamic(1, 0, i as i64 + 1, i as i32, 0);
        let frames = (i < MAX_STACK_OBSERVATIONS).then(|| {
            vec![CapturedFrame {
                function: i as u32,
                offset: Some(i as u64),
                matching_module: false,
            }]
        });
        collector.record_stack_charge(
            frames,
            "bulk_length",
            None,
            1,
            Some((0, i as u64 + 1, i as u32)),
            "exact_operation",
        );
    }
    let profile = collector.finish(0, 0);
    assert!(profile.stack_details_truncated);
    assert!(!profile.module_origin_map_truncated);
    assert!(!profile.module_origin_map_unavailable);
    assert_eq!(profile.stacks.len(), MAX_STACKS);
    assert!(profile.stack_charges.len() <= MAX_STACK_CHARGES + MAX_APIS);
    assert_eq!(
        profile
            .stack_charges
            .iter()
            .map(|charge| charge.instructions)
            .sum::<u64>(),
        count as u64
    );
    assert_eq!(profile.ledger.raw_instructions, count as u64);
}

#[test]
fn paired_guest_operation_scopes_restore_parent_after_return_trap_and_overflow() {
    let charges = Arc::new(InstructionPageCharges::default());
    let outer = charges.enter_host_operation(Some(10));
    let mut collector =
        InstructionChargeProfileCollector::new(None, None, Some(Arc::clone(&charges)));
    collector.push_operation_scope(Some(charges.enter_host_operation(Some(20))));
    collector.push_operation_scope(Some(charges.enter_host_operation(Some(30))));
    assert_eq!(charges.host_module_offset.load(Ordering::Relaxed), 30);
    collector.pop_operation_scope();
    assert_eq!(charges.host_module_offset.load(Ordering::Relaxed), 20);
    collector.clear_operation_scopes();
    assert_eq!(charges.host_module_offset.load(Ordering::Relaxed), 10);
    for index in 0..=MAX_OPERATION_SCOPES {
        collector
            .push_operation_scope(Some(charges.enter_host_operation(Some(100 + index as u64))));
    }
    assert!(!charges.host_operation_active.load(Ordering::Relaxed));
    collector.pop_operation_scope();
    assert!(charges.host_operation_active.load(Ordering::Relaxed));
    assert_eq!(
        charges.host_module_offset.load(Ordering::Relaxed),
        100 + MAX_OPERATION_SCOPES as u64 - 1
    );
    collector.clear_operation_scopes();
    assert_eq!(charges.host_module_offset.load(Ordering::Relaxed), 10);
    drop(outer);
    assert!(!charges.host_operation_active.load(Ordering::Relaxed));
}

#[test]
fn exact_operation_resolution_never_skips_an_unmapped_or_noncall_frame() {
    use ic_interfaces::execution_environment::{
        WasmInstructionModuleMetadata, WasmInstructionOriginRange,
    };
    let collector = InstructionChargeProfileCollector::new(
        Some(Arc::new(WasmInstructionModuleMetadata {
            instrumented_ranges: vec![
                WasmInstructionOriginRange {
                    instrumented_function_index: 1,
                    start_offset: 1,
                    end_offset: 2,
                    origin_id: Some(0),
                    kind: "original_call".to_string(),
                },
                WasmInstructionOriginRange {
                    instrumented_function_index: 2,
                    start_offset: 3,
                    end_offset: 4,
                    origin_id: None,
                    kind: "runtime_helper".to_string(),
                },
                WasmInstructionOriginRange {
                    instrumented_function_index: 3,
                    start_offset: 5,
                    end_offset: 6,
                    origin_id: Some(1),
                    kind: "original".to_string(),
                },
            ],
            ..Default::default()
        })),
        None,
        None,
    );
    let caller = CapturedFrame {
        function: 1,
        offset: Some(1),
        matching_module: true,
    };
    assert_eq!(
        collector.operation_origin(&[
            CapturedFrame {
                function: 2,
                offset: Some(3),
                matching_module: true
            },
            caller
        ]),
        Some((0, 1))
    );
    for first in [
        CapturedFrame {
            function: 9,
            offset: None,
            matching_module: true,
        },
        CapturedFrame {
            function: 1,
            offset: Some(2),
            matching_module: true,
        },
        CapturedFrame {
            function: 3,
            offset: Some(5),
            matching_module: true,
        },
        CapturedFrame {
            function: 1,
            offset: Some(1),
            matching_module: false,
        },
    ] {
        assert_eq!(collector.operation_origin(&[first, caller]), None);
    }
}

#[test]
fn detail_limits_preserve_observed_category_totals() {
    let mut collector = InstructionChargeProfileCollector::default();
    // A sparse high module-global ID must be retained before the cap.
    collector.observe_block(70_000, 0, 0, 1, 0, 7, 1);
    for id in 0..MAX_BLOCKS {
        collector.observe_block(id as i32, 0, 0, 1, 0, 2, 1);
    }
    // Existing rows keep accumulating after new detail has been limited.
    collector.observe_block(70_000, 0, 0, 1, 0, 7, 1);
    for origin in 0..=MAX_DYNAMIC {
        collector.observe_dynamic(3, 0, origin as i64, origin as i32, 0);
    }
    let profile = collector.finish(11, 13);
    assert!(profile.truncated);
    assert_eq!(profile.blocks.len(), MAX_BLOCKS);
    let high_block = profile
        .blocks
        .iter()
        .find(|row| row.block_id == 70_000)
        .unwrap();
    assert_eq!(high_block.hits, 2);
    assert_eq!(high_block.instructions, 14);
    assert_eq!(profile.dynamic_charges.len(), MAX_DYNAMIC);
    assert_eq!(
        profile.ledger.static_instructions,
        14 + MAX_BLOCKS as u64 * 2
    );
    assert_eq!(
        profile.ledger.dynamic_instructions,
        (MAX_DYNAMIC as u64 + 1) * 3
    );
    assert_eq!(profile.ledger.heap_page_instructions, 11);
    assert_eq!(profile.ledger.stable_page_instructions, 13);
    assert_eq!(
        profile.ledger.raw_instructions,
        38 + MAX_BLOCKS as u64 * 2 + (MAX_DYNAMIC as u64 + 1) * 3
    );
}

#[test]
fn overflow_is_visible_and_cannot_wrap_a_counter() {
    let mut collector = InstructionChargeProfileCollector::default();
    collector.observe_dynamic(u64::MAX, -1, -1, -1, 1);
    collector.observe_dynamic(1, -1, -1, -1, 1);
    let profile = collector.finish(0, 0);
    assert!(profile.truncated);
    assert_eq!(profile.ledger.raw_instructions, u64::MAX);
    assert_eq!(profile.ledger.stable_api_instructions, u64::MAX);
    assert_eq!(profile.dynamic_charges[0].function_index, None);
    assert_eq!(profile.dynamic_charges[0].instructions, u64::MAX);
}
