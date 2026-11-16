//! Installed update rollback probe using only the candidate Runtime.

#[cfg(target_arch = "wasm32")]
mod exports;

#[cfg(target_arch = "wasm32")]
mod callbacks;

#[cfg(target_arch = "wasm32")]
mod upgrade;

#[cfg(target_arch = "wasm32")]
mod identity;
