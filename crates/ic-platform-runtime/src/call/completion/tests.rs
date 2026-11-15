use super::{Completion, Outcome, RejectCode};
use std::num::NonZeroU32;

#[test]
fn reply_ownership_and_refund_presence_are_preserved() {
    for refund in [None, Some(0), Some(u128::MAX)] {
        let bytes = vec![0, 255, 128];
        let original = bytes.as_ptr();
        let value = Completion::from_reply(bytes, refund);
        assert_eq!(value.refunded_cycles(), refund);
        let Outcome::Reply(bytes) = value.outcome() else {
            panic!("expected reply")
        };
        assert_eq!(bytes.as_ptr(), original);
        assert_eq!(bytes, &[0, 255, 128]);
        let (Outcome::Reply(bytes), captured) = value.into_parts() else {
            panic!("expected reply")
        };
        assert_eq!(bytes.as_ptr(), original);
        assert_eq!(captured, refund);
    }
}

#[test]
fn rejection_catalogue_preserves_raw_values_and_uncertainty() {
    for (code, kind) in [
        (1, Some(RejectCode::SysFatal)),
        (2, Some(RejectCode::SysTransient)),
        (3, Some(RejectCode::DestinationInvalid)),
        (4, Some(RejectCode::CanisterReject)),
        (5, Some(RejectCode::CanisterError)),
        (6, Some(RejectCode::SysUnknown)),
        (7, None),
        (u32::MAX, None),
    ] {
        let message = String::from("  拒否\0\n ");
        let original = message.as_ptr();
        let value = Completion::from_rejection(NonZeroU32::new(code).unwrap(), message, Some(0));
        assert_eq!(
            matches!(value.outcome(), Outcome::OutcomeUnknown(_)),
            code == 6
        );
        let (Outcome::Rejected(rejection) | Outcome::OutcomeUnknown(rejection)) = value.outcome()
        else {
            panic!("unexpected reply")
        };
        assert_eq!(rejection.code(), code);
        assert_eq!(rejection.kind(), kind);
        if let Some(kind) = kind {
            assert_eq!(kind as u32, code);
        }
        assert_eq!(rejection.message(), "  拒否\0\n ");
        assert_eq!(rejection.message().as_ptr(), original);
    }
}

#[test]
fn empty_and_large_observations_are_not_decoded_or_truncated() {
    for bytes in [vec![], vec![0xff; 2 * 1024 * 1024]] {
        let length = bytes.len();
        let value = Completion::from_reply(bytes, None);
        let Outcome::Reply(bytes) = value.outcome() else {
            panic!("expected reply")
        };
        assert_eq!(bytes.len(), length);
    }
    let value = Completion::from_rejection(NonZeroU32::new(4).unwrap(), String::new(), None);
    let Outcome::Rejected(rejection) = value.outcome() else {
        panic!("expected rejection")
    };
    assert!(rejection.message().is_empty());
}
