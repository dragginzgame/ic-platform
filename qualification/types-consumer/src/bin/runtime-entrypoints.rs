#![forbid(unsafe_code)]

//! Manual update admission through the public Runtime boundary.

#[cfg(target_arch = "wasm32")]
mod runtime_entrypoints;

fn main() {
    #[cfg(target_arch = "wasm32")]
    runtime_entrypoints::check();
}
