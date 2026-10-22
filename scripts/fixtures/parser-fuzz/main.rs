#![no_main]

mod candid_malformed {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/candid_malformed.rs"
    ));
}

use ic_platform::{Account, Principal, account::ExternalAccount, principal::ExternalPrincipal};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    candid_malformed::exercise(data);
    if let Ok(text) = std::str::from_utf8(data) {
        let owned = Principal::from_text(text);
        if text.len() <= Principal::MAX_LENGTH_IN_TEXT {
            let external = ExternalPrincipal::from_text(text);
            assert_eq!(owned.is_ok(), external.is_ok());
            if let (Ok(owned), Ok(external)) = (owned, external) {
                assert_eq!(ExternalPrincipal::from(owned), external);
            }
        } else {
            assert!(owned.is_err());
        }
        let owned = text.parse::<Account>();
        if text.len() <= Account::MAX_LENGTH_IN_TEXT {
            let external = text.parse::<ExternalAccount>();
            assert_eq!(owned.is_ok(), external.is_ok());
            if let (Ok(owned), Ok(external)) = (owned, external) {
                assert_eq!(ExternalAccount::from(owned), external);
            }
        } else {
            assert!(owned.is_err());
        }
    }
    // Always exercise valid identities as well as arbitrary text.
    let principal = Principal::from_slice(&data[..data.len().min(29)]);
    assert_eq!(
        Principal::from_text(principal.to_text()).unwrap(),
        principal
    );
    let mut subaccount = [0; 32];
    let length = data.len().min(32);
    subaccount[..length].copy_from_slice(&data[..length]);
    let account = Account {
        owner: principal,
        subaccount: Some(subaccount),
    };
    assert_eq!(account.to_string().parse::<Account>().unwrap(), account);
});
