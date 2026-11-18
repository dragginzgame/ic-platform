use super::*;

fn limits() -> Limits {
    Limits {
        bytes: 65536,
        fields: 4096,
        allocation: 4096,
        items: 128,
    }
}

#[test]
fn literal_envelopes_preserve_presence_versions_and_application_errors() {
    let response = decode_get_value_response(&[16, 7, 26, 0, 40, 9], limits()).unwrap();
    assert_eq!(response.version, 7);
    assert_eq!(response.timestamp_nanoseconds, 9);
    assert_eq!(response.value, Some(&[][..]));
    assert_eq!(
        decode_get_value_response(&[], limits()).unwrap().value,
        None
    );
    let bytes = [10, 8, 8, 123, 18, 1, b'x', 26, 1, 255, 26, 1, 0];
    let r = decode_get_value_response(&bytes, limits()).unwrap();
    assert_eq!(
        r.error,
        Some(RegistryError {
            code: 123,
            reason: "x",
            key: &[255]
        })
    );
    assert!(
        decode_get_value_response(&[10, 0], limits())
            .unwrap()
            .error
            .is_some()
    );
    for bytes in [&[26, 0, 26, 0][..], &[16, 0, 16, 0], &[10, 4, 8, 0, 8, 0]] {
        assert_eq!(
            decode_get_value_response(bytes, limits()),
            Err(Error::DuplicateField)
        );
    }
    assert_eq!(
        decode_get_value_response(&[34, 0], limits()),
        Err(Error::ChunkedValue)
    );
    assert_eq!(
        decode_get_value_response(&[26, 2, 0], limits()),
        Err(Error::Malformed)
    );
    let mut cap = limits();
    cap.fields = 4;
    assert!(decode_get_value_response(&[10, 4, 8, 0, 18, 0, 26, 0], cap).is_ok());
    cap.fields = 3;
    assert_eq!(
        decode_get_value_response(&[10, 4, 8, 0, 18, 0, 26, 0], cap),
        Err(Error::FieldLimit)
    );
}

#[test]
fn requests_match_independent_protobuf_literals() {
    assert_eq!(
        encode_get_value_request(b"k", None, 3).unwrap(),
        [18, 1, b'k']
    );
    assert_eq!(
        encode_get_value_request(b"k", Some(0), 5).unwrap(),
        [10, 0, 18, 1, b'k']
    );
    assert_eq!(
        encode_get_value_request(b"k", Some(128), 8).unwrap(),
        [10, 3, 8, 128, 1, 18, 1, b'k']
    );
    for version in [None, Some(0), Some(127), Some(128), Some(u64::MAX)] {
        let bytes = encode_get_value_request(b"\0\xff", version, 64).unwrap();
        assert_eq!(
            encode_get_value_request(b"\0\xff", version, bytes.len()).unwrap(),
            bytes
        );
        assert_eq!(
            encode_get_value_request(b"\0\xff", version, bytes.len() - 1),
            Err(Error::OutputLimit)
        );
    }
}

#[test]
fn projections_keep_borrowed_presence_and_source_order() {
    let nodes = decode_subnet_list(&[18, 1, 4, 18, 0], limits()).unwrap();
    assert_eq!(nodes, vec![&[4][..], &[]]);
    let r = decode_node_record(&[122, 1, 4, 130, 1, 0, 138, 1, 0], limits()).unwrap();
    assert_eq!(r.node_operator_id, &[4]);
    assert_eq!(r.chip_id, Some(&[][..]));
    assert_eq!(r.hostos_version_id, Some(""));
    assert_eq!(decode_node_record(&[], limits()).unwrap().chip_id, None);
    let r = decode_subnet_record(
        &[26, 1, 4, 74, 1, b'v', 120, 2, 136, 1, 1, 224, 1, 0],
        limits(),
    )
    .unwrap();
    assert_eq!(r.membership, vec![&[4][..]]);
    assert_eq!(r.replica_version_id, "v");
    assert_eq!(r.subnet_type, 2);
    assert!(r.is_halted);
    assert!(!r.halt_at_cup_height);
    assert_eq!(
        decode_subnet_record(&[136, 1, 2], limits()),
        Err(Error::Malformed)
    );
    assert_eq!(
        decode_data_center_record(&[10, 1, 255], limits()),
        Err(Error::Malformed)
    );
    assert_eq!(
        decode_get_latest_version_response(&[8, 128, 1], limits()).unwrap(),
        128
    );
}

#[test]
fn repeated_and_unknown_fields_obey_all_budgets() {
    let bytes = [18, 1, 4, 18, 0];
    let mut cap = limits();
    cap.items = 1;
    assert_eq!(decode_subnet_list(&bytes, cap), Err(Error::ItemLimit));
    cap = limits();
    cap.allocation = 2 * std::mem::size_of::<&[u8]>();
    assert!(decode_subnet_list(&bytes, cap).is_ok());
    cap.allocation -= 1;
    assert_eq!(decode_subnet_list(&bytes, cap), Err(Error::AllocationLimit));
    cap = limits();
    cap.bytes = 4;
    assert_eq!(decode_subnet_list(&bytes, cap), Err(Error::ByteLimit));
    cap = limits();
    cap.fields = 1;
    assert_eq!(
        decode_subnet_list(&[8, 0, 18, 0], cap),
        Err(Error::FieldLimit)
    );
    assert_eq!(
        decode_subnet_list(&[8, 128], limits()),
        Err(Error::Malformed)
    );
}
