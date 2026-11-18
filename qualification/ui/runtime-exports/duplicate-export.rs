#![deny(unsafe_code)]
mod support;

#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "same",
    guard = support::guard,
    decode = support::decode,
    max_bytes = 8,
    on_error = support::fail,
    manual_reply
)]
async fn first(_input: Vec<u8>) {}

#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "same",
    guard = support::guard,
    decode = support::decode,
    max_bytes = 8,
    on_error = support::fail,
    manual_reply
)]
async fn second(_input: Vec<u8>) {}
