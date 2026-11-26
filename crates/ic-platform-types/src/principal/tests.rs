use super::{Principal, PrincipalError};
use serde::Deserialize;

#[test]
fn literal_identities_render_into_borrowed_storage() {
    for (bytes, text) in [
        (&[][..], "aaaaa-aa"),
        (&[4][..], "2vxsx-fae"),
        (
            &[0, 0, 0, 0, 0, 0, 0, 1, 1, 1][..],
            "rrkah-fqaaa-aaaaa-aaaaq-cai",
        ),
    ] {
        let identity = Principal::try_from_slice(bytes).unwrap();
        let mut buffer = [0xa5; Principal::MAX_LENGTH_IN_TEXT];
        let start = buffer.as_ptr();
        let output = identity.write_text(&mut buffer);
        assert_eq!(output, text);
        assert_eq!(output.as_ptr(), start);
        assert_eq!(identity.to_text(), text);
        assert_eq!(identity.to_string(), text);
        assert_eq!(Principal::from_text(text).unwrap(), identity);
        assert_eq!(
            Principal::from_text(text.to_ascii_uppercase()).unwrap(),
            identity
        );
    }
}

#[test]
fn bounded_bytes_tail_and_order_are_stable() {
    for len in 0..=29 {
        let bytes: Vec<_> = (0..len).map(|index| (index * 7) as u8).collect();
        let value = Principal::try_from_slice(&bytes).unwrap();
        assert_eq!(value.len() as usize, len);
        assert_eq!(value.as_slice(), bytes);
        assert!(value.as_fixed_bytes()[len..].iter().all(|byte| *byte == 0));
        let text = value.to_text();
        assert!(text.len() <= 63);
        assert_eq!(Principal::from_text(text).unwrap(), value);
    }
    assert_eq!(
        Principal::try_from_slice(&[0; 30]),
        Err(PrincipalError::BytesTooLong())
    );
    assert!(Principal::from_slice(&[255]) < Principal::from_slice(&[0, 0]));
    assert!(Principal::from_slice(&[0, 1]) < Principal::from_slice(&[0, 2]));
    assert!(Principal::management_canister().as_slice().is_empty());
    assert_eq!(Principal::anonymous().as_slice(), &[4]);
}

#[test]
fn text_refusals_keep_their_distinct_meanings() {
    for (text, error) in [
        ("".to_owned(), PrincipalError::TextTooShort()),
        ("!".to_owned(), PrincipalError::InvalidBase32()),
        ("aaaaa-ab".to_owned(), PrincipalError::InvalidBase32()),
        // Four decoded checksum bytes, with valid zero Base32 padding bits.
        (
            "baaaa-aa".to_owned(),
            PrincipalError::CheckSequenceNotMatch(),
        ),
        (
            "aaaaaaa".to_owned(),
            PrincipalError::AbnormalGrouped(Principal::management_canister()),
        ),
        (
            "-aaaaa-aa".to_owned(),
            PrincipalError::AbnormalGrouped(Principal::management_canister()),
        ),
        (" aaaaa-aa".to_owned(), PrincipalError::InvalidBase32()),
        ("a".repeat(64), PrincipalError::TextTooLong()),
    ] {
        assert_eq!(Principal::from_text(&text), Err(error), "{text:?}");
    }
    // Valid Base32 carrying thirty identity bytes must refuse before checksum.
    let text = "a".repeat(55);
    assert_eq!(
        Principal::from_text(text),
        Err(PrincipalError::TextTooLong())
    );
}

#[test]
fn serde_preserves_text_and_byte_admission_without_sequences() {
    use serde_test::{Configure, Token, assert_tokens};
    let principal = Principal::anonymous();
    assert_tokens(&principal.readable(), &[Token::Str("2vxsx-fae")]);
    assert_tokens(&principal.compact(), &[Token::Bytes(&[4])]);
    assert_eq!(serde_json::to_string(&principal).unwrap(), "\"2vxsx-fae\"");
    assert_eq!(
        serde_json::from_str::<Principal>("\"2VXSX-FAE\"").unwrap(),
        principal
    );
    for json in ["[4]", "4", "null", "\"2vxsx-faa\""] {
        assert!(serde_json::from_str::<Principal>(json).is_err());
    }
    let value = serde::de::value::BorrowedBytesDeserializer::<serde::de::value::Error>::new(&[4]);
    assert_eq!(Principal::deserialize(value).unwrap(), principal);
    let oversized = [0; 30];
    let value =
        serde::de::value::BorrowedBytesDeserializer::<serde::de::value::Error>::new(&oversized);
    assert!(Principal::deserialize(value).is_err());
}
