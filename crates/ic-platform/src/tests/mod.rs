//! Independent upstream oracles; none of this module is part of the library build.
#[cfg(all(
    feature = "cmc-types",
    feature = "cycles-ledger-types",
    feature = "icp-ledger-types",
    feature = "nns-registry-types"
))]
mod external_contract_consumer;
#[cfg(all(
    feature = "icrc-ledger-types",
    feature = "cmc-types",
    feature = "cycles-ledger-types",
    feature = "icp-ledger-types",
    feature = "nns-registry-types",
    feature = "certified-assets-types"
))]
mod external_contract_schema;
#[cfg(feature = "icrc-ledger-types")]
mod icrc_consumer;
#[cfg(all(
    feature = "cmc-types",
    feature = "cycles-ledger-types",
    feature = "certified-assets-types"
))]
mod owned_external_contracts;
#[cfg(feature = "candid-codec")]
pub(crate) mod reference;
