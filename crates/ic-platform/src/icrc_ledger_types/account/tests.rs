use super::*;
use std::error::Error;

fn detail(error: hex::FromHexError) -> ICRC1TextReprError {
    ICRC1TextReprError::InvalidSubaccount(format!("subaccount is not hex-encoded: {error}"))
}

#[test]
fn scalar_padding_preserves_utf8_parity_and_byte_error_offsets() {
    for suffix in ["é", "aé", "💎"] {
        assert_eq!(
            decode_subaccount(suffix),
            Err(detail(hex::FromHexError::OddLength))
        );
    }
    for (suffix, byte, index) in [("中", 0xe4, 63), ("éé", 0xc3, 62), ("中a", 0xe4, 62)] {
        assert_eq!(
            decode_subaccount(suffix),
            Err(detail(hex::FromHexError::InvalidHexCharacter {
                c: char::from(byte),
                index,
            }))
        );
    }
    assert_eq!(
        decode_subaccount(&"é".repeat(64)),
        Err(detail(hex::FromHexError::InvalidHexCharacter {
            c: char::from(0xc3),
            index: 0,
        }))
    );
}

#[test]
fn oversized_hex_validation_precedes_length_and_error_has_no_source() {
    assert_eq!(
        decode_subaccount(&"g".repeat(129)),
        Err(detail(hex::FromHexError::OddLength))
    );
    assert_eq!(
        decode_subaccount(&"f".repeat(130)),
        Err(ICRC1TextReprError::InvalidSubaccount(
            "subaccount is longer than 32 bytes".to_owned(),
        ))
    );
    assert_eq!(
        decode_subaccount(&format!("{}g", "f".repeat(129))),
        Err(detail(hex::FromHexError::InvalidHexCharacter {
            c: 'g',
            index: 129,
        }))
    );
    assert_eq!(
        decode_subaccount(""),
        Err(ICRC1TextReprError::DefaultSubaccountShouldBeOmitted)
    );
    let principal_error = "".parse::<Principal>().unwrap_err();
    let wrapped = ICRC1TextReprError::InvalidPrincipal(principal_error);
    assert!(wrapped.source().is_none());
}
