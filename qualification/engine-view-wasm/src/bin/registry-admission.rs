#![forbid(unsafe_code)]

//! Standalone Registry admission on the Wasm pointer-width boundary.

use ic_platform_protocol::registry_settings::{
    Error, Limits, SubnetSettings, decode_subnet_settings,
};

fn decode(bytes: &[u8], fields: usize) -> Result<SubnetSettings, Error> {
    std::hint::black_box(decode_subnet_settings(
        std::hint::black_box(bytes),
        Limits {
            bytes: bytes.len(),
            fields,
        },
    ))
}

fn main() {
    envelope_admission();
    assert_eq!(decode(&[], 0), Ok(SubnetSettings::default()));
    let present = decode(&[0xba, 1, 0, 0x92, 2, 0], 2).unwrap();
    assert!(!present.features.unwrap().http_requests);
    assert_eq!(present.features.unwrap().sev_enabled, None);
    assert_eq!(present.resource_limits.unwrap().maximum_state_size, None);
    let complete = [
        0xba, 1, 4, 0x18, 1, 0x48, 0, 0xc0, 1, 7, 0x92, 2, 8, 8, 0, 16, 2, 24, 3, 32, 4,
    ];
    let value = decode(&complete, 9).unwrap();
    assert!(value.features.unwrap().http_requests);
    assert_eq!(value.features.unwrap().sev_enabled, Some(false));
    assert_eq!(value.max_number_of_canisters, 7);
    let resource = value.resource_limits.unwrap();
    assert_eq!(resource.maximum_state_size, Some(0));
    assert_eq!(resource.maximum_state_delta, Some(2));
    assert_eq!(resource.maximum_query_instructions, Some(3));
    assert_eq!(resource.maximum_query_walltime_seconds, Some(4));
    assert_eq!(decode(&complete, 8), Err(Error::FieldLimit));
    assert_eq!(
        decode_subnet_settings(
            &complete,
            Limits {
                bytes: complete.len() - 1,
                fields: 9
            }
        ),
        Err(Error::ByteLimit)
    );
    // Root-message boundaries are valid partial records; all other truncations refuse.
    for end in 0..complete.len() {
        assert_eq!(
            decode(&complete[..end], 9).is_ok(),
            [0, 7, 10].contains(&end)
        );
    }
    let max = [0xc0, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1];
    assert_eq!(decode(&max, 1).unwrap().max_number_of_canisters, u64::MAX);
    for (bytes, expected) in [
        (&[0xc0, 1, 0, 0xc0, 1, 0][..], Error::DuplicateField),
        (&[0xba, 1, 4, 0x48, 0, 0x48, 0][..], Error::DuplicateField),
        (&[0xba, 1, 2, 0x48, 2][..], Error::Malformed),
        (&[11][..], Error::UnsupportedWireType),
        // Length 2^32 cannot fit usize on Wasm, and must never wrap to zero.
        (&[10, 0x80, 0x80, 0x80, 0x80, 0x10][..], Error::Malformed),
        (
            &[0xc0, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2][..],
            Error::Malformed,
        ),
    ] {
        assert_eq!(decode(bytes, 9), Err(expected));
    }
}

fn envelope_admission() {
    use ic_platform_protocol::registry::{self, Error, Limits};
    let limits = Limits {
        bytes: 4096,
        fields: 16,
        allocation: 64,
        items: 4,
    };
    let response = registry::decode_get_value_response(&[16, 7, 26, 0, 40, 9], limits).unwrap();
    assert_eq!(response.value, Some(&[][..]));
    assert_eq!(response.version, 7);
    assert_eq!(response.timestamp_nanoseconds, 9);
    assert_eq!(
        registry::decode_get_value_response(&[34, 0], limits),
        Err(Error::ChunkedValue)
    );
    assert_eq!(
        registry::decode_subnet_list(&[18, 0x80, 0x80, 0x80, 0x80, 0x10], limits),
        Err(Error::Malformed)
    );
    let mut bounded = limits;
    bounded.allocation = 2 * std::mem::size_of::<&[u8]>();
    assert_eq!(
        registry::decode_subnet_list(&[18, 1, 4, 18, 0], bounded)
            .unwrap()
            .len(),
        2
    );
    bounded.allocation -= 1;
    assert_eq!(
        registry::decode_subnet_list(&[18, 1, 4, 18, 0], bounded),
        Err(Error::AllocationLimit)
    );
    assert_eq!(
        registry::encode_get_value_request(b"k", Some(128), 8).unwrap(),
        [10, 3, 8, 128, 1, 18, 1, b'k']
    );
    assert_eq!(
        ic_platform_protocol::cloud_engine::encode_empty_args(6).unwrap(),
        b"DIDL\0\0"
    );
}
