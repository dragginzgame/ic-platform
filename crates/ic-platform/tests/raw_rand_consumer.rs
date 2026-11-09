//! Independent general-Candid reference, selected only for this test target.
use ic_platform::raw_rand;

fn reference_decode(bytes: &[u8]) -> Vec<u8> {
    candid::utils::decode_one_with_config(bytes, &candid::de::DecoderConfig::new()).unwrap()
}

#[test]
fn request_and_replica_reply_shape_match_reference_candid() {
    assert_eq!(
        raw_rand::encode().as_slice(),
        candid::encode_args(()).unwrap()
    );
    for value in [[0; 32], [255; 32], core::array::from_fn(|i| i as u8)] {
        let wire = candid::encode_one(value.to_vec()).unwrap();
        assert_eq!(&wire[..10], b"DIDL\x01\x6d\x7b\x01\x00\x20");
        assert_eq!(raw_rand::decode(&wire), Ok(value));
    }
}

#[test]
fn all_supported_leb_widths_match_general_candid() {
    // Two one-byte opcodes and four padding-compatible framing fields.
    let fields = [1u8, 0x6d, 0x7b, 1, 0, 32];
    for selected in 0..=6 {
        for width in 1..=9 {
            let mut wire = b"DIDL".to_vec();
            for (index, byte) in fields.iter().copied().enumerate() {
                let width = if (selected == 6 || selected == index) && index != 1 && index != 2 {
                    width
                } else {
                    1
                };
                let extension = 0;
                if width == 1 {
                    wire.push(byte);
                } else {
                    wire.push(byte | 0x80);
                    wire.extend(core::iter::repeat_n(extension | 0x80, width - 2));
                    wire.push(extension);
                }
            }
            wire.extend_from_slice(&[42; 32]);
            assert_eq!(raw_rand::decode(&wire), Ok([42; 32]));
            assert_eq!(reference_decode(&wire), [42; 32]);
        }
    }
}

#[test]
fn documented_restriction_rejects_valid_unused_type_entries() {
    // A second, unused vec nat8 type is valid Candid, outside this producer profile.
    let mut wire = b"DIDL\x02\x6d\x7b\x6d\x7b\x01\x00\x20".to_vec();
    wire.extend_from_slice(&[17; 32]);
    assert_eq!(reference_decode(&wire), [17; 32]);
    assert_eq!(
        raw_rand::decode(&wire),
        Err(raw_rand::DecodeError::TypeTable)
    );
}

#[test]
fn every_single_byte_mutation_is_fallible_and_successes_match_reference() {
    let original = candid::encode_one(vec![42u8; 32]).unwrap();
    for index in 0..original.len() {
        for byte in 0..=255 {
            let mut wire = original.clone();
            wire[index] = byte;
            if let Ok(random) = raw_rand::decode(&wire) {
                assert_eq!(reference_decode(&wire), random);
            }
        }
    }
}
