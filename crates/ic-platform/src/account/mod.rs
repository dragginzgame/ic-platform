//! Stable owned account identities and explicitly selected external ledger conversions.
//!
//! ```compile_fail
//! ic_platform::candid::contract::encode_one(ic_platform::Account::from(ic_platform::Principal::anonymous())).unwrap();
//! ```

pub use ic_platform_types::account::{Account, AccountError, DEFAULT_SUBACCOUNT, Subaccount};

/// Selected external Candid ledger DTO, available only with ledger integration.
#[cfg(feature = "icrc-ledger-types")]
pub use crate::icrc_ledger_types::Account as ExternalAccount;

#[cfg(feature = "icrc-ledger-types")]
impl From<ExternalAccount> for Account {
    fn from(value: ExternalAccount) -> Self {
        Self {
            owner: value.owner.into(),
            subaccount: value.subaccount,
        }
    }
}
#[cfg(feature = "icrc-ledger-types")]
impl From<Account> for ExternalAccount {
    fn from(value: Account) -> Self {
        Self {
            owner: value.owner.into(),
            subaccount: value.subaccount,
        }
    }
}
