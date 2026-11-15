use candidate_runtime::{
    call::{Call, CallFuture, Outcome, Wait},
    executor::{TaskHandle, TaskState, run_update},
};
use candidate_types::Principal;
use std::{
    cell::{Cell, RefCell},
    future::{Future, poll_fn},
    pin::Pin,
    task::Poll,
};

thread_local! {
    static HANDLES: RefCell<Vec<TaskHandle>> = const { RefCell::new(Vec::new()) };
    static A_POLLS: Cell<u32> = const { Cell::new(0) };
    static PAIR_POLLS: Cell<u32> = const { Cell::new(0) };
    static PAIR_DROPS: Cell<u32> = const { Cell::new(0) };
}

fn call(method: &'static str) -> CallFuture<'static, 'static> {
    Call::new(Principal::anonymous(), method, Wait::Unbounded)
        .owned_args(vec![0, 128, 255])
        .cycles(9)
        .execute()
}

async fn receive(method: &'static str, byte: u8, refund: u128) {
    let completion = call(method).await.unwrap();
    assert_eq!(completion.outcome(), &Outcome::Reply(vec![byte]));
    assert_eq!(completion.refunded_cycles(), Some(refund));
}

fn retain(handle: TaskHandle) {
    HANDLES.with(|handles| handles.borrow_mut().push(handle));
}

fn states(expected: &[TaskState]) {
    HANDLES.with(|handles| {
        let actual: Vec<_> = handles.borrow().iter().map(TaskHandle::state).collect();
        assert_eq!(actual, expected);
    });
}

struct PairDrop;
impl Drop for PairDrop {
    fn drop(&mut self) {
        PAIR_DROPS.with(|value| value.set(value.get() + 1));
    }
}

pub(super) fn step(phase: u32) {
    use TaskState::{Completed, Running, TerminatedByCleanup};
    match phase {
        0 => {
            let mut a = Box::pin(async {
                receive("a-first", 10, 101).await;
                // Submission from the resumed callback must retain the same task.
                receive("a-second", 11, 102).await;
            });
            retain(
                run_update(poll_fn(move |cx| {
                    A_POLLS.with(|value| value.set(value.get() + 1));
                    a.as_mut().poll(cx)
                }))
                .unwrap(),
            );
            retain(run_update(receive("b", 20, 201)).unwrap());
            states(&[Running, Running]);
        }
        1 => {
            states(&[Running, Completed]);
            A_POLLS.with(|value| assert_eq!(value.get(), 1));
        }
        2 => {
            states(&[Running, Completed]);
            A_POLLS.with(|value| assert_eq!(value.get(), 2));
        }
        3 => {
            states(&[Completed, Completed]);
            A_POLLS.with(|value| assert_eq!(value.get(), 3));
            retain(
                run_update(async {
                    let _drop = PairDrop;
                    let mut left = call("pair-left");
                    let mut right = call("pair-right");
                    poll_fn(|cx| {
                        PAIR_POLLS.with(|value| value.set(value.get() + 1));
                        assert!(Pin::new(&mut left).poll(cx).is_pending());
                        assert!(Pin::new(&mut right).poll(cx).is_pending());
                        Poll::<()>::Pending
                    })
                    .await;
                    panic!("cleanup must not resume the task");
                })
                .unwrap(),
            );
            retain(run_update(receive("survivor", 30, 301)).unwrap());
            states(&[Completed, Completed, Running, Running]);
        }
        4 | 5 => {
            states(&[Completed, Completed, TerminatedByCleanup, Running]);
            PAIR_POLLS.with(|value| assert_eq!(value.get(), 1));
            PAIR_DROPS.with(|value| assert_eq!(value.get(), 1));
        }
        6 => {
            states(&[Completed, Completed, TerminatedByCleanup, Completed]);
            PAIR_POLLS.with(|value| assert_eq!(value.get(), 1));
            PAIR_DROPS.with(|value| assert_eq!(value.get(), 1));
        }
        _ => panic!("unexpected interleaving phase"),
    }
}
