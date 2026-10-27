//! Standalone portable consumer of external CMC, ICP, Cycles Ledger and Registry contracts.
#[cfg(test)]
mod tests {
    use ic_platform::{
        candid::{DecodeError, DecodeLimits, contract, decode_exactly_one},
        cmc_types as cmc, cycles_ledger_types as cycles, icp_ledger_types as icp,
        nns_registry_types as registry,
        principal::ExternalPrincipal as Principal,
    };

    fn limits() -> DecodeLimits {
        DecodeLimits {
            reply_bytes: 16_384,
            decoding_quota: 100_000,
            skipping_quota: 0,
            type_entries: 128,
            header_bytes: 8192,
        }
    }
    fn roundtrip<T>(value: T)
    where
        T: contract::CandidType
            + for<'de> contract::serde::Deserialize<'de>
            + PartialEq
            + std::fmt::Debug,
    {
        let bytes = contract::encode_one(&value).unwrap();
        assert_eq!(decode_exactly_one::<T>(&bytes, limits()).unwrap(), value);
        assert!(matches!(
            decode_exactly_one::<T>(
                &bytes,
                DecodeLimits {
                    reply_bytes: bytes.len() - 1,
                    ..limits()
                }
            ),
            Err(DecodeError::ReplyTooLarge { .. })
        ));
    }
    #[test]
    fn cmc_rates_notifications_and_unbounded_cycle_amounts() {
        roundtrip(cmc::NotifyTopUpArg {
            block_index: u64::MAX,
            canister_id: Principal::anonymous(),
        });
        roundtrip(cmc::IcpXdrConversionRateResponse {
            data: cmc::IcpXdrConversionRate {
                timestamp_seconds: u64::MAX,
                xdr_permyriad_per_icp: 42,
            },
            hash_tree: vec![1, 2],
            certificate: vec![3, 4],
        });
        roundtrip(cmc::NotifyMintCyclesArg {
            block_index: 42,
            to_subaccount: Some(vec![0; 32]),
            deposit_memo: Some(vec![7; 8]),
        });
        let amount: contract::Nat = "340282366920938463463374607431768211456".parse().unwrap();
        roundtrip(cmc::NotifyTopUpResult::Ok(amount.clone()));
        roundtrip(cmc::NotifyMintCyclesResult::Ok(
            cmc::NotifyMintCyclesSuccess {
                block_index: amount.clone(),
                balance: amount.clone(),
                minted: amount,
            },
        ));
        for error in [
            cmc::NotifyError::Processing,
            cmc::NotifyError::TransactionTooOld(u64::MAX),
            cmc::NotifyError::InvalidTransaction("invalid".into()),
            cmc::NotifyError::Refunded {
                reason: "refund".into(),
                block_index: Some(7),
            },
            cmc::NotifyError::Other {
                error_code: u64::MAX,
                error_message: "other".into(),
            },
        ] {
            roundtrip(cmc::NotifyMintCyclesResult::Err(error));
        }
    }
    #[test]
    fn legacy_icp_transfer_and_exact_archive_query_callback() {
        roundtrip(icp::TransferArgs {
            memo: u64::MAX,
            amount: icp::Tokens { e8s: 10 },
            fee: icp::Tokens { e8s: 1 },
            from_subaccount: Some(vec![0; 32]),
            to: vec![7; 32],
            created_at_time: Some(icp::TimeStamp {
                timestamp_nanos: 99,
            }),
        });
        for error in [
            icp::TransferError::BadFee {
                expected_fee: icp::Tokens { e8s: 10_000 },
            },
            icp::TransferError::InsufficientFunds {
                balance: icp::Tokens { e8s: 0 },
            },
            icp::TransferError::TxTooOld {
                allowed_window_nanos: u64::MAX,
            },
            icp::TransferError::TxCreatedInFuture,
            icp::TransferError::TxDuplicate { duplicate_of: 42 },
        ] {
            roundtrip(icp::TransferResult::Err(error));
        }
        roundtrip(icp::GetBlocksArgs {
            start: u64::MAX,
            length: 1,
        });
        roundtrip(icp::QueryEncodedBlocksResponse {
            certificate: Some(vec![1, 2]),
            blocks: vec![vec![0, 255]],
            chain_length: 43,
            first_block_index: 42,
            archived_blocks: vec![icp::ArchivedEncodedBlocksRange {
                callback: icp::QueryEncodedBlocksFn::new(
                    Principal::anonymous(),
                    "get_encoded_blocks".into(),
                ),
                start: 0,
                length: 42,
            }],
        });
        roundtrip(icp::QueryEncodedBlocksResult::Ok(vec![vec![0, 255]]));
        roundtrip(icp::QueryEncodedBlocksResult::Err(
            icp::QueryArchiveError::BadFirstBlockIndex {
                requested_index: 0,
                first_valid_index: 42,
            },
        ));
        roundtrip(icp::QueryEncodedBlocksResult::Err(
            icp::QueryArchiveError::Other {
                error_code: u64::MAX,
                error_message: "archive".into(),
            },
        ));
    }
    #[test]
    fn cycles_creation_settings_duplicate_refund_and_all_withdraw_rejections() {
        let settings = cycles::CanisterSettings {
            controllers: Some(vec![Principal::anonymous()]),
            compute_allocation: Some(1u64.into()),
            memory_allocation: Some(2u64.into()),
            freezing_threshold: Some(3u64.into()),
            reserved_cycles_limit: Some(4u64.into()),
            wasm_memory_limit: Some(5u64.into()),
            wasm_memory_threshold: Some(6u64.into()),
            log_visibility: Some(cycles::LogVisibility::AllowedViewers(vec![
                Principal::anonymous(),
            ])),
            environment_variables: Some(vec![cycles::EnvironmentVariable {
                name: "key".into(),
                value: "value".into(),
            }]),
        };
        roundtrip(cycles::CreateCanisterArgs {
            from_subaccount: Some(vec![0; 32]),
            created_at_time: Some(99),
            amount: 100u64.into(),
            creation_args: Some(cycles::CmcCreateCanisterArgs {
                settings: Some(settings),
                subnet_selection: Some(cycles::SubnetSelection::Subnet {
                    subnet: Principal::anonymous(),
                }),
            }),
        });
        roundtrip(cycles::SubnetSelection::Filter(cycles::SubnetFilter {
            subnet_type: Some("application".into()),
        }));
        roundtrip(cycles::CreateCanisterResult::Ok(
            cycles::CreateCanisterSuccess {
                block_id: 42u64.into(),
                canister_id: Principal::anonymous(),
            },
        ));
        for error in [
            cycles::CreateCanisterError::InsufficientFunds {
                balance: 1u64.into(),
            },
            cycles::CreateCanisterError::TooOld,
            cycles::CreateCanisterError::CreatedInFuture { ledger_time: 99 },
            cycles::CreateCanisterError::TemporarilyUnavailable,
            cycles::CreateCanisterError::Duplicate {
                duplicate_of: 42u64.into(),
                canister_id: Some(Principal::anonymous()),
            },
            cycles::CreateCanisterError::FailedToCreate {
                fee_block: Some(42u64.into()),
                refund_block: Some(43u64.into()),
                error: "refunded".into(),
            },
            cycles::CreateCanisterError::GenericError {
                message: "error".into(),
                error_code: 1u64.into(),
            },
        ] {
            roundtrip(cycles::CreateCanisterResult::Err(error));
        }
        roundtrip(cycles::WithdrawArgs {
            amount: 42u64.into(),
            from_subaccount: None,
            to: Principal::anonymous(),
            created_at_time: Some(99),
        });
        for code in [
            cycles::RejectionCode::NoError,
            cycles::RejectionCode::CanisterError,
            cycles::RejectionCode::SysTransient,
            cycles::RejectionCode::DestinationInvalid,
            cycles::RejectionCode::Unknown,
            cycles::RejectionCode::SysFatal,
            cycles::RejectionCode::CanisterReject,
        ] {
            roundtrip(cycles::WithdrawResult::Err(
                cycles::WithdrawError::FailedToWithdraw {
                    fee_block: Some(42u64.into()),
                    rejection_code: code,
                    rejection_reason: "refunded".into(),
                },
            ));
        }
        for error in [
            cycles::WithdrawError::GenericError {
                message: "error".into(),
                error_code: 1u64.into(),
            },
            cycles::WithdrawError::TemporarilyUnavailable,
            cycles::WithdrawError::Duplicate {
                duplicate_of: 42u64.into(),
            },
            cycles::WithdrawError::BadFee {
                expected_fee: 1u64.into(),
            },
            cycles::WithdrawError::InvalidReceiver {
                receiver: Principal::anonymous(),
            },
            cycles::WithdrawError::CreatedInFuture { ledger_time: 99 },
            cycles::WithdrawError::TooOld,
            cycles::WithdrawError::InsufficientFunds {
                balance: 0u64.into(),
            },
        ] {
            roundtrip(cycles::WithdrawResult::Err(error));
        }
    }
    #[test]
    fn registry_explicit_target_and_all_response_states() {
        let target = Principal::from_slice(&[1]);
        roundtrip(registry::GetSubnetForCanisterRequest {
            principal: Some(target),
        });
        for subnet_id in [Some(Principal::from_slice(&[2])), None] {
            roundtrip(registry::GetSubnetForCanisterResponse::Ok(
                registry::GetSubnetForCanisterResponseOk { subnet_id },
            ));
        }
        roundtrip(registry::GetSubnetForCanisterResponse::Err(
            "not found".into(),
        ));
    }
}
