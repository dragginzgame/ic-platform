//! Host-only limits for instruction-profile dictionaries and retained traces.

use std::ffi::OsStr;

pub const ORIGIN_RANGES_ENV: &str = "POCKET_IC_INSTRUCTION_PROFILE_MAX_ORIGIN_RANGES";
pub const TRACE_LIMIT_MIB_ENV: &str = "POCKET_IC_MEMORY_ACCESS_TRACE_LIMIT_MIB";
pub const DEFAULT_TRACE_LIMIT_MIB: usize = 64;
const MIB: usize = 1024 * 1024;

fn parse_limit(name: &str, value: &OsStr) -> Result<usize, String> {
    value
        .to_str()
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or_else(|| format!("{name} must be a nonnegative integer that fits usize"))
}

/// Maximum final instrumented origin ranges retained per compiled module.
pub fn origin_range_limit() -> Result<usize, String> {
    std::env::var_os(ORIGIN_RANGES_ENV).map_or(
        Ok(ic_config::embedders::DEFAULT_INSTRUCTION_PROFILE_MAX_ORIGIN_RANGES),
        |value| parse_limit(ORIGIN_RANGES_ENV, &value),
    )
}

fn trace_limit_bytes(mib: usize) -> Result<usize, String> {
    if mib == 0 {
        return Err(format!("{TRACE_LIMIT_MIB_ENV} must be at least 1 MiB"));
    }
    mib.checked_mul(MIB)
        .ok_or_else(|| format!("{TRACE_LIMIT_MIB_ENV} exceeds the host address space"))
}

/// Conservative retained-data budget per PocketIC instance, not an RSS bound.
pub fn trace_memory_limit() -> Result<usize, String> {
    let mib = std::env::var_os(TRACE_LIMIT_MIB_ENV)
        .map_or(Ok(DEFAULT_TRACE_LIMIT_MIB), |value| {
            parse_limit(TRACE_LIMIT_MIB_ENV, &value)
        })?;
    trace_limit_bytes(mib)
}

#[cfg(test)]
mod tests {
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
}
