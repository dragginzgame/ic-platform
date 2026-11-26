//! Size and behavior witness driven by host-supplied IC argument bytes.

use candidate_runtime::entrypoints::EndpointError;
use candidate_types::Principal;
use std::convert::Infallible;

// principal_text
// Read a principal from runtime text and return its canonical text.
// limits : request up to 63 bytes
#[candidate_runtime::query(
    runtime = candidate_runtime,
    name = "principal_text",
    guard = admit,
    decode = decode,
    max_bytes = 63,
    on_error = fail,
    encode = encode
)]
fn principal_text(principal: Principal) -> String {
    principal.to_text()
}

fn admit() -> Result<(), Infallible> {
    Ok(())
}

fn decode(bytes: &[u8]) -> Result<Principal, ()> {
    Principal::from_text(std::str::from_utf8(bytes).map_err(|_| ())?).map_err(|_| ())
}

fn encode(text: String) -> Result<Vec<u8>, Infallible> {
    Ok(text.into_bytes())
}

fn fail(_: EndpointError<Infallible, (), Infallible>) -> ! {
    panic!("principal text refused")
}
