use super::{SubnetFrame, ViewFrame, candid, limits, new};
use ::candid::Principal;

fn frame(nodes: usize, skipped_bytes: usize, selected_text: usize) -> Vec<u8> {
    let principals: Vec<_> = (0..nodes)
        .map(|index| {
            let mut bytes = [1; 29];
            bytes[0] = u8::try_from(index).unwrap();
            Principal::from_slice(&bytes)
        })
        .collect();
    ::candid::encode_one(
        [ViewFrame {
            extra: vec!["z".repeat(skipped_bytes)],
            id: "e".repeat(selected_text),
            subnet_id: Some(Principal::anonymous()),
            engine_operator_id: None,
            subnet: Some(SubnetFrame {
                replica_version: "v".repeat(selected_text),
                maximum_state_size: Some(u64::MAX),
                halted: false,
                node_ids: principals.clone(),
            }),
            deleted_at: Some(i64::MIN),
            node_ids: principals,
        }]
        .as_slice(),
    )
    .unwrap()
}

#[derive(Clone, Copy, Debug)]
enum Budget {
    Work,
    Allocation,
    Skipping,
}

fn measure(bytes: &[u8], budget: Budget) -> usize {
    let cap = limits();
    let maximum = match budget {
        Budget::Work => cap.work,
        Budget::Allocation => cap.allocation,
        Budget::Skipping => cap.skipping,
    };
    let admits = |amount| {
        let mut cap = limits();
        match budget {
            Budget::Work => cap.work = amount,
            Budget::Allocation => cap.allocation = amount,
            Budget::Skipping => cap.skipping = amount,
        }
        match new::decode_get_engine_views_response(bytes, cap) {
            Ok(_) => true,
            Err(candid::Error::Limit) => false,
            Err(error) => panic!("unexpected candidate refusal: {error:?}"),
        }
    };
    assert!(admits(maximum), "Query ceiling exceeded: {budget:?}");
    let (mut low, mut high) = (0, maximum);
    while low < high {
        let mid = low + (high - low) / 2;
        if admits(mid) {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    assert!(admits(low));
    if low > 0 {
        assert!(!admits(low - 1));
    }
    low
}

#[test]
fn selected_workloads_fit_query_budgets() {
    let empty = ::candid::encode_one(Vec::<ViewFrame>::new()).unwrap();
    for (name, bytes) in [
        ("empty", empty),
        ("small", frame(4, 1, 1)),
        ("largest_lists", frame(128, 1, 256)),
        ("skipped_64k", frame(4, 65536, 1)),
    ] {
        assert!(new::decode_get_engine_views_response(&bytes, limits()).is_ok());
        {
            let work = measure(&bytes, Budget::Work);
            let allocation = measure(&bytes, Budget::Allocation);
            let skipping = measure(&bytes, Budget::Skipping);
            println!(
                "{name}: bytes={}, work={work}, allocation={allocation}, skipping={skipping}",
                bytes.len()
            );
        }
    }
}

#[test]
fn discarded_text_does_not_increase_logical_allocation() {
    let small = frame(4, 1, 1);
    let large = frame(4, 65536, 1);
    {
        assert_eq!(
            measure(&small, Budget::Allocation),
            measure(&large, Budget::Allocation)
        );
    }
    let mut malformed = large;
    let start = malformed
        .windows(32)
        .position(|window| window == [b'z'; 32])
        .unwrap();
    malformed[start + 65535] = 0xff;
    assert_eq!(
        new::decode_get_engine_views_response(&malformed, limits()),
        Err(candid::Error::Malformed)
    );
}

#[test]
fn zero_sized_discarded_vectors_stop_at_explicit_count_limit() {
    // vec record { 0:vec null; id:text; node_ids:vec principal }.
    let prefix = b"DIDL\x04\x6d\x01\x6c\x03\x00\x03\xdb\xb7\x01\x71\xbb\xf8\xfd\xed\x0f\x02\x6d\x68\x6d\x7f\x01\x00\x01";
    for (count, should_pass) in [
        (&[0x80, 0x08][..], true),
        (&[0x81, 0x08][..], false),
        (&[0xc0, 0x84, 0x3d][..], false),
    ] {
        let mut bytes = prefix.to_vec();
        bytes.extend_from_slice(count);
        bytes.extend_from_slice(&[1, b'e', 0]);
        let result = new::decode_get_engine_views_response(&bytes, limits());
        if should_pass {
            assert!(result.is_ok());
        } else {
            assert_eq!(result, Err(candid::Error::Limit));
        }
    }
}
