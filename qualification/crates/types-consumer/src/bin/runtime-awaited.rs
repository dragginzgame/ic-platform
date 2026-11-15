//! Multi-entry Wasm transport fixture; host invokes callbacks between entries.

#[cfg(target_arch = "wasm32")]
mod runtime_awaited;

fn main() {
    #[cfg(target_arch = "wasm32")]
    runtime_awaited::step();
}
