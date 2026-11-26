use super::{Account, AccountError, DEFAULT_SUBACCOUNT};
use crate::Principal;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

fn digest(value: &Account) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

#[test]
fn effective_identity_preserves_explicit_serialized_presence() {
    let absent = Account::from(Principal::anonymous());
    let zero = Account {
        subaccount: Some([0; 32]),
        ..absent
    };
    assert_eq!(absent, zero);
    assert_eq!(absent.cmp(&zero), std::cmp::Ordering::Equal);
    assert_eq!(digest(&absent), digest(&zero));
    assert_eq!(absent.effective_subaccount(), DEFAULT_SUBACCOUNT);
    assert_eq!(absent.to_string(), "2vxsx-fae");
    assert_eq!(zero.to_string(), "2vxsx-fae");
    assert_eq!(
        serde_json::to_string(&absent).unwrap(),
        r#"{"owner":"2vxsx-fae","subaccount":null}"#
    );
    let json = serde_json::to_value(zero).unwrap();
    assert_eq!(json["subaccount"], serde_json::json!([0; 32].to_vec()));
    assert_eq!(
        serde_json::from_value::<Account>(json).unwrap().subaccount,
        Some([0; 32])
    );
    assert_eq!(
        serde_json::from_str::<Account>(r#"{"owner":"2vxsx-fae"}"#)
            .unwrap()
            .subaccount,
        None
    );
    let mut one = zero;
    one.subaccount.as_mut().unwrap()[31] = 1;
    assert!(zero < one);
    assert_ne!(zero, one);
    assert!(Account::from(Principal::management_canister()) < absent);
}

#[test]
fn literal_text_and_caller_storage() {
    // Checksums independently calculated with Node's zlib CRC32 over owner || subaccount.
    let mut one = [0; 32];
    one[31] = 1;
    let mut abc = [0; 32];
    abc[30] = 0x0a;
    abc[31] = 0xbc;
    for (owner, subaccount, text) in [
        (Principal::management_canister(), one, "aaaaa-aa-nygwkoy.1"),
        (Principal::anonymous(), abc, "2vxsx-fae-tghg43a.abc"),
        (
            Principal::anonymous(),
            [255; 32],
            "2vxsx-fae-i7iipzy.ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        ),
    ] {
        let account = Account {
            owner,
            subaccount: Some(subaccount),
        };
        assert_eq!(text.parse::<Account>().unwrap(), account);
        assert_eq!(account.to_string(), text);
        let mut buffer = [0xa5; Account::MAX_LENGTH_IN_TEXT];
        let start = buffer.as_ptr();
        let rendered = account.write_text(&mut buffer);
        assert_eq!(rendered.as_ptr(), start);
        assert_eq!(rendered, text);
    }
    assert_eq!(
        "2VXSX-FAE-tghg43a.ABC"
            .parse::<Account>()
            .unwrap()
            .subaccount,
        Some(abc)
    );
    let longest = Account {
        owner: Principal::from_slice(&[255; 29]),
        subaccount: Some([255; 32]),
    };
    let text = longest.to_string();
    assert_eq!(text.len(), Account::MAX_LENGTH_IN_TEXT);
    assert_eq!(text.parse::<Account>().unwrap(), longest);
}

#[test]
fn icrc1_text_examples() {
    // https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-1/TextualEncoding.md#examples
    let owner_text = "k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae";
    let owner = Principal::from_text(owner_text).unwrap();
    let mut one = [0; 32];
    one[31] = 1;
    let increasing = std::array::from_fn(|index| (index + 1) as u8);
    for (suffix, subaccount) in [
        ("", None),
        ("-6cc627i.1", Some(one)),
        (
            "-dfxgiyy.102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
            Some(increasing),
        ),
    ] {
        let text = format!("{owner_text}{suffix}");
        let parsed = text.parse::<Account>().unwrap();
        assert_eq!(parsed.owner, owner);
        assert_eq!(parsed.subaccount, subaccount);
        assert_eq!(Account { owner, subaccount }.to_string(), text);
    }
    for suffix in ["-q6bn32y.", "-6cc627i.01", ".1"] {
        assert!(format!("{owner_text}{suffix}").parse::<Account>().is_err());
    }
    assert!(
        "k2t6j2nvnp4zjm3-25dtz6xhaac7boj5gayfoj3xs-i43lp-teztq-6ae"
            .parse::<Account>()
            .is_err()
    );
}

#[test]
fn malformed_text_keeps_refusal_precedence() {
    for (text, error) in [
        ("x".repeat(137), AccountError::TextTooLong),
        ("aaaaa-aa.1".into(), AccountError::MissingChecksum),
        (
            "aaaaa-aa-nygwkoy.01".into(),
            AccountError::LeadingZeroesInSubaccount,
        ),
        (
            "aaaaa-aa-nygwkoy.".into(),
            AccountError::DefaultSubaccountShouldBeOmitted,
        ),
        ("aaaaa-aa-nygwkoy.g".into(), AccountError::InvalidSubaccount),
        ("aaaaa-aa-nygwkoy.é".into(), AccountError::InvalidSubaccount),
        (
            "aaaaa-aa-nygwkoy.1.1".into(),
            AccountError::InvalidSubaccount,
        ),
        (
            format!("aaaaa-aa-nygwkoy.{}", "f".repeat(65)),
            AccountError::InvalidSubaccount,
        ),
        ("aaaaa-aa-NYGWKOY.1".into(), AccountError::InvalidChecksum),
        ("aaaaa-aa-nygwkoy.2".into(), AccountError::InvalidChecksum),
    ] {
        assert_eq!(text.parse::<Account>(), Err(error), "{text:?}");
    }
    assert!(matches!(
        "!-nygwkoy.1".parse::<Account>(),
        Err(AccountError::InvalidPrincipal(_))
    ));
}

#[test]
fn serde_binary_shape_and_subaccount_length_refusal() {
    use serde_test::{Configure, Token, assert_tokens};
    let account = Account {
        owner: Principal::anonymous(),
        subaccount: Some([0; 32]),
    };
    assert_tokens(
        &account.compact(),
        &[
            Token::Struct {
                name: "Account",
                len: 2,
            },
            Token::Str("owner"),
            Token::Bytes(&[4]),
            Token::Str("subaccount"),
            Token::Some,
            Token::Bytes(&[0; 32]),
            Token::StructEnd,
        ],
    );
    for len in [0, 1, 31, 33] {
        let json = serde_json::json!({"owner": "2vxsx-fae", "subaccount": vec![0; len]});
        assert!(serde_json::from_value::<Account>(json).is_err());
    }
}
