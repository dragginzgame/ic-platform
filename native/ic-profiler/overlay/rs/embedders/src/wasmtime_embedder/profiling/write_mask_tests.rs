use super::MemoryWriteProfileCollector;

#[test]
fn write_mask_repeated_overlapping_writes_preserve_width_and_frequency() {
    let collector = MemoryWriteProfileCollector::new();
    collector.observe_write(0, 10, 8, 0);
    collector.observe_write(0, 10, 8, 0);
    collector.observe_write(0, 14, 8, 3);
    collector.observe_write(1, 10, 4, 1);
    collector.observe_write(1, 12, 4, 1);
    collector.observe_write(0, 42, 0, 0);
    let profile = collector.finish();
    assert_eq!(profile.observed_writes, 5);
    assert_eq!(profile.observed_bytes, 32);
    assert!(!profile.truncated);
    let runs: Vec<_> = profile
        .ranges
        .iter()
        .map(|r| {
            (
                r.memory.as_str(),
                r.start,
                r.end,
                r.heap_writes,
                r.stable_writes,
            )
        })
        .collect();
    assert_eq!(
        runs,
        vec![
            ("heap", 10, 14, 2, 0),
            ("heap", 14, 18, 3, 0),
            ("heap", 18, 22, 1, 0),
            ("stable", 10, 12, 0, 1),
            ("stable", 12, 14, 0, 2),
            ("stable", 14, 16, 0, 1),
        ]
    );
}

#[test]
fn write_mask_bounds_sparse_ranges_and_export_without_dense_allocation() {
    let collector = MemoryWriteProfileCollector::with_limits(2, 1);
    collector.observe_write(1, 1_u64 << 48, 1_u64 << 32, 1);
    collector.observe_write(1, 1_u64 << 48, 1_u64 << 32, 1);
    collector.observe_write(0, 0, 8, 0);
    collector.observe_write(0, 16, 8, 0);
    let profile = collector.finish();
    assert_eq!(profile.observed_writes, 4);
    assert!(profile.truncated);
    assert_eq!(profile.ranges.len(), 1);
    assert_eq!(profile.max_unique_writes, 2);
    assert_eq!(profile.max_export_ranges, 1);
}

#[test]
fn write_mask_saturation_and_invalid_addresses_are_declared() {
    let collector = MemoryWriteProfileCollector::new();
    collector.observe_write(0, 0, u64::MAX, 0);
    collector.observe_write(0, 0, 1, 0);
    collector.observe_write(0, u64::MAX, 1, 0);
    let profile = collector.finish();
    assert!(profile.counters_saturated);
    assert!(profile.truncated);
    assert_eq!(profile.observed_bytes, u64::MAX);
    assert_eq!(profile.ranges[0].heap_writes, 2);
}
