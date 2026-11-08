use super::*;

fn reply(value: [u8; 32]) -> Vec<u8> {
    let mut wire = b"DIDL\x01\x6d\x7b\x01\x00\x20".to_vec();
    wire.extend_from_slice(&value);
    wire
}

#[test]
fn accepts_protocol_valid_zero_and_full_byte_range() {
    assert_eq!(encode(), b"DIDL\0\0");
    for value in [[0; 32], [255; 32], core::array::from_fn(|i| i as u8)] {
        assert_eq!(decode(&reply(value)), Ok(value));
    }
}

#[test]
fn every_truncated_prefix_fails() {
    let wire = reply([7; 32]);
    for length in 0..wire.len() {
        assert_eq!(decode(&wire[..length]), Err(DecodeError::Truncated));
    }
}

#[test]
fn refuses_wrong_schema_arity_length_magic_and_trailing_data() {
    for (offset, value, error) in [
        (0, b'X', DecodeError::InvalidMagic),
        (4, 0, DecodeError::TypeTable),
        (4, 2, DecodeError::TypeTable),
        (5, 0x6e, DecodeError::TypeTable),
        (6, 0x7c, DecodeError::TypeTable),
        (7, 0, DecodeError::ResultArity),
        (7, 2, DecodeError::ResultArity),
        (8, 1, DecodeError::ResultType),
        (8, 0x7b, DecodeError::ResultType),
        (9, 31, DecodeError::BlobLength),
        (9, 33, DecodeError::BlobLength),
    ] {
        let mut wire = reply([0; 32]);
        wire[offset] = value;
        assert_eq!(decode(&wire), Err(error));
    }
    let mut wire = reply([0; 32]);
    wire.push(0);
    assert_eq!(decode(&wire), Err(DecodeError::TrailingBytes));
    assert_eq!(
        decode(&[0; MAX_REPLY_BYTES + 1]),
        Err(DecodeError::TooLarge)
    );
}

#[test]
fn refuses_unterminated_and_incorrectly_extended_leb() {
    for bytes in [vec![0x81; 11], vec![0x81, 0x7f], vec![0x81, 0x80, 0x01]] {
        let mut wire = b"DIDL".to_vec();
        wire.extend(bytes);
        assert_eq!(decode(&wire), Err(DecodeError::TypeTable));
    }
    let mut wire = b"DIDL\x01\xed\x00".to_vec();
    wire.extend_from_slice(&reply([0; 32])[6..]);
    assert_eq!(decode(&wire), Err(DecodeError::TypeTable));
    let mut wire = b"DIDL\x81".to_vec();
    wire.extend_from_slice(&[0x80; 10]);
    assert_eq!(decode(&wire), Err(DecodeError::TypeTable));
}
