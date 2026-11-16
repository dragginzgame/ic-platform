//! Literal JSON contracts, independent of the Candid schema and encoder.

use candidate_types::Principal;
use ic_platform_protocol::cloud_engine::EngineView;

fn check(json: &str, accepted: bool) -> Option<EngineView> {
    let candidate = serde_json::from_str::<EngineView>(json);
    assert_eq!(candidate.is_ok(), accepted, "{json}: {candidate:?}");
    #[cfg(not(target_family = "wasm"))]
    {
        let published =
            serde_json::from_str::<legacy_platform::cloud_engine_types::EngineView>(json);
        assert_eq!(candidate.is_ok(), published.is_ok(), "admission: {json}");
        if let (Ok(new), Ok(old)) = (&candidate, &published) {
            assert_eq!(
                serde_json::to_value(new).unwrap(),
                serde_json::to_value(old).unwrap(),
                "selected values: {json}"
            );
        }
    }
    candidate.ok()
}

#[test]
fn literal_nested_values_preserve_numeric_and_identity_boundaries() {
    for timestamp in [i64::MIN, -1, 0, i64::MAX] {
        for maximum in [0, 1, u64::MAX] {
            let json = format!(
                r#"{{"id":" engine\u0000é ","subnet_id":"aaaaa-aa",
                "engine_operator_id":"2vxsx-fae","deleted_at":{timestamp},
                "node_ids":["2vxsx-fae","aaaaa-aa","2vxsx-fae"],
                "subnet":{{"replica_version":"","maximum_state_size":{maximum},
                "halted":false,"node_ids":["aaaaa-aa"]}}}}"#
            );
            let view = check(&json, true).unwrap();
            assert_eq!(view.id, " engine\0é ");
            assert_eq!(view.deleted_at, Some(timestamp));
            assert_eq!(view.subnet_id, Some(Principal::management_canister()));
            assert_eq!(view.engine_operator_id, Some(Principal::anonymous()));
            assert_eq!(
                view.node_ids,
                [
                    Principal::anonymous(),
                    Principal::management_canister(),
                    Principal::anonymous()
                ]
            );
            let subnet = view.subnet.as_ref().unwrap();
            assert_eq!(subnet.maximum_state_size, Some(maximum));
            assert_eq!(subnet.replica_version, "");
            assert!(!subnet.halted);
            assert_eq!(subnet.node_ids, [Principal::management_canister()]);
            let output = serde_json::to_string(&view).unwrap();
            // Require exact JSON integer tokens, not a lossy floating representation.
            assert!(output.contains(&format!("\"deleted_at\":{timestamp}")));
            assert!(output.contains(&format!("\"maximum_state_size\":{maximum}")));
            assert_eq!(check(&output, true).unwrap(), view);
        }
    }
}

#[test]
fn omitted_null_and_unknown_json_fields_retain_existing_policy() {
    let absent = check(r#"{"id":"","node_ids":[]}"#, true).unwrap();
    let explicit = check(r#"{"id":"","node_ids":[],"subnet_id":null,"engine_operator_id":null,"deleted_at":null,"subnet":null}"#, true).unwrap();
    assert_eq!(absent, explicit);
    let unknown = check(
        r#"{"id":"","node_ids":[],"future":{"items":[null,false,1,"x"]}}"#,
        true,
    )
    .unwrap();
    assert_eq!(absent, unknown);
    for optional in ["", ",\"maximum_state_size\":null"] {
        let json = format!(
            r#"{{"id":"x","node_ids":[],"subnet":{{"replica_version":"","halted":true,"node_ids":[]{optional}}}}}"#
        );
        let view = check(&json, true).unwrap();
        let subnet = view.subnet.unwrap();
        assert!(subnet.halted);
        assert_eq!(subnet.maximum_state_size, None);
    }
}

#[test]
fn malformed_selected_json_values_never_fall_back_to_absence() {
    for (field, values) in [
        (
            "subnet_id",
            &[r#""2vxsx-faa""#, r#""not-a-principal""#, "42", "[]"][..],
        ),
        ("engine_operator_id", &["false", "{}", r#""""#][..]),
        (
            "deleted_at",
            &[
                "9223372036854775808",
                "-9223372036854775809",
                "1.5",
                r#""0""#,
                "true",
            ][..],
        ),
        (
            "subnet",
            &[
                "false",
                "[]",
                "{}",
                r#"{"replica_version":"","node_ids":[]}"#,
            ][..],
        ),
    ] {
        for value in values {
            check(
                &format!(r#"{{"id":"x","node_ids":[],"{field}":{value}}}"#),
                false,
            );
        }
    }
    for value in ["-1", "18446744073709551616", "1.5", r#""0""#, "false"] {
        check(
            &format!(
                r#"{{"id":"x","node_ids":[],"subnet":{{"replica_version":"","halted":false,"node_ids":[],"maximum_state_size":{value}}}}}"#
            ),
            false,
        );
    }
    for json in [
        r#"{"node_ids":[]}"#,
        r#"{"id":"x"}"#,
        r#"{"id":null,"node_ids":[]}"#,
        r#"{"id":"x","node_ids":null}"#,
        r#"{"id":"x","node_ids":["2vxsx-faa"]}"#,
        r#"{"id":"x","node_ids":[[4]]}"#,
        r#"{"id":"x","node_ids":[],"id":"y"}"#,
        r#"{"id":"x","node_ids":[],"subnet_id":null,"subnet_id":null}"#,
        r#"{"id":"x","node_ids":[],"subnet":{"replica_version":"","halted":0,"node_ids":[]}}"#,
        r#"{"id":"x","node_ids":[],"subnet":{"replica_version":"","halted":false,"halted":true,"node_ids":[]}}"#,
        r#"{"id":"x","node_ids":[]} null"#,
    ] {
        check(json, false);
    }
}
