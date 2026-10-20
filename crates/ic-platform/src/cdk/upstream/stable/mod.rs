// Imported from ic-cdk 0.20.3; Apache-2.0. See ../NOTICE.md.
//! APIs to manage stable memory.
//!
//! You can check the [Internet Computer Specification](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-stable-memory)
//! for a in-depth explanation of stable memory.
use std::{error, fmt};

/// A possible error value when dealing with stable memory.
#[derive(Debug)]
pub enum StableMemoryError {
    /// No more stable memory could be allocated.
    OutOfMemory,
}

impl fmt::Display for StableMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfMemory => f.write_str("Out of memory"),
        }
    }
}

impl error::Error for StableMemoryError {}

/// Attempts to grow the stable memory by `new_pages` (added pages).
///
/// Returns an error if it wasn't possible. Otherwise, returns the previous
/// size that was reserved.
///
/// *Note*: Pages are 64KiB in WASM.
pub fn stable_grow(new_pages: u64) -> Result<u64, StableMemoryError> {
    match ic0::stable64_grow(new_pages) {
        u64::MAX => Err(StableMemoryError::OutOfMemory),
        pages => Ok(pages),
    }
}

/// Writes data to the stable memory location specified by an offset.
///
/// Warning - this will panic if `offset + buf.len()` exceeds the current size of stable memory.
/// Use `stable_grow` to request more stable memory if needed.
pub fn stable_write(offset: u64, buf: &[u8]) {
    ic0::stable64_write(buf, offset);
}

/// Reads data from the stable memory location specified by an offset.
pub fn stable_read(offset: u64, buf: &mut [u8]) {
    ic0::stable64_read(buf, offset);
}
