//! Cargo resolution boundary for this Runtime cohort.
//!
//! The manifest's shared `links` identity excludes a second guarded executor
//! supplier. Runtime selects this package by exact version and source path.
//! The compile-time cohort check also rejects mismatched Runtime/guard versions.
//! No executor or native library is implemented in this package.
#![no_std]
#![forbid(unsafe_code)]

/// Checks that a Runtime compilation uses its coordinated guard version.
///
/// Runtime invokes this in a const initializer with its package version. This
/// complements, but does not replace, Cargo source and `links` identity checks.
///
/// # Panics
/// Panics (or fails const evaluation) if the supplied version differs.
pub const fn assert_runtime_version(runtime: &str) {
    let expected = env!("CARGO_PKG_VERSION").as_bytes();
    let actual = runtime.as_bytes();
    assert!(
        actual.len() == expected.len(),
        "Runtime/guard version mismatch"
    );
    let mut index = 0;
    while index < expected.len() {
        assert!(
            actual[index] == expected[index],
            "Runtime/guard version mismatch"
        );
        index += 1;
    }
}

#[cfg(test)]
mod tests;
