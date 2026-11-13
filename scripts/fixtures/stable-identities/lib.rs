//! Compile-time identity proofs across distinct facade versions and the Types owner.

/// Return a current Principal through the next facade without conversion.
#[must_use]
pub fn principal(value: current::Principal) -> next::Principal {
    value
}
/// Return a next-facade Account through the current facade without conversion.
#[must_use]
pub fn account(value: next::Account) -> current::Account {
    value
}
/// Preserve the Principal error identity across facade versions.
#[must_use]
pub fn principal_error(value: current::PrincipalError) -> next::PrincipalError {
    value
}
/// Preserve the Account error identity across facade versions.
#[must_use]
pub fn account_error(value: next::AccountError) -> current::AccountError {
    value
}
/// Return a facade Principal as the canonical Types Principal without conversion.
#[must_use]
pub fn canonical(value: current::Principal) -> ic_platform_types::Principal {
    value
}

#[cfg(not(target_family = "wasm"))]
/// Require both facades' identity types to satisfy the same Wire schema trait.
pub fn schemas() {
    fn schema<T: ic_wire::typescript::Wire>() {}
    schema::<current::Principal>();
    schema::<next::Principal>();
    schema::<current::Account>();
    schema::<next::Account>();
}

#[cfg(feature = "current-runtime")]
/// Prepare a current Runtime call using the next facade's Principal identity.
pub fn prepared_call(target: next::Principal) -> current::call::Call<'static, 'static> {
    current::call::Call::new(target, "probe", current::call::WaitPolicy::Unbounded)
}

#[cfg(test)]
mod tests;
