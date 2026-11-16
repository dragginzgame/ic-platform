use ic_platform_protocol::cloud_engine::AvailableNodesPageResult;
use serde_json::{Value, json};

#[test]
fn page_arguments_preserve_public_values_and_match_independent_schema_bytes() {
    use ic_platform_protocol::cloud_engine::{
        ListAvailableNodesPageArgs, encode_list_available_nodes_page_args,
    };
    for limit in [
        Value::Null,
        json!("0"),
        json!("128"),
        json!("1393796574908163946345982392040522594123776"),
        json!("9".repeat(256)),
    ] {
        for start in [Value::Null, json!("aaaaa-aa"), json!("2vxsx-fae")] {
            for rejoining in [Value::Null, json!(""), json!(" \0é ".repeat(40))] {
                let json = json!({"limit": limit, "start": start, "rejoining": rejoining});
                let args: ListAvailableNodesPageArgs = serde_json::from_value(json).unwrap();
                let encoded = encode_list_available_nodes_page_args(&args, 4096).unwrap();
                assert_eq!(
                    encode_list_available_nodes_page_args(&args, encoded.len()).unwrap(),
                    encoded
                );
                assert!(encode_list_available_nodes_page_args(&args, encoded.len() - 1).is_err());
                let restored: ListAvailableNodesPageArgs =
                    serde_json::from_value(serde_json::to_value(&args).unwrap()).unwrap();
                assert_eq!(restored, args);
                #[cfg(not(target_family = "wasm"))]
                {
                    let wire = crate::fixtures::PageArgs {
                        limit: args
                            .limit
                            .as_ref()
                            .map(|n| n.0.to_str_radix(10).parse::<::candid::Nat>().unwrap()),
                        start: args
                            .start
                            .map(|p| ::candid::Principal::from_slice(p.as_slice())),
                        rejoining: args.rejoining,
                    };
                    assert_eq!(::candid::encode_one(&wire).unwrap(), encoded);
                }
            }
        }
    }
}

fn fixture(value: &Value) {
    let selected: AvailableNodesPageResult = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(&serde_json::to_value(&selected).unwrap(), value);
    #[cfg(not(target_family = "wasm"))]
    {
        use ic_platform_protocol::{
            candid::Limits, cloud_engine::decode_list_available_nodes_page_response,
        };
        let wire: crate::fixtures::PageResult = serde_json::from_value(value.clone()).unwrap();
        let bytes = ::candid::encode_one(&wire).unwrap();
        let cap = Limits {
            bytes: 65536,
            header: 4096,
            types: 64,
            work: 65536,
            allocation: 65536,
            vector: 16,
            skipping: 4096,
        };
        assert_eq!(
            decode_list_available_nodes_page_response(&bytes, cap).unwrap(),
            selected
        );
    }
}

#[test]
fn result_tags_errors_and_page_presence_match_public_contract() {
    fixture(&json!({"ok": null}));
    fixture(&json!({"ok": {"next": null, "registry_version": 0, "nodes": []}}));
    for class in [
        "type4", "type4_1", "type4_2", "type4_3", "type4_4", "type4_5",
    ] {
        fixture(
            &json!({"ok": {"next": "2vxsx-fae", "registry_version": u64::MAX, "nodes": [{
                "dc": "", "node_type": class, "node_id": "2vxsx-fae", "provider": "aaaaa-aa", "operator": "2vxsx-fae", "sev_enabled": true
            }]}}),
        );
    }
    for error in ["NotFound", "Unauthorized", "AlreadyExists", "NotVetted"] {
        fixture(&json!({"err": error}));
    }
    for error in [
        "Internal",
        "ControllerError",
        "InvalidTransition",
        "SlugConflict",
        "BadRequest",
    ] {
        fixture(&json!({"err": {error: " source\0é "}}));
    }
}

#[test]
fn malformed_json_cannot_become_unready() {
    for value in [
        json!({"Ok": null}),
        json!({"ok": {}}),
        json!({"err": "Future"}),
        json!({"err": {"Internal": false}}),
        json!({"ok": false}),
        json!({"ok": {"next": "bad principal", "registry_version": 0, "nodes": []}}),
        json!({"ok": {"next": null, "registry_version": 0, "nodes": [{
            "dc": null, "node_type": "unknown", "node_id": "2vxsx-fae", "provider": "aaaaa-aa", "operator": "2vxsx-fae", "sev_enabled": true
        }]}}),
    ] {
        assert!(serde_json::from_value::<AvailableNodesPageResult>(value).is_err());
    }
}
