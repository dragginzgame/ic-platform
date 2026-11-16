use ic_platform_protocol::{candid, cloud_engine as new};
use ic_platform_types::Principal;
use legacy_platform::{candid_codec, cloud_engine_types as old};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

// Deliberately minimal independent wire shape for the discarded-text cases.
legacy_platform::candid_record! {
    struct ExtraFrame {
        0 => extra: String,
        23_515 => id: String,
        4_257_184_827 => node_ids: Vec<Principal>,
    }
}

fn limits() -> candid::Limits {
    candid::Limits {
        bytes: 512 * 1024,
        header: 64 * 1024,
        types: 256,
        work: 2_000_000,
        allocation: 4 * 1024 * 1024,
        vector: 1024,
        skipping: 128 * 1024,
    }
}

fn published(bytes: &[u8]) -> Vec<old::EngineView> {
    published_result(bytes, limits()).unwrap()
}

fn published_result(
    bytes: &[u8],
    cap: candid::Limits,
) -> Result<Vec<old::EngineView>, candid_codec::decoder::Error> {
    old::decode_get_engine_views_response_with_skip_limit(
        bytes,
        candid_codec::decoder::Limits {
            bytes: cap.bytes,
            header: cap.header,
            types: cap.types,
            work: cap.work,
            allocation: cap.allocation,
        },
        cap.skipping,
    )
}

fn selected(nodes: usize, text: usize) -> Vec<u8> {
    let node_ids: Vec<_> = (0..nodes)
        .map(|index| {
            let mut bytes = [1; 29];
            bytes[0] = u8::try_from(index).unwrap();
            Principal::from_slice(&bytes)
        })
        .collect();
    candid_codec::encode_one(
        &[old::EngineView {
            id: "e".repeat(text),
            subnet_id: Some(Principal::anonymous()),
            engine_operator_id: None,
            subnet: Some(old::EngineSubnet {
                replica_version: "v".repeat(text),
                maximum_state_size: Some(u64::MAX),
                halted: false,
                node_ids: node_ids.clone(),
            }),
            deleted_at: Some(i64::MIN),
            node_ids,
        }]
        .as_slice(),
        limits().bytes,
    )
    .unwrap()
}

fn discarded(size: usize) -> Vec<u8> {
    candid_codec::encode_one(
        &[ExtraFrame {
            extra: "z".repeat(size),
            id: "e".into(),
            node_ids: vec![],
        }]
        .as_slice(),
        limits().bytes,
    )
    .unwrap()
}

fn measure<T>(decode: impl FnOnce() -> T) -> dhat::HeapStats {
    let profiler = dhat::Profiler::builder().testing().build();
    let result = std::hint::black_box(decode());
    let retained = dhat::HeapStats::get();
    drop(result);
    let released = dhat::HeapStats::get();
    drop(profiler);
    assert_eq!(released.curr_blocks, 0, "decode-owned allocations leaked");
    assert_eq!(released.curr_bytes, 0);
    retained
}

// Keep one test in this process: dhat's allocator and profiler are global.
#[test]
fn heap_allocations_fit_published_baseline() {
    let workloads = [
        (
            "empty",
            candid_codec::encode_one(&Vec::<old::EngineView>::new(), limits().bytes).unwrap(),
        ),
        ("small", selected(4, 1)),
        ("largest_lists", selected(128, 256)),
        ("discarded_1", discarded(1)),
        ("discarded_64k", discarded(65536)),
    ];
    let mut discarded_stats = Vec::new();
    for (name, bytes) in &workloads {
        // Validate equality outside the measured region, including Serde and
        // fixture construction. Both decoders then receive these exact bytes.
        assert_eq!(
            serde_json::to_value(published(bytes)).unwrap(),
            serde_json::to_value(new::decode_get_engine_views_response(bytes, limits()).unwrap())
                .unwrap()
        );
        let baseline = measure(|| published(bytes));
        let candidate = measure(|| new::decode_get_engine_views_response(bytes, limits()).unwrap());
        println!(
            "{name}: frame={}, published={baseline:?}, candidate={candidate:?}",
            bytes.len()
        );
        assert!(
            candidate.total_bytes <= baseline.total_bytes,
            "total bytes regressed: {name}"
        );
        assert!(
            candidate.max_bytes <= baseline.max_bytes,
            "peak bytes regressed: {name}"
        );
        if name.starts_with("discarded_") {
            discarded_stats.push((baseline, candidate));
        }
    }
    for index in 0..2 {
        let stats = |row: usize| {
            let (old, new) = &discarded_stats[row];
            let value = if index == 0 { old } else { new };
            (
                value.total_bytes,
                value.max_bytes,
                value.total_blocks,
                value.curr_bytes,
            )
        };
        assert_eq!(
            stats(0),
            stats(1),
            "discarded text allocated, decoder {index}"
        );
    }
    failed_decodes_release_partial_values();
}

fn failed_decodes_release_partial_values() {
    use candid_codec::decoder::Error as Old;
    // Keep this inside the existing serial profiler test, not another test
    // competing for the process-global allocator.
    for (name, bytes) in [
        ("small", selected(4, 1)),
        ("largest_lists", selected(128, 256)),
    ] {
        let truncated = &bytes[..bytes.len() - 1];
        refusal(
            name,
            "truncated",
            truncated,
            limits(),
            Old::Malformed,
            candid::Error::Malformed,
        );
        let mut trailing = bytes;
        trailing.push(0);
        refusal(
            name,
            "trailing",
            &trailing,
            limits(),
            Old::Trailing,
            candid::Error::Trailing,
        );
    }
    let bytes = selected(128, 256);
    let mut work = limits();
    work.work = 1000;
    refusal(
        "largest_lists",
        "work",
        &bytes,
        work,
        Old::Limit,
        candid::Error::Limit,
    );
    let mut allocation = limits();
    allocation.allocation = 2000;
    refusal(
        "largest_lists",
        "allocation",
        &bytes,
        allocation,
        Old::Limit,
        candid::Error::Limit,
    );
}

fn refusal(
    workload: &str,
    cause: &str,
    bytes: &[u8],
    cap: candid::Limits,
    old_error: candid_codec::decoder::Error,
    new_error: candid::Error,
) {
    let baseline = measure(|| {
        assert_eq!(published_result(bytes, cap).unwrap_err(), old_error);
    });
    let candidate = measure(|| {
        assert_eq!(
            new::decode_get_engine_views_response(bytes, cap).unwrap_err(),
            new_error
        );
    });
    for stats in [&baseline, &candidate] {
        assert!(
            stats.total_blocks > 0,
            "fixture must exercise allocation before refusal"
        );
        assert_eq!(stats.curr_blocks, 0);
        assert_eq!(stats.curr_bytes, 0);
    }
    println!("refusal {workload}/{cause}: published={baseline:?}, candidate={candidate:?}");
}
