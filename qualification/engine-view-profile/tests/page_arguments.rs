#![forbid(unsafe_code)]

//! Isolated allocation check for complete-output admission.
#![cfg(all(feature = "marketplace", not(target_family = "wasm")))]

use candidate_types::Principal;
use ic_platform_protocol::cloud_engine::{
    EncodeError, ListAvailableNodesPageArgs, encode_list_available_nodes_page_args as encode,
};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[test]
fn admission_precedes_the_single_output_allocation() {
    // Construct input magnitudes/text before profiling either operation.
    let args = ListAvailableNodesPageArgs {
        limit: Some("9".repeat(256).parse().unwrap()),
        start: Some(Principal::anonymous()),
        rejoining: Some("é".repeat(128)),
    };
    let expected = encode(&args, 4096).unwrap();
    let size = expected.len();
    for admitted in [false, true] {
        let cap = if admitted { size } else { size - 1 };
        let profiler = dhat::Profiler::builder().testing().build();
        let result = std::hint::black_box(encode(std::hint::black_box(&args), cap));
        let retained = dhat::HeapStats::get();
        let correct = if admitted {
            result.as_ref().is_ok_and(|bytes| bytes == &expected)
        } else {
            result == Err(EncodeError::OutputLimit { max_bytes: cap })
        };
        drop(result);
        let released = dhat::HeapStats::get();
        drop(profiler);
        assert!(correct);
        assert_eq!(retained.total_blocks, u64::from(admitted));
        assert_eq!(retained.total_bytes, if admitted { size as u64 } else { 0 });
        assert_eq!(released.curr_blocks, 0);
        assert_eq!(released.curr_bytes, 0);
    }
}
