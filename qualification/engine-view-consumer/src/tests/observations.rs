use ic_platform_protocol::{candid::Limits, cloud_engine};

fn limits() -> Limits {
    Limits {
        bytes: 4096,
        header: 1024,
        types: 16,
        work: 4096,
        allocation: 4096,
        vector: 16,
        skipping: 4096,
    }
}

#[cfg(not(target_family = "wasm"))]
fn old_limits() -> legacy_platform::candid_codec::decoder::Limits {
    let cap = limits();
    legacy_platform::candid_codec::decoder::Limits {
        bytes: cap.bytes,
        header: cap.header,
        types: cap.types,
        work: cap.work,
        allocation: cap.allocation,
    }
}

#[test]
fn public_fee_matches_literal_bits_and_published_decoder() {
    for bits in [
        0u64,
        1 << 63,
        0.25f64.to_bits(),
        (-1.0f64).to_bits(),
        f64::INFINITY.to_bits(),
        0x7ff8_0000_0000_1234,
    ] {
        let mut bytes = b"DIDL\0\x01\x72".to_vec();
        bytes.extend_from_slice(&bits.to_le_bytes());
        let fee = cloud_engine::decode_get_network_fee_response(&bytes, limits()).unwrap();
        assert_eq!(fee.to_bits(), bits);
        #[cfg(not(target_family = "wasm"))]
        assert_eq!(
            fee.to_bits(),
            legacy_platform::cloud_engine_types::decode_get_network_fee_response(
                &bytes,
                old_limits()
            )
            .unwrap()
            .to_bits()
        );
    }
}

#[test]
fn public_copy_status_matches_literal_json_and_published_decoder() {
    let mut bytes = b"DIDL\x01\x6c\x03\xc7\x8a\x8c\xab\x05\x78\x98\xce\xc7\xe7\x07\x78\xfe\xb4\xaf\xf1\x09\x7e\x01\0".to_vec();
    bytes.extend_from_slice(&u64::MAX.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes.push(1);
    let status = cloud_engine::decode_registry_copy_status_response(&bytes, limits()).unwrap();
    let json = serde_json::json!({"latest": u64::MAX, "version": 0, "caught_up": true});
    assert_eq!(serde_json::to_value(&status).unwrap(), json);
    assert_eq!(
        serde_json::from_value::<cloud_engine::RegistryCopyStatus>(json.clone()).unwrap(),
        status
    );
    #[cfg(not(target_family = "wasm"))]
    assert_eq!(
        serde_json::to_value(
            legacy_platform::cloud_engine_types::decode_registry_copy_status_response(
                &bytes,
                old_limits()
            )
            .unwrap()
        )
        .unwrap(),
        json
    );
}
