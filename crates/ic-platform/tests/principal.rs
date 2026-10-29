//! Owned identities need only the principal capability.
// Keep the identity in the ordinary library/Wasm proof as well as its tests.
pub use ic_platform::Principal;

#[test]
fn principal_needs_only_its_own_capability() {
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
