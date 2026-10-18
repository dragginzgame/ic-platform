//! Explicit upstream configuration and staged decoding for existing caller policy.
//!
//! Unlike Platform's finite-limit helpers, these APIs retain upstream defaults,
//! optional quotas/header configuration and diagnostic selection unchanged.
//! Callers own byte admission, stage error mapping, canonical re-encoding,
//! diagnostics disclosure and separate construction/value/finalization/drop
//! observations. No wrapper merges stages or supplies automatic limits.

pub use candid::de::{DecoderConfig, IDLDeserialize};
pub use candid::utils::{decode_args_with_config, decode_one_with_config};
