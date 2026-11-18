#![deny(unsafe_code)]
mod support;

fn fail(_: candidate_runtime::entrypoints::UpdateError<u8, u8>) {}

#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "bad",
    guard = support::guard,
    decode = support::decode,
    max_bytes = 8,
    on_error = fail,
    manual_reply
)]
async fn bad(_input: Vec<u8>) {}
