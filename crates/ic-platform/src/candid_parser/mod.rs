//! Explicit native host APIs for Candid parsing, checking and browser bindings.
//!
//! Callers own input/output admission, import resolution and diagnostics. Parsing
//! and generation have no automatic resource quotas. `check_prog` ignores imports;
//! inspect [`Dec`] before checking self-contained input. [`CandidSource::File`]
//! loads files/imports; its loaders and [`service_equal`] can print diagnostics.
//! Parse an [`IDLProg`] and use [`check_prog`] for silent in-memory checking.
//! Host retains service inventories and aggregate decoding budgets. This module
//! and its parser dependency are absent on Wasm, even with all features selected.

pub use candid_parser::bindings::{
    javascript::compile as compile_javascript, typescript::compile as compile_typescript,
};
pub use candid_parser::syntax::{Dec, IDLMergedProg};
pub use candid_parser::types::{
    FuncMode, Type, TypeInner, internal::TypeContainer, subtype::equal as type_equal,
};
pub use candid_parser::utils::{CandidSource, instantiate_candid, service_equal};
pub use candid_parser::{Error, IDLArgs, IDLProg, Result, TypeEnv, check_prog, parse_idl_args};
