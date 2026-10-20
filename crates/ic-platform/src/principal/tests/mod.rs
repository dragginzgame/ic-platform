use super::{ExternalPrincipal, Principal, PrincipalError};
use serde_test::{Configure, Token, assert_de_tokens, assert_tokens};

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
        assert_eq!(
            ExternalPrincipal::from(Principal::self_authenticating(input)),
            ExternalPrincipal::self_authenticating(input)
        );
    }
}
