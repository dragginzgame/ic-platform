use candidate_runtime::{
    call::{Call, Outcome, Wait},
    executor::{EntryError, TaskHandle, TaskState, run_update},
};
use candidate_types::Principal;
use std::cell::{Cell, RefCell};

thread_local! {
    static HANDLE: RefCell<Option<TaskHandle>> = const { RefCell::new(None) };
    static DROPS: Cell<u32> = const { Cell::new(0) };
    static COMPLETIONS: Cell<u32> = const { Cell::new(0) };
}
struct DropCheck;
impl Drop for DropCheck {
    fn drop(&mut self) {
        // Cleanup and ordinary completion both keep nested-entry exclusion
        // through destruction, without borrowing the executor across it.
        assert!(matches!(run_update(async {}), Err(EntryError::NestedEntry)));
        DROPS.with(|drops| drops.set(drops.get() + 1));
    }
}
fn start(method: &'static str) -> TaskHandle {
    run_update(async move {
        let _drop = DropCheck;
        assert!(matches!(run_update(async {}), Err(EntryError::NestedEntry)));
        let completion = Call::new(Principal::anonymous(), method, Wait::Unbounded)
            .owned_args(vec![0, 128, 255])
            .cycles(9)
            .execute()
            .await
            .unwrap();
        assert_eq!(completion.outcome(), &Outcome::Reply(vec![42]));
        assert_eq!(completion.refunded_cycles(), Some(17));
        COMPLETIONS.with(|done| done.set(done.get() + 1));
    })
    .unwrap()
}
fn observe(expected: TaskState, drops: u32, completions: u32) {
    HANDLE.with(|handle| assert_eq!(handle.borrow().as_ref().unwrap().state(), expected));
    DROPS.with(|value| assert_eq!(value.get(), drops));
    COMPLETIONS.with(|value| assert_eq!(value.get(), completions));
}
pub(super) fn step(phase: u32) {
    match phase {
        0 => {
            let handle = start("exec-reply");
            HANDLE.with(|slot| *slot.borrow_mut() = Some(handle));
            observe(TaskState::Running, 0, 0);
        }
        1 => {
            observe(TaskState::Completed, 1, 1);
            let handle = start("exec-cleanup");
            HANDLE.with(|slot| *slot.borrow_mut() = Some(handle));
            observe(TaskState::Running, 1, 1);
        }
        2 => {
            observe(TaskState::TerminatedByCleanup, 2, 1);
            // The executor retains this future even after its handle is dropped.
            drop(start("exec-after"));
        }
        3 => {
            observe(TaskState::TerminatedByCleanup, 3, 2);
            assert!(matches!(
                run_update(async { panic!("non-replicated task admitted") }),
                Err(EntryError::NonReplicated)
            ));
        }
        _ => panic!("unexpected executor phase"),
    }
}
