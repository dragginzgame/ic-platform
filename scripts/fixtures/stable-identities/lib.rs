pub fn principal(value: current::Principal) -> next::Principal {
    value
}
pub fn account(value: next::Account) -> current::Account {
    value
}
pub fn principal_error(value: current::PrincipalError) -> next::PrincipalError {
    value
}
pub fn account_error(value: next::AccountError) -> current::AccountError {
    value
}
pub fn canonical(value: current::Principal) -> ic_platform_types::Principal {
    value
}
#[cfg(feature = "external-ledger")]
#[test]
fn conversions_preserve_one_identity_and_explicit_zero_fields() {
    let owner = principal(current::Principal::anonymous());
    let registry: current::principal::ExternalPrincipal = owner.into();
    let owner = next::Principal::from(registry);
    let value = account(next::Account {
        owner,
        subaccount: Some([0; 32]),
    });
    let external: current::account::ExternalAccount = value.into();
    let restored = current::Account::from(external);
    assert_eq!(restored.subaccount, Some([0; 32]));
    assert_eq!(restored, current::Account::from(owner));
}

#[cfg(not(target_family = "wasm"))]
pub fn schemas() {
    fn schema<T: ic_wire::typescript::Wire>() {}
    schema::<current::Principal>();
    schema::<next::Principal>();
    schema::<current::Account>();
    schema::<next::Account>();
}

#[cfg(feature = "current-runtime")]
pub fn prepared_call(target: next::Principal) -> current::call::Call<'static, 'static> {
    current::call::Call::new(target, "probe", current::call::WaitPolicy::Unbounded)
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn real_schema_shapes_survive_facade_generations() {
    use ic_wire::typescript::native_schema;
    let principal = native_schema::<current::Principal>().unwrap();
    assert_eq!(principal, native_schema::<next::Principal>().unwrap());
    assert_eq!(principal["k"], "bytes");
    assert_eq!(principal["max"], 29);
    let account = native_schema::<current::Account>().unwrap();
    assert_eq!(account, native_schema::<next::Account>().unwrap());
    assert_eq!(account["fields"][0]["name"], "owner");
    assert_eq!(account["fields"][1]["name"], "subaccount");
}
