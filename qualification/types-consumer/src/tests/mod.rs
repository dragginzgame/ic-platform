#[cfg(not(target_family = "wasm"))]
mod dfinity;
#[cfg(not(target_family = "wasm"))]
mod published;

#[test]
fn literal_public_consumer() {
    super::fixtures::check();
}
