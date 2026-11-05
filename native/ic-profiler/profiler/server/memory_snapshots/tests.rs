use super::*;
use ic_replicated_state::PageIndex;

fn snapshot(id: &str, heap: PageMap, heap_bytes: u64, estimated: u64) -> MemorySnapshot {
    MemorySnapshot {
        metadata: MemorySnapshotMetadata {
            snapshot_id: id.to_string(),
            server_generation: "test-server".to_string(),
            server_binary_hash: None,
            instance_id: 0,
            instance_generation: "test-server:0".to_string(),
            canister_id: "test-canister".to_string(),
            canister_id_base64: "AA==".to_string(),
            subnet_id: "test-subnet".to_string(),
            label: None,
            module_hash: None,
            module_hash_kind: None,
            last_install_timestamp_nanos: None,
            scope: "committed_operation_boundary".to_string(),
            state_height: 0,
            state_time_nanos: 0,
            wasm_memory_size_bytes: heap_bytes,
            stable_memory_size_bytes: 0,
            trace_generation: 0,
            tracing_enabled: false,
            last_sequence_number: None,
            last_runtime_cost_sequence_number: None,
            estimated_retained_delta_bytes: estimated,
            retention_estimate_kind: "delta_page_bytes_plus_metadata_not_rss".to_string(),
        },
        heap: Some(heap),
        stable: None,
    }
}

#[test]
fn bytes_are_immutable_and_unallocated_padding_is_separate() {
    let mut map = PageMap::new_for_testing();
    let page = [0x2a; ic_sys::PAGE_SIZE];
    map.update(&[(PageIndex::new(0), &page)]);
    let captured = snapshot("before", map.clone(), PAGE_BYTES, 8192);
    let replacement = [0x7f; ic_sys::PAGE_SIZE];
    map.update(&[(PageIndex::new(0), &replacement)]);
    let bytes = captured.read_pages(MemoryKind::Heap, 0, 2).unwrap();
    assert_eq!(&bytes[..ic_sys::PAGE_SIZE], &page);
    assert!(bytes[ic_sys::PAGE_SIZE..].iter().all(|byte| *byte == 0));
    assert_eq!(
        captured.metadata().allocated_bytes(MemoryKind::Heap),
        PAGE_BYTES
    );
    assert_eq!(captured.metadata().allocated_bytes(MemoryKind::Stable), 0);
    assert!(
        captured
            .read_pages(MemoryKind::Stable, 0, 1)
            .unwrap()
            .iter()
            .all(|byte| *byte == 0)
    );
}

#[test]
fn sparse_large_address_reads_are_bounded_and_exact() {
    let page_index = (500 * 1024 * 1024 * 1024_u64 / PAGE_BYTES) - 1;
    let mut map = PageMap::new_for_testing();
    let page = [0x53; ic_sys::PAGE_SIZE];
    map.update(&[(
        PageIndex::new(page_index * PAGE_BYTES / ic_sys::PAGE_SIZE as u64),
        &page,
    )]);
    let mut captured = snapshot("sparse", map, 0, 8192);
    captured.stable = captured.heap.take();
    captured.metadata.stable_memory_size_bytes = (page_index + 1) * PAGE_BYTES;
    let bytes = captured
        .read_pages(MemoryKind::Stable, page_index, 1)
        .unwrap();
    assert_eq!(bytes.len(), PAGE_BYTES as usize);
    assert_eq!(&bytes[..ic_sys::PAGE_SIZE], &page);
    assert!(bytes[ic_sys::PAGE_SIZE..].iter().all(|byte| *byte == 0));
    assert!(captured.read_pages(MemoryKind::Heap, u64::MAX, 1).is_err());
    assert!(captured.read_pages(MemoryKind::Heap, 0, 0).is_err());
    assert!(
        captured
            .read_pages(MemoryKind::Heap, 0, MAX_READ_PAGES + 1)
            .is_err()
    );
}

#[test]
fn full_store_rejects_new_captures_without_evicting_old_ones() {
    let mut store = SnapshotStore::default();
    for index in 0..MAX_SNAPSHOTS {
        store
            .insert(Arc::new(snapshot(
                &index.to_string(),
                PageMap::new_for_testing(),
                0,
                8192,
            )))
            .unwrap();
    }
    assert!(
        store
            .insert(Arc::new(snapshot(
                "overflow",
                PageMap::new_for_testing(),
                0,
                8192
            )))
            .is_err()
    );
    assert_eq!(store.snapshots.len(), MAX_SNAPSHOTS);
    assert!(store.snapshots.contains_key("0"));
    let mut store = SnapshotStore::default();
    store
        .insert(Arc::new(snapshot(
            "all",
            PageMap::new_for_testing(),
            0,
            MAX_RETAINED_DELTA_BYTES as u64,
        )))
        .unwrap();
    assert!(
        store
            .insert(Arc::new(snapshot(
                "one-more",
                PageMap::new_for_testing(),
                0,
                1
            )))
            .is_err()
    );
    assert!(store.snapshots.contains_key("all"));
}

#[test]
fn metadata_counters_round_trip_without_javascript_precision_loss() {
    let mut captured = snapshot("json", PageMap::new_for_testing(), 0, 8192);
    captured.metadata.state_time_nanos = u64::MAX;
    captured.metadata.last_sequence_number = Some(u64::MAX - 1);
    let json = serde_json::to_value(captured.metadata()).unwrap();
    assert_eq!(json["state_time_nanos"], u64::MAX.to_string());
    assert_eq!(json["last_sequence_number"], (u64::MAX - 1).to_string());
    let decoded: MemorySnapshotMetadata = serde_json::from_value(json).unwrap();
    assert_eq!(decoded.state_time_nanos, u64::MAX);
    assert_eq!(decoded.last_sequence_number, Some(u64::MAX - 1));
}
