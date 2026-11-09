//! Independent equality against pinned external DIDs, not only self-roundtrips.
#![cfg(not(target_family = "wasm"))]
#[cfg(test)]
mod tests {
    use crate::candid_codec::Encode;
    use crate::{
        certified_assets_types as assets, cmc_types as cmc, cycles_ledger_types as cycles,
        icp_ledger_types as icp, nns_registry_types as registry,
    };
    use candid as contract;
    use candid_parser as parser;
    fn env(source: &str) -> parser::TypeEnv {
        let program: parser::IDLProg = source.parse().unwrap();
        let mut env = parser::TypeEnv::new();
        parser::check_prog(&mut env, &program).unwrap();
        env
    }
    fn equal<T: Encode>(env: &parser::TypeEnv, name: &str) {
        let mut env = env.clone();
        let declared = env.find_type(name).unwrap().clone();
        let (owned_env, actual) = crate::tests::reference::schema::<T>();
        let actual = env.merge_type(owned_env, actual);
        candid::types::subtype::equal(
            &mut std::collections::HashSet::new(),
            &env,
            &declared,
            &actual,
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    #[test]
    fn selected_icrc_types_match_frozen_registry_schemas() {
        use crate::icrc_ledger_types::{self as ledger, icrc21};
        // Generated independently by icrc-ledger-types =0.2.0 and Candid =0.10.39.
        // This includes recursive values, defaulted fields and query callbacks.
        let env = env(include_str!("../../../tests/fixtures/icrc-reference.did"));
        macro_rules! check {
            ($($name:ident),+ $(,)?) => { $(equal::<ledger::$name>(&env, concat!("Expected", stringify!($name)));)+ };
        }
        check!(
            Account,
            MetadataValue,
            Value,
            ICRC3Value,
            TransferArg,
            Memo,
            TransferError,
            AllowanceArgs,
            Allowance,
            GetArchivesArgs,
            ICRC3ArchiveInfo,
            ArchivedBlocks,
            BlockWithId,
            GetBlocksRequest,
            GetBlocksResult,
            ICRC3DataCertificate,
            SupportedBlockType
        );
        equal::<ledger::QueryArchiveFn<Vec<ledger::GetBlocksRequest>, ledger::GetBlocksResult>>(
            &env,
            "ExpectedQueryArchiveFn",
        );
        macro_rules! consent {
            ($($name:ident),+ $(,)?) => { $(equal::<icrc21::$name>(&env, concat!("Expected", stringify!($name)));)+ };
        }
        consent!(
            ConsentMessageMetadata,
            ConsentMessageRequest,
            ConsentMessageSpec,
            DisplayMessageType,
            ConsentInfo,
            ConsentMessage,
            FieldsDisplay,
            ErrorInfo,
            Icrc21Error
        );
        equal::<icrc21::Value>(&env, "ExpectedConsentValue");
    }
    #[test]
    fn icrc10_matches_external_named_records_and_independent_reply_bytes() {
        use crate::{
            candid_codec::decoder::{Limits, decode_one},
            icrc_ledger_types::icrc10::SupportedStandard,
        };
        // Independently express protocol facts from the pinned ICRC-10 DID.
        let source = "type SupportedStandard = record { name : text; url : text }; service : { icrc10_supported_standards : () -> (vec SupportedStandard) query }";
        let (_, (env, service)) =
            parser::utils::instantiate_candid(parser::utils::CandidSource::Text(source)).unwrap();
        equal::<SupportedStandard>(&env, "SupportedStandard");
        let method = env
            .get_method(&service, "icrc10_supported_standards")
            .unwrap();
        assert!(method.args.is_empty());
        assert_eq!(method.rets.len(), 1);
        let values = parser::parse_idl_args(
            "(vec { record { name = \"ICRC-10\"; url = \"https://example.test/standard\" } })",
        )
        .unwrap();
        let bytes = values.to_bytes_with_types(&env, &method.rets).unwrap();
        let decoded: Vec<SupportedStandard> = decode_one(
            &bytes,
            Limits {
                bytes: 4096,
                work: 100_000,
                allocation: 65536,
                types: 32,
                header: 1024,
            },
        )
        .unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].name, "ICRC-10");
        assert_eq!(decoded[0].url, "https://example.test/standard");
        assert_eq!(crate::tests::reference::oracle_bytes(&decoded), bytes);
    }
    #[test]
    fn cmc_matches_pinned_did_including_certification_and_notify_variants() {
        let env = env(include_str!("../../../tests/fixtures/external-cmc.did"));
        equal::<cmc::NotifyTopUpArg>(&env, "NotifyTopUpArg");
        equal::<cmc::NotifyTopUpResult>(&env, "NotifyTopUpResult");
        equal::<cmc::NotifyError>(&env, "NotifyError");
        equal::<cmc::NotifyMintCyclesArg>(&env, "NotifyMintCyclesArg");
        equal::<cmc::NotifyMintCyclesSuccess>(&env, "NotifyMintCyclesSuccess");
        equal::<cmc::NotifyMintCyclesResult>(&env, "NotifyMintCyclesResult");
        equal::<cmc::IcpXdrConversionRate>(&env, "IcpXdrConversionRate");
        equal::<cmc::IcpXdrConversionRateResponse>(&env, "IcpXdrConversionRateResponse");
    }
    #[test]
    fn icp_matches_legacy_transfer_and_recursive_archive_function_schema() {
        let env = env(include_str!("../../../tests/fixtures/external-icp.did"));
        equal::<icp::Tokens>(&env, "Tokens");
        equal::<icp::TimeStamp>(&env, "TimeStamp");
        equal::<icp::TransferArgs>(&env, "TransferArgs");
        equal::<icp::TransferError>(&env, "TransferError");
        equal::<icp::TransferResult>(&env, "TransferResult");
        equal::<icp::GetBlocksArgs>(&env, "GetBlocksArgs");
        equal::<icp::QueryArchiveError>(&env, "QueryArchiveError");
        equal::<icp::ArchivedEncodedBlocksRange>(&env, "ArchivedEncodedBlocksRange");
        equal::<icp::QueryEncodedBlocksResponse>(&env, "QueryEncodedBlocksResponse");
    }
    #[test]
    fn cycles_matches_its_own_settings_and_complete_creation_withdraw_errors() {
        let env = env(include_str!("../../../tests/fixtures/external-cycles.did"));
        equal::<cycles::CreateCanisterArgs>(&env, "CreateCanisterArgs");
        equal::<cycles::CmcCreateCanisterArgs>(&env, "CmcCreateCanisterArgs");
        equal::<cycles::CanisterSettings>(&env, "CanisterSettings");
        equal::<cycles::EnvironmentVariable>(&env, "EnvironmentVariable");
        equal::<cycles::LogVisibility>(&env, "LogVisibility");
        equal::<cycles::SubnetFilter>(&env, "SubnetFilter");
        equal::<cycles::SubnetSelection>(&env, "SubnetSelection");
        equal::<cycles::CreateCanisterSuccess>(&env, "CreateCanisterSuccess");
        equal::<cycles::CreateCanisterError>(&env, "CreateCanisterError");
        equal::<cycles::WithdrawArgs>(&env, "WithdrawArgs");
        equal::<cycles::WithdrawError>(&env, "WithdrawError");
        equal::<cycles::RejectionCode>(&env, "RejectionCode");
    }
    #[test]
    fn registry_optional_target_is_canonical_and_not_byte_identical_to_old_projection() {
        // Independently expressed protocol facts from Registry's pinned DID.
        let env = env(
            "type Request = record { \"principal\" : opt principal }; type Response = variant { Ok : record { subnet_id : opt principal }; Err : text };",
        );
        equal::<registry::GetSubnetForCanisterRequest>(&env, "Request");
        equal::<registry::GetSubnetForCanisterResponse>(&env, "Response");
        #[derive(contract::CandidType)]
        #[candid_path("candid")]
        struct LegacyRequired {
            principal: contract::Principal,
        }
        let principal = contract::Principal::from_slice(&[1]);
        let canonical = registry::GetSubnetForCanisterRequest {
            principal: Some(crate::Principal::from_slice(principal.as_slice())),
        };
        let canonical_bytes = crate::tests::reference::oracle_bytes(&canonical);
        let old_bytes = contract::encode_one(LegacyRequired { principal }).unwrap();
        assert_ne!(canonical_bytes, old_bytes);
        // Upstream subtyping admits the old required field; that is not equal wire identity.
        assert_eq!(
            crate::candid_codec::decoder::decode_one::<registry::GetSubnetForCanisterRequest>(
                &old_bytes,
                crate::tests::reference::limits()
            )
            .unwrap(),
            canonical
        );
        let args =
            parser::parse_idl_args("(record { \"principal\" = opt principal \"uuc56-gyb\" })")
                .unwrap();
        let expected = args
            .to_bytes_with_types(&env, &[env.find_type("Request").unwrap().clone()])
            .unwrap();
        assert_eq!(canonical_bytes, expected);
    }
    #[test]
    fn assets_match_complete_method_inputs_and_outputs() {
        let source = include_str!("../../../tests/fixtures/external-assets.did");
        let (_, (mut env, service)) =
            parser::utils::instantiate_candid(parser::utils::CandidSource::Text(source)).unwrap();
        fn method_type<T: Encode>(
            env: &mut parser::TypeEnv,
            service: &candid::types::Type,
            method: &str,
            input: bool,
        ) {
            let function = env.get_method(service, method).unwrap();
            assert_eq!(function.modes, [candid::types::FuncMode::Query]);
            let schema = if input {
                function.args[0].clone()
            } else {
                function.rets[0].clone()
            };
            let (owned_env, actual) = crate::tests::reference::schema::<T>();
            let actual = env.merge_type(owned_env, actual);
            candid::types::subtype::equal(
                &mut std::collections::HashSet::new(),
                env,
                &schema,
                &actual,
            )
            .unwrap();
        }
        method_type::<assets::GetArgs>(&mut env, &service, "get", true);
        method_type::<assets::GetResult>(&mut env, &service, "get", false);
        method_type::<assets::GetChunkArgs>(&mut env, &service, "get_chunk", true);
        method_type::<assets::GetChunkResult>(&mut env, &service, "get_chunk", false);
    }
    fn operation_type<T: Encode>(env: &mut parser::TypeEnv, declared: candid::types::Type) {
        let (owned_env, actual) = crate::tests::reference::schema::<T>();
        let actual = env.merge_type(owned_env, actual);
        candid::types::subtype::equal(
            &mut std::collections::HashSet::new(),
            env,
            &declared,
            &actual,
        )
        .unwrap();
    }
    fn operation<Req: Encode, Rep: Encode>(source: &str, method: &str) {
        let (_, (mut env, service)) =
            parser::utils::instantiate_candid(parser::utils::CandidSource::Text(source)).unwrap();
        let function = env.get_method(&service, method).unwrap().clone();
        assert_eq!(function.args.len(), 1, "{method} request arity");
        assert_eq!(function.rets.len(), 1, "{method} reply arity");
        operation_type::<Req>(&mut env, function.args[0].clone());
        operation_type::<Rep>(&mut env, function.rets[0].clone());
    }
    #[test]
    fn protocol_operations_match_independent_provider_methods_and_arity() {
        use crate::icrc_ledger_types as icrc;
        let cmc_did = include_str!("../../../tests/fixtures/external-cmc.did");
        operation::<cmc::NotifyTopUpArg, cmc::NotifyTopUpResult>(cmc_did, "notify_top_up");
        operation::<cmc::NotifyMintCyclesArg, cmc::NotifyMintCyclesResult>(
            cmc_did,
            "notify_mint_cycles",
        );
        let icp_did = include_str!("../../../tests/fixtures/external-icp.did");
        operation::<icp::TransferArgs, icp::TransferResult>(icp_did, "transfer");
        operation::<icp::GetBlocksArgs, icp::QueryEncodedBlocksResponse>(
            icp_did,
            "query_encoded_blocks",
        );
        let cycles_did = include_str!("../../../tests/fixtures/external-cycles.did");
        operation::<cycles::CreateCanisterArgs, cycles::CreateCanisterResult>(
            cycles_did,
            "create_canister",
        );
        operation::<cycles::WithdrawArgs, cycles::WithdrawResult>(cycles_did, "withdraw");
        operation::<icrc::Account, crate::candid_codec::Nat>(cycles_did, "icrc1_balance_of");
        operation::<icrc::TransferArg, Result<crate::candid_codec::Nat, icrc::TransferError>>(
            cycles_did,
            "icrc1_transfer",
        );
        operation::<Vec<icrc::GetBlocksRequest>, cycles::GetBlocksResult>(
            cycles_did,
            "icrc3_get_blocks",
        );
        let assets_did = include_str!("../../../tests/fixtures/external-assets.did");
        operation::<assets::GetArgs, assets::GetResult>(assets_did, "get");
        operation::<assets::GetChunkArgs, assets::GetChunkResult>(assets_did, "get_chunk");
        for (source, method) in [
            (cmc_did, "get_icp_xdr_conversion_rate"),
            (cycles_did, "icrc1_fee"),
            (cycles_did, "icrc3_get_tip_certificate"),
        ] {
            let (_, (env, service)) =
                parser::utils::instantiate_candid(parser::utils::CandidSource::Text(source))
                    .unwrap();
            let function = env.get_method(&service, method).unwrap();
            assert!(function.args.is_empty());
            assert_eq!(function.rets.len(), 1);
        }
    }
    #[test]
    fn cycles_provider_codec_preserves_nat64_instead_of_narrowing_standard_value() {
        use crate::candid_codec::decoder::{Limits, decode_one};
        let (_, (env, service)) =
            parser::utils::instantiate_candid(parser::utils::CandidSource::Text(include_str!(
                "../../../tests/fixtures/external-cycles.did"
            )))
            .unwrap();
        let response = env.get_method(&service, "icrc3_get_blocks").unwrap().rets[0].clone();
        let value=parser::parse_idl_args("(record { log_length = 1 : nat; blocks = vec { record { id = 0 : nat; block = variant { Nat64 = 7 : nat64 } } }; archived_blocks = vec {} })").unwrap();
        let bytes = value.to_bytes_with_types(&env, &[response]).unwrap();
        let limits = Limits {
            bytes: 65536,
            work: 1000000,
            allocation: 1000000,
            types: 256,
            header: 8192,
        };
        let decoded = decode_one::<cycles::GetBlocksResult>(&bytes, limits).unwrap();
        assert!(matches!(
            decoded.blocks[0].block,
            crate::icrc_ledger_types::Value::Nat64(7)
        ));
        assert!(decode_one::<crate::icrc_ledger_types::GetBlocksResult>(&bytes, limits).is_err());
    }
}
