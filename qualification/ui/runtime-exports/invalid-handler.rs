#![deny(unsafe_code)]
mod support;

#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "bad",
    guard = support::guard,
    decode = support::decode,
    max_bytes = 8,
    on_error = support::fail,
    manual_reply
)]
fn bad(_input: Vec<u8>) {}
