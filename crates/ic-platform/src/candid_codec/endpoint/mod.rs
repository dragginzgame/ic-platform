//! Explicit owned-codec adapters for pre-existing external Candid interfaces.
//!
//! Consumers select decode limits, reply bytes, argument arity and empty-envelope
//! policy. Guards run before byte admission and copying. The existing runtime
//! generator owns entrypoints and async execution. No schema printer or native
//! Candid method registration is selected; consumers retain external schemas.
use super::{
    arguments::DecodeArgs,
    decoder::{self, Arity, Limits},
};
pub use ic_platform_macros::{codec_query as query, codec_update as update};

/// Explicit admission of legacy requests with no envelope bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmptyArgs {
    /// Require an actual Candid argument envelope.
    Reject,
    /// Treat absent bytes as a zero-argument envelope, then apply the arity policy.
    Unit,
}
/// Admit the original scalar length before copying message data.
pub fn admit_len(actual: usize, limits: Limits) -> Result<(), decoder::Error> {
    if actual > limits.bytes {
        Err(decoder::Error::Limit)
    } else {
        Ok(())
    }
}
/// Decode using explicit owned-codec budgets; upstream work quotas are not interchangeable.
pub fn decode<'a, T: DecodeArgs<'a>>(
    bytes: &'a [u8],
    mut limits: Limits,
    empty: EmptyArgs,
    arity: Arity,
) -> Result<T, decoder::Error> {
    admit_len(bytes.len(), limits)?;
    let bytes = if bytes.is_empty() && empty == EmptyArgs::Unit {
        let empty = crate::candid_args::empty();
        // Synthetic framing does not count as received bytes; all other budgets remain.
        limits.bytes = limits.bytes.max(empty.len());
        empty
    } else {
        bytes
    };
    decoder::decode_args(bytes, limits, arity)
}
/// Admit and read an external argument sequence, without choosing caller authority.
pub fn read<T: for<'a> DecodeArgs<'a>>(
    limits: Limits,
    empty: EmptyArgs,
    arity: Arity,
) -> Result<T, decoder::Error> {
    admit_len(crate::ic0::msg_arg_data_size(), limits)?;
    let bytes = crate::cdk::api::msg_arg_data();
    decode(&bytes, limits, empty, arity)
}
#[cfg(test)]
mod tests;
