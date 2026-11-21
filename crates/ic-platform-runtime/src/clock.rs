//! Whole-unit views of the IC execution clock.

/// Current IC execution time in whole milliseconds since the Unix epoch.
/// # Panics
/// A native process has no IC execution clock.
#[must_use]
pub fn now_millis() -> u64 {
    crate::system::now() / 1_000_000
}

/// Current IC execution time in whole seconds since the Unix epoch.
/// # Panics
/// A native process has no IC execution clock.
#[must_use]
pub fn now_secs() -> u64 {
    crate::system::now() / 1_000_000_000
}
