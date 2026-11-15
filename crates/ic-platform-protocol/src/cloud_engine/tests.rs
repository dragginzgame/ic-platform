use super::{EncodeError, encode_get_engine_views_args};

// Literal frames authored from the Candid binary grammar, not a second encoder
// or the previous Platform implementation. The header describes vec text.
#[test]
fn empty_vector_is_one_argument() {
    let expected = b"DIDL\x01\x6d\x71\x01\x00\x00";
    assert_eq!(encode_get_engine_views_args(&[], 10), Ok(expected.to_vec()));
    assert_ne!(expected.as_slice(), b"DIDL\x00\x00");
}

#[test]
fn preserves_text_and_vector_order() {
    let ids = [String::from(" a\0"), String::new(), String::from("é")];
    let expected = b"DIDL\x01\x6d\x71\x01\x00\x03\x03 a\0\x00\x02\xc3\xa9";
    assert_eq!(
        encode_get_engine_views_args(&ids, expected.len()),
        Ok(expected.to_vec())
    );
    for max_bytes in 0..expected.len() {
        assert_eq!(
            encode_get_engine_views_args(&ids, max_bytes),
            Err(EncodeError::OutputLimit { max_bytes })
        );
    }
}

#[test]
fn text_length_uses_multiple_leb_bytes() {
    let ids = ["x".repeat(128)];
    let mut expected = b"DIDL\x01\x6d\x71\x01\x00\x01\x80\x01".to_vec();
    expected.extend_from_slice(&[b'x'; 128]);
    assert_eq!(encode_get_engine_views_args(&ids, 140), Ok(expected));
    assert_eq!(
        encode_get_engine_views_args(&ids, 139),
        Err(EncodeError::OutputLimit { max_bytes: 139 })
    );
}

#[test]
fn count_uses_multiple_leb_bytes_without_service_policy() {
    let ids = vec![String::new(); 128];
    let mut expected = b"DIDL\x01\x6d\x71\x01\x00\x80\x01".to_vec();
    expected.extend_from_slice(&[0; 128]);
    assert_eq!(encode_get_engine_views_args(&ids, 139), Ok(expected));
}

#[test]
fn refuses_even_empty_input_below_full_header_and_payload_size() {
    for max_bytes in 0..10 {
        assert_eq!(
            encode_get_engine_views_args(&[], max_bytes),
            Err(EncodeError::OutputLimit { max_bytes })
        );
    }
}
