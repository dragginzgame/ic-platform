//! Required external IC protocol codecs, independent of transport and recovery.
//!
//! Encode before submission and retain those exact bytes for intent journals.
//! Callers select the destination, wait, attachment, budgets and reply arity.
//! Returned protocol errors are decoded values, separate from transport failures.
//! Decoding authenticates nothing and never permits resubmission. The parent
//! module's Wasm depth limitation applies unless callers first select
//! [`super::DecodeLimits::with_nesting`] and admit the reply bytes.

use super::{DecodeError, DecodeLimits, config};
use candid::{de::IDLDeserialize, utils::ArgumentDecoder};

/// Caller-selected handling of the declared top-level reply arity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplyArity {
    /// Retain upstream optional omission and skipping within caller work quotas.
    Compatible,
    /// Require the declared value, even if optional, and refuse extra values
    /// before attempting to skip them. Zero-result operations require zero values.
    Exact,
}

fn reply<'a, T: ArgumentDecoder<'a>>(
    bytes: &'a [u8],
    limits: DecodeLimits,
    arity: ReplyArity,
    requires_value: bool,
) -> Result<T, DecodeError> {
    let mut decoder = IDLDeserialize::new_with_config(bytes, &config(bytes, limits)?)
        .map_err(DecodeError::Candid)?;
    if arity == ReplyArity::Exact && requires_value && decoder.is_done() {
        return Err(DecodeError::Candid(candid::Error::msg(
            "missing external protocol reply value",
        )));
    }
    let value = T::decode(&mut decoder).map_err(DecodeError::Candid)?;
    if arity == ReplyArity::Exact && !decoder.is_done() {
        return Err(DecodeError::Candid(candid::Error::msg(
            "unexpected additional external protocol reply values",
        )));
    }
    decoder.done().map_err(DecodeError::Candid)?;
    Ok(value)
}

#[cfg(any(
    feature = "management-canister-types",
    feature = "cmc-types",
    feature = "icp-ledger-types",
    feature = "icrc-ledger-types",
    feature = "nns-registry-types",
    feature = "cycles-ledger-types",
    all(feature = "certified-assets-types", not(target_family = "wasm"))
))]
macro_rules! single {
    ($module:ident, $method:literal, $request:ty, $response:ty) => {
        #[doc = concat!("External `", $method, "`: one request value and one result value.")]
        pub mod $module {
            /// Exact external method name; destination and transport remain caller-owned.
            pub const METHOD: &str = $method;
            /// Encode one argument without submitting, normalizing or mutating it.
            ///
            /// # Errors
            /// Returns the existing Candid encoding error.
            pub fn encode(request: &$request) -> Result<Vec<u8>, candid::Error> {
                candid::encode_one(request)
            }
            /// Decode one result, preserving the typed external success/error value.
            ///
            /// # Errors
            /// Refuses malformed, over-budget or arity-incompatible replies.
            pub fn decode(
                bytes: &[u8],
                limits: super::super::DecodeLimits,
                arity: super::super::ReplyArity,
            ) -> Result<$response, super::super::DecodeError> {
                super::super::reply::<($response,)>(bytes, limits, arity, true)
                    .map(|(value,)| value)
            }
        }
    };
}
macro_rules! empty_request {
    ($module:ident, $method:literal, $response:ty) => {
        #[doc = concat!("External `", $method, "`: zero request values and one result value.")]
        pub mod $module {
            /// Exact external method name.
            pub const METHOD: &str = $method;
            /// Encode the explicit zero-argument envelope, without submitting it.
            ///
            /// # Errors
            /// Returns the existing Candid encoding error.
            pub fn encode() -> Result<Vec<u8>, candid::Error> {
                candid::encode_args(())
            }
            /// Decode one result with caller-selected limits and arity.
            ///
            /// # Errors
            /// Refuses malformed, over-budget or arity-incompatible replies.
            pub fn decode(
                bytes: &[u8],
                limits: super::super::DecodeLimits,
                arity: super::super::ReplyArity,
            ) -> Result<$response, super::super::DecodeError> {
                super::super::reply::<($response,)>(bytes, limits, arity, true)
                    .map(|(value,)| value)
            }
        }
    };
}

#[cfg(feature = "management-canister-types")]
macro_rules! empty_reply {
    ($module:ident, $method:literal, $request:ty) => {
        #[doc = concat!("External `", $method, "`: one request value and zero reply values.")]
        pub mod $module {
            /// Exact external method name; no destination or spending authority.
            pub const METHOD: &str = $method;
            /// Encode the complete request before any caller-owned submission.
            ///
            /// # Errors
            /// Preserves the existing Candid encoding error.
            pub fn encode(request: &$request) -> Result<Vec<u8>, candid::Error> {
                candid::encode_one(request)
            }
            /// Decode zero results with explicit caller arity and resource policy.
            ///
            /// # Errors
            /// Refuses malformed, over-budget or arity-incompatible replies.
            pub fn decode(
                bytes: &[u8],
                limits: super::super::DecodeLimits,
                arity: super::super::ReplyArity,
            ) -> Result<(), super::super::DecodeError> {
                super::super::reply(bytes, limits, arity, false)
            }
        }
    };
}

/// Management protocol operations; no controller, settings or allocation policy.
pub mod management {
    empty_request!(raw_rand, "raw_rand", Vec<u8>);
    #[cfg(feature = "management-canister-types")]
    single!(
        canister_status,
        "canister_status",
        crate::management_canister_types::CanisterIdRecord,
        crate::management_canister_types::CanisterStatusResult
    );
    #[cfg(feature = "management-canister-types")]
    single!(
        canister_info,
        "canister_info",
        crate::management_canister_types::CanisterInfoArgs,
        crate::management_canister_types::CanisterInfoResult
    );
    #[cfg(feature = "management-canister-types")]
    empty_reply!(
        update_settings,
        "update_settings",
        crate::management_canister_types::UpdateSettingsArgs
    );
    #[cfg(feature = "management-canister-types")]
    single!(
        upload_chunk,
        "upload_chunk",
        crate::management_canister_types::UploadChunkArgs,
        crate::management_canister_types::ChunkHash
    );

    #[cfg(feature = "management-canister-types")]
    single!(
        stored_chunks,
        "stored_chunks",
        crate::management_canister_types::CanisterIdRecord,
        Vec<crate::management_canister_types::ChunkHash>
    );

    #[cfg(feature = "management-canister-types")]
    single!(
        list_canister_snapshots,
        "list_canister_snapshots",
        crate::management_canister_types::CanisterIdRecord,
        Vec<crate::management_canister_types::Snapshot>
    );

    #[cfg(feature = "management-canister-types")]
    single!(
        ecdsa_public_key,
        "ecdsa_public_key",
        crate::management_canister_types::EcdsaPublicKeyArgs,
        crate::management_canister_types::EcdsaPublicKeyResult
    );

    #[cfg(feature = "management-canister-types")]
    single!(
        sign_with_ecdsa,
        "sign_with_ecdsa",
        crate::management_canister_types::SignWithEcdsaArgs,
        crate::management_canister_types::SignWithEcdsaResult
    );

    #[cfg(feature = "management-canister-types")]
    empty_reply!(
        clear_chunk_store,
        "clear_chunk_store",
        crate::management_canister_types::CanisterIdRecord
    );

    #[cfg(feature = "management-canister-types")]
    empty_reply!(
        install_chunked_code,
        "install_chunked_code",
        crate::management_canister_types::InstallChunkedCodeArgs
    );

    #[cfg(feature = "management-canister-types")]
    empty_reply!(
        uninstall_code,
        "uninstall_code",
        crate::management_canister_types::UninstallCodeArgs
    );

    #[cfg(feature = "management-canister-types")]
    empty_reply!(
        start_canister,
        "start_canister",
        crate::management_canister_types::CanisterIdRecord
    );

    #[cfg(feature = "management-canister-types")]
    empty_reply!(
        stop_canister,
        "stop_canister",
        crate::management_canister_types::CanisterIdRecord
    );

    #[cfg(feature = "management-canister-types")]
    empty_reply!(
        delete_canister,
        "delete_canister",
        crate::management_canister_types::CanisterIdRecord
    );

    #[cfg(feature = "management-canister-types")]
    empty_reply!(
        deposit_cycles,
        "deposit_cycles",
        crate::management_canister_types::CanisterIdRecord
    );

    #[cfg(feature = "management-canister-types")]
    empty_reply!(
        delete_canister_snapshot,
        "delete_canister_snapshot",
        crate::management_canister_types::DeleteCanisterSnapshotArgs
    );
}

/// Direct CMC operations; distinct from Cycles Ledger creation and withdrawals.
#[cfg(feature = "cmc-types")]
pub mod cmc {
    empty_request!(
        get_icp_xdr_conversion_rate,
        "get_icp_xdr_conversion_rate",
        crate::cmc_types::IcpXdrConversionRateResponse
    );
    single!(
        notify_top_up,
        "notify_top_up",
        crate::cmc_types::NotifyTopUpArg,
        crate::cmc_types::NotifyTopUpResult
    );
    single!(
        notify_mint_cycles,
        "notify_mint_cycles",
        crate::cmc_types::NotifyMintCyclesArg,
        crate::cmc_types::NotifyMintCyclesResult
    );
}

/// Legacy ICP ledger operations; callers retain transfer and archive authority.
#[cfg(feature = "icp-ledger-types")]
pub mod icp_ledger {
    single!(
        transfer,
        "transfer",
        crate::icp_ledger_types::TransferArgs,
        crate::icp_ledger_types::TransferResult
    );
    single!(
        query_encoded_blocks,
        "query_encoded_blocks",
        crate::icp_ledger_types::GetBlocksArgs,
        crate::icp_ledger_types::QueryEncodedBlocksResponse
    );
}

/// ICRC operations retain the exact upstream DTO identities, including Accounts.
/// The standard get_blocks projection omits legacy Nat64; use the separate
/// cycles_ledger::get_blocks codec for that provider's wider value schema.
#[cfg(feature = "icrc-ledger-types")]
pub mod icrc {
    empty_request!(fee, "icrc1_fee", candid::Nat);
    single!(
        balance_of,
        "icrc1_balance_of",
        crate::icrc_ledger_types::Account,
        candid::Nat
    );
    single!(transfer, "icrc1_transfer", crate::icrc_ledger_types::TransferArg, Result<candid::Nat, crate::icrc_ledger_types::TransferError>);
    empty_request!(
        get_tip_certificate,
        "icrc3_get_tip_certificate",
        Option<crate::icrc_ledger_types::ICRC3DataCertificate>
    );
    single!(
        get_blocks,
        "icrc3_get_blocks",
        Vec<crate::icrc_ledger_types::GetBlocksRequest>,
        crate::icrc_ledger_types::GetBlocksResult
    );
}

/// NNS Registry operations; subnet interpretation remains consumer-owned.
#[cfg(feature = "nns-registry-types")]
pub mod nns_registry {
    single!(
        get_subnet_for_canister,
        "get_subnet_for_canister",
        crate::nns_registry_types::GetSubnetForCanisterRequest,
        crate::nns_registry_types::GetSubnetForCanisterResponse
    );
}

/// Cycles Ledger requests preserve debit, duplicate and topology fields unchanged.
#[cfg(feature = "cycles-ledger-types")]
pub mod cycles_ledger {
    #[cfg(feature = "icrc-ledger-types")]
    single!(
        get_blocks,
        "icrc3_get_blocks",
        Vec<crate::icrc_ledger_types::GetBlocksRequest>,
        crate::cycles_ledger_types::GetBlocksResult
    );
    single!(
        create_canister,
        "create_canister",
        crate::cycles_ledger_types::CreateCanisterArgs,
        crate::cycles_ledger_types::CreateCanisterResult
    );
    single!(
        withdraw,
        "withdraw",
        crate::cycles_ledger_types::WithdrawArgs,
        crate::cycles_ledger_types::WithdrawResult
    );
}

/// Native certified-asset acquisition; decoding does not authenticate content.
#[cfg(all(feature = "certified-assets-types", not(target_family = "wasm")))]
pub mod certified_assets {
    single!(
        get,
        "get",
        crate::certified_assets_types::GetArgs,
        crate::certified_assets_types::GetResult
    );
    single!(
        get_chunk,
        "get_chunk",
        crate::certified_assets_types::GetChunkArgs,
        crate::certified_assets_types::GetChunkResult
    );
}

/// Legacy archive callback signature. Consumers validate the callback's source
/// and method themselves; no guessed method or automatic archive follow is added.
#[cfg(feature = "icp-ledger-types")]
pub mod archive_encoded_blocks {
    /// Encode the admitted callback range, independently of its destination/method.
    ///
    /// # Errors
    /// Preserves the existing Candid encoding error.
    pub fn encode(
        request: &crate::icp_ledger_types::GetBlocksArgs,
    ) -> Result<Vec<u8>, candid::Error> {
        candid::encode_one(request)
    }
    /// Decode the callback's typed success/error without validating its authority.
    ///
    /// # Errors
    /// Refuses malformed, over-budget or arity-incompatible replies.
    pub fn decode(
        bytes: &[u8],
        limits: super::DecodeLimits,
        arity: super::ReplyArity,
    ) -> Result<crate::icp_ledger_types::QueryEncodedBlocksResult, super::DecodeError> {
        super::reply::<(crate::icp_ledger_types::QueryEncodedBlocksResult,)>(
            bytes, limits, arity, true,
        )
        .map(|(value,)| value)
    }
}
