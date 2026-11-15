use candidate_runtime::{
    call::{Call, CallFuture, Wait},
    executor::run_update,
};
use candidate_types::Principal;
use std::{
    cell::RefCell,
    future::{Future, poll_fn},
    pin::Pin,
    task::Poll,
};

thread_local! {
    static TRANSFER: RefCell<Option<CallFuture<'static, 'static>>> = const { RefCell::new(None) };
}

pub(super) fn step(phase: u32) {
    match phase {
        0 => {
            // Moving an unpolled future into its first task is valid.
            let mut future = Some(
                Call::new(Principal::anonymous(), "handoff", Wait::Unbounded)
                    .owned_args(vec![0, 128, 255])
                    .cycles(9)
                    .execute(),
            );
            drop(
                run_update(poll_fn(move |cx| {
                    let mut call = future.take().expect("original task must not resume");
                    assert!(Pin::new(&mut call).poll(cx).is_pending());
                    TRANSFER.with(|slot| *slot.borrow_mut() = Some(call));
                    Poll::<()>::Pending
                }))
                .unwrap(),
            );
        }
        1 => {
            let mut call = TRANSFER.with(|slot| slot.borrow_mut().take().unwrap());
            // A second root must not replace the first root's callback waker.
            drop(
                run_update(poll_fn(move |cx| {
                    let _ = Pin::new(&mut call).poll(cx);
                    Poll::<()>::Pending
                }))
                .unwrap(),
            );
        }
        _ => panic!("unexpected handoff phase"),
    }
}
