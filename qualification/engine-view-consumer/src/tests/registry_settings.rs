//! Independent public protobuf/JSON contracts; no production encoder is used.

use ic_platform_protocol::registry_settings::{Limits, SubnetSettings, decode_subnet_settings};
use serde_json::json;

#[cfg(not(target_family = "wasm"))]
mod envelope;

fn decode(bytes: &[u8]) -> SubnetSettings {
    decode_subnet_settings(
        bytes,
        Limits {
            bytes: bytes.len(),
            fields: 32,
        },
    )
    .unwrap()
}

#[test]
fn protobuf_presence_survives_public_json_crossing() {
    for (bytes, expected) in [
        (
            &[][..],
            json!({"features": null, "max_number_of_canisters": 0, "resource_limits": null}),
        ),
        (
            &[0xba, 1, 0, 0x92, 2, 0][..],
            json!({
                "features": {"http_requests": false, "sev_enabled": null},
                "max_number_of_canisters": 0,
                "resource_limits": {"maximum_state_size": null, "maximum_state_delta": null,
                    "maximum_query_instructions": null, "maximum_query_walltime_seconds": null}
            }),
        ),
        (
            &[0xba, 1, 2, 0x48, 0, 0x92, 2, 8, 8, 0, 16, 0, 24, 0, 32, 0][..],
            json!({
                "features": {"http_requests": false, "sev_enabled": false},
                "max_number_of_canisters": 0,
                "resource_limits": {"maximum_state_size": 0, "maximum_state_delta": 0,
                    "maximum_query_instructions": 0, "maximum_query_walltime_seconds": 0}
            }),
        ),
    ] {
        let value = decode(bytes);
        assert_eq!(serde_json::to_value(value).unwrap(), expected);
        assert_eq!(
            serde_json::from_value::<SubnetSettings>(expected).unwrap(),
            value
        );
    }
}

#[test]
fn maximum_u64_is_an_exact_integer_in_every_public_slot() {
    let expected = r#"{"features":null,"max_number_of_canisters":18446744073709551615,"resource_limits":{"maximum_state_size":18446744073709551615,"maximum_state_delta":18446744073709551615,"maximum_query_instructions":18446744073709551615,"maximum_query_walltime_seconds":18446744073709551615}}"#;
    let value: SubnetSettings = serde_json::from_str(expected).unwrap();
    assert_eq!(value.max_number_of_canisters, u64::MAX);
    let limits = value.resource_limits.unwrap();
    assert_eq!(limits.maximum_state_size, Some(u64::MAX));
    assert_eq!(limits.maximum_state_delta, Some(u64::MAX));
    assert_eq!(limits.maximum_query_instructions, Some(u64::MAX));
    assert_eq!(limits.maximum_query_walltime_seconds, Some(u64::MAX));
    assert_eq!(serde_json::to_string(&value).unwrap(), expected);
}

#[test]
fn json_omission_and_unknown_fields_have_explicit_semantics() {
    // JSON requires nonoptional scalars; protobuf absence supplies scalar defaults.
    assert!(serde_json::from_str::<SubnetSettings>("{}").is_err());
    assert_eq!(
        serde_json::from_str::<SubnetSettings>(r#"{"max_number_of_canisters":0}"#).unwrap(),
        decode(&[])
    );
    let absent =
        r#"{"max_number_of_canisters":0,"features":{"http_requests":false},"resource_limits":{}}"#;
    assert_eq!(
        serde_json::from_str::<SubnetSettings>(absent).unwrap(),
        decode(&[0xba, 1, 0, 0x92, 2, 0])
    );
    let future = r#"{"max_number_of_canisters":0,"future":[{},null],"features":{"http_requests":false,"future":1},"resource_limits":{"future":true}}"#;
    assert_eq!(
        serde_json::from_str::<SubnetSettings>(future).unwrap(),
        serde_json::from_str::<SubnetSettings>(absent).unwrap()
    );
}

#[test]
fn invalid_json_values_and_duplicates_refuse() {
    for bad in ["-1", "18446744073709551616", "1.5", "true", r#""0""#] {
        let json = format!(r#"{{"max_number_of_canisters":{bad}}}"#);
        assert!(
            serde_json::from_str::<SubnetSettings>(&json).is_err(),
            "{json}"
        );
        for field in [
            "maximum_state_size",
            "maximum_state_delta",
            "maximum_query_instructions",
            "maximum_query_walltime_seconds",
        ] {
            let json =
                format!(r#"{{"max_number_of_canisters":0,"resource_limits":{{"{field}":{bad}}}}}"#);
            assert!(
                serde_json::from_str::<SubnetSettings>(&json).is_err(),
                "{json}"
            );
        }
    }
    for body in [
        r#""max_number_of_canisters":null"#,
        r#""max_number_of_canisters":0,"max_number_of_canisters":0"#,
        r#""max_number_of_canisters":0,"features":null,"features":null"#,
        r#""max_number_of_canisters":0,"resource_limits":null,"resource_limits":null"#,
        r#""max_number_of_canisters":0,"features":{}"#,
        r#""max_number_of_canisters":0,"features":{"http_requests":0}"#,
        r#""max_number_of_canisters":0,"features":{"http_requests":false,"sev_enabled":0}"#,
        r#""max_number_of_canisters":0,"features":{"http_requests":false,"http_requests":false}"#,
        r#""max_number_of_canisters":0,"features":{"http_requests":false,"sev_enabled":null,"sev_enabled":null}"#,
    ] {
        let json = format!("{{{body}}}");
        assert!(
            serde_json::from_str::<SubnetSettings>(&json).is_err(),
            "{json}"
        );
    }
    for field in [
        "maximum_state_size",
        "maximum_state_delta",
        "maximum_query_instructions",
        "maximum_query_walltime_seconds",
    ] {
        let json = format!(
            r#"{{"max_number_of_canisters":0,"resource_limits":{{"{field}":null,"{field}":null}}}}"#
        );
        assert!(
            serde_json::from_str::<SubnetSettings>(&json).is_err(),
            "{json}"
        );
    }
}
