use super::{Account, AccountError, DEFAULT_SUBACCOUNT, Subaccount};
use crate::principal::{Principal, PrincipalError};
use serde_test::{Configure, Token, assert_de_tokens, assert_tokens};
use std::{
    cmp::Ordering,
    error::Error,
    hash::{Hash, Hasher},
};

fn with_last_byte(last: u8) -> Account {
    let mut subaccount = [0; 32];
    subaccount[31] = last;
    Account {
        owner: Principal::anonymous(),
        subaccount: Some(subaccount),
    }
}

#[test]
fn observed_texts_preserve_default_and_explicit_subaccounts() {
    let absent = Account::from(Principal::anonymous());
    assert_eq!(absent.subaccount, None);
    assert_eq!(absent.to_string(), "2vxsx-fae");
    assert_eq!(DEFAULT_SUBACCOUNT, &[0; 32]);
    for (last, text) in [
        (0, "2vxsx-fae"),
        (1, "2vxsx-fae-22yutvy.1"),
        (15, "2vxsx-fae-geewjua.f"),
        (16, "2vxsx-fae-xqawsji.10"),
        (255, "2vxsx-fae-rs2jnta.ff"),
    ] {
        let account = with_last_byte(last);
        assert_eq!(account.to_string(), text);
        let parsed = text.parse::<Account>().unwrap();
        assert_eq!(parsed, account);
        assert_eq!(
            parsed.subaccount,
            if last == 0 { None } else { account.subaccount }
        );
    }
}

#[test]
fn every_hex_width_is_minimal_right_aligned_and_case_compatible() {
    for owner in [
        Principal::management_canister(),
        Principal::anonymous(),
        Principal::from_slice(&[0x31; 29]),
    ] {
        for width in 1_usize..=64 {
            let mut subaccount: Subaccount = [0; 32];
            let first_byte = 32 - width.div_ceil(2);
            subaccount[first_byte..].fill(0xab);
            let expected_hex = if width % 2 == 0 {
                "ab".repeat(width / 2)
            } else {
                subaccount[first_byte] = 0x0b;
                format!("b{}", "ab".repeat(width / 2))
            };
            let account = Account {
                owner,
                subaccount: Some(subaccount),
            };
            let text = account.to_string();
            let (prefix, hex) = text.split_once('.').unwrap();
            let (owner_text, checksum) = prefix.rsplit_once('-').unwrap();
            assert_eq!(hex, expected_hex);
            assert_eq!(owner_text, owner.to_text());
            assert_eq!(checksum.len(), 7);
            assert!(
                checksum
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || (b'2'..=b'7').contains(&byte))
            );
            let parsed = text.parse::<Account>().unwrap();
            assert_eq!(parsed.owner, owner);
            assert_eq!(parsed.subaccount, Some(subaccount));
            let mixed_case = format!(
                "{}-{checksum}.{}",
                owner_text.to_ascii_uppercase(),
                hex.to_ascii_uppercase()
            );
            assert_eq!(mixed_case.parse::<Account>().unwrap(), account);
        }
    }
}

#[test]
fn text_ceiling_is_a_byte_bound_with_the_longest_form_admitted() {
    assert_eq!(Account::MAX_LENGTH_IN_TEXT, 136);
    let longest = Account {
        owner: Principal::from_slice(&[255; 29]),
        subaccount: Some([255; 32]),
    };
    let text = longest.to_string();
    assert_eq!(text.len(), 136);
    assert_eq!(text.parse::<Account>().unwrap(), longest);
    assert_eq!(
        format!("{text} ").parse::<Account>(),
        Err(AccountError::TextTooLong)
    );
    for text in ["!".repeat(137), "é".repeat(69), "💎".repeat(35)] {
        assert_eq!(text.parse::<Account>(), Err(AccountError::TextTooLong));
    }
    for text in ["é".repeat(68), "💎".repeat(34), "!".repeat(64)] {
        assert_eq!(
            text.parse::<Account>(),
            Err(AccountError::InvalidPrincipal(PrincipalError::TextTooLong()))
        );
    }
}

#[test]
fn malformed_combinations_preserve_error_precedence() {
    for (text, expected) in [
        (
            "",
            AccountError::InvalidPrincipal(PrincipalError::TextTooShort()),
        ),
        ("2vxsx-fae.1", AccountError::MissingChecksum),
        ("bad.0", AccountError::MissingChecksum),
        ("bad-aaaaaa.0", AccountError::MissingChecksum),
        ("bad-aaaaaaaa.0", AccountError::MissingChecksum),
        ("bad-aaaaaaa.0", AccountError::LeadingZeroesInSubaccount),
        ("bad-aaaaaaa.01", AccountError::LeadingZeroesInSubaccount),
        (
            "bad-aaaaaaa.1",
            AccountError::InvalidPrincipal(PrincipalError::InvalidBase32()),
        ),
        (
            "bad-aaaaaaa.",
            AccountError::InvalidPrincipal(PrincipalError::InvalidBase32()),
        ),
        (
            "2vxsx-fae-aaaaaaa.",
            AccountError::DefaultSubaccountShouldBeOmitted,
        ),
        (
            "2vxsx-fae-aaaaaaa.0",
            AccountError::LeadingZeroesInSubaccount,
        ),
        (
            "2vxsx-fae-aaaaaaa.01",
            AccountError::LeadingZeroesInSubaccount,
        ),
        (
            "2vxsx-fae-aaaaaaa.0g",
            AccountError::LeadingZeroesInSubaccount,
        ),
        ("2vxsx-fae-aaaaaaa.g", AccountError::InvalidSubaccount),
        ("2vxsx-fae-aaaaaaa..", AccountError::InvalidSubaccount),
        ("2vxsx-fae-aaaaaaa.1.2", AccountError::InvalidSubaccount),
        ("2vxsx-fae-aaaaaaa.1 ", AccountError::InvalidSubaccount),
        ("2vxsx-fae-aaaaaaa.１", AccountError::InvalidSubaccount),
        ("2vxsx-fae-aaaaaaa.٠", AccountError::InvalidSubaccount),
        ("2vxsx-fae-aaaaaaa.1", AccountError::InvalidChecksum),
        ("2vxsx-fae-!!!!!!!.1", AccountError::InvalidChecksum),
        ("2vxsx-fae-22YUTVY.1", AccountError::InvalidChecksum),
    ] {
        assert_eq!(text.parse::<Account>(), Err(expected), "input {text:?}");
    }
    let excessive_hex = "f".repeat(65);
    assert_eq!(
        format!("bad-aaaaaaa.{excessive_hex}").parse::<Account>(),
        Err(AccountError::InvalidPrincipal(
            PrincipalError::InvalidBase32()
        ))
    );
    assert_eq!(
        format!("2vxsx-fae-aaaaaaa.{excessive_hex}").parse::<Account>(),
        Err(AccountError::InvalidSubaccount)
    );
    assert_eq!(
        format!("2vxsx-fae-aaaaaaa.0{excessive_hex}").parse::<Account>(),
        Err(AccountError::LeadingZeroesInSubaccount)
    );
}

#[test]
fn checksum_and_payload_mutations_are_refused() {
    let account = with_last_byte(1);
    let text = account.to_string();
    let (prefix, hex) = text.split_once('.').unwrap();
    let (owner, checksum) = prefix.rsplit_once('-').unwrap();
    for index in 0..7 {
        let mut mutated = checksum.as_bytes().to_vec();
        mutated[index] = if mutated[index] == b'a' { b'b' } else { b'a' };
        let mutated = String::from_utf8(mutated).unwrap();
        assert_eq!(
            format!("{owner}-{mutated}.{hex}").parse::<Account>(),
            Err(AccountError::InvalidChecksum)
        );
    }
    assert_eq!(
        format!("{prefix}.2").parse::<Account>(),
        Err(AccountError::InvalidChecksum)
    );
    assert_eq!(
        format!("aaaaa-aa-{checksum}.{hex}").parse::<Account>(),
        Err(AccountError::InvalidChecksum)
    );
}

#[test]
fn malformed_unicode_never_panics_or_becomes_hex() {
    for symbol in ["é", "中", "💎", "\u{feff}", "\u{200d}"] {
        for count in 1..=40 {
            let hex = symbol.repeat(count);
            let text = format!("2vxsx-fae-aaaaaaa.{hex}");
            let expected = if text.len() > 136 {
                AccountError::TextTooLong
            } else {
                AccountError::InvalidSubaccount
            };
            assert_eq!(text.parse::<Account>(), Err(expected));
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug)]
struct HashCalls(Vec<String>);

impl Hasher for HashCalls {
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
fn logical_zero_equality_order_and_hash_preserve_standard_trait_calls() {
    let absent = Account::from(Principal::anonymous());
    let zero = with_last_byte(0);
    let one = with_last_byte(1);
    assert_eq!(absent, zero);
    assert_eq!(absent.cmp(&zero), Ordering::Equal);
    assert_eq!(absent.partial_cmp(&zero), Some(Ordering::Equal));
    assert!(zero < one);
    assert_eq!(absent.effective_subaccount(), &[0; 32]);
    assert_eq!(one.effective_subaccount()[31], 1);

    for account in [absent, zero, one] {
        let mut actual = HashCalls::default();
        account.hash(&mut actual);
        let mut expected = HashCalls::default();
        account.owner.hash(&mut expected);
        account.effective_subaccount().hash(&mut expected);
        assert_eq!(actual, expected);
    }
    let mut absent_calls = HashCalls::default();
    absent.hash(&mut absent_calls);
    let mut zero_calls = HashCalls::default();
    zero.hash(&mut zero_calls);
    assert_eq!(absent_calls, zero_calls);

    let short_owner = Account {
        owner: Principal::from_slice(&[255]),
        subaccount: Some([255; 32]),
    };
    let long_owner = Account::from(Principal::from_slice(&[0, 0]));
    assert!(short_owner < long_owner);
    let mut left = [0; 32];
    left[0] = 1;
    let mut right = [255; 32];
    right[0] = 0;
    assert!(
        Account {
            owner: absent.owner,
            subaccount: Some(left)
        } > Account {
            owner: absent.owner,
            subaccount: Some(right)
        }
    );
}

fn anonymous_cbor_prefix() -> Vec<u8> {
    let mut bytes = vec![0xa2, 0x65];
    bytes.extend_from_slice(b"owner");
    bytes.extend([0x41, 4, 0x6a]);
    bytes.extend_from_slice(b"subaccount");
    bytes
}

#[test]
fn serde_bytes_match_the_public_transcript_and_preserve_presence() {
    for account in [
        Account::from(Principal::anonymous()),
        with_last_byte(0),
        with_last_byte(1),
        with_last_byte(15),
        with_last_byte(16),
        with_last_byte(255),
    ] {
        let expected_sub = match account.subaccount {
            None => "null".to_owned(),
            Some(bytes) => format!(
                "[{}]",
                bytes
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        };
        let expected_json = format!("{{\"owner\":\"2vxsx-fae\",\"subaccount\":{expected_sub}}}");
        assert_eq!(serde_json::to_string(&account).unwrap(), expected_json);
        let json_account: Account = serde_json::from_str(&expected_json).unwrap();
        assert_eq!(json_account.subaccount, account.subaccount);
        assert_eq!(json_account.owner, account.owner);

        let mut expected_cbor = anonymous_cbor_prefix();
        let mut expected_postcard = vec![1, 4];
        match account.subaccount {
            None => {
                expected_cbor.push(0xf6);
                expected_postcard.push(0);
            }
            Some(bytes) => {
                expected_cbor.extend([0x58, 0x20]);
                expected_cbor.extend(bytes);
                expected_postcard.extend([1, 32]);
                expected_postcard.extend(bytes);
            }
        }
        assert_eq!(serde_cbor::to_vec(&account).unwrap(), expected_cbor);
        let cbor_account: Account = serde_cbor::from_slice(&expected_cbor).unwrap();
        assert_eq!(cbor_account.subaccount, account.subaccount);
        let mut scratch = [0; 80];
        assert_eq!(
            postcard::to_slice(&account, &mut scratch).unwrap(),
            expected_postcard
        );
        let postcard_account: Account = postcard::from_bytes(&expected_postcard).unwrap();
        assert_eq!(postcard_account.subaccount, account.subaccount);
    }
}

#[test]
fn serde_retains_struct_name_field_order_and_byte_callbacks() {
    assert_tokens(
        &Account::from(Principal::anonymous()).readable(),
        &[
            Token::Struct {
                name: "Account",
                len: 2,
            },
            Token::Str("owner"),
            Token::Str("2vxsx-fae"),
            Token::Str("subaccount"),
            Token::None,
            Token::StructEnd,
        ],
    );
    assert_tokens(
        &with_last_byte(0).compact(),
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
    assert_de_tokens(
        &with_last_byte(0).compact(),
        &[
            Token::Struct {
                name: "Account",
                len: 2,
            },
            Token::Str("owner"),
            Token::ByteBuf(&[4]),
            Token::Str("subaccount"),
            Token::Some,
            Token::ByteBuf(&[0; 32]),
            Token::StructEnd,
        ],
    );
}

#[test]
fn serde_defaults_unknown_fields_duplicates_and_exact_lengths() {
    for json in [
        "{\"owner\":\"2vxsx-fae\"}",
        "{\"ignored\":{\"nested\":[1,2]},\"owner\":\"2vxsx-fae\"}",
    ] {
        let account: Account = serde_json::from_str(json).unwrap();
        assert_eq!(account.owner, Principal::anonymous());
        assert_eq!(account.subaccount, None);
    }
    for json in [
        "{}",
        "{\"subaccount\":null}",
        "{\"owner\":\"2vxsx-fae\",\"owner\":\"2vxsx-fae\"}",
        "{\"owner\":\"2vxsx-fae\",\"subaccount\":null,\"subaccount\":null}",
        "\"2vxsx-fae\"",
    ] {
        assert!(serde_json::from_str::<Account>(json).is_err());
    }
    for length in [0, 1, 31, 32, 33, 64] {
        let json = serde_json::json!({"owner": "2vxsx-fae", "subaccount": vec![0; length]});
        let json_result = serde_json::from_value::<Account>(json);
        assert_eq!(json_result.is_ok(), length == 32);
        for major_type in [0x58, 0x98] {
            let mut cbor = anonymous_cbor_prefix();
            cbor.extend([major_type, u8::try_from(length).unwrap()]);
            cbor.extend(std::iter::repeat_n(0, length));
            let result = serde_cbor::from_slice::<Account>(&cbor);
            assert_eq!(result.is_ok(), length == 32);
            if let Ok(account) = result {
                assert_eq!(account.subaccount, Some([0; 32]));
            }
        }
    }
}

#[test]
fn errors_preserve_exact_display_and_principal_source() {
    for (error, display) in [
        (AccountError::TextTooLong, "account text is too long"),
        (AccountError::MissingChecksum, "missing account checksum"),
        (AccountError::InvalidChecksum, "invalid account checksum"),
        (
            AccountError::InvalidSubaccount,
            "subaccount must contain at most 64 hex digits",
        ),
        (
            AccountError::LeadingZeroesInSubaccount,
            "subaccount should not have leading zeroes",
        ),
        (
            AccountError::DefaultSubaccountShouldBeOmitted,
            "default subaccount should be omitted",
        ),
    ] {
        assert_eq!(error.to_string(), display);
        assert!(error.source().is_none());
        assert_eq!(error.clone(), error);
    }
    let error = AccountError::InvalidPrincipal(PrincipalError::TextTooShort());
    assert_eq!(error.to_string(), "invalid principal: Text is too short.");
    assert_eq!(
        error.source().unwrap().downcast_ref::<PrincipalError>(),
        Some(&PrincipalError::TextTooShort())
    );
    assert_eq!(format!("{error:?}"), "InvalidPrincipal(TextTooShort)");
}
