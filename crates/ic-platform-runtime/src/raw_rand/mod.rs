//! One bounded management canister randomness request.

#[cfg(test)]
mod tests;

use crate::call::{Call, Completion, Outcome, Rejection, SubmitError, Wait};
use ic_platform_protocol::raw_rand as wire;
use ic_platform_types::Principal;

/// Failure of one request, preserving transport uncertainty and wire errors.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// The call could not be submitted.
    Submission(SubmitError),
    /// The call received a reject code and message.
    Rejected(Rejection),
    /// The IC stopped waiting without establishing the outcome.
    OutcomeUnknown(Rejection),
    /// The request or reply failed the canonical raw_rand wire profile.
    Frame(wire::Error),
}

/// Request exactly 32 bytes from the management canister with zero attached cycles.
/// The caller supplies the bounded wait in seconds and owns the resulting bytes.
///
/// # Errors
/// Returns the submission, rejection, uncertain outcome or protocol error from
/// this single attempt. No retry is performed.
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
