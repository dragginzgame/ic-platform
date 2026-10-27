//! Explicit native host APIs for required external Candid parsing and checking.
//!
//! Callers own input/output admission, import resolution and diagnostics. Parsing
//! has no automatic resource quotas. `check_prog` ignores imports;
//! inspect [`Dec`] before checking self-contained input. [`CandidSource::File`]
//! loads files/imports; its loaders and [`service_equal`] can print diagnostics.
//! Parse an [`IDLProg`] and use [`check_prog`] for silent in-memory checking.
//! Host retains service inventories and aggregate decoding budgets. This module
//! and its parser dependency are absent on Wasm, even with all features selected.
//! Dynamic values and type/value serializers support native refusal fixtures;
//! callers retain fixture admission and production decoding policy.

pub use candid_parser::ser::{TypeSerialize, ValueSerializer};
pub use candid_parser::syntax::Dec;
pub use candid_parser::types::{
    FuncMode, Type, TypeInner, internal::TypeContainer, subtype::equal as type_equal,
    value::IDLField,
};
pub use candid_parser::utils::{CandidSource, instantiate_candid, service_equal};
pub use candid_parser::{
    Error, IDLArgs, IDLProg, IDLValue, Result, TypeEnv, check_prog, parse_idl_args,
};
