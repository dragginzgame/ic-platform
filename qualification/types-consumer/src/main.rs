#![forbid(unsafe_code)]

//! Independent public identity consumer, also executed as standalone Wasm.

#[cfg(all(test, not(target_family = "wasm")))]
mod tests;

mod fixtures;

fn main() {
    fixtures::check();
}
