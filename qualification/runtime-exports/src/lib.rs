//! Real generated IC update exports; only export attributes use the approved exception.

#[cfg(all(
    target_arch = "wasm32",
    not(any(feature = "raw", feature = "principal-text"))
))]
mod exports;
#[cfg(all(target_arch = "wasm32", feature = "principal-text"))]
mod principal_text;
#[cfg(all(target_arch = "wasm32", feature = "raw"))]
mod raw;
