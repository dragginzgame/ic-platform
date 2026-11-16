use ::candid::{CandidType, Deserialize, Principal};
use engine_view_consumer::fixtures::{FullViewFrame, SubnetFrame};
use ic_platform_protocol::{candid, cloud_engine as new};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

// Deliberately minimal independent wire shape for the discarded-text cases.
#[derive(CandidType, Deserialize)]
struct ExtraFrame {
    #[serde(rename = "")]
    extra: String,
    id: String,
    node_ids: Vec<Principal>,
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

fn selected(nodes: usize, text: usize) -> Vec<u8> {
    let node_ids: Vec<_> = (0..nodes)
        .map(|index| {
            let mut bytes = [1; 29];
            bytes[0] = u8::try_from(index).unwrap();
            Principal::from_slice(&bytes)
        })
        .collect();
    ::candid::encode_one(
        [FullViewFrame {
            id: "e".repeat(text),
            subnet_id: Some(Principal::anonymous()),
            engine_operator_id: None,
            subnet: Some(SubnetFrame {
                replica_version: "v".repeat(text),
                maximum_state_size: Some(u64::MAX),
                halted: false,
                node_ids: node_ids.clone(),
            }),
            deleted_at: Some(i64::MIN),
            node_ids,
        }]
        .as_slice(),
    )
    .unwrap()
}

fn discarded(size: usize) -> Vec<u8> {
    ::candid::encode_one(
        [ExtraFrame {
            extra: "z".repeat(size),
            id: "e".into(),
            node_ids: vec![],
        }]
        .as_slice(),
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
fn heap_allocations_fit_retained_baseline() {
    let workloads = [
        (
            "empty",
            ::candid::encode_one(Vec::<FullViewFrame>::new()).unwrap(),
        ),
        ("small", selected(4, 1)),
        ("largest_lists", selected(128, 256)),
        ("discarded_1", discarded(1)),
        ("discarded_64k", discarded(65536)),
    ];
    let mut discarded_stats = Vec::new();
    for (name, bytes) in &workloads {
        let views = new::decode_get_engine_views_response(bytes, limits()).unwrap();
        if *name == "empty" {
            assert!(views.is_empty());
        } else {
            assert_eq!(views.len(), 1);
            let view = &views[0];
            let (nodes, text) = if *name == "largest_lists" {
                (128, 256)
            } else if *name == "small" {
                (4, 1)
            } else {
                (0, 1)
            };
            assert_eq!(view.id, "e".repeat(text));
            assert_eq!(view.node_ids.len(), nodes);
            if nodes > 0 {
                let subnet = view.subnet.as_ref().unwrap();
                assert_eq!(subnet.replica_version, "v".repeat(text));
                assert_eq!(subnet.maximum_state_size, Some(u64::MAX));
                assert!(!subnet.halted);
                assert_eq!(subnet.node_ids, view.node_ids);
                for (index, principal) in view.node_ids.iter().enumerate() {
                    let mut expected = [1; 29];
                    expected[0] = u8::try_from(index).unwrap();
                    assert_eq!(principal.as_slice(), expected);
                }
                assert_eq!(view.subnet_id.unwrap().as_slice(), &[4]);
                assert_eq!(view.deleted_at, Some(i64::MIN));
            } else {
                assert!(view.subnet.is_none());
                assert!(view.subnet_id.is_none());
                assert!(view.deleted_at.is_none());
            }
            assert!(view.engine_operator_id.is_none());
        }
        drop(views);
        // Frozen Linux x86_64 baseline from the retained heap report.
        let baseline = match *name {
            "empty" => 616,
            "small" => 1058,
            "largest_lists" => 9008,
            _ => 425,
        };
        let candidate = measure(|| new::decode_get_engine_views_response(bytes, limits()).unwrap());
        println!(
            "{name}: frame={}, historical_bound={baseline}, candidate={candidate:?}",
            bytes.len()
        );
        assert!(
            candidate.total_bytes <= baseline,
            "total bytes regressed: {name}"
        );
        assert!(
            candidate.max_bytes <= usize::try_from(baseline).unwrap(),
            "peak bytes regressed: {name}"
        );
        if name.starts_with("discarded_") {
            discarded_stats.push(candidate);
        }
    }
    let stats = |row: usize| {
        let value = &discarded_stats[row];
        (
            value.total_bytes,
            value.max_bytes,
            value.total_blocks,
            value.curr_bytes,
        )
    };
    assert_eq!(stats(0), stats(1), "discarded text allocated");
    failed_decodes_release_partial_values();
}

fn failed_decodes_release_partial_values() {
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
            candid::Error::Malformed,
        );
        let mut trailing = bytes;
        trailing.push(0);
        refusal(
            name,
            "trailing",
            &trailing,
            limits(),
            candid::Error::Trailing,
        );
    }
    let bytes = selected(128, 256);
    let mut work = limits();
    work.work = 1000;
    refusal("largest_lists", "work", &bytes, work, candid::Error::Limit);
    let mut allocation = limits();
    allocation.allocation = 2000;
    refusal(
        "largest_lists",
        "allocation",
        &bytes,
        allocation,
        candid::Error::Limit,
    );
}

fn refusal(
    workload: &str,
    cause: &str,
    bytes: &[u8],
    cap: candid::Limits,
    new_error: candid::Error,
) {
    let candidate = measure(|| {
        assert_eq!(
            new::decode_get_engine_views_response(bytes, cap).unwrap_err(),
            new_error
        );
    });
    {
        let stats = &candidate;
        assert!(
            stats.total_blocks > 0,
            "fixture must exercise allocation before refusal"
        );
        assert_eq!(stats.curr_blocks, 0);
        assert_eq!(stats.curr_bytes, 0);
    }
    println!("refusal {workload}/{cause}: candidate={candidate:?}");
}
