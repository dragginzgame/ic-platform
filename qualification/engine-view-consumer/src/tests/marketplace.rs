use ic_platform_protocol::candid::Nat;

mod availability;

#[test]
fn natural_json_forms_preserve_exact_values_and_refuse_bad_inputs() {
    for (input, expected) in [
        ("0", "0"),
        ("18446744073709551615", "18446744073709551615"),
        (
            r#""1393796574908163946345982392040522594123776""#,
            "1393796574908163946345982392040522594123776",
        ),
        (
            "[0,0,0,0,4096]",
            "1393796574908163946345982392040522594123776",
        ),
    ] {
        let value: Nat = serde_json::from_str(input).unwrap();
        assert_eq!(value.0.to_str_radix(10), expected);
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<Nat>(&json).unwrap(), value);
        #[cfg(not(target_family = "wasm"))]
        {
            let old: ic_platform::candid_codec::Nat = serde_json::from_str(input).unwrap();
            assert_eq!(old.0, value.0);
            assert_eq!(serde_json::to_string(&old).unwrap(), json);
        }
    }
    for input in [
        "-1",
        "1.5",
        "true",
        "null",
        "[4294967296]",
        "[-1]",
        r#""-1""#,
        r#""no""#,
    ] {
        assert!(serde_json::from_str::<Nat>(input).is_err());
        #[cfg(not(target_family = "wasm"))]
        assert!(serde_json::from_str::<ic_platform::candid_codec::Nat>(input).is_err());
    }
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn marketplace_matches_published_records_across_all_classes() {
    use ic_platform::{candid_codec, cloud_engine_types as old};
    use ic_platform_protocol::{candid, cloud_engine as new};
    use ic_platform_types::Principal;
    let cap = candid::Limits {
        bytes: 65536,
        header: 4096,
        types: 32,
        work: 100_000,
        allocation: 65536,
        vector: 64,
        skipping: 4096,
    };
    for (index, class) in [
        old::NodeType::Type4,
        old::NodeType::Type4_1,
        old::NodeType::Type4_2,
        old::NodeType::Type4_3,
        old::NodeType::Type4_4,
        old::NodeType::Type4_5,
    ]
    .into_iter()
    .enumerate()
    {
        let old_rows = vec![old::MarketplaceEntry {
            key: " opaque\0é ".into(),
            entry: old::PriceEntry {
                dc: (index % 2 == 0).then(String::new),
                updated_at: if index % 2 == 0 { i64::MIN } else { i64::MAX },
                node_type: class,
                provider: (index % 2 == 0).then(Principal::anonymous),
                price: old::Price {
                    gross_cycles: "9".repeat(256).parse().unwrap(),
                    net_cycles: 0u64.into(),
                },
            },
        }];
        let bytes = candid_codec::encode_one(&old_rows, cap.bytes).unwrap();
        let rows = new::decode_list_marketplace_prices_response(&bytes, cap).unwrap();
        assert_eq!(
            rows[0].entry.price.gross_cycles.0.to_str_radix(10),
            "9".repeat(256)
        );
        assert_eq!(
            serde_json::to_value(&rows).unwrap(),
            serde_json::to_value(&old_rows).unwrap()
        );
        let decoded: Vec<new::MarketplaceEntry> =
            serde_json::from_value(serde_json::to_value(&rows).unwrap()).unwrap();
        assert_eq!(decoded, rows);
        let old_cap = candid_codec::decoder::Limits {
            bytes: cap.bytes,
            header: cap.header,
            types: cap.types,
            work: cap.work,
            allocation: cap.allocation,
        };
        let roundtrip = old::decode_list_marketplace_prices_response(&bytes, old_cap).unwrap();
        assert_eq!(
            serde_json::to_value(roundtrip).unwrap(),
            serde_json::to_value(rows).unwrap()
        );
    }
}
