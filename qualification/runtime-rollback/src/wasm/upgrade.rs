//! Bounded peer handshake for observing a stop/drain before caller upgrade.

use candidate_runtime::{
    call::{Call, Outcome, Wait},
    entrypoints::UpdateError,
    system,
};
use candidate_types::Principal;
use std::cell::Cell;

// upgrade_probe
// Forward the held peer reply to the caller awaiting the upgrade probe.
// guard  : admit always succeeds
// limits : 59 input bytes maximum
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "upgrade_probe",
    guard = admit,
    decode = pair,
    max_bytes = 59,
    on_error = fail,
    manual_reply
)]
async fn upgrade_probe((peer, caller): (Principal, Principal)) {
    let completion = Call::new(peer, "hold_upgrade", Wait::Unbounded)
        .owned_args(caller.as_slice().to_vec())
        .execute()
        .await
        .unwrap();
    let Outcome::Reply(bytes) = completion.outcome() else {
        panic!("held peer must reply")
    };
    system::reply(bytes).unwrap();
}

// hold_upgrade
// Hold a remote call open until release or the fixture round-trip limit.
// guard  : admit always succeeds
// limits : 29 input bytes maximum; at most 128 round trips
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "hold_upgrade",
    guard = admit,
    decode = principal,
    max_bytes = 29,
    on_error = fail,
    manual_reply
)]
async fn hold_upgrade(caller: Principal) {
    STARTED.with(|value| value.set(true));
    // Cross-subnet round trips leave the driver an observable pending ingress.
    // The explicit bound prevents an accidentally unreleased infinite loop.
    for _ in 0..128 {
        if RELEASE.with(Cell::get) {
            break;
        }
        let completion = Call::new(caller, "probe", Wait::Unbounded)
            .owned_args(vec![0])
            .execute()
            .await
            .unwrap();
        // Stopping rejects new requests but still processes the original
        // caller's response. This peer remains independently releasable.
        assert!(matches!(
            completion.outcome(),
            Outcome::Reply(_) | Outcome::Rejected(_)
        ));
    }
    FINISHED.with(|value| value.set(true));
    system::reply(if RELEASE.with(Cell::get) {
        b"released"
    } else {
        b"limit"
    })
    .unwrap();
}

// release_upgrade
// Release the pending remote handler.
// guard  : admit always succeeds
// limits : empty input
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "release_upgrade",
    guard = admit,
    decode = empty,
    max_bytes = 0,
    on_error = fail,
    manual_reply
)]
async fn release_upgrade((): ()) {
    RELEASE.with(|value| value.set(true));
    system::reply(b"released").unwrap();
}

// upgrade_status
// Return the remote handler’s started and finished flags.
// guard  : admit always succeeds
// limits : empty input
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "upgrade_status",
    guard = admit,
    decode = empty,
    max_bytes = 0,
    on_error = fail,
    manual_reply
)]
async fn upgrade_status((): ()) {
    system::reply(&[
        u8::from(STARTED.with(Cell::get)),
        u8::from(FINISHED.with(Cell::get)),
    ])
    .unwrap();
}

thread_local! {
    static STARTED: Cell<bool> = const { Cell::new(false) };
    static RELEASE: Cell<bool> = const { Cell::new(false) };
    static FINISHED: Cell<bool> = const { Cell::new(false) };
}

/// Admit every caller to this qualification fixture.
fn admit() -> Result<(), u8> {
    Ok(())
}

fn principal(bytes: &[u8]) -> Result<Principal, u8> {
    if bytes.is_empty() {
        return Err(1);
    }
    Principal::try_from_slice(bytes).map_err(|_| 2)
}

fn pair(bytes: &[u8]) -> Result<(Principal, Principal), u8> {
    let (&len, bytes) = bytes.split_first().ok_or(1)?;
    let len = usize::from(len);
    if len > bytes.len() {
        return Err(2);
    }
    Ok((principal(&bytes[..len])?, principal(&bytes[len..])?))
}

fn empty(bytes: &[u8]) -> Result<(), u8> {
    if bytes.is_empty() { Ok(()) } else { Err(1) }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Generated failure handler owns the error"
)]
fn fail(error: UpdateError<u8, u8>) -> ! {
    panic!("upgrade admission: {error:?}")
}
