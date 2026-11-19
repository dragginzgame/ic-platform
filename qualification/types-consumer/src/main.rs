#![forbid(unsafe_code)]

//! Independent public identity consumer, also executed as standalone Wasm.

mod fixtures;
#[cfg(test)]
mod tests;

fn main() {
    fixtures::check();
}
