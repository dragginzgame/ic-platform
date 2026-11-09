use super::*;
use crate::candid_codec::arguments::encode_args;
fn limits() -> Limits {
    Limits {
        bytes: 1024,
        header: 512,
        types: 32,
        work: 10000,
        allocation: 10000,
    }
}
#[test]
fn empty_policy_never_chooses_argument_arity() {
    assert!(decode::<()>(&[], limits(), EmptyArgs::Reject, Arity::Exact).is_err());
    assert_eq!(
        decode::<()>(&[], limits(), EmptyArgs::Unit, Arity::Exact),
        Ok(())
    );
    assert_eq!(
        decode::<(Option<u64>,)>(&[], limits(), EmptyArgs::Unit, Arity::Exact),
        Err(decoder::Error::Arity)
    );
    assert_eq!(
        decode::<(Option<u64>,)>(&[], limits(), EmptyArgs::Unit, Arity::Compatible),
        Ok((None,))
    );
    assert!(decode::<(u64,)>(&[], limits(), EmptyArgs::Unit, Arity::Compatible).is_err());
}
#[test]
fn input_limits_and_trailing_values_remain_enforced() {
    let bytes = encode_args(&(42u64, 99u64), 1024).unwrap();
    assert_eq!(
        decode::<(u64,)>(&bytes, limits(), EmptyArgs::Reject, Arity::Exact),
        Err(decoder::Error::Arity)
    );
    assert_eq!(
        decode::<(u64,)>(&bytes, limits(), EmptyArgs::Reject, Arity::Compatible),
        Ok((42,))
    );
    for n in 0..bytes.len() {
        assert!(
            decode::<(u64,)>(&bytes[..n], limits(), EmptyArgs::Reject, Arity::Compatible).is_err()
        );
    }
    assert_eq!(admit_len(1025, limits()), Err(decoder::Error::Limit));
    assert!(
        decode::<()>(
            &[],
            Limits {
                work: 0,
                ..limits()
            },
            EmptyArgs::Unit,
            Arity::Exact
        )
        .is_err()
    );
    assert!(
        decode::<()>(
            &[],
            Limits {
                header: 0,
                ..limits()
            },
            EmptyArgs::Unit,
            Arity::Exact
        )
        .is_err()
    );
}
