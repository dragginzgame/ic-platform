use super::*;
use std::error::Error;

#[test]
fn growth_mapping_preserves_all_non_sentinel_u64_values() {
    for pages in [
        0,
        1,
        u64::from(u32::MAX),
        u64::from(u32::MAX) + 1,
        u64::MAX - 1,
    ] {
        assert_eq!(growth_result(pages).unwrap(), pages);
    }
    assert!(matches!(
        growth_result(u64::MAX),
        Err(StableMemoryError::OutOfMemory)
    ));
}

#[test]
fn growth_error_has_the_exact_message_and_no_source() {
    let error = StableMemoryError::OutOfMemory;
    assert_eq!(error.to_string(), "Out of memory");
    assert_eq!(format!("{error:?}"), "OutOfMemory");
    assert!(error.source().is_none());
}

#[test]
fn stable_api_retains_64_bit_offsets_and_argument_order() {
    let _: fn(u64) -> Result<u64, StableMemoryError> = stable_grow;
    let _: fn(u64, &[u8]) = stable_write;
    let _: fn(u64, &mut [u8]) = stable_read;
}
