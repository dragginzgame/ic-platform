//! Ledger-types-only consumer: the DTO does not select Platform codec APIs.
use ic_platform::icrc_ledger_types::icrc10::SupportedStandard;

/// Construct an external record without any direct Candid or Serde dependency.
pub fn standard(name: String, url: String) -> SupportedStandard {
    SupportedStandard { name, url }
}

#[test]
fn discovery_record_keeps_application_selected_values() {
    let value = standard("application standard".into(), "opaque URL text".into());
    assert_eq!(value.clone(), value);
    assert_eq!(value.name, "application standard");
    assert_eq!(value.url, "opaque URL text");
}
