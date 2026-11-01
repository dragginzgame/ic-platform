use super::RejectCode;
use serde_test::{Token, assert_tokens};

#[test]
fn serde_variant_names_stay_external_compatible() {
    for (code, variant) in [
        (RejectCode::SysFatal, "SysFatal"),
        (RejectCode::SysTransient, "SysTransient"),
        (RejectCode::DestinationInvalid, "DestinationInvalid"),
        (RejectCode::CanisterReject, "CanisterReject"),
        (RejectCode::CanisterError, "CanisterError"),
        (RejectCode::SysUnknown, "SysUnknown"),
    ] {
        assert_tokens(
            &code,
            &[Token::UnitVariant {
                name: "RejectCode",
                variant,
            }],
        );
    }
}
