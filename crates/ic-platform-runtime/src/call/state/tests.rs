use super::{Completion, Shared, State, publish, replace_waker, retire};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    task::Wake,
};
use std::{
    sync::{Arc, Mutex},
    task::Waker,
};

struct Reentrant {
    state: Shared,
    events: Arc<AtomicUsize>,
}
impl Wake for Reentrant {
    fn wake(self: Arc<Self>) {
        assert!(self.state.try_lock().is_ok(), "wake under completion lock");
        self.events.fetch_add(10, Ordering::SeqCst);
    }
}
impl Drop for Reentrant {
    fn drop(&mut self) {
        assert!(self.state.try_lock().is_ok(), "drop under completion lock");
        self.events.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn production_transitions_release_lock_before_user_wake_and_drop() {
    for mode in 0..4 {
        let state = Arc::new(Mutex::new(State::Pending(None)));
        let events = Arc::new(AtomicUsize::new(0));
        let waker = Waker::from(Arc::new(Reentrant {
            state: Arc::clone(&state),
            events: Arc::clone(&events),
        }));
        replace_waker(&state, &waker);
        drop(waker);
        match mode {
            0 => replace_waker(&state, Waker::noop()),
            1 => publish(&state, Completion::from_reply(vec![42], Some(37))),
            2 => retire(&state, State::Abandoned),
            _ => retire(&state, State::Cleanup),
        }
        assert_eq!(
            events.load(Ordering::SeqCst),
            if mode == 1 { 11 } else { 1 }
        );
        if mode == 1 {
            let guard = state.lock().unwrap();
            let State::Complete(value) = &*guard else {
                panic!("missing completion")
            };
            assert_eq!(value, &Completion::from_reply(vec![42], Some(37)));
        }
        if mode == 2 {
            publish(&state, Completion::from_reply(vec![99], None));
            assert!(matches!(*state.lock().unwrap(), State::Abandoned));
        }
        retire(&state, State::Consumed);
    }
}
