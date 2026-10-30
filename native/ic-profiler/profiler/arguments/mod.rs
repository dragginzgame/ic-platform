//! Native request primitives; each surface retains its admission order and diagnostics.

use serde_json::Value;

pub(crate) const DEFAULT_PROFILE_EXECUTIONS: u64 = 20;
pub(crate) use pocket_ic::common::rest::MAX_INSTRUCTION_PROFILING_EXECUTIONS;

pub(crate) enum DecimalError {
    Syntax,
    Overflow,
}

pub(crate) fn unsigned_decimal(value: &str) -> Result<u64, DecimalError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(DecimalError::Syntax);
    }
    value.parse().map_err(|_| DecimalError::Overflow)
}

pub(crate) fn unsigned_argument(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| unsigned_decimal(value.as_str()?).ok())
}
