use candidate_types::{Account, Principal};
use ic_platform_types::{Account as PublishedAccount, Principal as PublishedPrincipal};

#[test]
fn published_identity_compatibility() {
    for len in 0..=29 {
        let bytes: Vec<_> = (0..len).map(|i| (i * 13) as u8).collect();
        let owner = Principal::from_slice(&bytes);
        let published = PublishedPrincipal::from_slice(&bytes);
        assert_eq!(owner.to_text(), published.to_text());
        for subaccount in [None, Some([0; 32]), Some([1; 32]), Some([255; 32])] {
            let local = Account { owner, subaccount };
            let old = PublishedAccount {
                owner: published,
                subaccount,
            };
            assert_eq!(local.to_string(), old.to_string());
            assert_eq!(
                serde_json::to_value(local).unwrap(),
                serde_json::to_value(old).unwrap()
            );
            let text = old.to_string();
            assert_eq!(text.parse::<Account>().unwrap(), local);
            // Probe every one-byte text mutation without treating the oracle as authority.
            for offset in 0..text.len() {
                let mut changed = text.as_bytes().to_vec();
                changed[offset] = b'!';
                let changed = String::from_utf8(changed).unwrap();
                assert_eq!(
                    changed.parse::<Account>().is_ok(),
                    changed.parse::<PublishedAccount>().is_ok()
                );
            }
        }
    }
}

#[test]
fn caller_buffers_preserve_every_subaccount_bit_and_owner_length() {
    // Exercise every possible first-nonzero bit location, including the maximal
    // owner/text boundary. Expected text comes from the published formatter.
    for length in 0..=29 {
        let bytes = vec![0xa5; length];
        let owner = Principal::from_slice(&bytes);
        let old_owner = PublishedPrincipal::from_slice(&bytes);
        for bit in 0..256 {
            let mut subaccount = [0; 32];
            subaccount[bit / 8] = 1 << (bit % 8);
            let value = Account {
                owner,
                subaccount: Some(subaccount),
            };
            let old = PublishedAccount {
                owner: old_owner,
                subaccount: Some(subaccount),
            };
            let expected = old.to_string();
            let mut buffer = [0; Account::MAX_LENGTH_IN_TEXT];
            assert_eq!(value.write_text(&mut buffer), expected);
            assert_eq!(value.to_string(), expected);
            assert_eq!(
                expected.parse::<Account>().unwrap().subaccount,
                Some(subaccount)
            );
            assert_eq!(
                expected.to_uppercase().parse::<Account>().is_ok(),
                expected.to_uppercase().parse::<PublishedAccount>().is_ok()
            );
        }
    }
}

#[test]
fn principal_refusals_match_published_typed_contract() {
    use candidate_types::PrincipalError as New;
    use ic_platform_types::PrincipalError as Old;
    let compare = |input: &str| match (
        Principal::from_text(input),
        PublishedPrincipal::from_text(input),
    ) {
        (Ok(new), Ok(old)) => assert_eq!(new.as_slice(), old.as_slice()),
        (Err(New::BytesTooLong()), Err(Old::BytesTooLong()))
        | (Err(New::InvalidBase32()), Err(Old::InvalidBase32()))
        | (Err(New::TextTooShort()), Err(Old::TextTooShort()))
        | (Err(New::TextTooLong()), Err(Old::TextTooLong()))
        | (Err(New::CheckSequenceNotMatch()), Err(Old::CheckSequenceNotMatch())) => {}
        (Err(New::AbnormalGrouped(new)), Err(Old::AbnormalGrouped(old))) => {
            assert_eq!(new.as_slice(), old.as_slice())
        }
        other => panic!("different Principal admission for {input:?}: {other:?}"),
    };
    for input in ["", "a", "aaaa", "-----", "aaaaaaa", "aaaaa-aa-", "é", "☃"] {
        compare(input);
    }
    for length in [62, 63, 64, 65, 136, 1024] {
        compare(&"a".repeat(length));
    }
    for length in 0..=29 {
        let text = Principal::from_slice(&vec![0x5a; length]).to_text();
        compare(&text);
        compare(&text.to_uppercase());
        compare(&text.replace('-', ""));
        let mut changed = text.as_bytes().to_vec();
        for last_symbol in *b"abcdefghijklmnopqrstuvwxyz234567" {
            changed[text.len() - 1] = last_symbol;
            compare(std::str::from_utf8(&changed).unwrap());
        }
        for end in 0..text.len() {
            compare(&text[..end]);
            for replacement in *b"az0!-" {
                let mut changed = text.as_bytes().to_vec();
                changed[end] = replacement;
                compare(std::str::from_utf8(&changed).unwrap());
            }
        }
    }
}

#[cfg(feature = "self-authentication")]
#[test]
fn published_derivation_compatibility() {
    for len in [0, 1, 28, 29, 55, 56, 63, 64, 65, 127, 128, 1024] {
        let bytes = vec![0x5a; len];
        assert_eq!(
            Principal::self_authenticating(&bytes).as_slice(),
            PublishedPrincipal::self_authenticating(&bytes).as_slice()
        );
    }
}
