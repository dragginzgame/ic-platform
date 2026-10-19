// Imported from ic-cdk 0.20.3; Apache-2.0. See ../NOTICE.md.
//! APIs to manage stable memory.
//!
//! You can check the [Internet Computer Specification](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-stable-memory)
//! for a in-depth explanation of stable memory.
// mod canister;
// #[cfg(test)]
// mod tests;

use std::{error, fmt};

/// WASM page size in bytes.
#[cfg(feature = "candid")]
pub const WASM_PAGE_SIZE_IN_BYTES: u64 = 64 * 1024; // 64KB

static CANISTER_STABLE_MEMORY: CanisterStableMemory = CanisterStableMemory {};

/// A trait defining the stable memory API which each canister running on the IC can make use of
pub trait StableMemory {
    /// Gets current size of the stable memory (in WASM pages).
    #[cfg(feature = "candid")]
    fn stable_size(&self) -> u64;

    /// Attempts to grow the stable memory by `new_pages` (added pages).
    ///
    /// Returns an error if it wasn't possible. Otherwise, returns the previous
    /// size that was reserved.
    ///
    /// *Note*: Pages are 64KiB in WASM.
    fn stable_grow(&self, new_pages: u64) -> Result<u64, StableMemoryError>;

    /// Writes data to the stable memory location specified by an offset.
    ///
    /// Warning - this will panic if `offset + buf.len()` exceeds the current size of stable memory.
    /// Use `stable_grow` to request more stable memory if needed.
    fn stable_write(&self, offset: u64, buf: &[u8]);

    /// Reads data from the stable memory location specified by an offset.
    fn stable_read(&self, offset: u64, buf: &mut [u8]);
}

/// A possible error value when dealing with stable memory.
#[derive(Debug)]
pub enum StableMemoryError {
    /// No more stable memory could be allocated.
    OutOfMemory,
    /// Attempted to read more stable memory than had been allocated.
    OutOfBounds,
}

impl fmt::Display for StableMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfMemory => f.write_str("Out of memory"),
            Self::OutOfBounds => f.write_str("Read exceeds allocated memory"),
        }
    }
}

impl error::Error for StableMemoryError {}

/// A standard implementation of [`StableMemory`].
///
/// Useful for creating legacy checkpoint adapters.
#[derive(Default, Debug, Copy, Clone)]
pub struct CanisterStableMemory {}

impl StableMemory for CanisterStableMemory {
    #[cfg(feature = "candid")]
    fn stable_size(&self) -> u64 {
        ic0::stable64_size()
    }

    fn stable_grow(&self, new_pages: u64) -> Result<u64, StableMemoryError> {
        match ic0::stable64_grow(new_pages) {
            u64::MAX => Err(StableMemoryError::OutOfMemory),
            x => Ok(x),
        }
    }

    fn stable_write(&self, offset: u64, buf: &[u8]) {
        ic0::stable64_write(buf, offset);
    }

    fn stable_read(&self, offset: u64, buf: &mut [u8]) {
        ic0::stable64_read(buf, offset);
    }
}

/// Gets current size of the stable memory (in WASM pages).
#[cfg(feature = "candid")]
pub fn stable_size() -> u64 {
    CANISTER_STABLE_MEMORY.stable_size()
}

/// Attempts to grow the stable memory by `new_pages` (added pages).
///
/// Returns an error if it wasn't possible. Otherwise, returns the previous
/// size that was reserved.
///
/// *Note*: Pages are 64KiB in WASM.
pub fn stable_grow(new_pages: u64) -> Result<u64, StableMemoryError> {
    CANISTER_STABLE_MEMORY.stable_grow(new_pages)
}

/// Writes data to the stable memory location specified by an offset.
///
/// Warning - this will panic if `offset + buf.len()` exceeds the current size of stable memory.
/// Use `stable_grow` to request more stable memory if needed.
pub fn stable_write(offset: u64, buf: &[u8]) {
    CANISTER_STABLE_MEMORY.stable_write(offset, buf);
}

/// Reads data from the stable memory location specified by an offset.
pub fn stable_read(offset: u64, buf: &mut [u8]) {
    CANISTER_STABLE_MEMORY.stable_read(offset, buf);
}

/// Returns a copy of the stable memory.
///
/// This will map the whole memory (even if not all of it has been written to).
///
/// # Panics
///
/// When the bytes of the stable memory cannot fit into a `Vec` which constrained by the usize.
#[cfg(feature = "candid")]
pub fn stable_bytes() -> Vec<u8> {
    let size = (stable_size() << 16)
        .try_into()
        .expect("overflow: stable memory too large to read in one go");
    let mut vec = Vec::with_capacity(size);
    ic0::stable64_read_uninit(&mut vec.spare_capacity_mut()[..size], 0);
    // SAFETY: ic0::stable64_read writes to all of `vec[0..size]`, so `set_len` is safe to call with the new size.
    unsafe {
        vec.set_len(size);
    }
    vec
}

#[cfg(feature = "candid")]
mod legacy {
    use super::*;
    use std::io;
    /// Performs generic IO (read, write, and seek) on stable memory.
    ///
    /// Warning: When using write functionality, this will overwrite any existing
    /// data in stable memory as it writes, so ensure you set the `offset` value
    /// accordingly if you wish to preserve existing data.
    ///
    /// Will attempt to grow the memory as it writes,
    /// and keep offsets and total capacity.
    #[derive(Debug)]
    pub struct StableIO<M: StableMemory = CanisterStableMemory> {
        /// The offset of the next write.
        offset: u64,

        /// The capacity, in pages.
        capacity: u64,

        /// The stable memory to write data to.
        memory: M,
    }

    impl Default for StableIO {
        fn default() -> Self {
            Self::with_memory(CanisterStableMemory::default(), 0)
        }
    }

    impl<M: StableMemory> StableIO<M> {
        /// Creates a new `StableIO` which writes to the selected memory
        pub fn with_memory(memory: M, offset: u64) -> Self {
            let capacity = memory.stable_size();
            Self {
                offset,
                capacity,
                memory,
            }
        }

        /// Attempts to grow the memory by adding new pages.
        pub fn grow(&mut self, new_pages: u64) -> Result<(), StableMemoryError> {
            let old_page_count = self.memory.stable_grow(new_pages)?;
            self.capacity = old_page_count + new_pages;
            Ok(())
        }

        /// Writes a byte slice to the buffer.
        ///
        /// # Errors
        ///
        /// When it cannot grow the memory to accommodate the new data.
        pub fn write(&mut self, buf: &[u8]) -> Result<usize, StableMemoryError> {
            let required_capacity_bytes = self.offset + buf.len() as u64;
            let required_capacity_pages = required_capacity_bytes.div_ceil(WASM_PAGE_SIZE_IN_BYTES);
            let current_pages = self.capacity;
            let additional_pages_required = required_capacity_pages.saturating_sub(current_pages);

            if additional_pages_required > 0 {
                self.grow(additional_pages_required)?;
            }

            self.memory.stable_write(self.offset, buf);
            self.offset += buf.len() as u64;
            Ok(buf.len())
        }
    }

    impl<M: StableMemory> io::Write for StableIO<M> {
        fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
            self.write(buf)
                .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))
        }

        fn flush(&mut self) -> Result<(), io::Error> {
            // Noop.
            Ok(())
        }
    }

    // impl_stable_io!(u32);
    // impl_stable_io!(u64);

    /// A writer to the stable memory.
    ///
    /// Warning: This will overwrite any existing data in stable memory as it writes, so ensure you set
    /// the `offset` value accordingly if you wish to preserve existing data.
    ///
    /// Will attempt to grow the memory as it writes,
    /// and keep offsets and total capacity.
    #[derive(Debug)]
    pub struct StableWriter<M: StableMemory = CanisterStableMemory>(StableIO<M>);

    #[allow(clippy::derivable_impls)]
    impl Default for StableWriter {
        #[inline]
        fn default() -> Self {
            Self(StableIO::default())
        }
    }

    impl<M: StableMemory> io::Write for StableWriter<M> {
        #[inline]
        fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
            io::Write::write(&mut self.0, buf)
        }

        #[inline]
        fn flush(&mut self) -> Result<(), io::Error> {
            io::Write::flush(&mut self.0)
        }
    }
}
#[cfg(feature = "candid")]
pub(super) use legacy::StableWriter;
