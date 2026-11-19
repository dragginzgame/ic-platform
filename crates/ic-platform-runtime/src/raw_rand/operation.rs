use super::{Error, wire};
use crate::call::{Call, Completion, Outcome, SubmitError, Wait};
use candidate_types::Principal;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

/// Request exactly 32 bytes from the management canister with zero attached cycles.
/// The caller supplies the bounded wait in seconds and owns the resulting bytes.
///
/// # Errors
/// Returns the submission, rejection, uncertain outcome or protocol error from
/// this single attempt. No retry is performed.
#[cfg(target_arch = "wasm32")]
pub async fn request(wait_seconds: u32) -> Result<[u8; 32], Error> {
    let call = prepare(wait_seconds)?;
    finish(call.execute().await)
}

fn prepare(wait_seconds: u32) -> Result<Call<'static, 'static>, Error> {
    Ok(Call::new(
        Principal::management_canister(),
        wire::METHOD,
        Wait::Bounded {
            seconds: wait_seconds,
        },
    )
    .args(wire::encode(6).map_err(Error::Frame)?))
}

fn finish(result: Result<Completion, SubmitError>) -> Result<[u8; 32], Error> {
    match result.map_err(Error::Submission)?.into_parts().0 {
        Outcome::Reply(bytes) => wire::decode(&bytes, 42).map_err(Error::Frame),
        Outcome::Rejected(error) => Err(Error::Rejected(error)),
        Outcome::OutcomeUnknown(error) => Err(Error::OutcomeUnknown(error)),
    }
}
