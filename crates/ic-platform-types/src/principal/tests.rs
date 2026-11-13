use super::{Principal, PrincipalError};
use data_encoding::BASE32_NOPAD;
use serde_test::{Configure, Token, assert_de_tokens, assert_tokens};
use std::hash::{Hash, Hasher};

const MANAGEMENT: Principal = Principal::management_canister();
const ANONYMOUS: Principal = Principal::anonymous();
const CONSTRUCTED: Principal = Principal::from_slice(&[1, 2, 0]);
const FALLIBLE: Result<Principal, PrincipalError> = Principal::try_from_slice(&[1, 2, 0]);
const TOO_LONG: Result<Principal, PrincipalError> = Principal::try_from_slice(&[0; 30]);
const SIGNIFICANT: &[u8] = CONSTRUCTED.as_slice();
const FIXED: &[u8; 29] = CONSTRUCTED.as_fixed_bytes();
const LENGTH: u8 = CONSTRUCTED.len();

#[test]
fn constructors_and_accessors_remain_const() {
    assert!(MANAGEMENT.as_slice().is_empty());
    assert_eq!(ANONYMOUS.as_slice(), [4]);
    assert_eq!(FALLIBLE, Ok(CONSTRUCTED));
    assert_eq!(TOO_LONG, Err(PrincipalError::BytesTooLong()));
    assert_eq!(SIGNIFICANT, [1, 2, 0]);
    assert_eq!(&FIXED[..3], [1, 2, 0]);
    assert!(FIXED[3..].iter().all(|byte| *byte == 0));
    assert_eq!(LENGTH, 3);
    assert_eq!(Principal::MAX_LENGTH_IN_BYTES, 29);
    assert_eq!(Principal::MAX_LENGTH_IN_TEXT, 63);
}

#[test]
fn every_significant_length_preserves_bytes_and_zero_padding() {
    for length in 0..=29 {
        let bytes: Vec<u8> = (0..length)
            .map(|index| u8::try_from(index * 7 + length).expect("fixture byte fits"))
            .collect();
        let principal = Principal::from_slice(&bytes);
        assert_eq!(usize::from(principal.len()), length);
        assert_eq!(principal.as_slice(), bytes);
        assert_eq!(principal.as_ref(), bytes);
        assert_eq!(&principal.as_fixed_bytes()[..length], bytes);
        assert!(
            principal.as_fixed_bytes()[length..]
                .iter()
                .all(|byte| *byte == 0)
        );
        assert_eq!(Principal::try_from(bytes.as_slice()), Ok(principal));
        assert_eq!(Principal::try_from(&bytes), Ok(principal));
        assert_eq!(Principal::try_from(bytes.clone()), Ok(principal));

        let oracle = ic_principal::Principal::from_slice(&bytes);
        let text = principal.to_text();
        assert_eq!(text, oracle.to_text());
        assert_eq!(Principal::from_text(&text), Ok(principal));
        assert_eq!(text.parse::<Principal>(), Ok(principal));
        assert_eq!(Principal::try_from(text.as_str()), Ok(principal));
        assert_eq!(principal.to_string(), text);
        assert_eq!(
            ic_principal::Principal::from_text(&text)
                .unwrap()
                .as_slice(),
            bytes
        );
        assert!(text.len() <= 63);
        if length == 29 {
            assert_eq!(text.len(), 63);
        }
    }
}

#[test]
fn length_precedes_lexicographic_order_and_trailing_zero_is_significant() {
    let mut values = [
        Principal::from_slice(&[0, 0]),
        Principal::from_slice(&[255]),
        Principal::from_slice(&[0]),
        MANAGEMENT,
        Principal::from_slice(&[0, 1]),
    ];
    values.sort();
    assert_eq!(
        values,
        [
            MANAGEMENT,
            Principal::from_slice(&[0]),
            Principal::from_slice(&[255]),
            Principal::from_slice(&[0, 0]),
            Principal::from_slice(&[0, 1]),
        ]
    );
    assert_ne!(MANAGEMENT, Principal::from_slice(&[0]));
    assert_ne!(Principal::from_slice(&[1]), Principal::from_slice(&[1, 0]));
}

#[test]
fn oversized_binary_conversions_are_fallible() {
    for length in [30, 31, 255, 256] {
        let bytes = vec![0; length];
        let expected = Err(PrincipalError::BytesTooLong());
        assert_eq!(Principal::try_from_slice(&bytes), expected);
        assert_eq!(Principal::try_from(bytes.as_slice()), expected);
        assert_eq!(Principal::try_from(&bytes), expected);
        assert_eq!(Principal::try_from(bytes), expected);
    }
}

#[test]
#[should_panic]
#[expect(
    unused_must_use,
    reason = "The test asserts constructor refusal by panic, without a returned identity"
)]
fn infallible_constructor_panics_above_the_binary_bound() {
    Principal::from_slice(&[0; 30]);
}

#[derive(Default)]
struct HashTranscript(Vec<String>);

impl Hasher for HashTranscript {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.push(format!("bytes:{bytes:?}"));
    }

    fn write_u8(&mut self, value: u8) {
        self.0.push(format!("u8:{value}"));
    }

    fn write_usize(&mut self, value: usize) {
        self.0.push(format!("usize:{value}"));
    }
}

#[test]
fn debug_and_hash_match_the_public_observation_shape() {
    for bytes in [&[][..], &[4][..], &[1, 2][..], &[255; 29][..]] {
        let mut fixed = [0_u8; 29];
        fixed[..bytes.len()].copy_from_slice(bytes);
        let principal = Principal::from_slice(bytes);
        assert_eq!(
            format!("{principal:?}"),
            format!("Principal {{ len: {}, bytes: {fixed:?} }}", bytes.len())
        );
        let mut transcript = HashTranscript::default();
        principal.hash(&mut transcript);
        assert_eq!(
            transcript.0,
            [
                format!("u8:{}", bytes.len()),
                "usize:29".to_owned(),
                format!("bytes:{fixed:?}"),
            ]
        );
    }
}

#[test]
fn case_is_admitted_but_dash_grouping_must_be_canonical() {
    assert_eq!(MANAGEMENT.to_text(), "aaaaa-aa");
    assert_eq!(ANONYMOUS.to_text(), "2vxsx-fae");
    assert_eq!(Principal::from_text("AAAAA-AA"), Ok(MANAGEMENT));
    assert_eq!(Principal::from_text("aAaAa-aA"), Ok(MANAGEMENT));
    for text in ["aaaaaaa", "-aaaaa-aa", "aaaaa-aa-", "aaaaa--aa", "aaa-aaaa"] {
        assert_eq!(
            Principal::from_text(text),
            Err(PrincipalError::AbnormalGrouped(MANAGEMENT))
        );
    }
}

#[test]
fn malformed_text_reports_the_specified_error_precedence() {
    for (text, error) in [
        ("", PrincipalError::TextTooShort()),
        ("-", PrincipalError::TextTooShort()),
        ("aa", PrincipalError::TextTooShort()),
        ("a", PrincipalError::InvalidBase32()),
        ("aaaaa-a", PrincipalError::InvalidBase32()),
        ("aaaaa-ab", PrincipalError::InvalidBase32()),
        ("aaaaa-aa=", PrincipalError::InvalidBase32()),
        ("aaaaa-aa ", PrincipalError::InvalidBase32()),
        (" aaaaa-aa", PrincipalError::InvalidBase32()),
        ("aaaaa-a0", PrincipalError::InvalidBase32()),
        ("aaaaa-aq", PrincipalError::CheckSequenceNotMatch()),
        ("-aaaaa-aq", PrincipalError::CheckSequenceNotMatch()),
        ("\0aaaaa-aa", PrincipalError::InvalidBase32()),
        ("é", PrincipalError::InvalidBase32()),
        ("💎", PrincipalError::InvalidBase32()),
    ] {
        assert_eq!(Principal::from_text(text), Err(error), "input {text:?}");
    }
    assert_eq!(
        Principal::from_text("a".repeat(62)),
        Err(PrincipalError::InvalidBase32())
    );
    assert_eq!(
        Principal::from_text("a".repeat(63)),
        Err(PrincipalError::TextTooLong())
    );
    for text in [
        "!".repeat(64),
        "-".repeat(64),
        "é".repeat(32),
        "💎".repeat(16),
    ] {
        assert_eq!(
            Principal::from_text(text),
            Err(PrincipalError::TextTooLong())
        );
    }
    assert_eq!(
        Principal::from_text("é".repeat(31)),
        Err(PrincipalError::InvalidBase32())
    );
}

fn ungrouped_payload(bytes: &[u8], checksum: u32) -> String {
    let mut checked = checksum.to_be_bytes().to_vec();
    checked.extend_from_slice(bytes);
    let mut encoded = vec![0; BASE32_NOPAD.encode_len(checked.len())];
    BASE32_NOPAD.encode_mut(&checked, &mut encoded);
    String::from_utf8(encoded)
        .expect("Base32 is ASCII")
        .to_ascii_lowercase()
}

#[test]
fn decoded_length_is_checked_before_checksum_or_grouping() {
    for length in [30, 35] {
        let bytes = vec![7; length];
        let checksum = crc32fast::hash(&bytes);
        for checksum in [checksum, checksum ^ 1] {
            let text = ungrouped_payload(&bytes, checksum);
            assert!(text.len() <= 63);
            assert_eq!(
                Principal::from_text(text),
                Err(PrincipalError::TextTooLong())
            );
        }
    }
    let bytes = [1, 2, 3];
    let text = ungrouped_payload(&bytes, crc32fast::hash(&bytes) ^ 1);
    assert_eq!(
        Principal::from_text(text),
        Err(PrincipalError::CheckSequenceNotMatch())
    );
}

#[test]
fn bounded_ascii_and_unicode_inputs_do_not_panic() {
    for byte in 0_u8..=127 {
        for length in [0, 1, 2, 6, 7, 8, 62, 63, 64] {
            let text = char::from(byte).to_string().repeat(length);
            let result = Principal::from_text(text);
            if length > 63 {
                assert_eq!(result, Err(PrincipalError::TextTooLong()));
            }
        }
    }
    for text in ["é", "中", "💎", "\u{feff}", "\u{200d}"] {
        for count in 1..=22 {
            let input = text.repeat(count);
            let expected = if input.len() > 63 {
                PrincipalError::TextTooLong()
            } else {
                PrincipalError::InvalidBase32()
            };
            assert_eq!(Principal::from_text(input), Err(expected));
        }
    }
}

#[test]
fn serde_uses_text_or_significant_bytes_with_no_marker() {
    let principal = Principal::from_slice(&[1, 2]);
    assert_tokens(&principal.readable(), &[Token::Str("w3gef-eqbai")]);
    assert_tokens(&principal.compact(), &[Token::Bytes(&[1, 2])]);
    assert_de_tokens(&principal.compact(), &[Token::BorrowedBytes(&[1, 2])]);
    assert_de_tokens(&principal.compact(), &[Token::ByteBuf(&[1, 2])]);
    assert_de_tokens(&principal.readable(), &[Token::String("w3gef-eqbai")]);
    assert_de_tokens(&principal.compact(), &[Token::Str("w3gef-eqbai")]);
    assert_de_tokens(&principal.compact(), &[Token::String("w3gef-eqbai")]);
    assert_de_tokens(&principal.readable(), &[Token::Bytes(&[1, 2])]);
    assert_de_tokens(&principal.readable(), &[Token::ByteBuf(&[1, 2])]);
    assert_eq!(
        serde_json::to_string(&principal).unwrap(),
        "\"w3gef-eqbai\""
    );
    assert_eq!(serde_cbor::to_vec(&principal).unwrap(), [0x42, 1, 2]);
    let mut scratch = [0_u8; 32];
    assert_eq!(
        postcard::to_slice(&principal, &mut scratch).unwrap(),
        [2, 1, 2]
    );
}

#[test]
fn serde_round_trips_all_lengths_and_refuses_numeric_sequences() {
    for length in 0..=29 {
        let principal = Principal::from_slice(&vec![0; length]);
        let json = serde_json::to_string(&principal).unwrap();
        assert_eq!(serde_json::from_str::<Principal>(&json).unwrap(), principal);
        let cbor = serde_cbor::to_vec(&principal).unwrap();
        assert_eq!(
            serde_cbor::from_slice::<Principal>(&cbor).unwrap(),
            principal
        );
        let mut scratch = [0_u8; 32];
        let encoded = postcard::to_slice(&principal, &mut scratch).unwrap();
        assert_eq!(encoded.len(), length + 1);
        assert_eq!(
            postcard::from_bytes::<Principal>(encoded).unwrap(),
            principal
        );
    }
    assert!(serde_json::from_str::<Principal>("[4]").is_err());
    assert!(serde_json::from_str::<Principal>("\"aaaaaaa\"").is_err());
    assert!(serde_cbor::from_slice::<Principal>(&[0x81, 4]).is_err());
    assert!(serde_cbor::from_slice::<Principal>(&[0x80]).is_err());
    assert_eq!(
        serde_cbor::from_slice::<Principal>(&[
            0x68, b'a', b'a', b'a', b'a', b'a', b'-', b'a', b'a'
        ])
        .unwrap(),
        MANAGEMENT
    );
    let mut oversized_cbor = vec![0x58, 30];
    oversized_cbor.extend([0; 30]);
    assert!(serde_cbor::from_slice::<Principal>(&oversized_cbor).is_err());
    let mut oversized_postcard = vec![30];
    oversized_postcard.extend([0; 30]);
    assert!(postcard::from_bytes::<Principal>(&oversized_postcard).is_err());
}

#[test]
fn empty_tuple_errors_preserve_display_debug_and_serde_shape() {
    for (error, name, display) in [
        (
            PrincipalError::BytesTooLong(),
            "BytesTooLong",
            "Bytes is longer than 29 bytes.",
        ),
        (
            PrincipalError::InvalidBase32(),
            "InvalidBase32",
            "Text must be in valid Base32 encoding.",
        ),
        (
            PrincipalError::TextTooShort(),
            "TextTooShort",
            "Text is too short.",
        ),
        (
            PrincipalError::TextTooLong(),
            "TextTooLong",
            "Text is too long.",
        ),
        (
            PrincipalError::CheckSequenceNotMatch(),
            "CheckSequenceNotMatch",
            "CRC32 check sequence doesn't match with calculated from Principal bytes.",
        ),
    ] {
        assert_eq!(error.to_string(), display);
        assert_eq!(format!("{error:?}"), name);
        let json = format!("{{\"{name}\":[]}}");
        assert_eq!(serde_json::to_string(&error).unwrap(), json);
        assert_eq!(
            serde_json::from_str::<PrincipalError>(&json).unwrap(),
            error
        );
        assert_tokens(
            &error.clone().readable(),
            &[
                Token::TupleVariant {
                    name: "PrincipalError",
                    variant: name,
                    len: 0,
                },
                Token::TupleVariantEnd,
            ],
        );
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }
}

#[test]
fn grouping_error_carries_the_identity_in_text_and_binary_serde() {
    let error = PrincipalError::AbnormalGrouped(MANAGEMENT);
    assert_eq!(
        error.to_string(),
        "Text should be separated by - (dash) every 5 characters: expected \"aaaaa-aa\""
    );
    assert_eq!(
        serde_json::to_string(&error).unwrap(),
        "{\"AbnormalGrouped\":\"aaaaa-aa\"}"
    );
    assert_tokens(
        &error.clone().readable(),
        &[
            Token::NewtypeVariant {
                name: "PrincipalError",
                variant: "AbnormalGrouped",
            },
            Token::Str("aaaaa-aa"),
        ],
    );
    assert_tokens(
        &error.compact(),
        &[
            Token::NewtypeVariant {
                name: "PrincipalError",
                variant: "AbnormalGrouped",
            },
            Token::Bytes(&[]),
        ],
    );
}
