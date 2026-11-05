use super::*;

#[test]
fn touched_pagination_is_a_sparse_stable_heap_union_with_exact_totals() {
    let evidence = [vec![(0, 2), (9, 10)], vec![(7_000_000, 8_000_000)]];
    let (pages, total) = touched_window(&evidence, SnapshotMemory::Both, 2, 3);
    assert_eq!(total, 1_000_003);
    assert_eq!(
        pages,
        vec![
            (MemoryKind::Heap, 9),
            (MemoryKind::Stable, 7_000_000),
            (MemoryKind::Stable, 7_000_001)
        ]
    );
    let (pages, total) = touched_window(&evidence, SnapshotMemory::Stable, 999_999, 256);
    assert_eq!(total, 1_000_000);
    assert_eq!(pages, vec![(MemoryKind::Stable, 7_999_999)]);
    let (pages, total) = touched_window(&evidence, SnapshotMemory::Both, 1_000_003, 256);
    assert_eq!(total, 1_000_003);
    assert!(pages.is_empty());
}

#[test]
fn mixed_touched_query_preserves_exact_lifecycle_ids_and_pagination() {
    let query = SnapshotImageQuery::from_arguments(&json!({
        "view": "reads", "memory": "both", "selection": "touched", "page_offset": "9007199254740993",
        "page_count": 256, "logical_execution_id": "18446744073709551615"
    }))
    .unwrap();
    assert_eq!(query.memory, Some(SnapshotMemory::Both));
    assert_eq!(query.view, Some(SnapshotView::Reads));
    assert_eq!(query.selection, Some(SnapshotSelection::Touched));
    assert_eq!(query.page_offset.as_deref(), Some("9007199254740993"));
    assert_eq!(
        query.logical_execution_id.as_deref(),
        Some("18446744073709551615")
    );
    for value in [
        json!({"selection":"guessed"}),
        json!({"view":"guessed"}),
        json!({"attempt_id":-1}),
        json!({"page_offset":"1.5"}),
    ] {
        assert!(SnapshotImageQuery::from_arguments(&value).is_err());
    }
}

#[test]
fn byte_read_ranges_preserve_addresses_and_counts_without_rounding() {
    let ranges: Vec<MemorySnapshotReadRange> = serde_json::from_value(json!([{
        "memory":"stable", "start":"9007199254740993", "end":"9007199254741000",
        "heap_reads":"0", "stable_direct_reads":"1", "stable_native_reads":"18446744073709551615"
    }]))
    .unwrap();
    assert_eq!(ranges[0].start, 9_007_199_254_740_993);
    assert_eq!(ranges[0].end, 9_007_199_254_741_000);
    assert_eq!(ranges[0].stable_native_reads, u64::MAX);
    assert!(
        serde_json::from_value::<Vec<MemorySnapshotReadRange>>(
            json!([{"memory": "heap", "start": "1.5"}])
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<Vec<MemorySnapshotReadRange>>(json!({"available": true})).is_err()
    );
}

#[test]
fn write_mask_arguments_and_ranges_preserve_exact_byte_counts() {
    let query = SnapshotImageQuery::from_arguments(&json!({
        "view": "writes", "memory": "both", "compare_to": "before"
    }))
    .unwrap();
    assert_eq!(query.view, Some(SnapshotView::Writes));
    let ranges: Vec<MemorySnapshotWriteRange> = serde_json::from_value(json!([{
        "memory": "heap", "start": "9007199254740993", "end": "9007199254741001",
        "heap_writes": "18446744073709551615", "stable_writes": 0
    }]))
    .unwrap();
    assert_eq!(ranges[0].start, 9_007_199_254_740_993);
    assert_eq!(ranges[0].end, 9_007_199_254_741_001);
    assert_eq!(ranges[0].heap_writes, u64::MAX);
    assert_eq!(ranges[0].stable_writes, 0);
    assert!(
        serde_json::from_value::<Vec<MemorySnapshotWriteRange>>(
            json!([{"memory": "heap", "start": -1}])
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<Vec<MemorySnapshotWriteRange>>(json!({"available": true}))
            .is_err()
    );
}

#[test]
fn image_arguments_preserve_exact_u64_pages_and_reject_ambiguous_inputs() {
    let query = SnapshotImageQuery::from_arguments(&json!({
        "memory": "stable", "start_page": "9007199254740993", "page_count": "16", "compare_to": "before"
    })).unwrap();
    assert_eq!(query.start_page.as_deref(), Some("9007199254740993"));
    assert_eq!(query.memory, Some(SnapshotMemory::Stable));
    assert_eq!(query.compare_to.as_deref(), Some("before"));
    for value in [
        json!({"start_page": -1}),
        json!({"start_page": "1e3"}),
        json!({"start_page": "18446744073709551616"}),
        json!({"memory": "wasm"}),
        json!({"page_count": 1.5}),
        json!({"compare_to": 4}),
    ] {
        assert!(
            SnapshotImageQuery::from_arguments(&value).is_err(),
            "{value}"
        );
    }
    assert_eq!(
        parse_quantity("18446744073709551615", "page").unwrap(),
        u64::MAX
    );
}
