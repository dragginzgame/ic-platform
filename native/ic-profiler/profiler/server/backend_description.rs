//! IC/PocketIC facts owned by this compiled native provider.
use serde_json::{Value, json};
use std::sync::OnceLock;

pub(crate) fn description() -> &'static Value {
    static DESCRIPTION: OnceLock<Value> = OnceLock::new();
    DESCRIPTION.get_or_init(|| json!({
        "availability": "available",
        "backend": "instrumented-pocket-ic",
        "value_kind": "compiled_source_description",
        "source_sha256": {
            "rs/execution_environment/src/scheduler.rs": hex::encode(ic_crypto_sha2::Sha256::hash(include_bytes!("../../../execution_environment/src/scheduler.rs"))),
            "rs/config/src/subnet_config.rs": hex::encode(ic_crypto_sha2::Sha256::hash(include_bytes!("../../../config/src/subnet_config.rs"))),
            "rs/pocket_ic_server/src/pocket_ic.rs": hex::encode(ic_crypto_sha2::Sha256::hash(include_bytes!("../../src/pocket_ic.rs")))
        },
        "scheduler_model": scheduler_model(),
        "round_semantics": {"initial_budget":"actual signed starting counter; nominal per-worker limit minus a slice reserve plus one can differ from the configured nominal limit","global_accounting":"subtract maximum worker consumption per inner iteration plus all-canister finalization; summed worker work is separate","execution_accounting":"guest instructions include IC API/page weights; dirty-copy round charges, per-slice overhead and productive-canister-pass overhead are distinct","negative_remaining":"initial counter exhaustion does not by itself imply exceeding the nominal configured per-worker limit","heap_delta":"heap_delta_bytes is the logical scheduler delta, not physical RAM/RSS or disk writes; the configured max_heap_delta_per_iteration_bytes also gates the round-cumulative sum of worker deltas checked between inner iterations; missing legacy measurements remain null","slice_boundary":"the configured instruction slice limit is a metering gate; a metered block can cross it, and DTS carries the counter overrun into the next slice; a small overrun alone is not a protocol violation"},
        "coverage_notes": ["PocketIC direct ingress bypasses native envelope authentication; HTTP gateway can verify it.", "PocketIC uses real XNet stream/payload/routing code with in-process transport and stubbed certification signature verification.", "PocketIC threshold replies do not measure distributed DKG/IDKG or production signing."],
        "components": {"ic.accounting":"real","ic.ingress":"path_dependent","ic.crypto":"path_dependent","ic.consensus":"emulated_or_absent","ic.p2p":"absent","ic.certification":"path_dependent","ic.messaging":"real","ic.xnet":"path_dependent","ic.execution":"real","ic.query":"real","ic.compilation":"real","ic.sandbox":"real","ic.memory":"real","ic.state":"real","ic.management":"real","ic.chain_key":"emulated","ic.adapter":"path_dependent","ic.control":"external_or_absent","ic.host":"observed_boundary_only","ic.observability":"real"}
    }))
}

pub(crate) fn explain() -> Value {
    ic_system_profiler::explain_with_backend(description())
}

fn scheduler_model() -> Value {
    let config = ic_config::subnet_config::SubnetConfig::new(
        ic_registry_subnet_type::SubnetType::Application,
    )
    .scheduler_config;
    let round = config.max_instructions_per_round.get();
    let slice = config
        .max_instructions_per_slice
        .get()
        .max(config.max_instructions_per_install_code_slice.get());
    let initial = i128::from(round) - i128::from(slice) + 1;
    let workers = config.scheduler_cores;
    json!({
        "scope":"The instrumented replicated canister scheduler; logical instruction progression, not a wall-clock timeline or a production throughput guarantee. Read each round's captured configuration for actual limits.",
        "iteration_steps":[
            "Before each inner iteration, eligible canisters are assigned to per-worker lists according to the deterministic schedule, with separate long/new execution pools.",
            "Each worker processes its assigned canisters sequentially. It can execute several queued messages/slices from one canister before moving to its next assigned canister, subject to budget, heap-delta, blocked-install and long-execution rules. A canister is not executed concurrently on multiple workers.",
            "Workers do not steal work from another worker's list. All workers join at an iteration barrier; a worker that finishes its list early waits.",
            "The scheduler collects results and finalizes the iteration, checks stopping conditions and inducts same-subnet messages if continuing. Another iteration rebuilds the schedule when eligible work and allowance remain."
        ],
        "budget_accounting":"Every worker starts an iteration with a copy of the current remaining counter. The shared counter loses that iteration's maximum worker consumption plus global finalization, not the sum of worker consumption. The busiest worker can change between iterations, so the charged path is not necessarily one physical thread's cumulative work.",
        "round_charges":"Worker consumption includes guest instructions, dirty-page-copy round charges, per-slice overhead and productive canister-pass overhead. Global finalization is added separately. These scheduler charges are distinct from the billed guest instruction ledger.",
        "dispatch_reserve":"For nominal round allowance R and S=max(update_slice,install_code_slice), the initial counter is R-S+1. Workers check this counter before starting another execution slice; they do not wait for one thread to accumulate R instructions. A slice started with positive remaining can finish after the counter becomes negative. Many small messages can exhaust dispatch near the initial counter instead of using the whole nominal allowance.",
        "example_application_defaults":{
            "nominal_round_instructions_per_worker":round.to_string(),
            "max_slice_instructions":config.max_instructions_per_slice.get().to_string(),
            "initial_dispatch_counter":initial.to_string(),
            "scheduler_workers":workers.to_string(),
            "approximate_aggregate_nominal_round_instructions":(u128::from(round) * workers as u128).to_string(),
            "max_regular_message_instructions":config.max_instructions_per_message.get().to_string(),
            "interpretation":"Illustrative defaults, not captured limits or an exact hard ceiling: overhead and metered-block overshoot require their own accounting. Aggregate capacity assumes busy, balanced workers; it is not a fungible subnet pool. The message cap spans slices/rounds; a slice cap is not a hard canister-round quota."
        },
        "chart":"Logical rounds · charged worker path / nominal worker budget divides the sum of iteration maxima plus finalization by R. Dispatch may stop well below the nominal allowance after reserving the largest slice; use the compiled example or captured counters. Use initial/remaining counters and actual stop reasons; do not interpret the bar as summed worker utilization or physical CPU utilization.",
        "other_stops":"The heap-delta guard, no eligible work, no progress and other captured stopping conditions can end the round before its instruction counter is exhausted. Logical heap delta is not RSS or disk traffic.",
        "sources":["rs/execution_environment/src/scheduler.rs: inner_round, execute_canisters_in_inner_round, execute_canisters_on_thread and initial round-counter reservation", "rs/execution_environment/src/scheduler/round_schedule.rs: start_iteration and partition_canisters_to_cores"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backend_examples_use_compiled_defaults_and_retain_source_identity() {
        let config = ic_config::subnet_config::SubnetConfig::new(
            ic_registry_subnet_type::SubnetType::Application,
        )
        .scheduler_config;
        let result = explain();
        let example = &result["rounds"]["scheduler_model"]["example_application_defaults"];
        assert_eq!(
            example["nominal_round_instructions_per_worker"],
            config.max_instructions_per_round.get().to_string()
        );
        assert_eq!(
            example["max_regular_message_instructions"],
            config.max_instructions_per_message.get().to_string()
        );
        assert_eq!(
            result["backend_description"]["source_sha256"]
                .as_object()
                .unwrap()
                .len(),
            3
        );
        assert_eq!(
            result["components"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["component_id"] == "ic.chain_key")
                .unwrap()["pocketic_implementation"],
            "emulated"
        );
        assert!(
            result["coverage_notes"][0]
                .as_str()
                .unwrap()
                .contains("PocketIC")
        );
    }
}
