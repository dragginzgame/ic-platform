use super::*;
use serde_test::{Token, assert_de_tokens};
use std::{collections::hash_map::DefaultHasher, hash::Hasher};

#[test]
fn default_identity_preserves_representation() {
    let absent = Account::from(Principal::anonymous());
    let explicit = Account {
        subaccount: Some([0; 32]),
        ..absent
    };
    assert_eq!(absent, explicit);
    assert_eq!(absent.cmp(&explicit), std::cmp::Ordering::Equal);
    let mut left = DefaultHasher::new();
    let mut right = DefaultHasher::new();
    absent.hash(&mut left);
    explicit.hash(&mut right);
    assert_eq!(left.finish(), right.finish());
    assert_ne!(absent.subaccount, explicit.subaccount);
    assert_eq!(absent.to_string(), explicit.to_string());
}

#[test]
fn binary_owner_has_no_marker() {
    use serde_test::Configure;
    let account = Account::from(Principal::from_slice(&[2, 1]));
    for bytes in [Token::BorrowedBytes(&[2, 1]), Token::ByteBuf(&[2, 1])] {
        assert_de_tokens(
            &account.compact(),
            &[
                Token::Struct {
                    name: "Account",
                    len: 2,
                },
                Token::Str("owner"),
                bytes,
                Token::Str("subaccount"),
                Token::None,
                Token::StructEnd,
            ],
        );
    }
}

#[test]
fn bounded_text_and_original_vector() {
    let text = "k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae-6cc627i.1";
    let account: Account = text.parse().unwrap();
    assert_eq!(account.subaccount.unwrap()[31], 1);
    assert_eq!(account.to_string(), text);
    assert_eq!(
        "x".repeat(1024 * 1024).parse::<Account>(),
        Err(AccountError::TextTooLong)
    );
    assert!(text.replace(".1", ".01").parse::<Account>().is_err());
    assert!(text.replace(".1", ".z").parse::<Account>().is_err());
    assert!(
        text.replace("6cc627i", "aaaaaaa")
            .parse::<Account>()
            .is_err()
    );
}

#[test]
fn serde_preserves_explicit_zero_and_rejects_wrong_lengths() {
    use serde_test::{Configure, assert_de_tokens_error, assert_ser_tokens};
    let account = Account {
        owner: Principal::from_slice(&[2, 1]),
        subaccount: Some([0; 32]),
    };
    let mut tokens = vec![
        Token::Struct {
            name: "Account",
            len: 2,
        },
        Token::Str("owner"),
        Token::Bytes(&[2, 1]),
        Token::Str("subaccount"),
        Token::Some,
        Token::Tuple { len: 32 },
    ];
    tokens.extend([Token::U8(0); 32]);
    tokens.extend([Token::TupleEnd, Token::StructEnd]);
    assert_ser_tokens(&account.compact(), &tokens);
    assert_de_tokens(&account.compact(), &tokens);
    tokens.remove(6);
    tokens.pop(); // Deserialization stops at the invalid array, before StructEnd.
    assert_de_tokens_error::<serde_test::Compact<Account>>(
        &tokens,
        "invalid length 31, expected an array of length 32",
    );
}
