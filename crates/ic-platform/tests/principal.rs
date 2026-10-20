//! The unconditional identity surface works without optional capabilities.
use ic_platform::Principal;

#[test]
fn principal_is_available_without_features() {
    let _: ic_platform::PrincipalError = Principal::from_text("invalid").unwrap_err();
    let principal = Principal::from_text("aaaaa-aa").unwrap();
    assert_eq!(principal, Principal::management_canister());
    assert_eq!(principal.to_text(), "aaaaa-aa");
    assert_eq!(Principal::from_slice(principal.as_slice()), principal);

    #[cfg(feature = "candid")]
    {
        let contract_principal: ic_platform::candid::contract::Principal = principal.into();
        assert_eq!(Principal::from(contract_principal), principal);
    }
}

#[test]
fn owned_identity_roundtrips_at_external_boundaries() {
    for len in 0..=29 {
        let owned = Principal::from_slice(&[2; 29][..len]);
        let external: ic_platform::principal::ExternalPrincipal = owned.into();
        assert_eq!(external.as_slice(), owned.as_slice());
        assert_eq!(Principal::from(external), owned);
        assert_eq!(Principal::from_text(owned.to_text()).unwrap(), owned);
    }
    assert!(Principal::try_from_slice(&[0; 30]).is_err());
}

#[test]
fn owned_account_needs_no_ledger_or_candid_feature() {
    use ic_platform::{Account, Principal, Subaccount};
    let mut subaccount: Subaccount = [0; 32];
    subaccount[31] = 1;
    let account = Account {
        owner: Principal::anonymous(),
        subaccount: Some(subaccount),
    };
    assert_eq!(account.to_string().parse::<Account>().unwrap(), account);
}
