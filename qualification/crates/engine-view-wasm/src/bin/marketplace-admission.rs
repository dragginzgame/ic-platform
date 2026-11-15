//! Executed standalone marketplace/page corpus, independent of size acceptance.

use ic_platform_protocol::{
    candid::{Error, Limits},
    cloud_engine::{
        self as engine, AvailableNodesPageResult as Reply, EngineError, ListAvailableNodesPageArgs,
        NodeType,
    },
};

fn limits() -> Limits {
    Limits {
        bytes: 65536,
        header: 4096,
        types: 64,
        work: 100_000,
        allocation: 65536,
        vector: 64,
        skipping: 4096,
    }
}

fn hex(text: &str) -> Vec<u8> {
    text.split_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).unwrap())
        .collect()
}

// Literal schemas independently assembled from the selected #166 declaration.
const PRICES: &str = "44 49 44 4c 07 6d 01 6c 02 9f 93 c6 02 71 f2 f0 99 84 07 02 6c 05 ff ae 01 03 b7 ff f5 81 01 74 d7 fa 89 89 01 04 b1 e2 e7 99 02 05 89 9d ad c4 0c 06 6e 71 6b 06 ba e0 ad d6 01 7f ac 8b cf c5 0f 7f ad 8b cf c5 0f 7f ae 8b cf c5 0f 7f af 8b cf c5 0f 7f b0 8b cf c5 0f 7f 6e 68 6c 02 a8 db 82 e4 09 7d ef 97 90 a9 0b 7d 01 00";
const PAGE: &str = "44 49 44 4c 09 6b 02 9c c2 01 01 e5 8e b4 02 08 6e 02 6c 03 f3 89 8a c8 04 03 d6 ee f5 a0 06 78 b1 f0 fa fd 09 04 6e 68 6d 05 6c 07 ff ae 01 06 d7 fa 89 89 01 07 b8 aa cc b7 01 68 b1 e2 e7 99 02 68 a4 b7 cc a3 04 68 a6 fb ec a3 0b 7e 9c cc b3 f7 0f 71 6e 71 6b 06 ba e0 ad d6 01 7f ac 8b cf c5 0f 7f ad 8b cf c5 0f 7f ae 8b cf c5 0f 7f af 8b cf c5 0f 7f b0 8b cf c5 0f 7f 6b 09 dd c6 a0 11 71 ec ea a7 7f 71 cc ce f0 c6 06 71 bd b1 98 ef 06 71 cf a0 de f2 06 7f d4 b4 c5 9a 09 7f f4 f6 d8 a2 09 7f e1 8b d8 8a 0c 7f aa 98 d9 d1 0e 71 01 00";

fn prices() {
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
        let mut bytes = hex(PRICES);
        bytes.extend_from_slice(&[1, 1, b'k', 1, 0]);
        bytes.extend_from_slice(&i64::MIN.to_le_bytes());
        bytes.extend_from_slice(&[index as u8, 0]);
        bytes.extend_from_slice(&[128; 20]);
        bytes.extend_from_slice(&[1, 0]);
        let rows =
            engine::decode_list_marketplace_prices_response(std::hint::black_box(&bytes), limits())
                .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].entry.node_type, class);
        assert_eq!(rows[0].entry.updated_at, i64::MIN);
        assert_eq!(rows[0].entry.dc.as_deref(), Some(""));
        assert!(rows[0].entry.provider.is_none());
        assert_eq!(
            rows[0].entry.price.gross_cycles.0.to_str_radix(10),
            "1393796574908163946345982392040522594123776"
        );
        assert_eq!(rows[0].entry.price.net_cycles.0.to_str_radix(10), "0");
        for end in 0..bytes.len() {
            assert_eq!(
                engine::decode_list_marketplace_prices_response(&bytes[..end], limits()),
                Err(Error::Malformed)
            );
        }
        let mut cap = limits();
        cap.allocation = 0;
        assert_eq!(
            engine::decode_list_marketplace_prices_response(&bytes, cap),
            Err(Error::Limit)
        );
    }
    let mut oversized = hex(PRICES);
    oversized.extend_from_slice(&[128, 128, 128, 128, 16]); // 2^32 rows must not wrap on Wasm
    assert_eq!(
        engine::decode_list_marketplace_prices_response(&oversized, limits()),
        Err(Error::Limit)
    );
}

fn reply(payload: &[u8], cap: Limits) -> Result<Reply, Error> {
    let mut bytes = hex(PAGE);
    bytes.extend_from_slice(payload);
    engine::decode_list_available_nodes_page_response(std::hint::black_box(&bytes), cap)
}

fn pages() {
    assert_eq!(reply(&[0, 0], limits()), Ok(Reply::Ok(None)));
    let mut empty = vec![0, 1, 0];
    empty.extend_from_slice(&0u64.to_le_bytes());
    empty.push(0);
    let Reply::Ok(Some(page)) = reply(&empty, limits()).unwrap() else {
        panic!("expected empty page")
    };
    assert!(page.nodes.is_empty());
    assert!(page.next.is_none());
    let mut payload = vec![0, 1, 1, 1, 1, 4];
    payload.extend_from_slice(&u64::MAX.to_le_bytes());
    payload.extend_from_slice(&[1, 1, 0, 5, 1, 1, 4, 1, 0, 1, 1, 4, 1, 2, b'i', b'p']);
    let Reply::Ok(Some(page)) = reply(&payload, limits()).unwrap() else {
        panic!("expected present page")
    };
    assert_eq!(page.registry_version, u64::MAX);
    assert_eq!(page.next.unwrap().as_slice(), &[4]);
    assert_eq!(page.nodes.len(), 1);
    assert_eq!(page.nodes[0].node_type, NodeType::Type4_5);
    assert_eq!(page.nodes[0].provider.as_slice(), &[]);
    assert!(page.nodes[0].sev_enabled);
    let mut cap = limits();
    cap.skipping = 4;
    assert!(reply(&payload, cap).is_ok());
    cap.skipping = 3;
    assert_eq!(reply(&payload, cap), Err(Error::Limit));
    for field in 0..6 {
        let mut cap = limits();
        match field {
            0 => cap.bytes = 1,
            1 => cap.header = 1,
            2 => cap.types = 0,
            3 => cap.work = 0,
            4 => cap.allocation = 0,
            _ => cap.vector = 0,
        }
        assert_eq!(reply(&payload, cap), Err(Error::Limit));
    }
    for end in 0..payload.len() {
        assert_eq!(reply(&payload[..end], limits()), Err(Error::Malformed));
    }
    let mut unknown = hex(PAGE);
    let offset = unknown
        .windows(6)
        .position(|w| w == [0xb0, 0x8b, 0xcf, 0xc5, 0x0f, 0x7f])
        .unwrap();
    unknown[offset] += 1;
    unknown.extend_from_slice(&payload);
    assert_eq!(
        engine::decode_list_available_nodes_page_response(&unknown, limits()),
        Err(Error::Type)
    );
    *payload.last_mut().unwrap() = 255;
    assert_eq!(reply(&payload, limits()), Err(Error::Malformed));
    for (index, error) in [
        EngineError::Internal("x".into()),
        EngineError::ControllerError("x".into()),
        EngineError::InvalidTransition("x".into()),
        EngineError::SlugConflict("x".into()),
        EngineError::NotFound,
        EngineError::Unauthorized,
        EngineError::AlreadyExists,
        EngineError::NotVetted,
        EngineError::BadRequest("x".into()),
    ]
    .into_iter()
    .enumerate()
    {
        let mut payload = vec![1, index as u8];
        if !matches!(index, 4..=7) {
            payload.extend_from_slice(&[1, b'x']);
        }
        assert_eq!(reply(&payload, limits()), Ok(Reply::Err(error)));
    }
    assert_eq!(reply(&[1, 9], limits()), Err(Error::Malformed));
    assert_eq!(reply(&[0, 2], limits()), Err(Error::Malformed));
    assert_eq!(
        engine::decode_list_available_nodes_page_response(b"DIDL\0\0", limits()),
        Err(Error::Arity)
    );
}

fn requests() {
    for (limit, digits) in [
        ("0", vec![0]),
        (
            "1393796574908163946345982392040522594123776",
            [vec![128; 20], vec![1]].concat(),
        ),
    ] {
        let args = ListAvailableNodesPageArgs {
            limit: Some(limit.parse().unwrap()),
            start: None,
            rejoining: Some("".into()),
        };
        let mut expected = hex(
            "44 49 44 4c 04 6c 03 bb be 84 a8 07 01 e2 e8 ad a0 08 02 e5 df 81 c0 0d 03 6e 7d 6e 68 6e 71 01 00",
        );
        expected.push(1);
        expected.extend(digits);
        expected.extend_from_slice(&[0, 1, 0]);
        assert_eq!(
            engine::encode_list_available_nodes_page_args(
                std::hint::black_box(&args),
                expected.len()
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            engine::encode_list_available_nodes_page_args(&args, expected.len() - 1),
            Err(engine::EncodeError::OutputLimit {
                max_bytes: expected.len() - 1
            })
        );
    }
}

fn observations() {
    for bits in [
        0u64,
        1 << 63,
        0.25f64.to_bits(),
        f64::INFINITY.to_bits(),
        0x7ff8_0000_0000_1234,
    ] {
        let mut bytes = b"DIDL\0\x01\x72".to_vec();
        bytes.extend_from_slice(&bits.to_le_bytes());
        assert_eq!(
            engine::decode_get_network_fee_response(std::hint::black_box(&bytes), limits())
                .unwrap()
                .to_bits(),
            bits
        );
    }
    let mut bytes =
        hex("44 49 44 4c 01 6c 03 c7 8a 8c ab 05 78 98 ce c7 e7 07 78 fe b4 af f1 09 7e 01 00");
    bytes.extend_from_slice(&u64::MAX.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes.push(1);
    let status =
        engine::decode_registry_copy_status_response(std::hint::black_box(&bytes), limits())
            .unwrap();
    assert_eq!(
        (status.latest, status.version, status.caught_up),
        (u64::MAX, 0, true)
    );
}

fn main() {
    prices();
    pages();
    requests();
    observations();
}
