#![deny(unsafe_code)]
fn guard() -> Result<(), u8> { Ok(()) }
fn decode(_: &[u8]) -> Result<(), u8> { Ok(()) }
fn encode(_: ()) -> Result<Vec<u8>, u8> { Ok(vec![]) }
fn fail<T: std::fmt::Debug>(error: T) -> ! { panic!("{error:?}") }
#[candidate_runtime::query(runtime = candidate_runtime, guard = guard, decode = decode, max_bytes = 8, on_error = fail, name = "bad", manual_reply)]
async fn bad((): ()) {}
