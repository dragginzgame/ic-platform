#![forbid(unsafe_code)]

//! Independent public identity consumer, also executed as standalone Wasm.

#[cfg(test)]
mod tests;

mod fixtures;

fn main() {
    fixtures::check();
}
