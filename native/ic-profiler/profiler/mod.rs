//! PocketIC profiling implementation: canister attribution, whole-system observation,
//! compact viewer and built-in agent reference. Public root aliases preserve API paths.

pub(crate) mod rendering;

#[path = "server/backend_description.rs"]
pub(crate) mod backend_description;
#[path = "server/client_api.rs"]
pub(crate) mod client_api;

#[path = "server/computation_catalog.rs"]
pub mod computation_catalog;
#[path = "server/computation_catalog_notes.rs"]
pub(crate) mod computation_catalog_notes;
#[path = "server/instruction_charge_rules.rs"]
pub(crate) mod instruction_charge_rules;
#[path = "server/instruction_profiler_config.rs"]
pub mod instruction_profiler_config;
#[path = "server/memory_access_tracing.rs"]
pub(crate) mod memory_access_tracing;
#[path = "server/memory_access_viewer.rs"]
pub mod memory_access_viewer;
#[path = "server/memory_snapshot_api.rs"]
pub(crate) mod memory_snapshot_api;
#[path = "server/memory_snapshot_image.rs"]
pub(crate) mod memory_snapshot_image;
#[path = "server/memory_snapshots.rs"]
pub(crate) mod memory_snapshots;
#[path = "server/system_profile_api/mod.rs"]
pub mod system_profile_api;
#[path = "server/system_profile_cli.rs"]
pub mod system_profile_cli;
#[path = "server/system_profile_metrics.rs"]
pub(crate) mod system_profile_metrics;
#[path = "server/system_profile_render.rs"]
pub(crate) mod system_profile_render;

#[path = "server/profiling_manual.rs"]
pub(crate) mod profiling_manual;

#[path = "server/catalog_cli.rs"]
pub mod catalog_cli;

#[path = "server/system_profile_product/mod.rs"]
pub(crate) mod system_profile_product;
