use super::*;

fn limits() -> Limits {
    Limits {
        bytes: 1024,
        fields: 32,
        allocation: 1024,
        items: 16,
    }
}

#[test]
fn request_matches_independent_protocol_bytes_and_admits_exact_budget() {
    let expected = b"\x12\x0bsubnet_list";
    // The key is eleven bytes, including its underscore.
    assert_eq!(
        encode_get_value_request(b"subnet_list", None, 13).unwrap(),
        expected
    );
    assert_eq!(
        encode_get_value_request(b"subnet_list", None, 12),
        Err(Error::ByteLimit)
    );
    assert_eq!(
        encode_get_value_request(b"x", Some(300), 8).unwrap(),
        [10, 3, 8, 172, 2, 18, 1, b'x']
    );
    assert_eq!(
        encode_get_value_request(b"", Some(0), 6).unwrap(),
        [10, 2, 8, 0, 18, 0]
    );
    let max = encode_get_value_request(b"", Some(u64::MAX), 15).unwrap();
    assert_eq!(&max[..3], &[10, 11, 8]);
    assert_eq!(
        &max[3..13],
        &[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]
    );
    assert_eq!(
        encode_get_value_request(b"", Some(u64::MAX), 14),
        Err(Error::ByteLimit)
    );
}

#[test]
fn inline_response_preserves_versions_time_and_borrowed_empty_presence() {
    let bytes = [16, 172, 2, 26, 3, 18, 1, 4, 40, 200, 1];
    let response = decode_get_value_response(&bytes, limits()).unwrap();
    assert_eq!(response.version, 300);
    assert_eq!(response.timestamp_nanoseconds, 200);
    assert_eq!(response.value, Some(&bytes[5..8]));
    assert!(response.error.is_none());
    assert_eq!(
        decode_get_value_response(&[], limits()).unwrap().value,
        None
    );
    assert_eq!(
        decode_get_value_response(&[26, 0], limits()).unwrap().value,
        Some(&[][..])
    );
    let wide = [16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1];
    assert_eq!(
        decode_get_value_response(&wide, limits()).unwrap().version,
        u64::MAX
    );
}

#[test]
fn source_errors_preserve_unknown_codes_key_and_text_with_shared_field_budget() {
    let bytes = [10, 9, 8, 231, 7, 18, 1, b'x', 26, 1, b'k'];
    let response = decode_get_value_response(&bytes, limits()).unwrap();
    assert_eq!(
        response.error,
        Some(RegistryError {
            code: 999,
            reason: "x",
            key: b"k"
        })
    );
    assert_eq!(
        decode_get_value_response(
            &bytes,
            Limits {
                fields: 3,
                ..limits()
            }
        ),
        Err(Error::FieldLimit)
    );
    assert_eq!(
        decode_get_value_response(&[10, 3, 18, 1, 255], limits()),
        Err(Error::Malformed)
    );
    assert_eq!(
        decode_get_value_response(&[10, 4, 8, 1, 8, 2], limits()),
        Err(Error::DuplicateField)
    );
}

#[test]
fn unknown_fields_are_bounded_and_all_wire_failures_are_closed() {
    for bytes in [
        &[48, 7][..],
        &[49, 0, 0, 0, 0, 0, 0, 0, 0][..],
        &[50, 2, 1, 2][..],
        &[53, 0, 0, 0, 0][..],
    ] {
        assert!(decode_get_value_response(bytes, limits()).is_ok());
        assert_eq!(
            decode_get_value_response(
                bytes,
                Limits {
                    fields: 0,
                    ..limits()
                }
            ),
            Err(Error::FieldLimit)
        );
    }
    for bytes in [
        &[0][..],
        &[16][..],
        &[26, 3, 1][..],
        &[17, 0, 0, 0, 0, 0, 0, 0, 0][..],
        &[16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2][..],
        &[26, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1][..],
        &[49, 0][..],
        &[53, 0][..],
    ] {
        assert_eq!(
            decode_get_value_response(bytes, limits()),
            Err(Error::Malformed)
        );
    }
    for byte in [51, 52, 54, 55] {
        assert_eq!(
            decode_get_value_response(&[byte], limits()),
            Err(Error::UnsupportedWireType)
        );
    }
    assert_eq!(
        decode_get_value_response(&[16, 1, 16, 2], limits()),
        Err(Error::DuplicateField)
    );
    assert_eq!(
        decode_get_value_response(&[26, 0, 26, 0], limits()),
        Err(Error::DuplicateField)
    );
    assert_eq!(
        decode_get_value_response(&[34, 0], limits()),
        Err(Error::ChunkedValue)
    );
    assert_eq!(
        decode_get_value_response(&[26, 0, 34, 0], limits()),
        Err(Error::ChunkedValue)
    );
    assert_eq!(
        decode_get_value_response(
            &[16, 1],
            Limits {
                bytes: 1,
                ..limits()
            }
        ),
        Err(Error::ByteLimit)
    );
}

#[test]
fn subnet_entries_preserve_raw_values_and_enforce_limits_before_allocation() {
    let bytes = [18, 1, 4, 18, 0, 18, 2, 7, 8, 48, 1];
    assert_eq!(
        decode_subnet_list(&bytes, limits()).unwrap(),
        [&[4][..], &[][..], &[7, 8][..]]
    );
    assert!(
        decode_subnet_list(
            &[],
            Limits {
                items: 0,
                allocation: 0,
                ..limits()
            }
        )
        .unwrap()
        .is_empty()
    );
    assert_eq!(
        decode_subnet_list(
            &bytes,
            Limits {
                items: 2,
                ..limits()
            }
        ),
        Err(Error::ItemLimit)
    );
    assert_eq!(
        decode_subnet_list(
            &bytes,
            Limits {
                allocation: 3 * std::mem::size_of::<&[u8]>() - 1,
                ..limits()
            }
        ),
        Err(Error::AllocationLimit)
    );
    assert_eq!(
        decode_subnet_list(
            &bytes,
            Limits {
                fields: 3,
                ..limits()
            }
        ),
        Err(Error::FieldLimit)
    );
    assert_eq!(
        decode_subnet_list(&[16, 1], limits()),
        Err(Error::Malformed)
    );
    assert_eq!(
        decode_subnet_list(&[18, 2, 4], limits()),
        Err(Error::Malformed)
    );
    assert_eq!(
        decode_subnet_list(
            &bytes,
            Limits {
                bytes: bytes.len() - 1,
                ..limits()
            }
        ),
        Err(Error::ByteLimit)
    );
}
