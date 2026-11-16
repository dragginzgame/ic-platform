use super::{hex, limits};
use crate::{
    candid::{self, Error, Nat},
    cloud_engine::{NodeType, decode_list_marketplace_prices_response as decode},
};

// Independent binary grammar: vec record {key:text; entry:record {dc:opt text;
// updated_at:int64; node_type:variant {six null cases}; provider:opt principal;
// price:record {gross_cycles:nat; net_cycles:nat}}}. No production encoder.
const HEADER: &str = "44 49 44 4c 07 6d 01 6c 02 9f 93 c6 02 71 f2 f0 99 84 07 02 6c 05 ff ae 01 03 b7 ff f5 81 01 74 d7 fa 89 89 01 04 b1 e2 e7 99 02 05 89 9d ad c4 0c 06 6e 71 6b 06 ba e0 ad d6 01 7f ac 8b cf c5 0f 7f ad 8b cf c5 0f 7f ae 8b cf c5 0f 7f af 8b cf c5 0f 7f b0 8b cf c5 0f 7f 6e 68 6c 02 a8 db 82 e4 09 7d ef 97 90 a9 0b 7d 01 00";

fn frame(class: u8, time: i64, present: bool) -> Vec<u8> {
    let mut bytes = hex(HEADER);
    bytes.extend_from_slice(&[1, 1, b'k']); // one row, opaque key
    if present {
        bytes.extend_from_slice(&[1, 0]);
    } else {
        bytes.push(0);
    }
    bytes.extend_from_slice(&time.to_le_bytes());
    bytes.push(class);
    if present {
        bytes.extend_from_slice(&[1, 1, 1, 4]);
    } else {
        bytes.push(0);
    }
    bytes.extend_from_slice(&[0x80; 20]);
    bytes.extend_from_slice(&[1, 0]); // gross 2^140, net zero
    bytes
}

#[test]
fn six_classes_large_naturals_and_presence_are_preserved() {
    for (index, class) in [
        NodeType::Type4,
        NodeType::Type4_1,
        NodeType::Type4_2,
        NodeType::Type4_3,
        NodeType::Type4_4,
        NodeType::Type4_5,
    ]
    .into_iter()
    .enumerate()
    {
        for time in [i64::MIN, -1, 0, i64::MAX] {
            for present in [false, true] {
                let rows = decode(&frame(index as u8, time, present), limits()).unwrap();
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].key, "k");
                let row = &rows[0].entry;
                assert_eq!(row.node_type, class);
                assert_eq!(row.updated_at, time);
                assert_eq!(row.dc.as_deref(), present.then_some(""));
                assert_eq!(
                    row.provider,
                    present.then_some(candidate_types::Principal::anonymous())
                );
                assert_eq!(
                    row.price.gross_cycles.0.to_str_radix(10),
                    "1393796574908163946345982392040522594123776"
                );
                assert_eq!(row.price.net_cycles, Nat::from(0));
            }
        }
    }
}

#[test]
fn malformed_marketplace_and_unknown_classes_refuse() {
    let bytes = frame(0, 0, false);
    for end in 0..bytes.len() {
        assert_eq!(decode(&bytes[..end], limits()), Err(Error::Malformed));
    }
    assert_eq!(decode(&frame(6, 0, false), limits()), Err(Error::Malformed));
    let mut unknown = frame(5, 0, false);
    // Change the final type4_5 label to an unrecognized adjacent label.
    let offset = unknown
        .windows(6)
        .position(|w| w == [0xb0, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f])
        .unwrap();
    unknown[offset] += 1;
    assert_eq!(decode(&unknown, limits()), Err(Error::Type));
    let mut wrong_payload = frame(0, 0, false);
    let offset = wrong_payload
        .windows(6)
        .position(|w| w == [0xba, 0xe0, 0xad, 0xd6, 1, 0x7f])
        .unwrap();
    wrong_payload[offset + 5] = 0x7e;
    assert_eq!(decode(&wrong_payload, limits()), Err(Error::Type));
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(decode(&trailing, limits()), Err(Error::Trailing));
    assert_eq!(decode(b"DIDL\0\0", limits()), Err(Error::Arity));
    assert_eq!(decode(b"DIDL\0\x02\x7f\x7f", limits()), Err(Error::Arity));
    for budget in 0..6 {
        let mut cap = limits();
        match budget {
            0 => cap.bytes = bytes.len() - 1,
            1 => cap.header = 107,
            2 => cap.types = 6,
            3 => cap.work = 0,
            4 => cap.allocation = 0,
            _ => cap.vector = 0,
        }
        assert_eq!(decode(&bytes, cap), Err(Error::Limit));
    }
}

#[test]
fn natural_conversion_is_metered_and_accepts_bounded_nonminimal_values() {
    let bytes = b"DIDL\0\x01\x7d\0";
    let mut cap = limits();
    cap.allocation = 9; // one radix digit plus one rounded 64-bit magnitude word
    cap.work = 10; // seven header bytes, value traversal, byte read, conversion
    assert_eq!(candid::decode::<Nat>(bytes, cap).unwrap(), Nat::from(0));
    cap.allocation = 8;
    assert_eq!(candid::decode::<Nat>(bytes, cap), Err(Error::Limit));
    cap.allocation = 9;
    cap.work = 9;
    assert_eq!(candid::decode::<Nat>(bytes, cap), Err(Error::Limit));
    assert_eq!(
        candid::decode::<Nat>(b"DIDL\0\x01\x7d\x80\0", limits()).unwrap(),
        Nat::from(0)
    );
    assert_eq!(
        candid::decode::<Nat>(b"DIDL\0\x01\x7d\x80", limits()),
        Err(Error::Malformed)
    );
    assert_eq!(
        candid::decode::<Nat>(b"DIDL\0\x01\x7c\0", limits()),
        Err(Error::Type)
    );
}
