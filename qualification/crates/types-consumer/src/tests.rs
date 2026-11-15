use candidate_types::{Account, Principal};
use ic_platform_types::{Account as PublishedAccount, Principal as PublishedPrincipal};

#[test]
fn literal_public_consumer() {
    super::fixtures::check();
}

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
