//! External Candid argument admission without a runtime or implicit empty request.
use super::{DecodeError, DecodeLimits, nesting::NestingError};
use candid::utils::ArgumentDecoder;

/// Caller-selected treatment of an absent argument envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmptyArgs {
    /// Require an actual Candid envelope, including for zero arguments.
    Reject,
    /// Admit missing bytes as the zero-argument envelope for a legacy interface.
    Unit,
}

/// Artifact-selected argument byte, work, type-table and header allowances.
/// Decoding also rejects wire nesting beyond 16 constructed levels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArgumentLimits {
    /// Maximum original envelope bytes, checked before reading message data.
    pub max_bytes: usize,
    /// Candid decoding work allowance.
    pub decoding_quota: usize,
    /// Candid work allowance for skipped fields and arguments.
    pub skipping_quota: usize,
    /// Maximum type-table entries.
    pub max_type_len: usize,
    /// Maximum header bytes.
    pub max_header_len: usize,
}

/// Argument admission or decoding failure, retaining the underlying typed cause.
#[derive(Debug, thiserror::Error)]
pub enum ArgumentDecodeError {
    /// Original argument envelope is too large.
    #[error("argument envelope is {actual} bytes; maximum is {maximum}")]
    TooLarge {
        /// Observed bytes.
        actual: usize,
        /// Artifact-selected allowance.
        maximum: usize,
    },
    /// Malformed, incompatible or over-budget Candid arguments.
    #[error("invalid or over-budget Candid arguments")]
    Candid(#[source] candid::Error),
}
impl ArgumentDecodeError {
    /// Recover the typed wire-admission refusal through the existing error variant.
    pub fn nesting_error(&self) -> Option<&NestingError> {
        match self {
            Self::Candid(candid::Error::Custom(error)) => error.downcast_ref(),
            _ => None,
        }
    }
}
impl ArgumentLimits {
    /// Check a scalar byte length before copying the message or decoding it.
    ///
    /// # Errors
    /// Refuses lengths above the artifact's selected allowance.
    pub fn admit_len(self, actual: usize) -> Result<(), ArgumentDecodeError> {
        if actual > self.max_bytes {
            return Err(ArgumentDecodeError::TooLarge {
                actual,
                maximum: self.max_bytes,
            });
        }
        Ok(())
    }
    /// Decode an admitted external argument tuple with an explicit empty policy.
    ///
    /// Zero values use `()`, one uses `(T,)`, and multiple use `(T, U, ...)`.
    /// Additional values retain Candid-compatible, work-bounded skipping.
    ///
    /// # Errors
    /// Refuses byte/work/header/type limits and malformed or incompatible values.
    pub fn decode<'a, T: ArgumentDecoder<'a>>(
        self,
        bytes: &'a [u8],
        empty: EmptyArgs,
    ) -> Result<T, ArgumentDecodeError> {
        self.admit_len(bytes.len())?;
        let bytes = if bytes.is_empty() && empty == EmptyArgs::Unit {
            crate::candid_args::empty().as_slice()
        } else {
            bytes
        };
        super::decode_args(
            bytes,
            DecodeLimits {
                reply_bytes: self.max_bytes.max(bytes.len()),
                decoding_quota: self.decoding_quota,
                skipping_quota: self.skipping_quota,
                type_entries: self.max_type_len,
                header_bytes: self.max_header_len,
            },
        )
        .map_err(|error| match error {
            DecodeError::ReplyTooLarge { actual, maximum } => {
                ArgumentDecodeError::TooLarge { actual, maximum }
            }
            DecodeError::Candid(error) => ArgumentDecodeError::Candid(error),
        })
    }
}
