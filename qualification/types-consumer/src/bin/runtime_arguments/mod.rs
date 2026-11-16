use candidate_runtime::{
    entrypoints::{AdmissionError, prepare},
    system::ArgumentTooLarge,
};
use std::cell::Cell;

pub(super) fn guarded() {
    let stage = Cell::new(0);
    let denied: Result<Vec<u8>, AdmissionError<u8, u8>> = prepare(
        usize::MAX,
        || {
            stage.set(stage.get() + 1);
            Err(7)
        },
        |_| panic!("denied guard must not decode"),
    );
    assert_eq!(denied, Err(AdmissionError::Guard(7)));
    assert_eq!(stage.get(), 1);

    let oversized: Result<Vec<u8>, AdmissionError<u8, u8>> = prepare(
        2,
        || {
            assert_eq!(stage.get(), 1);
            stage.set(2);
            Ok(())
        },
        |_| panic!("oversized arguments must not decode"),
    );
    assert_eq!(
        oversized,
        Err(AdmissionError::Arguments(ArgumentTooLarge {
            actual: 3,
            maximum: 2
        }))
    );
    assert_eq!(stage.get(), 2);

    let malformed: Result<Vec<u8>, AdmissionError<u8, u8>> = prepare(
        1,
        || {
            assert_eq!(stage.get(), 2);
            stage.set(3);
            Ok(())
        },
        |bytes| {
            assert_eq!(stage.get(), 3);
            assert_eq!(bytes, [2]);
            stage.set(4);
            Err(9)
        },
    );
    assert_eq!(malformed, Err(AdmissionError::Decode(9)));
    assert_eq!(stage.get(), 4);

    let accepted: Result<Vec<u8>, AdmissionError<u8, u8>> = prepare(
        2,
        || {
            assert_eq!(stage.get(), 4);
            stage.set(5);
            Ok(())
        },
        |bytes| {
            assert_eq!(stage.get(), 5);
            stage.set(6);
            Ok(bytes.to_vec())
        },
    );
    assert_eq!(accepted, Ok(vec![0, 255]));
    assert_eq!(stage.get(), 6);
}
