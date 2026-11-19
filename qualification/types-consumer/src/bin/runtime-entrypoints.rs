#![forbid(unsafe_code)]

//! Manual update admission through the public Runtime boundary.

mod runtime_entrypoints;

fn main() {
    runtime_entrypoints::check();
}
