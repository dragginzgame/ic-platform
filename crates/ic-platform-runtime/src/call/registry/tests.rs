use super::Registry;

#[test]
fn zero_stale_and_duplicate_handles_do_not_gain_authority() {
    let mut registry = Registry::new();
    assert_eq!(registry.take(0), None);
    let first = registry.insert("first").unwrap();
    assert_eq!(first, 1);
    assert_eq!(registry.take(first), Some("first"));
    let second = registry.insert("second").unwrap();
    assert_eq!(second, 2);
    assert_eq!(registry.take(first), None);
    assert_eq!(registry.take(second), Some("second"));
    assert_eq!(registry.take(second), None);
}

#[test]
fn exhaustion_returns_owned_value_and_never_wraps() {
    let mut registry = Registry::new();
    registry.last = u32::MAX - 1;
    assert_eq!(registry.insert("last"), Ok(u32::MAX));
    assert_eq!(registry.insert("retained"), Err("retained"));
    assert_eq!(registry.take(u32::MAX), Some("last"));
    assert_eq!(registry.insert("still exhausted"), Err("still exhausted"));
}
