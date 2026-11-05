use super::*;

#[test]
fn explicit_limits_reject_invalid_and_overflowing_values() {
    assert_eq!(parse_limit(ORIGIN_RANGES_ENV, OsStr::new("0")), Ok(0));
    assert_eq!(
        parse_limit(ORIGIN_RANGES_ENV, OsStr::new("1048576")),
        Ok(1_048_576)
    );
    for value in ["", "-1", "1.5", "unlimited", "184467440737095516160"] {
        assert!(parse_limit(ORIGIN_RANGES_ENV, OsStr::new(value)).is_err());
    }
    assert!(trace_limit_bytes(0).is_err());
    assert!(trace_limit_bytes(usize::MAX).is_err());
    assert_eq!(trace_limit_bytes(256), Ok(256 * MIB));
}
