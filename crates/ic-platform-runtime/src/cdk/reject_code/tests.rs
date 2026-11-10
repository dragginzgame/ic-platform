use super::{RejectCode, UnknownRejectCode};
use serde::Deserialize;
use serde::de::value::{Error, StrDeserializer};
use serde_test::{Token, assert_de_tokens, assert_tokens};

const CASES: [(RejectCode, u64, &str, &str, u32); 6] = [
    (RejectCode::SysFatal, 1, "SYS_FATAL", "SysFatal", 0),
    (
        RejectCode::SysTransient,
        2,
        "SYS_TRANSIENT",
        "SysTransient",
        1,
    ),
    (
        RejectCode::DestinationInvalid,
        3,
        "DESTINATION_INVALID",
        "DestinationInvalid",
        2,
    ),
    (
        RejectCode::CanisterReject,
        4,
        "CANISTER_REJECT",
        "CanisterReject",
        3,
    ),
    (
        RejectCode::CanisterError,
        5,
        "CANISTER_ERROR",
        "CanisterError",
        4,
    ),
    (RejectCode::SysUnknown, 6, "SYS_UNKNOWN", "SysUnknown", 5),
];

#[test]
fn protocol_numbers_and_names_match_the_catalogue() {
    for (code, number, label, variant, _) in CASES {
        assert_eq!(RejectCode::try_from(number), Ok(code));
        assert_eq!(code as u64, number);
        assert_eq!(code.to_string(), label);
        assert_eq!(format!("{code:?}"), variant);
    }
}

#[test]
fn unsupported_numbers_preserve_the_entire_input() {
    let boundary = 1_u64 << 32;
    for number in [
        0,
        7,
        8,
        boundary - 2,
        boundary - 1,
        boundary,
        boundary + 1,
        boundary + 2,
        boundary + 3,
        boundary + 4,
        boundary + 5,
        boundary + 6,
        u64::MAX,
    ] {
        assert_eq!(RejectCode::try_from(number), Err(UnknownRejectCode(number)));
    }
}

#[test]
fn serde_retains_unit_variants_and_zero_based_indices() {
    for (code, _, _, variant, index) in CASES {
        assert_tokens(
            &code,
            &[Token::UnitVariant {
                name: "RejectCode",
                variant,
            }],
        );
        assert_de_tokens(
            &code,
            &[
                Token::Enum { name: "RejectCode" },
                Token::U32(index),
                Token::Unit,
            ],
        );
    }
}

#[test]
fn serde_refuses_unknown_variant_names() {
    for name in ["FutureReject", "SYS_UNKNOWN", "", "7"] {
        let input = StrDeserializer::<Error>::new(name);
        assert!(RejectCode::deserialize(input).is_err());
    }
}

#[test]
fn public_call_surface_preserves_the_catalogue_identity() {
    let code: crate::cdk::call::RejectCode = RejectCode::CanisterReject;
    assert_eq!(crate::cdk::call::RejectCode::try_from(4), Ok(code));

    let unknown = crate::cdk::call::RejectCode::try_from(u64::MAX)
        .expect_err("an unsupported code must remain an error through the public path");
    assert_eq!(unknown.0, u64::MAX);
}
