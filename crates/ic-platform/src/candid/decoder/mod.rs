//! Explicit upstream configuration and staged decoding for existing caller policy.
//!
//! Unlike Platform's finite-limit helpers, these APIs retain upstream defaults,
//! optional quotas/header configuration and diagnostic selection unchanged.
//! Callers own byte admission, stage error mapping, canonical re-encoding,
//! diagnostics disclosure and separate construction/value/finalization/drop
//! observations. No wrapper merges stages or supplies automatic limits.
//! `decode_one` and `decode_args` use upstream defaults and diagnostics; they are
//! distinct from the caller-bounded helpers in the parent module.

pub use candid::de::{DecoderConfig, IDLDeserialize};
pub use candid::utils::{decode_args, decode_args_with_config, decode_one, decode_one_with_config};
