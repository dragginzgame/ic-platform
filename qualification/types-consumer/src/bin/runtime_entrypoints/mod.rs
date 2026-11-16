use candidate_runtime::{
    entrypoints::{AdmissionError, UpdateError, update_manual},
    executor::{EntryError, TaskState, run_update},
    system::{ArgumentTooLarge, ReplyError, reply},
};
use std::{cell::Cell, future::Ready, rc::Rc};

fn never(_: Vec<u8>) -> Ready<()> {
    panic!("refused input must not invoke the handler")
}

pub(super) fn check() {
    assert_eq!(reply(&[]), Err(ReplyError::NoActiveTask));
    let denied = update_manual(
        100,
        || Err(7_u8),
        |_| -> Result<Vec<u8>, u8> { panic!("guard refused") },
        never,
    );
    assert!(matches!(
        denied,
        Err(UpdateError::Admission(AdmissionError::Guard(7)))
    ));

    let oversized = update_manual(
        2,
        || Ok::<_, u8>(()),
        |_| -> Result<Vec<u8>, u8> { panic!("size refused") },
        never,
    );
    assert!(matches!(
        oversized,
        Err(UpdateError::Admission(AdmissionError::Arguments(
            ArgumentTooLarge {
                actual: 3,
                maximum: 2
            }
        )))
    ));

    let malformed = update_manual(
        1,
        || Ok::<_, u8>(()),
        |bytes| -> Result<Vec<u8>, u8> {
            assert_eq!(bytes, [9]);
            Err(8)
        },
        never,
    );
    assert!(matches!(
        malformed,
        Err(UpdateError::Admission(AdmissionError::Decode(8)))
    ));

    let invoked = Rc::new(Cell::new(0));
    let observed = Rc::clone(&invoked);
    let accepted = update_manual(
        2,
        || Ok::<_, u8>(()),
        |bytes| Ok::<_, u8>(bytes.to_vec()),
        move |bytes| {
            observed.set(observed.get() + 1);
            assert_eq!(bytes, [0, 255]);
            // Even handler construction, not just its future's poll, is task-owned.
            assert!(matches!(run_update(async {}), Err(EntryError::NestedEntry)));
            async move {
                assert_eq!(reply(&bytes), Ok(()));
                assert_eq!(reply(b"duplicate"), Err(ReplyError::AlreadyReplied));
            }
        },
    )
    .unwrap();
    assert_eq!(invoked.get(), 1);
    assert_eq!(accepted.state(), TaskState::Completed);

    let outer = run_update(async {
        let nested = update_manual(
            0,
            || -> Result<(), u8> { panic!("nested guard must not run") },
            |_| Ok::<_, u8>(vec![]),
            never,
        );
        assert!(matches!(
            nested,
            Err(UpdateError::Entry(EntryError::NestedEntry))
        ));
        assert_eq!(reply(&[]), Ok(()));
    })
    .unwrap();
    assert_eq!(outer.state(), TaskState::Completed);

    let non_replicated = update_manual(
        0,
        || -> Result<(), u8> { panic!("non-replicated guard must not run") },
        |_| Ok::<_, u8>(vec![]),
        never,
    );
    assert!(matches!(
        non_replicated,
        Err(UpdateError::Entry(EntryError::NonReplicated))
    ));
}
