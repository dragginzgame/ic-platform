use super::{ExternalPrincipal, Principal, PrincipalError};
use serde_test::{Configure, Token, assert_de_tokens, assert_tokens};

#[test]
fn error_messages_and_sources_remain_stable() {
    use std::error::Error;
    for (error, message) in [
        (
            PrincipalError::BytesTooLong(),
            "Bytes is longer than 29 bytes.",
        ),
        (
            PrincipalError::InvalidBase32(),
            "Text must be in valid Base32 encoding.",
        ),
        (PrincipalError::TextTooShort(), "Text is too short."),
        (PrincipalError::TextTooLong(), "Text is too long."),
        (
            PrincipalError::CheckSequenceNotMatch(),
            "CRC32 check sequence doesn't match with calculated from Principal bytes.",
        ),
        (
            PrincipalError::AbnormalGrouped(Principal::management_canister()),
            "Text should be separated by - (dash) every 5 characters: expected \"aaaaa-aa\"",
        ),
    ] {
        assert_eq!(error.to_string(), message);
        assert!(error.source().is_none());
    }
}

#[test]
fn deserialization_preserves_structured_format_errors() {
    use serde::Deserialize;

    #[derive(Debug, PartialEq)]
    enum FormatError {
        UnexpectedToken,
        Custom(String),
    }
    impl std::fmt::Display for FormatError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }
    impl std::error::Error for FormatError {}
    impl serde::de::Error for FormatError {
        fn custom<T: std::fmt::Display>(message: T) -> Self {
            Self::Custom(message.to_string())
        }
    }
    struct RefusingDeserializer(bool);
    impl<'de> serde::Deserializer<'de> for RefusingDeserializer {
        type Error = FormatError;

        fn is_human_readable(&self) -> bool {
            self.0
        }
        fn deserialize_any<V: serde::de::Visitor<'de>>(
            self,
            _: V,
        ) -> Result<V::Value, Self::Error> {
            Err(FormatError::UnexpectedToken)
        }
        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
            map struct enum identifier ignored_any
        }
    }
    for human_readable in [true, false] {
        assert_eq!(
            Principal::deserialize(RefusingDeserializer(human_readable)),
            Err(FormatError::UnexpectedToken)
        );
    }
}

#[test]
fn all_lengths_preserve_text_bytes_order_and_padding() {
    for len in 0..=Principal::MAX_LENGTH_IN_BYTES {
        for seed in 0..=255u8 {
            let bytes: Vec<_> = (0..len).map(|i| seed.wrapping_add(i as u8)).collect();
            let owned = Principal::try_from_slice(&bytes).unwrap();
            let external = ExternalPrincipal::from_slice(&bytes);
            assert_eq!(owned.to_text(), external.to_text());
            assert_eq!(Principal::from_text(owned.to_text()).unwrap(), owned);
            assert_eq!(
                Principal::from_text(owned.to_text().to_uppercase()).unwrap(),
                owned
            );
            assert_eq!(owned.len() as usize, len);
            assert!(owned.as_fixed_bytes()[len..].iter().all(|&b| b == 0));
            assert_eq!(ExternalPrincipal::from(owned), external);
            assert_eq!(Principal::from(external), owned);
        }
    }
    let a = Principal::from_slice(&[255]);
    let b = Principal::from_slice(&[0, 0]);
    assert_eq!(a.cmp(&b), ExternalPrincipal::from(a).cmp(&b.into()));
    let empty = Principal::management_canister();
    let zero = Principal::from_slice(&[0]);
    assert_eq!(empty.as_fixed_bytes(), zero.as_fixed_bytes());
    assert_ne!(empty, zero);
}

#[test]
fn malformed_text_and_bytes_are_fallible_and_bounded() {
    assert_eq!(
        Principal::try_from_slice(&[0; 30]),
        Err(PrincipalError::BytesTooLong())
    );
    for (text, error) in [
        ("aaaaa", PrincipalError::TextTooShort()),
        ("aaaaa-a", PrincipalError::InvalidBase32()),
        ("é", PrincipalError::InvalidBase32()),
        (
            "aaaaaaa",
            PrincipalError::AbnormalGrouped(Principal::management_canister()),
        ),
        ("aaaaa-ai", PrincipalError::CheckSequenceNotMatch()),
    ] {
        assert_eq!(Principal::from_text(text), Err(error), "{text}");
    }
    let text = format!("{}aaaaa-aa", "-".repeat(1_048_576));
    assert_eq!(
        Principal::from_text(text),
        Err(PrincipalError::TextTooLong())
    );
    assert_eq!(
        Principal::from_text("a".repeat(64)),
        Err(PrincipalError::TextTooLong())
    );
    assert_eq!(
        Principal::from_text("a".repeat(63)),
        Err(PrincipalError::TextTooLong())
    );
}

#[test]
fn serde_bytes_never_consume_a_candid_marker() {
    for bytes in [&[][..], &[4][..], &[2, 1][..], &[0; 29][..]] {
        let p = Principal::from_slice(bytes);
        assert_tokens(&p.compact(), &[Token::Bytes(bytes)]);
        assert_de_tokens(&p.compact(), &[Token::ByteBuf(bytes)]);
    }
    assert_tokens(
        &Principal::anonymous().readable(),
        &[Token::Str("2vxsx-fae")],
    );
    serde_test::assert_de_tokens_error::<serde_test::Compact<Principal>>(
        &[Token::ByteBuf(&[0; 30])],
        "Bytes is longer than 29 bytes.",
    );
}

#[test]
fn self_authentication_keeps_upstream_vectors() {
    let expected = [
        209, 74, 2, 140, 42, 58, 43, 201, 71, 97, 2, 187, 40, 130, 52, 196, 21, 162, 176, 31, 130,
        142, 166, 42, 197, 179, 228, 47, 2,
    ];
    assert_eq!(Principal::self_authenticating([]).as_slice(), expected);
    for input in [&[][..], b"public key bytes", &[42; 1024][..]] {
        let owned = input.to_vec();
        let expected = ExternalPrincipal::self_authenticating(input);
        assert_eq!(
            ExternalPrincipal::from(Principal::self_authenticating(&owned)),
            expected
        );
        assert_eq!(
            ExternalPrincipal::from(Principal::self_authenticating(owned)),
            expected
        );
        assert_eq!(
            ExternalPrincipal::from(Principal::self_authenticating(input)),
            expected
        );
    }
}

#[test]
fn self_authentication_evaluates_custom_as_ref_once() {
    use std::cell::Cell;
    struct PublicKey<'a>(&'a Cell<usize>);
    impl AsRef<[u8]> for PublicKey<'_> {
        fn as_ref(&self) -> &[u8] {
            self.0.set(self.0.get() + 1);
            b"public key bytes"
        }
    }
    let calls = Cell::new(0);
    let actual = Principal::self_authenticating(PublicKey(&calls));
    assert_eq!(calls.get(), 1);
    assert_eq!(
        ExternalPrincipal::from(actual),
        ExternalPrincipal::self_authenticating(b"public key bytes")
    );
}
