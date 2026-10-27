//! New facade exports must retain exact registry type identities.
#[test]
fn selected_management_and_ledger_exports_have_exact_upstream_identity() {
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
    same!(
        ledger::TransferArg,
        icrc_ledger_types::icrc1::transfer::TransferArg
    );
    same!(ledger::Memo, icrc_ledger_types::icrc1::transfer::Memo);
    same!(
        ledger::icrc21::ConsentMessageRequest,
        icrc_ledger_types::icrc21::requests::ConsentMessageRequest
    );
    same!(
        ledger::icrc21::ConsentInfo,
        icrc_ledger_types::icrc21::responses::ConsentInfo
    );
    same!(
        ledger::icrc21::Value,
        icrc_ledger_types::icrc21::responses::Value
    );
    same!(
        ledger::icrc21::Icrc21Error,
        icrc_ledger_types::icrc21::errors::Icrc21Error
    );
    assert_ne!(
        TypeId::of::<ledger::Value>(),
        TypeId::of::<ledger::icrc21::Value>()
    );
}
