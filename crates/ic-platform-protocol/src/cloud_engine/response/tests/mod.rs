use super::{EngineView, decode_get_engine_views_response};
use crate::candid::{Error, Limits};

#[cfg(feature = "cloud-engine-marketplace")]
mod arguments;
#[cfg(feature = "cloud-engine-marketplace")]
mod availability;
#[cfg(feature = "cloud-engine-marketplace")]
mod marketplace;
mod observations;

fn limits() -> Limits {
    Limits {
        bytes: 65536,
        header: 4096,
        types: 256,
        work: 100_000,
        allocation: 65536,
        vector: 1024,
        skipping: 65536,
    }
}

// Independently authored from Candid's binary grammar. These literals do not
// use the projection macro, a production encoder, or old Platform test data.
fn hex(text: &str) -> Vec<u8> {
    text.split_whitespace()
        .map(|v| u8::from_str_radix(v, 16).unwrap())
        .collect()
}
fn minimal() -> Vec<u8> {
    hex("44 49 44 4c 03 6d 01 6c 02 db b7 01 71 bb f8 fd ed 0f 02 6d 68 01 00 01 01 65 00")
}
fn full() -> Vec<u8> {
    hex("44 49 44 4c 08
        6d 01
        6c 06 db b7 01 71 bd 86 9d 8b 04 02 99 8b eb 90 06 02 dd ad 9c d0 09 03 d9 e5 d5 d6 0a 04 bb f8 fd ed 0f 05
        6e 68
        6e 06
        6e 74
        6d 68
        6c 04 d1 a5 c6 9c 04 71 ae a4 9c f5 05 07 e0 e8 dc b5 0b 7e bb f8 fd ed 0f 05
        6e 78
        01 00
        01 01 65
        01 01 01 01
        00
        01 00 01 ff ff ff ff ff ff ff ff 00 01 01 01 04
        01 00 00 00 00 00 00 00 80
        02 01 01 01 01 01 04")
}
fn decode(bytes: &[u8]) -> Result<Vec<EngineView>, Error> {
    decode_get_engine_views_response(bytes, limits())
}

#[test]
fn selected_fields_preserve_absence_and_numeric_extremes() {
    let values = decode(&minimal()).unwrap();
    let view = &values[0];
    assert_eq!(view.id, "e");
    assert!(view.node_ids.is_empty());
    assert!(view.subnet.is_none());
    assert!(view.subnet_id.is_none());
    assert!(view.engine_operator_id.is_none());
    assert!(view.deleted_at.is_none());
    let values = decode(&full()).unwrap();
    let view = &values[0];
    assert_eq!(view.deleted_at, Some(i64::MIN));
    assert_eq!(view.subnet_id.unwrap().as_slice(), &[1]);
    assert_eq!(
        view.node_ids
            .iter()
            .map(|id| id.as_slice())
            .collect::<Vec<_>>(),
        [&[1][..], &[4][..]]
    );
    let subnet = view.subnet.as_ref().unwrap();
    assert_eq!(subnet.replica_version, "");
    assert_eq!(subnet.maximum_state_size, Some(u64::MAX));
    assert!(!subnet.halted);
    assert_eq!(subnet.node_ids[0].as_slice(), &[4]);
}

#[test]
fn every_truncated_prefix_refuses_and_trailing_bytes_refuse() {
    let bytes = full();
    for end in 0..bytes.len() {
        assert!(decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(decode(&trailing), Err(Error::Trailing));
}

#[test]
fn exact_arity_and_required_fields() {
    assert_eq!(decode(b"DIDL\0\0"), Err(Error::Arity));
    assert_eq!(decode(b"DIDL\0\x02\x7f\x7f"), Err(Error::Arity));
    let mut missing = minimal();
    // Change the id label to a valid unknown label; no id remains in the record.
    missing[10] = 0xda;
    assert_eq!(decode(&missing), Err(Error::Type));
}

#[test]
fn malformed_present_optional_does_not_become_absent() {
    // vec record { id:text; subnet:opt bool; node_ids:vec principal }.
    let frame = hex(
        "44 49 44 4c 04 6d 01 6c 03 db b7 01 71 dd ad 9c d0 09 03 bb f8 fd ed 0f 02 6d 68 6e 7e 01 00 01 01 65 01 01 00",
    );
    assert_eq!(decode(&frame), Err(Error::Type));
    let mut invalid_tag = frame;
    let tag = invalid_tag.len() - 3;
    invalid_tag[tag] = 2;
    assert_eq!(decode(&invalid_tag), Err(Error::Malformed));
}

fn with_unknown_text(text: &[u8]) -> Vec<u8> {
    // Field zero precedes both selected fields; the selected mapping cannot
    // accidentally manufacture the expected unknown-field encoding.
    let mut bytes =
        hex("44 49 44 4c 03 6d 01 6c 03 00 71 db b7 01 71 bb f8 fd ed 0f 02 6d 68 01 00 01");
    assert!(text.len() < 128);
    bytes.push(text.len() as u8);
    bytes.extend_from_slice(text);
    bytes.extend_from_slice(&[1, b'e', 0]);
    bytes
}

#[test]
fn skipped_text_is_validated_without_retaining_its_allocation() {
    assert_eq!(
        decode(&with_unknown_text(b"discard me")),
        decode(&minimal())
    );
    assert_eq!(decode(&with_unknown_text(&[0xff])), Err(Error::Malformed));
    let short = with_unknown_text(b"x");
    let long = with_unknown_text(&[b'x'; 100]);
    let min = minimum(&short, |l, n| l.allocation = n);
    let mut cap = limits();
    cap.allocation = min;
    assert!(decode_get_engine_views_response(&long, cap).is_ok());
}

fn minimum(bytes: &[u8], set: impl Fn(&mut Limits, usize)) -> usize {
    let mut low = 0;
    let mut high = 100_000;
    while low < high {
        let mid = low + (high - low) / 2;
        let mut cap = limits();
        set(&mut cap, mid);
        if decode_get_engine_views_response(bytes, cap).is_ok() {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    low
}

#[test]
fn exact_and_one_below_every_budget() {
    let bytes = full();
    let setters: [fn(&mut Limits, usize); 6] = [
        |l, n| l.bytes = n,
        |l, n| l.header = n,
        |l, n| l.types = n,
        |l, n| l.work = n,
        |l, n| l.allocation = n,
        |l, n| l.vector = n,
    ];
    for set in setters {
        let exact = minimum(&bytes, set);
        assert!(exact > 0 && exact < 100_000);
        let mut cap = limits();
        set(&mut cap, exact);
        assert!(decode_get_engine_views_response(&bytes, cap).is_ok());
        set(&mut cap, exact - 1);
        assert_eq!(
            decode_get_engine_views_response(&bytes, cap),
            Err(Error::Limit)
        );
    }
    let bytes = with_unknown_text(b"discard");
    let mut cap = limits();
    cap.skipping = 9; // value traversal + length + seven bytes
    assert!(decode_get_engine_views_response(&bytes, cap).is_ok());
    cap.skipping = 8;
    assert_eq!(
        decode_get_engine_views_response(&bytes, cap),
        Err(Error::Limit)
    );
    cap = limits();
    cap.skipping = 0;
    assert!(decode_get_engine_views_response(&minimal(), cap).is_ok());
}

#[test]
fn skipped_work_is_cumulative_and_consumes_total_work() {
    let mut twice = with_unknown_text(b"xx");
    // Make two records with unknown text; same table, two independent discards.
    let values = twice[26..].to_vec();
    twice[25] = 2;
    twice.extend_from_slice(&values);
    let mut cap = limits();
    cap.skipping = 7;
    assert_eq!(
        decode_get_engine_views_response(&twice, cap),
        Err(Error::Limit)
    );
    cap.skipping = 8;
    assert_eq!(
        decode_get_engine_views_response(&twice, cap).unwrap().len(),
        2
    );
    let needed = minimum(&twice, |l, n| l.work = n);
    cap.work = needed - 1;
    assert_eq!(
        decode_get_engine_views_response(&twice, cap),
        Err(Error::Limit)
    );
}

#[test]
fn malformed_type_tables_refuse_even_when_vector_is_empty() {
    let mut bad = minimal();
    bad[6] = 9; // vector element points outside table
    assert_eq!(decode(&bad), Err(Error::Malformed));
    let mut duplicate = minimal();
    // Two equally sized label encodings are not needed to exercise the parser.
    duplicate.splice(13..18, [0xdb, 0xb7, 1]);
    assert_eq!(decode(&duplicate), Err(Error::Malformed));
    assert_eq!(decode(b"DIDL\x01\x71\x01\x00"), Err(Error::Unsupported));
    assert_eq!(
        decode(b"DIDL\xff\xff\xff\xff\xff\xff\xff\xff\xff\x02"),
        Err(Error::Malformed)
    );
}

#[test]
fn invalid_principal_and_boolean_refuse() {
    let mut bytes = full();
    let size = bytes.len();
    bytes[size - 2] = 30; // principal length exceeds identity bound
    assert_eq!(decode(&bytes), Err(Error::Malformed));
    let mut bytes = full();
    // Find the end of the maximum-state-size fixed bytes: next byte is halted.
    let offset = bytes.windows(8).position(|v| v == [255; 8]).unwrap() + 8;
    bytes[offset] = 2;
    assert_eq!(decode(&bytes), Err(Error::Malformed));
}

fn nested_unknown(count: u8, cyclic: bool) -> Vec<u8> {
    let mut frame = b"DIDL".to_vec();
    frame.push(3 + count);
    frame.extend_from_slice(&hex(
        "6d 01 6c 03 00 03 db b7 01 71 bb f8 fd ed 0f 02 6d 68",
    ));
    for index in 0..count {
        frame.push(0x6e);
        frame.push(if index + 1 < count {
            4 + index
        } else if cyclic {
            3
        } else {
            0x7f
        });
    }
    frame.extend_from_slice(&[1, 0, 1]);
    frame.extend(std::iter::repeat_n(1, usize::from(count)));
    if cyclic {
        frame.push(0);
    }
    frame.extend_from_slice(&[1, b'e', 0]);
    frame
}

#[test]
fn constructed_depth_covers_skipped_types_and_values() {
    assert!(decode(&nested_unknown(14, false)).is_ok());
    assert_eq!(decode(&nested_unknown(15, false)), Err(Error::Limit));
    // One recursive definition has shallow type depth but deep runtime values.
    let mut frame = nested_unknown(1, true);
    let pos = frame.len() - 4;
    frame.splice(pos..pos, std::iter::repeat_n(1, 13));
    assert_eq!(decode(&frame), Err(Error::Limit));
}

#[test]
fn skips_validates_variants_and_reference_values() {
    // Unknown field uses a variant with one function arm; service and function
    // references must retain their transparent-reference tags and UTF-8 method.
    let mut frame = hex(
        "44 49 44 4c 05 6d 01 6c 03 00 03 db b7 01 71 bb f8 fd ed 0f 02 6d 68 6b 01 00 04 6a 00 00 01 01 01 00 01 00 01 01 01 04 01 6d 01 65 00",
    );
    assert!(decode(&frame).is_ok());
    let end = frame.len();
    frame[end - 4] = 0xff;
    assert_eq!(decode(&frame), Err(Error::Malformed));
}

#[test]
fn huge_zero_sized_skipped_vector_is_bounded() {
    let frame = hex(
        "44 49 44 4c 04 6d 01 6c 03 00 03 db b7 01 71 bb f8 fd ed 0f 02 6d 68 6d 7f 01 00 01 ff ff ff ff 0f 01 65 00",
    );
    assert_eq!(decode(&frame), Err(Error::Limit));
}

#[test]
fn zero_unsigned_and_maximum_signed_values_are_not_defaults() {
    let mut frame = full();
    let size_pos = frame.windows(8).position(|v| v == [255; 8]).unwrap();
    frame[size_pos..size_pos + 8].fill(0);
    let time_pos = frame.len() - 15;
    frame[time_pos..time_pos + 8].copy_from_slice(&i64::MAX.to_le_bytes());
    let views = decode(&frame).unwrap();
    assert_eq!(
        views[0].subnet.as_ref().unwrap().maximum_state_size,
        Some(0)
    );
    assert_eq!(views[0].deleted_at, Some(i64::MAX));
}

#[test]
fn discarded_integers_are_not_limited_to_machine_width() {
    for kind in [0x7d, 0x7c] {
        let mut frame = with_unknown_text(b"");
        frame[10] = kind;
        frame.splice(26..27, std::iter::repeat_n(0x80, 20).chain([0]));
        assert!(decode(&frame).is_ok());
        // Unterminated arbitrary-width value still refuses under input/work limits.
        frame.truncate(46);
        assert!(decode(&frame).is_err());
    }
}
