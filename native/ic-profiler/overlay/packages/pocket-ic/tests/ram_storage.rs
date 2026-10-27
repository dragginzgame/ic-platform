#![cfg(target_os = "linux")]

use candid::Principal;
use pocket_ic::common::storage::{ram_parent, ram_requested_from_env, validate_ram_directory};
use pocket_ic::{PocketIc, PocketIcBuilder, PocketIcState};
use reqwest::blocking::Client;
use serde_json::Value;

const PARITY_WAT: &str = r#"
    (module
      (import "ic0" "msg_reply_data_append" (func $append (param i64 i64)))
      (import "ic0" "msg_reply" (func $reply))
      (import "ic0" "performance_counter" (func $counter (param i32) (result i64)))
      (import "ic0" "stable64_grow" (func $stable_grow (param i64) (result i64)))
      (import "ic0" "stable64_read" (func $stable_read (param i64 i64 i64)))
      (import "ic0" "stable64_write" (func $stable_write (param i64 i64 i64)))
      (memory (export "memory") i64 4)

      (func (export "canister_update exercise")
        (drop (call $stable_grow (i64.const 1)))
        (call $stable_read (i64.const 131072) (i64.const 0) (i64.const 8))
        (call $stable_read (i64.const 131072) (i64.const 0) (i64.const 8))
        (i64.store (i64.const 65536) (i64.const 42))
        (call $stable_write (i64.const 0) (i64.const 65536) (i64.const 8))
        (call $stable_read (i64.const 131072) (i64.const 0) (i64.const 8))
        (memory.fill (i64.const 196608) (i32.const 7) (i64.const 17))
        (memory.copy (i64.const 196640) (i64.const 196608) (i64.const 17))
        (drop (memory.grow (i64.const 1)))
        (memory.fill (i64.const 262144) (i32.const 1) (i64.const 0))
        (memory.copy (i64.const 262144) (i64.const 0) (i64.const 0))
        (call $stable_write (i64.const 0) (i64.const 0) (i64.const 0))
        (call $stable_read (i64.const 0) (i64.const 0) (i64.const 0))
        (i64.store (i64.const 0) (call $counter (i32.const 0)))
        (call $append (i64.const 0) (i64.const 8))
        (call $reply))

      (func (export "canister_query state")
        (i64.store (i64.const 0) (i64.load (i64.const 65536)))
        (i64.store (i64.const 8) (i64.load (i64.const 131072)))
        (i64.store (i64.const 16) (i64.load (i64.const 196608)))
        (i64.store (i64.const 24) (i64.load (i64.const 196640)))
        (i64.store (i64.const 32) (memory.size))
        (call $append (i64.const 0) (i64.const 40))
        (call $reply))

      (func (export "canister_update fail")
        (i64.store (i64.const 65536) (i64.const 99))
        (call $stable_write (i64.const 0) (i64.const 65536) (i64.const 8))
        unreachable))
    "#;

fn storage(pic: &PocketIc) -> Value {
    Client::new()
        .get(pic.get_server_url().join("storage").unwrap())
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap()
}

#[test]
fn ram_launch_isolated_from_disk_preserves_checkpoint_execution_and_masks() {
    if let Err(reason) = validate_ram_directory(&ram_parent()) {
        eprintln!("Skipping RAM storage regression: {reason}");
        return;
    }
    if ram_requested_from_env().unwrap() {
        eprintln!(
            "Skipping disk/RAM isolation comparison because POCKET_IC_RAM explicitly selects RAM for all builders"
        );
        return;
    }
    // Concurrent RAM builders must converge on their own RAM server, including
    // the first-launch race, while a disk builder uses a different server.
    let (ram_a, ram_b) = std::thread::scope(|scope| {
        let a = scope.spawn(|| {
            PocketIcBuilder::new()
                .with_application_subnet()
                .with_ram_storage()
                .build()
        });
        let b = scope.spawn(|| {
            PocketIcBuilder::new()
                .with_application_subnet()
                .with_ram_storage()
                .build()
        });
        (a.join().unwrap(), b.join().unwrap())
    });
    let ram_url = ram_a.get_server_url();
    assert_eq!(ram_url, ram_b.get_server_url());
    assert_eq!(storage(&ram_a)["ram_backed"], true);
    let seed = PocketIcBuilder::new()
        .with_application_subnet()
        .with_state(PocketIcState::new())
        .build();
    let disk_url = seed.get_server_url();
    assert_ne!(disk_url, ram_url);
    assert_eq!(storage(&seed)["ram_backed"], false);
    drop((ram_a, ram_b));

    // One persisted starting state avoids differences from separate installs.
    let canister = seed.create_canister();
    seed.add_cycles(canister, 2_000_000_000_000);
    seed.install_canister(canister, wat::parse_str(PARITY_WAT).unwrap(), vec![], None);
    seed.update_call(canister, Principal::anonymous(), "exercise", vec![])
        .unwrap();
    let state = seed.drop_and_take_state().unwrap();
    let disk = PocketIcBuilder::new()
        .with_server_url(disk_url)
        .with_read_only_state(&state)
        .build();
    let ram = PocketIcBuilder::new()
        .with_server_url(ram_url)
        .with_ram_storage()
        .with_read_only_state(&state)
        .build();
    assert_eq!(storage(&disk)["ram_backed"], false);
    assert_eq!(storage(&ram)["ram_backed"], true);
    assert_eq!(disk.get_time(), ram.get_time());
    let initial_cycles = disk.cycle_balance(canister);
    assert_eq!(initial_cycles, ram.cycle_balance(canister));
    disk.start_memory_profiling(canister, 8, true, true);
    ram.start_memory_profiling(canister, 8, true, true);
    assert_eq!(
        disk.update_call(canister, Principal::anonymous(), "exercise", vec![])
            .unwrap(),
        ram.update_call(canister, Principal::anonymous(), "exercise", vec![])
            .unwrap()
    );
    let state_reply = disk
        .query_call(canister, Principal::anonymous(), "state", vec![])
        .unwrap();
    assert_eq!(
        state_reply,
        ram.query_call(canister, Principal::anonymous(), "state", vec![])
            .unwrap()
    );
    let stable = disk.get_stable_memory(canister);
    assert_eq!(stable, ram.get_stable_memory(canister));
    assert!(
        disk.update_call(canister, Principal::anonymous(), "fail", vec![])
            .is_err()
    );
    assert!(
        ram.update_call(canister, Principal::anonymous(), "fail", vec![])
            .is_err()
    );
    for pic in [&disk, &ram] {
        assert_eq!(
            pic.query_call(canister, Principal::anonymous(), "state", vec![])
                .unwrap(),
            state_reply
        );
        assert_eq!(pic.get_stable_memory(canister), stable);
        pic.stop_memory_access_tracing(canister);
    }
    assert_eq!(disk.cycle_balance(canister), ram.cycle_balance(canister));
    assert!(disk.cycle_balance(canister) < initial_cycles);
    assert_eq!(disk.get_time(), ram.get_time());
    let disk_trace = disk.get_memory_access_trace(canister);
    let ram_trace = ram.get_memory_access_trace(canister);
    assert_eq!(
        disk_trace.runtime_configurations,
        ram_trace.runtime_configurations
    );
    let read = disk_trace.executions[0].read_profile.as_ref().unwrap();
    assert!(!read.truncated && !read.counters_saturated);
    assert!(
        read.ranges
            .iter()
            .any(|range| range.stable_direct_reads > 0)
    );
    assert!(
        read.ranges
            .iter()
            .any(|range| range.stable_native_reads > 0)
    );
    let write = disk_trace.executions[0].write_profile.as_ref().unwrap();
    assert!(!write.truncated && !write.counters_saturated);
    assert!(write.ranges.iter().any(|range| range.stable_writes > 0));
    // The second identical state query is cached after the trapped update
    // rolls back, so it adds no physical Wasm execution.
    assert_eq!(disk_trace.executions.len(), 3);
    assert_eq!(disk_trace.executions.len(), ram_trace.executions.len());
    for (disk, ram) in disk_trace.executions.iter().zip(&ram_trace.executions) {
        assert_eq!(disk.function, ram.function);
        assert_eq!(disk.outcome, ram.outcome);
        assert_eq!(disk.instructions, ram.instructions);
        assert_eq!(disk.page_access_instructions, ram.page_access_instructions);
        assert_eq!(disk.accessed_pages, ram.accessed_pages);
        assert_eq!(disk.dirty_pages, ram.dirty_pages);
        assert_eq!(disk.stable_accessed_pages, ram.stable_accessed_pages);
        assert_eq!(disk.stable_dirty_pages, ram.stable_dirty_pages);
        assert_eq!(disk.read_profile, ram.read_profile);
        assert_eq!(disk.write_profile, ram.write_profile);
        let disk = disk.instruction_profile.as_ref().unwrap();
        let ram = ram.instruction_profile.as_ref().unwrap();
        assert!(disk.ledger.reconciled && ram.ledger.reconciled);
        assert_eq!(disk.ledger, ram.ledger);
        assert_eq!(disk.rules, ram.rules);
        assert_eq!(disk.blocks, ram.blocks);
        assert_eq!(disk.api_calls, ram.api_calls);
        assert_eq!(disk.dynamic_charges, ram.dynamic_charges);
    }
}
