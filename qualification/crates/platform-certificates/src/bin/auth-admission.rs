//! Execute the isolated Auth issuer-policy contract on native and Wasm.
#![forbid(unsafe_code)]

mod auth_admission;

fn main() {
    auth_admission::tests::run();
}
