use super::assert_runtime_version;

#[test]
fn coordinated_version_is_admitted() {
    assert_runtime_version(env!("CARGO_PKG_VERSION"));
}

#[test]
#[should_panic(expected = "Runtime/guard version mismatch")]
fn other_cohort_is_refused() {
    assert_runtime_version("0.00.0");
}

#[test]
#[should_panic(expected = "Runtime/guard version mismatch")]
fn partial_version_is_refused() {
    assert_runtime_version("0");
}
