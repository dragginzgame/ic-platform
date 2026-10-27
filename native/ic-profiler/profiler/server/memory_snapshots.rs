//! Immutable diagnostic memory captures. No canister code or management call
//! runs here: a capture retains the already committed operation-boundary state.
//!
//! The budget estimates retained delta pages, not process RSS. Checkpoint file
//! mappings, allocator sharing and other owners of the same pages are excluded.

use crate::{InstanceId, memory_access_tracing::server_identity};
use base64::prelude::*;
use ic_replicated_state::{
    PageMap,
    canister_state::{WASM_PAGE_SIZE_IN_BYTES, execution_state::ExecutionState},
    page_map::Buffer,
};
use ic_types::{CanisterId, SubnetId};
use pocket_ic::common::rest::trace_u64;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock},
};

pub(crate) const MAX_SNAPSHOTS: usize = 8;
pub(crate) const MAX_RETAINED_DELTA_BYTES: usize = 128 * 1024 * 1024;
pub(crate) const MAX_READ_PAGES: u32 = 64;
pub(crate) const PAGE_BYTES: u64 = WASM_PAGE_SIZE_IN_BYTES as u64;
const MAX_LABEL_BYTES: usize = 1024;
// Deliberately count every snapshot independently, even if its delta is shared.
const ESTIMATED_DELTA_PAGE_METADATA_BYTES: usize = 256;
const ESTIMATED_CAPTURE_METADATA_BYTES: usize = 8 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MemoryKind {
    Heap,
    Stable,
}

/// Counters use decimal strings in JSON to preserve exact Wasm64 addresses.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub(crate) struct MemorySnapshotMetadata {
    pub snapshot_id: String,
    pub server_generation: String,
    pub server_binary_hash: Option<String>,
    pub instance_id: InstanceId,
    /// Instance IDs are append-only within this server generation.
    pub instance_generation: String,
    pub canister_id: String,
    pub canister_id_base64: String,
    pub subnet_id: String,
    pub label: Option<String>,
    pub module_hash: Option<String>,
    pub module_hash_kind: Option<String>,
    #[serde(with = "trace_u64::option")]
    #[schemars(with = "Option<String>")]
    pub last_install_timestamp_nanos: Option<u64>,
    /// This includes all work committed by the preceding operation, possibly
    /// other messages, callbacks or timers. It is not a per-message snapshot.
    pub scope: String,
    #[serde(with = "trace_u64")]
    #[schemars(with = "String")]
    pub state_height: u64,
    #[serde(with = "trace_u64")]
    #[schemars(with = "String")]
    pub state_time_nanos: u64,
    #[serde(with = "trace_u64")]
    #[schemars(with = "String")]
    pub wasm_memory_size_bytes: u64,
    #[serde(with = "trace_u64")]
    #[schemars(with = "String")]
    pub stable_memory_size_bytes: u64,
    #[serde(with = "trace_u64")]
    #[schemars(with = "String")]
    pub trace_generation: u64,
    pub tracing_enabled: bool,
    #[serde(with = "trace_u64::option")]
    #[schemars(with = "Option<String>")]
    pub last_sequence_number: Option<u64>,
    #[serde(with = "trace_u64::option")]
    #[schemars(with = "Option<String>")]
    pub last_runtime_cost_sequence_number: Option<u64>,
    /// Delta page bytes plus conservative per-page and fixed metadata overhead;
    /// neither an exact live allocation size nor a checkpoint-file size.
    #[serde(with = "trace_u64")]
    #[schemars(with = "String")]
    pub estimated_retained_delta_bytes: u64,
    pub retention_estimate_kind: String,
}

impl MemorySnapshotMetadata {
    pub(crate) fn allocated_bytes(&self, memory: MemoryKind) -> u64 {
        match memory {
            MemoryKind::Heap => self.wasm_memory_size_bytes,
            MemoryKind::Stable => self.stable_memory_size_bytes,
        }
    }
}

pub(crate) struct MemorySnapshot {
    metadata: MemorySnapshotMetadata,
    heap: Option<PageMap>,
    stable: Option<PageMap>,
}

impl MemorySnapshot {
    pub(crate) fn metadata(&self) -> &MemorySnapshotMetadata {
        &self.metadata
    }

    /// Returns exactly the requested pages, zero padding outside allocation.
    /// Such padding is NOT memory data: the caller must use `allocated_bytes`
    /// to distinguish allocated zero bytes from unallocated addresses.
    pub(crate) fn read_pages(
        &self,
        memory: MemoryKind,
        start_page: u64,
        page_count: u32,
    ) -> Result<Vec<u8>, String> {
        if !(1..=MAX_READ_PAGES).contains(&page_count) {
            return Err(format!("page_count must be between 1 and {MAX_READ_PAGES}"));
        }
        let offset = start_page
            .checked_mul(PAGE_BYTES)
            .ok_or_else(|| "memory address overflow".to_string())?;
        let length = u64::from(page_count) * PAGE_BYTES;
        offset
            .checked_add(length)
            .ok_or_else(|| "memory address overflow".to_string())?;
        let mut bytes = vec![0; length as usize];
        let available = self
            .metadata
            .allocated_bytes(memory)
            .saturating_sub(offset)
            .min(length);
        if available != 0 {
            let page_map = match memory {
                MemoryKind::Heap => self.heap.as_ref(),
                MemoryKind::Stable => self.stable.as_ref(),
            }
            .ok_or_else(|| "captured memory contents are unavailable".to_string())?;
            let offset = usize::try_from(offset)
                .map_err(|_| "memory address is not representable on this host".to_string())?;
            Buffer::new(page_map.clone()).read(&mut bytes[..available as usize], offset);
        }
        Ok(bytes)
    }
}

#[derive(Default)]
struct SnapshotStore {
    snapshots: BTreeMap<String, Arc<MemorySnapshot>>,
    estimated_retained_delta_bytes: usize,
}

impl SnapshotStore {
    fn insert(&mut self, snapshot: Arc<MemorySnapshot>) -> Result<(), String> {
        let id = &snapshot.metadata.snapshot_id;
        if self.snapshots.contains_key(id) {
            return Err("snapshot ID already exists".to_string());
        }
        if self.snapshots.len() >= MAX_SNAPSHOTS {
            return Err(format!(
                "snapshot limit reached ({MAX_SNAPSHOTS} globally); delete a retained snapshot before capturing another"
            ));
        }
        let estimated = usize::try_from(snapshot.metadata.estimated_retained_delta_bytes)
            .map_err(|_| "snapshot retained-delta estimate overflow".to_string())?;
        let retained = self
            .estimated_retained_delta_bytes
            .checked_add(estimated)
            .ok_or_else(|| "snapshot retained-delta estimate overflow".to_string())?;
        if retained > MAX_RETAINED_DELTA_BYTES {
            return Err(format!(
                "snapshot retained-delta estimate exceeds the global {MAX_RETAINED_DELTA_BYTES}-byte budget; delete retained snapshots before capturing another"
            ));
        }
        self.estimated_retained_delta_bytes = retained;
        self.snapshots.insert(id.clone(), snapshot);
        Ok(())
    }
}

fn store() -> &'static Mutex<SnapshotStore> {
    static STORE: OnceLock<Mutex<SnapshotStore>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(SnapshotStore::default()))
}

/// All state arguments must come from the same immutable labeled state and the
/// cursor must be sampled while the PocketIC operation owns the instance.
#[allow(clippy::too_many_arguments)]
pub(crate) fn capture_snapshot(
    snapshot_id: String,
    instance_id: InstanceId,
    canister_id: CanisterId,
    subnet_id: SubnetId,
    state_height: u64,
    state_time_nanos: u64,
    execution_state: Option<&ExecutionState>,
    label: Option<String>,
    trace_generation: u64,
    last_sequence_number: Option<u64>,
    last_runtime_cost_sequence_number: Option<u64>,
    tracing_enabled: bool,
) -> Result<Arc<MemorySnapshot>, String> {
    if label
        .as_ref()
        .is_some_and(|label| label.len() > MAX_LABEL_BYTES)
    {
        return Err(format!(
            "snapshot label exceeds {MAX_LABEL_BYTES} UTF-8 bytes"
        ));
    }
    let delta_pages = execution_state.map_or(0, |state| {
        state
            .wasm_memory
            .page_map
            .num_delta_pages()
            .saturating_add(state.stable_memory.page_map.num_delta_pages())
    });
    let estimated = delta_pages
        .checked_mul(ic_sys::PAGE_SIZE + ESTIMATED_DELTA_PAGE_METADATA_BYTES)
        .and_then(|bytes| bytes.checked_add(ESTIMATED_CAPTURE_METADATA_BYTES))
        .and_then(|bytes| bytes.checked_add(label.as_ref().map_or(0, String::len)))
        .ok_or_else(|| "snapshot retained-delta estimate overflow".to_string())?;
    let memory_size = |pages: u64| {
        pages
            .checked_mul(PAGE_BYTES)
            .ok_or_else(|| "captured memory size overflow".to_string())
    };
    let heap_size =
        memory_size(execution_state.map_or(0, |state| state.wasm_memory.size.get() as u64))?;
    let stable_size =
        memory_size(execution_state.map_or(0, |state| state.stable_memory.size.get() as u64))?;
    let identity = server_identity();
    let metadata = MemorySnapshotMetadata {
        snapshot_id: snapshot_id.clone(),
        server_generation: identity.generation.clone(),
        server_binary_hash: identity.binary_hash.clone(),
        instance_id,
        instance_generation: format!("{}:{instance_id}", identity.generation),
        canister_id: canister_id.to_string(),
        canister_id_base64: BASE64_STANDARD.encode(canister_id.get().as_slice()),
        subnet_id: subnet_id.to_string(),
        label,
        module_hash: execution_state
            .map(|state| hex::encode(state.wasm_binary.binary.module_hash())),
        module_hash_kind: execution_state.map(|state| {
            if state
                .wasm_binary
                .binary
                .as_slice()
                .starts_with(&[0x1f, 0x8b])
            {
                "installed_compressed_sha256"
            } else {
                "installed_uncompressed_sha256"
            }
            .to_string()
        }),
        last_install_timestamp_nanos: execution_state
            .and_then(|state| state.last_install_timestamp)
            .map(|time| time.as_nanos_since_unix_epoch()),
        scope: "committed_operation_boundary".to_string(),
        state_height,
        state_time_nanos,
        wasm_memory_size_bytes: heap_size,
        stable_memory_size_bytes: stable_size,
        trace_generation,
        tracing_enabled,
        last_sequence_number,
        last_runtime_cost_sequence_number,
        estimated_retained_delta_bytes: estimated as u64,
        retention_estimate_kind: "delta_page_bytes_plus_metadata_not_rss".to_string(),
    };
    let snapshot = Arc::new(MemorySnapshot {
        metadata,
        heap: execution_state.map(|state| state.wasm_memory.page_map.clone()),
        stable: execution_state.map(|state| state.stable_memory.page_map.clone()),
    });
    store().lock().unwrap().insert(Arc::clone(&snapshot))?;
    Ok(snapshot)
}

/// Opaque IDs remain fetchable after their canister or instance is deleted.
pub(crate) fn get_snapshot(snapshot_id: &str) -> Result<Arc<MemorySnapshot>, String> {
    store()
        .lock()
        .unwrap()
        .snapshots
        .get(snapshot_id)
        .cloned()
        .ok_or_else(|| "memory snapshot not found (deleted or server restarted)".to_string())
}

pub(crate) fn delete_snapshot(snapshot_id: &str) -> Result<bool, String> {
    let mut store = store().lock().unwrap();
    let Some(snapshot) = store.snapshots.remove(snapshot_id) else {
        return Ok(false);
    };
    store.estimated_retained_delta_bytes = store
        .estimated_retained_delta_bytes
        .saturating_sub(snapshot.metadata.estimated_retained_delta_bytes as usize);
    Ok(true)
}

#[cfg(test)]
mod tests {
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
}
