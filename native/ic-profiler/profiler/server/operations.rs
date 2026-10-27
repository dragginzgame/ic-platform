//! Profiling-specific PocketIC operations. This file is a child module of
//! pocket_ic so it can use its private routing/state without widening visibility.
use super::PocketIc;
use crate::memory_access_tracing::server_identity;
use crate::state_api::state::{OpOut, PocketIcError};
use crate::{OpId, Operation};
use ic_interfaces_state_manager::StateReader;
use ic_metrics::MetricsRegistry;
use ic_state_machine_tests::StateMachine;
use ic_types::CanisterId;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Retains immutable memory maps at a committed operation boundary. This does
/// not execute a round, a canister method or a management-canister operation.
#[derive(Clone, Debug)]
pub struct CaptureMemorySnapshot {
    instance_id: crate::InstanceId,
    canister_id: CanisterId,
    label: Option<String>,
    snapshot_id: String,
}

impl CaptureMemorySnapshot {
    pub fn new(
        instance_id: crate::InstanceId,
        canister_id: CanisterId,
        label: Option<String>,
    ) -> Self {
        Self {
            instance_id,
            canister_id,
            label,
            // Capture requests must not share the graph's operation cache.
            snapshot_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl Operation for CaptureMemorySnapshot {
    fn retry_if_busy(&self) -> bool {
        true
    }

    fn compute(&self, pic: &mut PocketIc) -> OpOut {
        let Some(subnet) = pic.try_route_canister(self.canister_id) else {
            return OpOut::Error(PocketIcError::CanisterNotFound(self.canister_id));
        };
        // One labeled immutable state supplies both memories, identity, size,
        // timestamp and height; never race separate live-memory downloads.
        let labeled = subnet.state_manager.get_latest_state();
        let height = labeled.height().get();
        let state = labeled.take();
        let Some(canister) = state.canister_state(&self.canister_id) else {
            return OpOut::Error(PocketIcError::CanisterNotFound(self.canister_id));
        };
        let (generation, last_sequence, last_runtime_sequence, tracing_enabled) = pic
            .subnets
            .memory_access_trace_recorder
            .snapshot_cursor(self.canister_id);
        match crate::memory_snapshots::capture_snapshot(
            self.snapshot_id.clone(),
            self.instance_id,
            self.canister_id,
            subnet.get_subnet_id(),
            height,
            state.metadata.batch_time.as_nanos_since_unix_epoch(),
            canister.execution_state.as_ref(),
            self.label.clone(),
            generation,
            last_sequence,
            last_runtime_sequence,
            tracing_enabled,
        ) {
            Ok(snapshot) => match serde_json::to_vec(snapshot.metadata()) {
                Ok(bytes) => OpOut::Bytes(bytes),
                Err(error) => OpOut::Error(PocketIcError::MemorySnapshotError(error.to_string())),
            },
            Err(error) => OpOut::Error(PocketIcError::MemorySnapshotError(error)),
        }
    }

    fn id(&self) -> OpId {
        OpId(format!("capture_memory_snapshot({})", self.snapshot_id))
    }
}

/// Starts a bounded IC charge profiling window for a canister.
#[derive(Copy, Clone, Debug)]
pub struct StartInstructionProfiling {
    pub canister_id: CanisterId,
    pub max_executions: u64,
    pub read_mask: bool,
    pub write_mask: bool,
}

impl Operation for StartInstructionProfiling {
    fn retry_if_busy(&self) -> bool {
        true
    }

    fn compute(&self, pic: &mut PocketIc) -> OpOut {
        match pic.try_route_canister(self.canister_id) {
            Some(subnet) if subnet.canister_exists(self.canister_id) => {
                if pic
                    .subnets
                    .memory_access_trace_recorder
                    .start_profiling_masks(
                        self.canister_id,
                        self.max_executions,
                        self.read_mask,
                        self.write_mask,
                    )
                {
                    OpOut::NoOutput
                } else {
                    OpOut::Error(PocketIcError::MemoryAccessTraceBudgetExceeded)
                }
            }
            _ => OpOut::Error(PocketIcError::CanisterNotFound(self.canister_id)),
        }
    }

    fn id(&self) -> OpId {
        OpId(format!(
            "start_instruction_profiling({},{},{},{})",
            self.canister_id, self.max_executions, self.read_mask, self.write_mask
        ))
    }
}

/// Starts recording the Wasm memory accesses of all message executions of a canister.
#[derive(Copy, Clone, Debug)]
pub struct StartMemoryAccessTracing {
    pub canister_id: CanisterId,
}

impl Operation for StartMemoryAccessTracing {
    fn retry_if_busy(&self) -> bool {
        true
    }

    fn compute(&self, pic: &mut PocketIc) -> OpOut {
        match pic.try_route_canister(self.canister_id) {
            Some(subnet) if subnet.canister_exists(self.canister_id) => {
                if pic
                    .subnets
                    .memory_access_trace_recorder
                    .start(self.canister_id)
                {
                    OpOut::NoOutput
                } else {
                    OpOut::Error(PocketIcError::MemoryAccessTraceBudgetExceeded)
                }
            }
            _ => OpOut::Error(PocketIcError::CanisterNotFound(self.canister_id)),
        }
    }

    fn id(&self) -> OpId {
        OpId(format!("start_memory_access_tracing({})", self.canister_id))
    }
}

/// Stops recording the Wasm memory accesses of a canister (keeps the recorded ones).
#[derive(Copy, Clone, Debug)]
pub struct StopMemoryAccessTracing {
    pub canister_id: CanisterId,
}

impl Operation for StopMemoryAccessTracing {
    fn retry_if_busy(&self) -> bool {
        true
    }

    fn compute(&self, pic: &mut PocketIc) -> OpOut {
        pic.subnets
            .memory_access_trace_recorder
            .stop(self.canister_id);
        OpOut::NoOutput
    }

    fn id(&self) -> OpId {
        OpId(format!("stop_memory_access_tracing({})", self.canister_id))
    }
}

/// Returns the Wasm memory accesses recorded for a canister so far.
#[derive(Copy, Clone, Debug)]
pub struct GetMemoryAccessTrace {
    pub canister_id: CanisterId,
}

impl Operation for GetMemoryAccessTrace {
    fn compute(&self, pic: &mut PocketIc) -> OpOut {
        OpOut::MemoryAccessTrace(Arc::new(
            pic.subnets
                .memory_access_trace_recorder
                .get(self.canister_id),
        ))
    }

    fn id(&self) -> OpId {
        OpId(format!("get_memory_access_trace({})", self.canister_id))
    }
}

/// Deletes the Wasm memory accesses recorded for a canister so far.
#[derive(Clone, Debug)]
pub struct ClearMemoryAccessTrace {
    pub canister_id: CanisterId,
    pub expected_server_generation: Option<String>,
    pub expected_trace_generation: Option<u64>,
    pub expected_last_sequence_number: Option<u64>,
    pub expected_last_runtime_cost_sequence_number: Option<Option<u64>>,
}

impl Operation for ClearMemoryAccessTrace {
    fn retry_if_busy(&self) -> bool {
        true
    }

    fn compute(&self, pic: &mut PocketIc) -> OpOut {
        if self
            .expected_server_generation
            .as_deref()
            .is_some_and(|expected| expected != server_identity().generation)
        {
            return OpOut::Error(PocketIcError::MemoryAccessTraceChanged);
        }
        let recorder = &pic.subnets.memory_access_trace_recorder;
        if let Some(generation) = self.expected_trace_generation {
            if !recorder.clear_if_unchanged(
                self.canister_id,
                generation,
                self.expected_last_sequence_number,
                self.expected_last_runtime_cost_sequence_number,
            ) {
                return OpOut::Error(PocketIcError::MemoryAccessTraceChanged);
            }
        } else {
            recorder.clear(self.canister_id);
        }
        OpOut::NoOutput
    }

    fn id(&self) -> OpId {
        OpId(format!(
            "clear_memory_access_trace({},{:?},{:?},{:?},{:?})",
            self.canister_id,
            self.expected_server_generation,
            self.expected_trace_generation,
            self.expected_last_sequence_number,
            self.expected_last_runtime_cost_sequence_number
        ))
    }
}

impl PocketIc {
    /// Clone registry handles only. Gathering happens after the API's instance
    /// locks have been released; these handles do not keep StateMachines alive.
    pub(crate) fn profiling_metrics_registries(&self) -> Vec<(String, String, MetricsRegistry)> {
        type Entry = (std::sync::Weak<StateMachine>, u64);
        static GENERATIONS: std::sync::OnceLock<std::sync::Mutex<BTreeMap<usize, Entry>>> =
            std::sync::OnceLock::new();
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let mut generations = GENERATIONS
            .get_or_init(|| std::sync::Mutex::new(BTreeMap::new()))
            .lock()
            .unwrap();
        generations.retain(|_, (weak, _)| weak.strong_count() > 0);
        self.subnets
            .get_all()
            .into_iter()
            .map(|subnet| {
                let sm = &subnet.state_machine;
                let key = Arc::as_ptr(sm) as usize;
                let (_, generation) = generations.entry(key).or_insert_with(|| {
                    (
                        Arc::downgrade(sm),
                        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                    )
                });
                (
                    sm.get_subnet_id().to_string(),
                    format!("registry-{generation}"),
                    sm.metrics_registry().clone(),
                )
            })
            .collect()
    }
}

/// These are observation-side lifecycle records, never replicated state or
/// pricing inputs. Results may be polled repeatedly, so consumers deduplicate
/// by request identity and cohort. Latency ends when the API observes a result,
/// rather than claiming this is the exact canister commit timestamp.
pub(crate) fn profile_direct_request(
    operation: &str,
    request_id: &str,
    subnet: Option<String>,
    canister: Option<String>,
    method: Option<String>,
    result: Option<&str>,
) {
    if !ic_system_profiler::enabled() {
        return;
    }
    let mut fields = vec![
        ("request_id", request_id.to_string()),
        ("cohort", "root_ingress".to_string()),
        ("ingress_path", "direct".to_string()),
        ("completion_source", "client_observed".to_string()),
    ];
    if let Some(value) = subnet {
        fields.push(("subnet_id", value));
    }
    if let Some(value) = canister {
        fields.push(("canister_id", value));
    }
    if let Some(value) = method {
        fields.push(("method", value));
    }
    if let Some(value) = result {
        fields.push(("result", value.to_string()));
    }
    ic_system_profiler::event_fields("ic.ingress.direct", operation, &fields);
}
