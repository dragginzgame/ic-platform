//! Owned wire mappings against the independently retained upstream encoder.
#![cfg(not(target_family = "wasm"))]
use crate::{certified_assets_types::*, cmc_types::*};
fn check<
    T: crate::candid_codec::Encode
        + for<'a> crate::candid_codec::decoder::Decode<'a>
        + std::fmt::Debug
        + PartialEq,
>(
    value: T,
) {
    crate::tests::reference::roundtrip(&value);
}
#[test]
fn cmc_all_requests_response_branches_and_large_naturals() {
    check(NotifyTopUpArg {
        block_index: u64::MAX,
        canister_id: crate::Principal::from_slice(&[42; 29]),
    });
    for present in [false, true] {
        check(NotifyMintCyclesArg {
            block_index: u64::MAX,
            to_subaccount: present.then(|| vec![0; 32]),
            deposit_memo: present.then(|| vec![255; 8]),
        });
        check(NotifyTopUpResult::Err(NotifyError::Refunded {
            reason: "refund 🌍".into(),
            block_index: present.then_some(u64::MAX),
        }));
    }
    for error in [
        NotifyError::Processing,
        NotifyError::TransactionTooOld(u64::MAX),
        NotifyError::InvalidTransaction("invalid".into()),
        NotifyError::Other {
            error_code: u64::MAX,
            error_message: "other".into(),
        },
    ] {
        check(NotifyMintCyclesResult::Err(error));
    }
    let amount: crate::candid_codec::Nat =
        "340282366920938463463374607431768211456".parse().unwrap();
    check(NotifyTopUpResult::Ok(amount.clone()));
    check(NotifyMintCyclesResult::Ok(NotifyMintCyclesSuccess {
        block_index: amount.clone(),
        minted: amount.clone(),
        balance: amount,
    }));
    check(IcpXdrConversionRateResponse {
        data: IcpXdrConversionRate {
            timestamp_seconds: u64::MAX,
            xdr_permyriad_per_icp: 42,
        },
        hash_tree: vec![0, 255],
        certificate: vec![1, 2, 3],
    });
}
#[test]
fn assets_requests_results_optional_digest_and_large_index() {
    check(GetArgs {
        key: "/🌍".into(),
        accept_encodings: vec!["identity".into(), "gzip".into()],
    });
    for sha256 in [None, Some(vec![7; 32])] {
        let amount = "340282366920938463463374607431768211456"
            .parse::<crate::candid_codec::Nat>()
            .unwrap();
        check(GetResult {
            content: vec![0, 255],
            content_type: "text/html".into(),
            content_encoding: "identity".into(),
            sha256: sha256.clone(),
            total_length: amount.clone(),
        });
        check(GetChunkArgs {
            key: "/🌍".into(),
            content_encoding: "gzip".into(),
            index: amount,
            sha256,
        });
    }
    check(GetChunkResult { content: vec![] });
    check(GetChunkResult {
        content: vec![0, 255],
    });
}

#[test]
fn cycles_settings_selection_and_all_errors() {
    use crate::cycles_ledger_types::*;
    let principal = crate::Principal::from_slice(&[42; 29]);
    for log_visibility in [
        LogVisibility::Controllers,
        LogVisibility::Public,
        LogVisibility::AllowedViewers(vec![principal]),
    ] {
        check(CreateCanisterArgs {
            amount: "340282366920938463463374607431768211456".parse().unwrap(),
            from_subaccount: Some(vec![0; 32]),
            created_at_time: Some(u64::MAX),
            creation_args: Some(CmcCreateCanisterArgs {
                subnet_selection: Some(SubnetSelection::Subnet { subnet: principal }),
                settings: Some(CanisterSettings {
                    controllers: Some(vec![principal]),
                    compute_allocation: Some(100u64.into()),
                    memory_allocation: Some(0u64.into()),
                    freezing_threshold: Some(u64::MAX.into()),
                    reserved_cycles_limit: Some(1u64.into()),
                    wasm_memory_limit: Some(2u64.into()),
                    wasm_memory_threshold: Some(3u64.into()),
                    log_visibility: Some(log_visibility),
                    environment_variables: Some(vec![EnvironmentVariable {
                        name: "NAME".into(),
                        value: "🌍".into(),
                    }]),
                }),
            }),
        });
    }
    check(CreateCanisterArgs {
        amount: 0u64.into(),
        from_subaccount: None,
        created_at_time: None,
        creation_args: None,
    });
    check(SubnetSelection::Filter(SubnetFilter {
        subnet_type: Some("application".into()),
    }));
    check(CreateCanisterResult::Ok(CreateCanisterSuccess {
        block_id: 1u64.into(),
        canister_id: principal,
    }));
    for error in [
        CreateCanisterError::GenericError {
            message: "error".into(),
            error_code: u128::MAX.into(),
        },
        CreateCanisterError::TemporarilyUnavailable,
        CreateCanisterError::Duplicate {
            duplicate_of: 1u64.into(),
            canister_id: Some(principal),
        },
        CreateCanisterError::CreatedInFuture {
            ledger_time: u64::MAX,
        },
        CreateCanisterError::FailedToCreate {
            error: "failure".into(),
            refund_block: None,
            fee_block: Some(1u64.into()),
        },
        CreateCanisterError::TooOld,
        CreateCanisterError::InsufficientFunds {
            balance: 0u64.into(),
        },
    ] {
        check(CreateCanisterResult::Err(error));
    }
    check(WithdrawArgs {
        to: principal,
        from_subaccount: None,
        created_at_time: Some(u64::MAX),
        amount: u128::MAX.into(),
    });
    check(WithdrawResult::Ok(u128::MAX.into()));
    for code in [
        RejectionCode::NoError,
        RejectionCode::CanisterError,
        RejectionCode::SysTransient,
        RejectionCode::DestinationInvalid,
        RejectionCode::Unknown,
        RejectionCode::SysFatal,
        RejectionCode::CanisterReject,
    ] {
        check(WithdrawResult::Err(WithdrawError::FailedToWithdraw {
            fee_block: Some(1u64.into()),
            rejection_code: code,
            rejection_reason: "failure".into(),
        }));
    }
    for error in [
        WithdrawError::GenericError {
            message: "error".into(),
            error_code: u128::MAX.into(),
        },
        WithdrawError::TemporarilyUnavailable,
        WithdrawError::Duplicate {
            duplicate_of: 1u64.into(),
        },
        WithdrawError::BadFee {
            expected_fee: 1u64.into(),
        },
        WithdrawError::InvalidReceiver {
            receiver: principal,
        },
        WithdrawError::CreatedInFuture {
            ledger_time: u64::MAX,
        },
        WithdrawError::TooOld,
        WithdrawError::InsufficientFunds {
            balance: 0u64.into(),
        },
    ] {
        check(WithdrawResult::Err(error));
    }
}

#[test]
#[cfg(feature = "icp-ledger-types")]
fn icp_transfer_and_archive_callbacks_preserve_wire() {
    use crate::icp_ledger_types::*;
    check(TransferArgs {
        memo: u64::MAX,
        amount: Tokens { e8s: 10 },
        fee: Tokens { e8s: 1 },
        from_subaccount: Some(vec![0; 32]),
        to: vec![42; 32],
        created_at_time: Some(TimeStamp {
            timestamp_nanos: u64::MAX,
        }),
    });
    check(TransferResult::Ok(u64::MAX));
    for error in [
        TransferError::BadFee {
            expected_fee: Tokens { e8s: 1 },
        },
        TransferError::InsufficientFunds {
            balance: Tokens { e8s: 0 },
        },
        TransferError::TxTooOld {
            allowed_window_nanos: u64::MAX,
        },
        TransferError::TxCreatedInFuture,
        TransferError::TxDuplicate {
            duplicate_of: u64::MAX,
        },
    ] {
        check(TransferResult::Err(error));
    }
    check(GetBlocksArgs {
        start: 0,
        length: u64::MAX,
    });
    for principal in [
        crate::Principal::management_canister(),
        crate::Principal::anonymous(),
        crate::Principal::from_slice(&[42; 29]),
    ] {
        check(QueryEncodedBlocksFn::new(principal, "archive_🌍".into()));
        check(QueryEncodedBlocksResponse {
            certificate: Some(vec![0, 255]),
            blocks: vec![vec![], vec![0, 255]],
            chain_length: u64::MAX,
            first_block_index: 0,
            archived_blocks: vec![ArchivedEncodedBlocksRange {
                callback: QueryEncodedBlocksFn::new(principal, "archive".into()),
                start: 0,
                length: u64::MAX,
            }],
        });
    }
    check(QueryEncodedBlocksResponse {
        certificate: None,
        blocks: vec![],
        chain_length: 0,
        first_block_index: 0,
        archived_blocks: vec![],
    });
    check(QueryEncodedBlocksResult::Ok(vec![vec![0, 255]]));
    for error in [
        QueryArchiveError::BadFirstBlockIndex {
            requested_index: 0,
            first_valid_index: u64::MAX,
        },
        QueryArchiveError::Other {
            error_code: u64::MAX,
            error_message: "archive".into(),
        },
    ] {
        check(QueryEncodedBlocksResult::Err(error));
    }
}
