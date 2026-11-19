#![forbid(unsafe_code)]

//! Multi-entry Wasm transport fixture; host invokes callbacks between entries.

mod runtime_awaited;

fn main() {
    runtime_awaited::step();
}
