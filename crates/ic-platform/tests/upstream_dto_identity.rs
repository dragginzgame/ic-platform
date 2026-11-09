//! Management and ledger DTOs are owned; their protocol compatibility is checked separately.
#[test]
fn management_exports_have_owned_identity() {
    use ic_platform::{icrc_ledger_types as ledger, management_canister_types as mgmt};
    use std::any::TypeId;
    macro_rules! owned {
        ($facade:ty, $upstream:ty) => {
            assert_ne!(TypeId::of::<$facade>(), TypeId::of::<$upstream>());
        };
    }
    owned!(
        mgmt::CanisterSettings,
        ic_management_canister_types::CanisterSettings
    );
    owned!(
        mgmt::CanisterInstallMode,
        ic_management_canister_types::CanisterInstallMode
    );
    owned!(mgmt::Change, ic_management_canister_types::Change);
    owned!(
        mgmt::ChangeOrigin,
        ic_management_canister_types::ChangeOrigin
    );
    owned!(
        mgmt::InstallChunkedCodeArgs,
        ic_management_canister_types::InstallChunkedCodeArgs
    );
    owned!(
        mgmt::UninstallCodeArgs,
        ic_management_canister_types::UninstallCodeArgs
    );
    owned!(
        mgmt::EcdsaPublicKeyArgs,
        ic_management_canister_types::EcdsaPublicKeyArgs
    );
    owned!(
        mgmt::SignWithEcdsaArgs,
        ic_management_canister_types::SignWithEcdsaArgs
    );
    assert_ne!(
        TypeId::of::<ledger::Value>(),
        TypeId::of::<ledger::icrc21::Value>()
    );
}
