//! PocketIC profiling implementation and stable public module aliases.

pub(crate) mod rendering;
mod server;

pub(crate) use server::{
    backend_description, computation_catalog_notes, instruction_charge_rules,
    memory_access_tracing, memory_snapshot_api, memory_snapshot_image, memory_snapshots,
    profiling_manual, system_profile_metrics, system_profile_product, system_profile_render,
};
pub use server::{
    catalog_cli, computation_catalog, instruction_profiler_config, memory_access_viewer,
    system_profile_api, system_profile_cli,
};
