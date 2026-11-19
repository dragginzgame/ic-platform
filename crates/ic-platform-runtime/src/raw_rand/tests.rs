use super::*;
use std::num::NonZeroU32;

#[test]
fn request_preserves_bound_and_attaches_no_cycles() {
    for seconds in [0, 10, 300, u32::MAX] {
        let call = prepare(seconds).unwrap();
        assert_eq!(call.target(), Principal::management_canister());
        assert_eq!(call.method(), "raw_rand");
        assert_eq!(call.raw_args(), b"DIDL\0\0");
        assert_eq!(call.wait(), Wait::Bounded { seconds });
        assert_eq!(call.attached_cycles(), 0);
    }
}

#[test]
fn only_a_complete_valid_reply_yields_bytes() {
    let mut reply = b"DIDL\x01\x6d\x7b\x01\x00\x20".to_vec();
    let seed = std::array::from_fn(|i| i as u8);
    reply.extend_from_slice(&seed);
    let decode = |bytes| finish(Ok(Completion::from_reply(bytes, None)));
    assert_eq!(decode(reply.clone()), Ok(seed));
    assert_eq!(
        decode(reply[..41].to_vec()),
        Err(Error::Frame(wire::Error::Length))
    );
    reply[0] = 0;
    assert_eq!(
        decode(reply.clone()),
        Err(Error::Frame(wire::Error::Framing))
    );
    reply.push(0);
    assert_eq!(decode(reply), Err(Error::Frame(wire::Error::ByteLimit)));
}

#[test]
fn transport_failures_preserve_the_original_observation() {
    assert_eq!(
        finish(Err(SubmitError::CallbackCapacity)),
        Err(Error::Submission(SubmitError::CallbackCapacity))
    );
    for code in [4, 6, 99] {
        let result =
            Completion::from_rejection(NonZeroU32::new(code).unwrap(), "message".into(), None);
        let rejection = match (code, finish(Ok(result))) {
            (6, Err(Error::OutcomeUnknown(r))) | (4 | 99, Err(Error::Rejected(r))) => r,
            other => panic!("unexpected transport observation: {other:?}"),
        };
        assert_eq!(rejection.code(), code);
        assert_eq!(rejection.message(), "message");
    }
}
