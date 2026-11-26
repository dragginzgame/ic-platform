use candidate_types::Principal;
use ic_principal::Principal as DfinityPrincipal;

#[test]
fn principal_matches_dfinity_text_and_bytes() {
    for length in 0..=29 {
        for seed in [0_u8, 85, 170, 255] {
            let bytes: Vec<_> = (0..length).map(|i| seed.wrapping_add(i * 7)).collect();
            let expected = DfinityPrincipal::from_slice(&bytes).to_text();
            assert_eq!(Principal::from_slice(&bytes).to_text(), expected);
            for text in [expected.clone(), expected.to_ascii_uppercase()] {
                let dfinity = DfinityPrincipal::from_text(&text).unwrap();
                assert_eq!(
                    Principal::from_text(&text).unwrap().as_slice(),
                    dfinity.as_slice()
                );
            }

            let mut changed = expected.into_bytes();
            let last = changed.len() - 1;
            for symbol in *b"abcdefghijklmnopqrstuvwxyz234567" {
                changed[last] = symbol;
                let text = std::str::from_utf8(&changed).unwrap();
                let local = Principal::from_text(text);
                let dfinity = DfinityPrincipal::from_text(text);
                assert_eq!(
                    local.as_ref().ok().map(Principal::as_slice),
                    dfinity.as_ref().ok().map(DfinityPrincipal::as_slice),
                    "{text}"
                );
            }
        }
    }

    for text in ["", "!", "aaaaaaa", "aaaaa-aa-", "aaaaa-ab", "baaaa-aa", "é"] {
        assert!(Principal::from_text(text).is_err(), "{text}");
        assert!(DfinityPrincipal::from_text(text).is_err(), "{text}");
    }
}
