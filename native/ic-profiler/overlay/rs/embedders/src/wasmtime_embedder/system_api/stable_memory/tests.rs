use super::*;

#[test]
fn system_profile_direct_reads_preserve_copy_and_bounds() {
    let capture = ic_system_profiler::start(ic_system_profiler::RunConfig {
        name: "direct-stable-read-counters".to_string(),
        ..Default::default()
    })
    .unwrap();
    let memory = StableMemory::new(ic_replicated_state::Memory::new_for_testing());
    let mut bytes = [0x55; 8];
    memory
        .stable_read_without_bounds_checks(1, 0, 3, &mut Heap::unchecked(&mut bytes))
        .unwrap();
    assert_eq!(bytes, [0x55, 0, 0, 0, 0x55, 0x55, 0x55, 0x55]);
    assert_eq!(memory.system_profile_direct_reads(), (1, 3));
    assert!(
        memory
            .stable_read_without_bounds_checks(7, 0, 2, &mut Heap::unchecked(&mut bytes))
            .is_err()
    );
    assert_eq!(memory.system_profile_direct_reads(), (1, 3));
    ic_system_profiler::stop(
        capture["run_id"].as_str().unwrap(),
        &ic_system_profiler::StopOptions::default(),
    )
    .unwrap();
}
