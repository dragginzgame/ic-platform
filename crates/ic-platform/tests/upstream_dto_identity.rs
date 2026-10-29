//! Management DTOs retain registry identities; selected ledger DTOs are Platform-owned.
#[test]
fn management_exports_retain_exact_upstream_identity() {
    use ic_platform::{icrc_ledger_types as ledger, management_canister_types as mgmt};
    use std::any::TypeId;
    macro_rules! same {
        ($facade:ty, $upstream:ty) => {
            assert_eq!(TypeId::of::<$facade>(), TypeId::of::<$upstream>());
        };
    }
    same!(
        mgmt::CanisterSettings,
        ic_management_canister_types::CanisterSettings
    );
    same!(
        mgmt::CanisterInstallMode,
        ic_management_canister_types::CanisterInstallMode
    );
    same!(mgmt::Change, ic_management_canister_types::Change);
    same!(
        mgmt::ChangeOrigin,
        ic_management_canister_types::ChangeOrigin
    );
    same!(
        mgmt::InstallChunkedCodeArgs,
        ic_management_canister_types::InstallChunkedCodeArgs
    );
    same!(
        mgmt::UninstallCodeArgs,
        ic_management_canister_types::UninstallCodeArgs
    );
    same!(
        mgmt::EcdsaPublicKeyArgs,
        ic_management_canister_types::EcdsaPublicKeyArgs
    );
    same!(
        mgmt::SignWithEcdsaArgs,
        ic_management_canister_types::SignWithEcdsaArgs
    );
    assert_ne!(
        TypeId::of::<ledger::Value>(),
        TypeId::of::<ledger::icrc21::Value>()
    );
}
