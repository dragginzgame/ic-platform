//! Explicit adapters for required external Candid interfaces, never owned Wire RPC.
//!
//! Select `candid-endpoint`, supply artifact-owned argument limits and the explicit
//! empty-envelope policy, and retain caller admission/authentication guards. The
//! adapters delegate entrypoints, async contexts and replies to the existing
//! generator/executor. Invalid arguments and encoding failures trap; access guards
//! retain their ordered rejection semantics. No parser or retry owner is added.
//! Arguments deeper than 16 constructed levels are refused before Candid runs.
pub use super::arguments::{ArgumentDecodeError, ArgumentLimits, EmptyArgs};
pub use ic_platform_macros::{candid_query as query, candid_update as update};

/// Read and decode an external argument tuple after admitting its scalar length.
/// Raw adapters retain their own dispatch, authorization and error projection.
///
/// # Errors
/// Refuses oversized messages before copying and preserves typed decoding causes.
pub fn read<T: for<'de> super::contract::ArgumentDecoder<'de>>(
    limits: ArgumentLimits,
    empty: EmptyArgs,
) -> Result<T, ArgumentDecodeError> {
    limits.admit_len(crate::ic0::msg_arg_data_size())?;
    let bytes = crate::cdk::api::msg_arg_data();
    limits.decode(&bytes, empty)
}

// Generator-only coupling: registration does not select the service printer.
#[doc(hidden)]
pub use candid::candid_method as __candid_method;
