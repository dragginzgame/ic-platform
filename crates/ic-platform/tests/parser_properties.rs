//! Reproducible generated parser corpus; no cryptographic behavior is changed.
use ic_platform::{Account, Principal, principal::ExternalPrincipal};

#[test]
fn generated_unicode_and_boundary_inputs_are_total() {
    let alphabet = ['a', '0', '-', '.', 'é', '中', '\0', '\u{10ffff}'];
    let mut state = 0x1234_5678_u64;
    for length in 0..=160 {
        for _ in 0..32 {
            let text: String = (0..length)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    alphabet[state as usize % alphabet.len()]
                })
                .collect();
            if let Ok(value) = Principal::from_text(&text) {
                assert_eq!(Principal::from_text(value.to_text()).unwrap(), value);
                assert_eq!(ExternalPrincipal::from_text(&text).unwrap(), value.into());
            }
            if let Ok(value) = text.parse::<Account>() {
                assert_eq!(value.to_string().parse::<Account>().unwrap(), value);
            }
        }
    }
}

#[test]
fn generated_account_roundtrips_preserve_effective_identity() {
    let mut state = 0x8765_4321_u64;
    for index in 0..4096 {
        let mut bytes = [0u8; 32];
        for byte in &mut bytes {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state as u8;
        }
        let account = Account {
            owner: Principal::from_slice(&bytes[..index % 30]),
            subaccount: Some(bytes),
        };
        assert_eq!(account.to_string().parse::<Account>().unwrap(), account);
        #[cfg(feature = "icrc-ledger-types")]
        {
            let external: ic_platform::account::ExternalAccount = account.into();
            assert_eq!(external.to_string(), account.to_string());
        }
    }
}
