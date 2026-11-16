//! Execute retained independent Auth signature vectors on native and Wasm.
#![forbid(unsafe_code)]
mod crypto_admission;
fn main() {
    crypto_admission::tests::run();
}
