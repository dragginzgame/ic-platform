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

#[cfg(test)]
mod tests;
