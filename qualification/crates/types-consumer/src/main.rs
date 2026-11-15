//! Independent public identity consumer, also executed as standalone Wasm.

mod fixtures;

fn main() {
    fixtures::check();
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests;
