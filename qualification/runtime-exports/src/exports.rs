use candidate_runtime::{
    entrypoints::{Inspection, InspectionDecision},
    system,
};

// inspect
// Admit small echo ingress from a canister controller.
// limits : method name up to 4 bytes, request at most 4 bytes
#[candidate_runtime::inspect_message(runtime = candidate_runtime, max_method_bytes = 4)]
fn inspect(message: Inspection<'_>) -> InspectionDecision {
    if message.method_name() == "echo"
        && message.argument_size() <= 4
        && system::is_controller(&message.caller())
    {
        InspectionDecision::Accept
    } else {
        InspectionDecision::Reject
    }
}

// echo
// Reply with the admitted bytes.
// guard  : admit accepts every caller
// limits : request up to 4 bytes, reply echoes the admitted bytes
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "echo",
    guard = admit,
    decode = decode,
    max_bytes = 4,
    on_error = fail,
    manual_reply
)]
async fn echo(input: Vec<u8>) {
    system::reply(&input).unwrap();
}

// probe
// Echo bytes on the inter-canister lane.
// limits : request up to 4 bytes, reply echoes the admitted bytes
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "probe",
    guard = admit,
    decode = decode,
    max_bytes = 4,
    on_error = fail,
    encode = Ok::<Vec<u8>, u8>
)]
fn probe(input: Vec<u8>) -> Vec<u8> {
    input
}

// denied
// Exercise unconditional refusal before handler execution.
// guard  : deny refuses all callers
// limits : request up to 4 bytes, no reply
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "denied",
    guard = deny,
    decode = decode,
    max_bytes = 4,
    on_error = fail,
    manual_reply
)]
async fn denied(_input: Vec<u8>) {
    panic!("guard must refuse before handler")
}

// disabled
// Exercise removal of an export disabled by cfg.
// limits : request up to 4 bytes, no exported reply
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "disabled",
    guard = missing,
    decode = missing,
    max_bytes = 4,
    on_error = missing,
    manual_reply
)]
#[cfg(any())]
async fn disabled(_input: Missing) {
    missing();
}

// conditional
// Exercise removal of an export disabled by cfg_attr.
// limits : request up to 4 bytes, no exported reply
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "conditional",
    guard = missing,
    decode = missing,
    max_bytes = 4,
    on_error = missing,
    manual_reply
)]
#[cfg_attr(all(), cfg(any()))]
async fn conditional(_input: Missing) {
    missing();
}

/// Admit every caller to this qualification fixture.
fn admit() -> Result<(), u8> {
    Ok(())
}

/// Refuse every caller before decoding.
fn deny() -> Result<(), u8> {
    Err(7)
}

fn decode(bytes: &[u8]) -> Result<Vec<u8>, u8> {
    if bytes.first() == Some(&255) {
        Err(9)
    } else {
        Ok(bytes.to_vec())
    }
}

fn fail(error: impl core::fmt::Debug) -> ! {
    panic!("admission refused: {error:?}")
}
