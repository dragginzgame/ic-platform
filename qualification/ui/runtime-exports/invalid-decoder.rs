#![deny(unsafe_code)]

mod support;

fn decode(_: u64) -> Result<Vec<u8>, u8> {
    Ok(vec![])
}

#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "bad",
    guard = support::guard,
    decode = decode,
    max_bytes = 8,
    on_error = support::fail,
    manual_reply
)]
async fn bad(_input: Vec<u8>) {}
