use ic_platform_protocol::cloud_engine::AvailableNodesPageResult;
use serde_json::{Value, json};

#[test]
fn page_arguments_preserve_public_values_and_match_published_bytes() {
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
                let args: ListAvailableNodesPageArgs =
                    serde_json::from_value(json.clone()).unwrap();
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
                    use legacy_platform::cloud_engine_types as old;
                    let old_args: old::ListAvailableNodesPageArgs =
                        serde_json::from_value(json).unwrap();
                    assert_eq!(
                        old::encode_list_available_nodes_page_args(&old_args, 4096).unwrap(),
                        encoded
                    );
                    assert_eq!(
                        serde_json::to_value(old_args).unwrap(),
                        serde_json::to_value(args).unwrap()
                    );
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
        use legacy_platform::{candid_codec, cloud_engine_types as old};
        let published: old::AvailableNodesPageResult =
            serde_json::from_value(value.clone()).unwrap();
        let bytes = candid_codec::encode_one(&published, 65536).unwrap();
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
        let old_cap = candid_codec::decoder::Limits {
            bytes: cap.bytes,
            header: cap.header,
            types: cap.types,
            work: cap.work,
            allocation: cap.allocation,
        };
        assert_eq!(
            &serde_json::to_value(
                old::decode_list_available_nodes_page_response(&bytes, old_cap).unwrap()
            )
            .unwrap(),
            value
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
        assert!(serde_json::from_value::<AvailableNodesPageResult>(value.clone()).is_err());
        #[cfg(not(target_family = "wasm"))]
        assert!(
            serde_json::from_value::<legacy_platform::cloud_engine_types::AvailableNodesPageResult>(
                value
            )
            .is_err()
        );
    }
}
