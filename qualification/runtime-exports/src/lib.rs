//! Real generated IC update exports; only export attributes use the approved exception.

#[cfg(all(target_arch = "wasm32", not(feature = "raw")))]
mod exports;
#[cfg(all(target_arch = "wasm32", feature = "raw"))]
mod raw;
