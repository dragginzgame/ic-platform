//! Exact upstream manager identities for IC Memory's infrastructure boundary.
//!
//! IC Memory retains allocation, checked reopening and exclusive-lease policy.
//! These exports do not authorize unmanaged application storage.

pub use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
