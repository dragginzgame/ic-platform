//! Exact upstream substrate types used by governed IC Storage adapters.
//!
//! These exports retain upstream initialization, failure and persisted-byte
//! contracts. They grant no allocation authority: production callers use IC
//! Storage's governed constructors and IC Memory's committed runtime handles.
//! Default backing and restricted views support those owners and isolated tests;
//! exposing them does not authorize unmanaged application storage.

pub use ic_stable_structures::{
    BTreeMap, BTreeSet, Cell, DefaultMemoryImpl, Log, Memory, MinHeap, RestrictedMemory, Storable,
    Vec, VectorMemory,
};

pub mod btreemap;
pub mod btreeset;
pub mod log;
pub mod memory_manager;
pub mod storable;

#[cfg(test)]
mod tests;
