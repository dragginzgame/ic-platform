//! Composition fixture only: no transport, authentication or Query report policy.

use ic_platform_protocol::registry_settings::{self, SubnetSettings};
use legacy_platform::nns_registry_types::protobuf;

#[derive(Debug, PartialEq, Eq)]
enum Refusal<'a> {
    Envelope(protobuf::Error),
    Application(protobuf::RegistryError<'a>),
    MissingContent,
    Settings(registry_settings::Error),
}

#[derive(Debug, PartialEq, Eq)]
struct Projection {
    settings: SubnetSettings,
    mutation_version: u64,
    mutation_timestamp_nanoseconds: u64,
}

// Match the proposed consumer's separate envelope/payload admission budgets.
// Neither codec exposes a cross-codec remaining-budget token: these are not a
// shared 4,096-field allowance across both independently bounded operations.
fn project(bytes: &[u8]) -> Result<Projection, Refusal<'_>> {
    let response = protobuf::decode_get_value_response(
        bytes,
        protobuf::Limits {
            bytes: 64 * 1024,
            fields: 4096,
            allocation: 128 * 1024,
            items: 4096,
        },
    )
    .map_err(Refusal::Envelope)?;
    if let Some(error) = response.error {
        return Err(Refusal::Application(error));
    }
    let settings = registry_settings::decode_subnet_settings(
        response.value.ok_or(Refusal::MissingContent)?,
        registry_settings::Limits {
            bytes: 64 * 1024,
            fields: 4096,
        },
    )
    .map_err(Refusal::Settings)?;
    Ok(Projection {
        settings,
        mutation_version: response.version,
        mutation_timestamp_nanoseconds: response.timestamp_nanoseconds,
    })
}

#[test]
fn inline_settings_retain_mutation_evidence_and_presence() {
    // Envelope: version 7, inline features {HTTP true, SEV false}, timestamp 9.
    let bytes = [16, 7, 26, 7, 0xba, 1, 4, 0x18, 1, 0x48, 0, 40, 9];
    let value = project(&bytes).unwrap();
    assert_eq!(value.mutation_version, 7);
    assert_eq!(value.mutation_timestamp_nanoseconds, 9);
    let features = value.settings.features.unwrap();
    assert!(features.http_requests);
    assert_eq!(features.sev_enabled, Some(false));
    assert_eq!(value.settings.resource_limits, None);
    assert_eq!(project(&[]), Err(Refusal::MissingContent));
    assert_eq!(project(&[16, 7, 40, 9]), Err(Refusal::MissingContent));
    assert_eq!(
        project(&[26, 0]).unwrap().settings,
        SubnetSettings::default()
    );
    assert!(
        project(&[26, 3, 0xba, 1, 0])
            .unwrap()
            .settings
            .features
            .is_some()
    );
}

#[test]
fn application_errors_precede_payload_interpretation() {
    // Unknown application code 123, reason "x", opaque key 0xff; malformed
    // selected payload must not hide the source application error.
    let bytes = [10, 8, 8, 123, 18, 1, b'x', 26, 1, 255, 26, 1, 0];
    assert_eq!(
        project(&bytes),
        Err(Refusal::Application(protobuf::RegistryError {
            code: 123,
            reason: "x",
            key: &[255],
        }))
    );
    // Even a present empty error is an error, not a success discriminator.
    assert!(matches!(
        project(&[10, 0, 26, 0]),
        Err(Refusal::Application(_))
    ));
}

#[test]
fn envelope_and_payload_refusals_remain_distinct() {
    for (bytes, error) in [
        (&[34, 0][..], protobuf::Error::ChunkedValue),
        (&[26, 0, 34, 0][..], protobuf::Error::ChunkedValue),
        (&[26, 0, 26, 0][..], protobuf::Error::DuplicateField),
        (&[26, 2, 0][..], protobuf::Error::Malformed),
        (&[16, 0, 16, 0][..], protobuf::Error::DuplicateField),
    ] {
        assert_eq!(project(bytes), Err(Refusal::Envelope(error)));
    }
    for (bytes, error) in [
        (&[26, 1, 0][..], registry_settings::Error::Malformed),
        (
            &[26, 6, 0xc0, 1, 0, 0xc0, 1, 0][..],
            registry_settings::Error::DuplicateField,
        ),
        (
            &[26, 5, 0xba, 1, 2, 0x48, 2][..],
            registry_settings::Error::Malformed,
        ),
    ] {
        assert_eq!(project(bytes), Err(Refusal::Settings(error)));
    }
}

#[test]
fn composition_keeps_both_field_admission_boundaries() {
    let mut envelope = [8 * 6, 0].repeat(4095);
    envelope.extend_from_slice(&[26, 0]);
    assert!(project(&envelope).is_ok());
    envelope.extend_from_slice(&[8 * 6, 0]);
    assert_eq!(
        project(&envelope),
        Err(Refusal::Envelope(protobuf::Error::FieldLimit))
    );
    // One envelope field containing 4,097 payload fields exhausts only the
    // payload allowance. 8,194 bytes is encoded literally as 0x82, 0x40.
    let mut exact = vec![26, 0x80, 0x40];
    exact.extend_from_slice(&[8, 0].repeat(4096));
    assert!(project(&exact).is_ok());
    let mut nested = vec![26, 0x82, 0x40];
    nested.extend_from_slice(&[8, 0].repeat(4097));
    assert_eq!(
        project(&nested),
        Err(Refusal::Settings(registry_settings::Error::FieldLimit))
    );
    let oversized = vec![0; 64 * 1024 + 1];
    assert_eq!(
        project(&oversized),
        Err(Refusal::Envelope(protobuf::Error::ByteLimit))
    );
}
