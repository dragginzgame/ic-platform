//! PocketIC profiling implementation: canister attribution, whole-system observation,
//! compact viewer and built-in agent reference. Public root aliases preserve API paths.

pub(crate) mod arguments;
pub(crate) mod rendering;

pub(crate) mod backend_description;
pub(crate) mod client_api;

pub mod computation_catalog;
pub(crate) mod computation_catalog_notes;
pub(crate) mod instruction_charge_rules;
pub mod instruction_profiler_config;
pub(crate) mod memory_access_tracing;
pub mod memory_access_viewer;
pub(crate) mod memory_snapshot_api;
pub(crate) mod memory_snapshot_image;
pub(crate) mod memory_snapshots;
pub mod system_profile_api;
pub mod system_profile_cli;
pub(crate) mod system_profile_metrics;
pub(crate) mod system_profile_render;

pub(crate) mod profiling_manual;

pub mod catalog_cli;

pub(crate) mod system_profile_product;
