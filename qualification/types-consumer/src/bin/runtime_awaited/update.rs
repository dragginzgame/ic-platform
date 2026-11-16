use candidate_runtime::{
    call::{Call, Outcome, Wait},
    entrypoints::update_manual,
    executor::{TaskHandle, TaskState},
    system::{ReplyError, reply},
};
use candidate_types::Principal;
use std::cell::{Cell, RefCell};

thread_local! {
    static HANDLE: RefCell<Option<TaskHandle>> = const { RefCell::new(None) };
    static GUARDS: Cell<u32> = const { Cell::new(0) };
    static DECODES: Cell<u32> = const { Cell::new(0) };
    static HANDLERS: Cell<u32> = const { Cell::new(0) };
    static COMPLETIONS: Cell<u32> = const { Cell::new(0) };
    static DROPS: Cell<u32> = const { Cell::new(0) };
}

struct ObserveDrop(ReplyError);
impl Drop for ObserveDrop {
    fn drop(&mut self) {
        assert_eq!(reply(b"destructor"), Err(self.0));
        DROPS.with(|value| value.set(value.get() + 1));
    }
}

fn start(method: &'static str) {
    let handle = update_manual(
        3,
        || {
            GUARDS.with(|value| value.set(value.get() + 1));
            Ok::<_, u8>(())
        },
        |bytes| {
            DECODES.with(|value| value.set(value.get() + 1));
            assert_eq!(bytes, [0, 128, 255]);
            Ok::<_, u8>(bytes.to_vec())
        },
        move |input| {
            HANDLERS.with(|value| value.set(value.get() + 1));
            async move {
                let mut drop_check = ObserveDrop(ReplyError::NoActiveTask);
                let response = Call::new(Principal::anonymous(), method, Wait::Unbounded)
                    .owned_args(input.clone())
                    .cycles(9)
                    .execute()
                    .await
                    .unwrap();
                // Reply bytes are deliberately different from the admitted input.
                assert_eq!(input, [0, 128, 255]);
                assert_eq!(response.outcome(), &Outcome::Reply(vec![77]));
                assert_eq!(response.refunded_cycles(), Some(700));
                assert_eq!(reply(&input), Ok(()));
                drop_check.0 = ReplyError::AlreadyReplied;
                assert_eq!(reply(b"duplicate"), Err(ReplyError::AlreadyReplied));
                COMPLETIONS.with(|value| value.set(value.get() + 1));
            }
        },
    )
    .unwrap();
    assert_eq!(handle.state(), TaskState::Running);
    HANDLE.with(|slot| *slot.borrow_mut() = Some(handle));
}

fn observe(state: TaskState, admitted: u32, completed: u32, dropped: u32) {
    HANDLE.with(|slot| assert_eq!(slot.borrow().as_ref().unwrap().state(), state));
    GUARDS.with(|value| assert_eq!(value.get(), admitted));
    DECODES.with(|value| assert_eq!(value.get(), admitted));
    HANDLERS.with(|value| assert_eq!(value.get(), admitted));
    COMPLETIONS.with(|value| assert_eq!(value.get(), completed));
    DROPS.with(|value| assert_eq!(value.get(), dropped));
}

pub(super) fn step(phase: u32) {
    match phase {
        0 => {
            start("update-reply");
            observe(TaskState::Running, 1, 0, 0);
        }
        1 => {
            observe(TaskState::Completed, 1, 1, 1);
            start("update-cleanup");
            observe(TaskState::Running, 2, 1, 1);
        }
        2 => {
            observe(TaskState::TerminatedByCleanup, 2, 1, 2);
            start("update-after");
            observe(TaskState::Running, 3, 1, 2);
        }
        3 => observe(TaskState::Completed, 3, 2, 3),
        _ => panic!("unexpected update phase"),
    }
}
