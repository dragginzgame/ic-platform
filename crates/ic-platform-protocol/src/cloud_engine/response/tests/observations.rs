use super::{hex, limits};
use crate::{
    candid::Error,
    cloud_engine::{decode_get_network_fee_response, decode_registry_copy_status_response},
};

#[test]
fn fee_preserves_ieee_bits_without_business_admission() {
    for bits in [
        0,
        1u64 << 63,
        0.25f64.to_bits(),
        (-1.0f64).to_bits(),
        f64::MAX.to_bits(),
        1,
        f64::INFINITY.to_bits(),
        f64::NEG_INFINITY.to_bits(),
        0x7ff8_0000_0000_1234,
    ] {
        let mut frame = b"DIDL\0\x01\x72".to_vec();
        frame.extend_from_slice(&bits.to_le_bytes());
        assert_eq!(
            decode_get_network_fee_response(&frame, limits())
                .unwrap()
                .to_bits(),
            bits
        );
        let mut cap = limits();
        cap.bytes = frame.len();
        assert!(decode_get_network_fee_response(&frame, cap).is_ok());
        cap.bytes -= 1;
        assert_eq!(
            decode_get_network_fee_response(&frame, cap).unwrap_err(),
            Error::Limit
        );
        for end in 0..frame.len() {
            assert_eq!(
                decode_get_network_fee_response(&frame[..end], limits()).unwrap_err(),
                Error::Malformed
            );
        }
        frame.push(0);
        assert_eq!(
            decode_get_network_fee_response(&frame, limits()).unwrap_err(),
            Error::Trailing
        );
    }
    assert_eq!(
        decode_get_network_fee_response(b"DIDL\0\0", limits()).unwrap_err(),
        Error::Arity
    );
    assert_eq!(
        decode_get_network_fee_response(b"DIDL\0\x02\x72\x72", limits()).unwrap_err(),
        Error::Arity
    );
    assert_eq!(
        decode_get_network_fee_response(b"DIDL\0\x01\x78\0\0\0\0\0\0\0\0", limits()).unwrap_err(),
        Error::Type
    );
}

#[test]
fn status_retains_versions_and_source_flag_independently() {
    // record {latest:nat64; version:nat64; caught_up:bool}, independently
    // computed Candid labels in ascending order, one argument of table type 0.
    let header =
        hex("44 49 44 4c 01 6c 03 c7 8a 8c ab 05 78 98 ce c7 e7 07 78 fe b4 af f1 09 7e 01 00");
    for (latest, version, flag) in [(0u64, u64::MAX, true), (u64::MAX, 0, false)] {
        let mut frame = header.clone();
        frame.extend_from_slice(&latest.to_le_bytes());
        frame.extend_from_slice(&version.to_le_bytes());
        frame.push(u8::from(flag));
        let status = decode_registry_copy_status_response(&frame, limits()).unwrap();
        assert_eq!(
            (status.latest, status.version, status.caught_up),
            (latest, version, flag)
        );
        for end in 0..frame.len() {
            assert_eq!(
                decode_registry_copy_status_response(&frame[..end], limits()).unwrap_err(),
                Error::Malformed
            );
        }
        for budget in [0, 1, 2, 3, 4] {
            let mut cap = limits();
            match budget {
                0 => cap.bytes = frame.len() - 1,
                1 => cap.header = header.len() - 1,
                2 => cap.types = 0,
                3 => cap.work = 0,
                _ => cap.allocation = 0,
            }
            assert_eq!(
                decode_registry_copy_status_response(&frame, cap),
                Err(Error::Limit)
            );
        }
        *frame.last_mut().unwrap() = 2;
        assert_eq!(
            decode_registry_copy_status_response(&frame, limits()),
            Err(Error::Malformed)
        );
        *frame.last_mut().unwrap() = 0;
        frame.push(0);
        assert_eq!(
            decode_registry_copy_status_response(&frame, limits()),
            Err(Error::Trailing)
        );
    }
    assert_eq!(
        decode_registry_copy_status_response(b"DIDL\0\0", limits()),
        Err(Error::Arity)
    );
    assert_eq!(
        decode_registry_copy_status_response(b"DIDL\0\x02\x7f\x7f", limits()),
        Err(Error::Arity)
    );
    assert_eq!(
        decode_registry_copy_status_response(b"DIDL\x01\x6c\0\x01\0", limits()),
        Err(Error::Type)
    );
}
