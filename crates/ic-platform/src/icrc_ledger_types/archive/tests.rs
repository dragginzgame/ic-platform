use super::*;

struct NoTraits;

#[test]
fn cloning_needs_no_marker_traits_and_owned_conversions_move_the_method() {
    let owner = Principal::management_canister();
    let reference = QueryArchiveFn::<NoTraits, NoTraits>::new(owner, "archive.é");
    let cloned = reference.clone();
    assert_eq!(cloned.canister_id, owner);
    assert_eq!(cloned.method, "archive.é");

    let allocation = reference.method.as_ptr();
    let function = Func::from(reference);
    assert_eq!(function.principal, owner);
    assert_eq!(function.method.as_ptr(), allocation);
    let restored: Result<QueryArchiveFn<NoTraits, NoTraits>, String> = function.try_into();
    let restored = restored.unwrap();
    assert_eq!(restored.canister_id, owner);
    assert_eq!(restored.method.as_ptr(), allocation);
    assert_eq!(restored.method, "archive.é");
}

#[test]
fn development_only_registry_bridges_preserve_identity_and_unrestricted_methods() {
    for owner in [
        Principal::management_canister(),
        Principal::anonymous(),
        Principal::try_from_slice(&[42; 29]).unwrap(),
    ] {
        for method in ["", "archive.é!?", "\0"] {
            let reference = QueryArchiveFn::<NoTraits, NoTraits>::new(owner, method);
            let registry = candid::types::reference::Func::from(reference);
            assert_eq!(registry.principal.as_slice(), owner.as_slice());
            assert_eq!(registry.method, method);
            let restored: Result<QueryArchiveFn<NoTraits, NoTraits>, String> = registry.try_into();
            let restored = restored.unwrap();
            assert_eq!(restored.canister_id, owner);
            assert_eq!(restored.method, method);
        }
    }
}
