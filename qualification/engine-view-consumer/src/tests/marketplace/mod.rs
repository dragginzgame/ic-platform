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
    }
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn marketplace_matches_independent_schema_across_all_classes() {
    use crate::fixtures as wire;
    use ::candid::Principal;
    use ic_platform_protocol::{candid, cloud_engine as new};
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
        wire::NodeClass::Type4,
        wire::NodeClass::Type4_1,
        wire::NodeClass::Type4_2,
        wire::NodeClass::Type4_3,
        wire::NodeClass::Type4_4,
        wire::NodeClass::Type4_5,
    ]
    .into_iter()
    .enumerate()
    {
        let wire_rows = vec![wire::MarketplaceEntry {
            key: " opaque\0é ".into(),
            entry: wire::PriceEntry {
                dc: (index % 2 == 0).then(String::new),
                updated_at: if index % 2 == 0 { i64::MIN } else { i64::MAX },
                node_type: class,
                provider: (index % 2 == 0).then(Principal::anonymous),
                price: wire::Price {
                    gross_cycles: "9".repeat(256).parse().unwrap(),
                    net_cycles: 0u64.into(),
                },
            },
        }];
        let bytes = ::candid::encode_one(&wire_rows).unwrap();
        let rows = new::decode_list_marketplace_prices_response(&bytes, cap).unwrap();
        assert_eq!(
            rows[0].entry.price.gross_cycles.0.to_str_radix(10),
            "9".repeat(256)
        );
        assert_eq!(rows[0].key, " opaque\0é ");
        assert_eq!(rows[0].entry.dc, (index % 2 == 0).then(String::new));
        assert_eq!(
            rows[0].entry.updated_at,
            if index % 2 == 0 { i64::MIN } else { i64::MAX }
        );
        assert_eq!(
            serde_json::to_value(rows[0].entry.node_type).unwrap(),
            [
                "type4", "type4_1", "type4_2", "type4_3", "type4_4", "type4_5"
            ][index]
        );
        assert_eq!(
            rows[0].entry.provider.map(|p| p.as_slice().to_vec()),
            (index % 2 == 0).then(|| vec![4])
        );
        assert_eq!(rows[0].entry.price.net_cycles.0.to_str_radix(10), "0");
        let decoded: Vec<new::MarketplaceEntry> =
            serde_json::from_value(serde_json::to_value(&rows).unwrap()).unwrap();
        assert_eq!(decoded, rows);
    }
}
