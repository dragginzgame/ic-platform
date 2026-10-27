//! Observation-only adapters; deterministic pricing stays in the parent module.

use ic_interfaces::execution_environment::{TracedRuntimeCost, WasmMemoryAccessTracer};
use ic_types::{CanisterId, SubnetId};
use std::sync::Arc;

/// Mirror only committed host-accounting observations into the run profiler.
/// The existing journal remains responsible for speculative-operation rollback.
/// These values describe the deterministic ledger; they never determine it.
pub(super) fn profile_runtime_cost(
    canister_id: Option<CanisterId>,
    subnet_id: Option<SubnetId>,
    cost: &TracedRuntimeCost,
) {
    if !ic_system_profiler::enabled() {
        return;
    }
    let mut fields = vec![
        ("rule_id", cost.rule_id.to_string()),
        ("unit", cost.unit.as_str().to_string()),
        ("kind", cost.kind.as_str().to_string()),
        ("amount", cost.amount.to_string()),
        ("description", cost.description.clone()),
        ("accounting_scope", "committed_host_ledger".to_string()),
    ];
    if let Some(id) = canister_id {
        fields.push(("canister_id", id.to_string()));
    }
    if let Some(id) = subnet_id {
        fields.push(("subnet_id", id.to_string()));
    }
    if let Some(amount) = cost.nominal_cycles {
        fields.push(("nominal_cycles", amount.to_string()));
    }
    if let Some(time) = cost.time {
        fields.push(("ic_time_ns", time.as_nanos_since_unix_epoch().to_string()));
    }
    if let Some(round) = cost.round {
        fields.push(("round", round.to_string()));
    }
    if let Some(id) = cost.call_context_id {
        fields.push(("call_context_id", id.get().to_string()));
    }
    if let Some(id) = &cost.message_id {
        fields.push(("request_id", id.clone()));
    }
    if let Some(id) = cost.callback_id {
        fields.push(("callback_id", id.to_string()));
    }
    if let Some(bytes) = cost.payload_bytes {
        fields.push(("payload_bytes", bytes.to_string()));
    }
    if let Some(instructions) = cost.instructions {
        fields.push(("instructions", instructions.to_string()));
    }
    // Keep source-site configuration and incomplete-journal flags with the
    // observation. The bounded profiler validates and limits attribute bytes.
    fields.extend(
        cost.details
            .iter()
            .map(|(key, value)| (key.as_str(), value.clone())),
    );
    ic_system_profiler::event_fields("ic.accounting.pricing", "runtime_cost", &fields);
}

/// A host-only sink lets instrumented real replicas observe their ledger even
/// without configuring the separate canister memory/instruction profiler.
struct SystemRuntimeCostObserver;

impl WasmMemoryAccessTracer for SystemRuntimeCostObserver {
    fn is_tracing_enabled(&self, _canister_id: CanisterId) -> bool {
        false
    }
    fn record(
        &self,
        _canister_id: CanisterId,
        _execution: ic_interfaces::execution_environment::TracedWasmExecution,
    ) {
    }
}

pub(super) fn system_runtime_cost_observer() -> Arc<dyn WasmMemoryAccessTracer> {
    static OBSERVER: std::sync::OnceLock<Arc<dyn WasmMemoryAccessTracer>> =
        std::sync::OnceLock::new();
    Arc::clone(OBSERVER.get_or_init(|| Arc::new(SystemRuntimeCostObserver)))
}
