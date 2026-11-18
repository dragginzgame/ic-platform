#[cfg(test)]
mod tests;

use super::Completion;
use std::{
    sync::{Arc, Mutex},
    task::Waker,
};

pub(super) enum State {
    Pending(Option<Waker>),
    Complete(Completion),
    Abandoned,
    Cleanup,
    Consumed,
}
pub(super) type Shared = Arc<Mutex<State>>;

pub(super) fn replace_waker(shared: &Shared, waker: &Waker) {
    // Even cloning or dropping a waker can execute arbitrary user code.
    let mut replacement = Some(waker.clone());
    let previous = {
        let mut state = shared.lock().unwrap();
        match &mut *state {
            State::Pending(old) => std::mem::replace(old, replacement.take()),
            _ => None,
        }
    };
    drop(previous);
    drop(replacement);
}

pub(super) fn retire(shared: &Shared, replacement: State) {
    let previous = {
        let mut state = shared.lock().unwrap();
        std::mem::replace(&mut *state, replacement)
    };
    drop(previous);
}

pub(super) fn publish(shared: &Shared, completion: Completion) {
    let mut completion = Some(completion);
    let previous = {
        let mut state = shared.lock().unwrap();
        if matches!(*state, State::Pending(_)) {
            Some(std::mem::replace(
                &mut *state,
                State::Complete(completion.take().unwrap()),
            ))
        } else {
            assert!(matches!(*state, State::Abandoned));
            None
        }
    };
    if let Some(State::Pending(Some(waker))) = previous {
        waker.wake();
    }
    drop(completion);
}
