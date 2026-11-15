use super::*;

fn decode(bytes: &[u8]) -> Result<SubnetSettings, Error> {
    decode_subnet_settings(
        bytes,
        Limits {
            bytes: bytes.len(),
            fields: 64,
        },
    )
}

// Literal protobuf fixtures are independent of the production dispatch. Field
// numbers come from the pinned replica schema documented in qualification.
const COMPLETE: &[u8] = &[
    0xba, 1, 4, 0x18, 1, 0x48, 0, // features: HTTP true, SEV explicitly false
    0xc0, 1, 7, // maximum canisters
    0x92, 2, 8, 8, 0, 16, 2, 24, 3, 32, 4, // all four resource limits
];

#[test]
fn presence_and_raw_values() {
    assert_eq!(decode(&[]), Ok(SubnetSettings::default()));
    let empty = decode(&[0xba, 1, 0, 0x92, 2, 0]).unwrap();
    assert_eq!(empty.features, Some(SubnetFeatures::default()));
    assert_eq!(empty.resource_limits, Some(ResourceLimits::default()));
    let value = decode(COMPLETE).unwrap();
    assert_eq!(
        value.features,
        Some(SubnetFeatures {
            http_requests: true,
            sev_enabled: Some(false)
        })
    );
    assert_eq!(value.max_number_of_canisters, 7);
    assert_eq!(
        value.resource_limits,
        Some(ResourceLimits {
            maximum_state_size: Some(0),
            maximum_state_delta: Some(2),
            maximum_query_instructions: Some(3),
            maximum_query_walltime_seconds: Some(4),
        })
    );
    assert_eq!(
        decode(&[0xba, 1, 4, 0x18, 0, 0x48, 1]).unwrap().features,
        Some(SubnetFeatures {
            http_requests: false,
            sev_enabled: Some(true)
        })
    );
    let reversed = [
        0x92, 2, 8, 32, 4, 24, 3, 16, 2, 8, 0, 0xc0, 1, 7, 0xba, 1, 4, 0x48, 0, 0x18, 1,
    ];
    assert_eq!(decode(&reversed), Ok(value));
}

#[test]
fn byte_and_field_limits_include_nested_and_unknown_fields() {
    assert!(
        decode_subnet_settings(
            COMPLETE,
            Limits {
                bytes: COMPLETE.len(),
                fields: 9
            }
        )
        .is_ok()
    );
    assert_eq!(
        decode_subnet_settings(
            COMPLETE,
            Limits {
                bytes: COMPLETE.len(),
                fields: 8
            }
        ),
        Err(Error::FieldLimit)
    );
    assert_eq!(
        decode_subnet_settings(
            COMPLETE,
            Limits {
                bytes: COMPLETE.len() - 1,
                fields: 9
            }
        ),
        Err(Error::ByteLimit)
    );
    assert_eq!(
        decode_subnet_settings(
            &[],
            Limits {
                bytes: 0,
                fields: 0
            }
        ),
        Ok(SubnetSettings::default())
    );
    // Unknown field inside a selected message also consumes the shared budget.
    assert_eq!(
        decode_subnet_settings(
            &[0xba, 1, 2, 8, 0],
            Limits {
                bytes: 5,
                fields: 1
            }
        ),
        Err(Error::FieldLimit)
    );
    assert!(
        decode_subnet_settings(
            &[0xba, 1, 2, 8, 0],
            Limits {
                bytes: 5,
                fields: 2
            }
        )
        .is_ok()
    );
}

#[test]
fn duplicates_refuse_even_for_defaults_and_empty_messages() {
    for bytes in [
        &[0xba, 1, 0, 0xba, 1, 0][..],
        &[0xc0, 1, 0, 0xc0, 1, 0],
        &[0x92, 2, 0, 0x92, 2, 0],
        &[0xba, 1, 4, 0x18, 0, 0x18, 0],
        &[0xba, 1, 4, 0x48, 0, 0x48, 0],
    ] {
        assert_eq!(decode(bytes), Err(Error::DuplicateField));
    }
    for tag in [8, 16, 24, 32] {
        assert_eq!(
            decode(&[0x92, 2, 4, tag, 0, tag, 0]),
            Err(Error::DuplicateField)
        );
    }
}

#[test]
fn unknown_fields_skip_valid_framing_only() {
    let unknown = [
        8, 0, 8, 1, 17, 0, 0, 0, 0, 0, 0, 0, 0, 26, 2, 255, 255, 37, 0, 0, 0, 0,
    ];
    assert_eq!(decode(&unknown), Ok(SubnetSettings::default()));
    for bytes in [&[8][..], &[17, 0], &[26, 2, 0], &[37, 0]] {
        assert_eq!(decode(bytes), Err(Error::Malformed));
    }
    for tag in [11, 12, 14, 15] {
        assert_eq!(decode(&[tag]), Err(Error::UnsupportedWireType));
    }
}

#[test]
fn selected_wire_types_and_boolean_values_are_strict() {
    for bytes in [
        &[0xb8, 1, 0][..],
        &[0xc2, 1, 0],
        &[0x90, 2, 0],
        &[0xba, 1, 2, 0x1a, 0],
        &[0xba, 1, 2, 0x4a, 0],
        &[0xba, 1, 2, 0x18, 2],
        &[0xba, 1, 2, 0x48, 2],
    ] {
        assert_eq!(decode(bytes), Err(Error::Malformed));
    }
    for tag in [10, 18, 26, 34] {
        assert_eq!(decode(&[0x92, 2, 2, tag, 0]), Err(Error::Malformed));
    }
}

#[test]
fn varint_bounds_and_field_numbers() {
    let max = [0xc0, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1];
    assert_eq!(decode(&max).unwrap().max_number_of_canisters, u64::MAX);
    let mut overflow = max;
    overflow[11] = 2;
    assert_eq!(decode(&overflow), Err(Error::Malformed));
    overflow[11] = 128;
    assert_eq!(decode(&overflow), Err(Error::Malformed));
    assert_eq!(decode(&max[..11]), Err(Error::Malformed));
    // Non-minimal but bounded protobuf varints are accepted.
    assert_eq!(
        decode(&[0xc0, 1, 0x80, 0]).unwrap().max_number_of_canisters,
        0
    );
    assert_eq!(decode(&[0, 0]), Err(Error::Malformed));
    assert_eq!(
        decode(&[0x80, 0x80, 0x80, 0x80, 0x10, 0]),
        Err(Error::Malformed)
    );
    assert!(decode(&[0xf8, 0xff, 0xff, 0xff, 0x0f, 0]).is_ok());
    assert_eq!(
        decode(&[26, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1]),
        Err(Error::Malformed)
    );
}

#[test]
fn truncation_inside_each_selected_message_refuses() {
    for bytes in [
        &[0xba, 1, 1, 0x18][..],
        &[0x92, 2, 1, 8],
        &[0xba, 1, 3, 0x18, 1],
        &[0x92, 2, 3, 8, 1],
    ] {
        assert_eq!(decode(bytes), Err(Error::Malformed));
    }
}
