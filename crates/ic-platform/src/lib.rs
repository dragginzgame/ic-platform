//! Controlled dependencies for IC runtime integration and required IC Candid interfaces.
//!
//! All owned application endpoints use IC Wire, including public endpoints.
//! Candid adapts required management/system, NNS/SNS, ledger and other Candid
//! interfaces; caller location does not select the codec. Existing Candid
//! checkpoints remain a legacy persisted-data compatibility exception.
//!
//! `principal` selects Principal and Account; `candid-codec` selects owned binary encoding.
//! Legacy Candid decoder, parser and printer exports are removed.
//! `agent` selects native IC transport and identities, with consumer-owned policy.
//! `ic-protocol` selects native typed packet codecs without Agent or runtime.
//! `ic-protocol-fixtures` additionally selects native development protocol fixtures.
//! `bls` selects BLS keys, signatures and verification with upstream allocation support.
//! `canister-sig-creation` selects canister public-key formats and hashing.
//! `signature-verification` selects upstream canister-signature verification.
//! `runtime` selects raw adapters and prepared `call` requests; ordinary entrypoints select Runtime directly.
//! `ic0` selects qualified raw system calls without Candid or an executor.
//! `raw-rand` selects the dependency-free management entropy codec, without transport.
//! `candid-args` selects small, dependency-free external argument encoders.
//! `candid-codec` selects the owned typed binary codec without registry Candid.
//! `management-canister-types` selects external management HTTP/history/snapshot/status DTOs.
//! `icrc-ledger-types` selects external ledger DTOs without storage integration.
//! Consumers retain protocol schemas, resource limits and lifecycle policy.

#[cfg(feature = "principal")]
pub mod account;
#[cfg(feature = "principal")]
pub use account::{Account, AccountError, Subaccount};

#[cfg(feature = "principal")]
pub mod principal;
#[cfg(feature = "raw-rand")]
pub mod raw_rand;
#[cfg(feature = "principal")]
pub use principal::{Principal, PrincipalError};

#[cfg(all(feature = "agent", not(target_family = "wasm")))]
pub mod agent;

#[cfg(all(feature = "ic-protocol", not(target_family = "wasm")))]
pub mod ic_protocol;

#[cfg(feature = "bls")]
pub mod bls;
#[cfg(feature = "runtime")]
pub use ic_platform_runtime::call;
#[cfg(feature = "candid-args")]
pub mod candid_args;
#[cfg(feature = "candid-codec")]
pub mod candid_codec;
#[cfg(feature = "canister-sig-creation")]
pub mod canister_sig_creation;
#[cfg(feature = "runtime")]
pub mod cdk;
#[cfg(feature = "certification")]
pub mod certification;
#[cfg(all(feature = "certified-assets-types", not(target_family = "wasm")))]
pub mod certified_assets_types;
#[cfg(feature = "cmc-types")]
pub mod cmc_types;
#[cfg(feature = "cycles-ledger-types")]
pub mod cycles_ledger_types;
#[cfg(feature = "ecdsa")]
pub mod ecdsa;
#[cfg(feature = "ed25519")]
pub mod ed25519;
#[cfg(feature = "executor")]
pub use ic_platform_runtime::executor;
#[cfg(feature = "ic0")]
pub use ic_platform_runtime::ic0;
#[cfg(feature = "icp-ledger-types")]
pub mod icp_ledger_types;
#[cfg(feature = "nns-registry-types")]
pub mod nns_registry_types;

#[cfg(feature = "icrc-ledger-types")]
pub mod icrc_ledger_types;
#[cfg(feature = "management-canister-types")]
pub mod management_canister_types;
#[cfg(feature = "signature-verification")]
pub mod signature_verification;

#[cfg(feature = "runtime")]
#[doc(hidden)]
pub use ic_platform_runtime::__platform_println;

#[cfg(test)]
mod tests;
