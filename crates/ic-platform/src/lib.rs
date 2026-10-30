//! Controlled dependencies for IC runtime integration and required IC Candid interfaces.
//!
//! All owned application endpoints use IC Wire, including public endpoints.
//! Candid adapts required management/system, NNS/SNS, ledger and other Candid
//! interfaces; caller location does not select the codec. Existing Candid
//! checkpoints remain a legacy persisted-data compatibility exception.
//!
//! `principal` selects Principal and Account. `candid` selects bounded decoding;
//! `executor` selects the shared CDK executor without a Candid dependency.
//! `candid-parser` selects native external Candid parsing and type checking.
//! `candid-service` separately selects Candid service declaration generation.
//! `agent` selects native IC transport and identities, with consumer-owned policy.
//! `pocket-ic` selects native simulator client identities; Testkit owns lifecycle policy.
//! `secp256k1` selects native offline public-key derivation with upstream identities.
//! `bls` selects BLS keys, signatures and verification with upstream allocation support.
//! `blake3` selects the upstream hash function, streaming hasher and digest identity.
//! `canister-sig-creation` selects canister public-key formats and hashing.
//! `signature-verification` selects upstream canister-signature verification.
//! `cdk` selects owned explicit-codec entrypoints and a Candid-free runtime.
//! `runtime` selects those adapters and prepared `call` requests without the entrypoint generator.
//! `ic0` selects qualified raw system calls without Candid or an executor.
//! `management-canister-types` selects external management HTTP/history/snapshot/status DTOs.
//! `icrc-ledger-types` selects external ledger DTOs without storage integration.
//! Consumers retain protocol schemas, resource limits and lifecycle policy.

#[cfg(feature = "principal")]
pub mod account;
#[cfg(feature = "principal")]
pub use account::{Account, AccountError, Subaccount};

#[cfg(any(feature = "canister-sig-creation", feature = "signature-verification"))]
mod signature_upstream;

#[cfg(feature = "principal")]
pub mod principal;
#[cfg(feature = "principal")]
pub use principal::{Principal, PrincipalError};

#[cfg(all(feature = "agent", not(target_family = "wasm")))]
pub mod agent;

#[cfg(feature = "blake3")]
pub mod blake3;
#[cfg(feature = "bls")]
pub mod bls;
#[cfg(feature = "runtime")]
pub mod call;
#[cfg(feature = "candid")]
pub mod candid;
#[cfg(all(feature = "candid-parser", not(target_family = "wasm")))]
pub mod candid_parser;
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
pub mod executor;
#[cfg(feature = "ic0")]
pub mod ic0;
#[cfg(feature = "icp-ledger-types")]
pub mod icp_ledger_types;
#[cfg(feature = "nns-registry-types")]
pub mod nns_registry_types;

#[cfg(feature = "icrc-ledger-types")]
pub mod icrc_ledger_types;
#[cfg(feature = "management-canister-types")]
pub mod management_canister_types;
#[cfg(all(feature = "secp256k1", not(target_family = "wasm")))]
pub mod secp256k1;
#[cfg(feature = "signature-verification")]
pub mod signature_verification;
#[cfg(feature = "stable-structures")]
pub mod stable_structures;

#[cfg(all(feature = "pocket-ic", not(target_family = "wasm")))]
pub mod pocket_ic;
