//! The unconditional identity surface works without optional capabilities.
use ic_platform::Principal;

#[test]
fn principal_is_available_without_features() {
    let principal = Principal::from_text("aaaaa-aa").unwrap();
    assert_eq!(principal, Principal::management_canister());
    assert_eq!(principal.to_text(), "aaaaa-aa");
    assert_eq!(Principal::from_slice(principal.as_slice()), principal);

    #[cfg(feature = "candid")]
    {
        let contract_principal: ic_platform::candid::contract::Principal = principal;
        assert_eq!(contract_principal, principal);
    }
}
