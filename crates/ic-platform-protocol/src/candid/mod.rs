//! Resource bounds and failures for the selected binary Candid projections.
//!
//! This is not a general-purpose Candid API. Decoding authenticates nothing.

#[cfg(feature = "cloud-engine-marketplace")]
mod nat;
mod wire;
#[cfg(feature = "cloud-engine-marketplace")]
pub use nat::Nat;
pub(crate) use wire::{Decode, Reader, Schema};

/// Explicit budgets for one complete response; none has an implicit default.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Complete input size, including its type table.
    pub bytes: usize,
    /// Maximum end offset of the header and argument type list.
    pub header: usize,
    /// Number of constructed type-table entries.
    pub types: usize,
    /// Cumulative byte reads and traversal operations.
    pub work: usize,
    /// Cumulative logical storage, including type-table storage and retained data.
    /// This is not a bound on allocator overhead or total process memory.
    pub allocation: usize,
    /// Maximum elements in each value vector, including discarded vectors.
    pub vector: usize,
    /// Cumulative work on discarded values, also charged against `work`.
    pub skipping: usize,
}

/// A closed decoding refusal; malformed values never silently become absent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Invalid bytes, labels, references or a truncated value.
    Malformed,
    /// Unsupported wire constructor or opaque reference.
    Unsupported,
    /// A selected value has an incompatible type or a required field is absent.
    Type,
    /// The message does not contain exactly one argument.
    Arity,
    /// Bytes follow the complete value.
    Trailing,
    /// A caller budget or the fixed 16-level constructed-wire ceiling is exceeded.
    Limit,
    /// The allocator refused admitted storage.
    Allocation,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Malformed => "malformed Candid message",
            Self::Unsupported => "unsupported Candid representation",
            Self::Type => "incompatible Candid projection",
            Self::Arity => "expected exactly one Candid argument",
            Self::Trailing => "trailing Candid bytes",
            Self::Limit => "Candid decoding limit exceeded",
            Self::Allocation => "Candid allocation failed",
        })
    }
}
impl std::error::Error for Error {}

pub(crate) fn decode<T: Decode>(input: &[u8], limits: Limits) -> Result<T, Error> {
    wire::decode(input, limits)
}
