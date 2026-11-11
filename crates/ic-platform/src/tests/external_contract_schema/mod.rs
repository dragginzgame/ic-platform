//! Independent equality against selected protocol schemas, not only self-roundtrips.
#![cfg(not(target_family = "wasm"))]
#[cfg(test)]
mod tests {
    use crate::candid_codec::Encode;
    use crate::tests::reference::{assert_named_schema as equal, frozen_schema};
    use crate::{
        certified_assets_types as assets, cmc_types as cmc, cycles_ledger_types as cycles,
        icp_ledger_types as icp, nns_registry_types as registry,
    };
    use candid as contract;
    use candid::types::TypeEnv;
    fn env(provider: &str) -> TypeEnv {
        let source = match provider {
            "cmc" => include_str!("../../../tests/fixtures/owned-cmc-schema.json"),
            "icp" => include_str!("../../../tests/fixtures/owned-icp-schema.json"),
            "cycles" => include_str!("../../../tests/fixtures/owned-cycles-schema.json"),
            "assets" => include_str!("../../../tests/fixtures/owned-assets-schema.json"),
            "registry" => include_str!("../../../tests/fixtures/registry-schema.json"),
            "icrc10" => include_str!("../../../tests/fixtures/icrc10-schema.json"),
            _ => panic!("unknown frozen provider"),
        };
        frozen_schema(source)
    }
    fn provider(name: &str) -> (TypeEnv, candid::types::Type) {
        let env = env(name);
        let service = env.find_type("$service").unwrap().clone();
        (env, service)
    }
    fn value(name: &str) -> Vec<u8> {
        let values: std::collections::BTreeMap<String, Vec<u8>> =
            serde_json::from_str(include_str!("../../../tests/fixtures/protocol-values.json"))
                .unwrap();
        values[name].clone()
    }
    #[test]
    fn icrc10_matches_external_named_records_and_independent_reply_bytes() {
        use crate::{
            candid_codec::decoder::{Limits, decode_one},
            icrc_ledger_types::icrc10::SupportedStandard,
        };
        let (env, service) = provider("icrc10");
        equal::<SupportedStandard>(&env, "SupportedStandard");
        let method = env
            .get_method(&service, "icrc10_supported_standards")
            .unwrap();
        assert!(method.args.is_empty());
        assert_eq!(method.rets.len(), 1);
        let bytes = value("icrc10");
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
    fn cmc_matches_selected_schema_including_certification_and_notify_variants() {
        let env = env("cmc");
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
        let env = env("icp");
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
        let env = env("cycles");
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
        let env = env("registry");
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
        assert_eq!(canonical_bytes, value("registry"));
    }
    #[test]
    fn assets_match_complete_method_inputs_and_outputs() {
        let (mut env, service) = provider("assets");
        fn method_type<T: Encode>(
            env: &mut TypeEnv,
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
    fn operation_type<T: Encode>(env: &mut TypeEnv, declared: candid::types::Type) {
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
        let (mut env, service) = provider(source);
        let function = env.get_method(&service, method).unwrap().clone();
        assert_eq!(function.args.len(), 1, "{method} request arity");
        assert_eq!(function.rets.len(), 1, "{method} reply arity");
        operation_type::<Req>(&mut env, function.args[0].clone());
        operation_type::<Rep>(&mut env, function.rets[0].clone());
    }
    #[test]
    fn protocol_operations_match_independent_provider_methods_and_arity() {
        use crate::icrc_ledger_types as icrc;
        let cmc_provider = "cmc";
        operation::<cmc::NotifyTopUpArg, cmc::NotifyTopUpResult>(cmc_provider, "notify_top_up");
        operation::<cmc::NotifyMintCyclesArg, cmc::NotifyMintCyclesResult>(
            cmc_provider,
            "notify_mint_cycles",
        );
        let icp_provider = "icp";
        operation::<icp::TransferArgs, icp::TransferResult>(icp_provider, "transfer");
        operation::<icp::GetBlocksArgs, icp::QueryEncodedBlocksResponse>(
            icp_provider,
            "query_encoded_blocks",
        );
        let cycles_provider = "cycles";
        operation::<cycles::CreateCanisterArgs, cycles::CreateCanisterResult>(
            cycles_provider,
            "create_canister",
        );
        operation::<cycles::WithdrawArgs, cycles::WithdrawResult>(cycles_provider, "withdraw");
        operation::<icrc::Account, crate::candid_codec::Nat>(cycles_provider, "icrc1_balance_of");
        operation::<icrc::TransferArg, Result<crate::candid_codec::Nat, icrc::TransferError>>(
            cycles_provider,
            "icrc1_transfer",
        );
        operation::<Vec<icrc::GetBlocksRequest>, cycles::GetBlocksResult>(
            cycles_provider,
            "icrc3_get_blocks",
        );
        let assets_provider = "assets";
        operation::<assets::GetArgs, assets::GetResult>(assets_provider, "get");
        operation::<assets::GetChunkArgs, assets::GetChunkResult>(assets_provider, "get_chunk");
        for (source, method) in [
            (cmc_provider, "get_icp_xdr_conversion_rate"),
            (cycles_provider, "icrc1_fee"),
            (cycles_provider, "icrc3_get_tip_certificate"),
        ] {
            let (env, service) = provider(source);
            let function = env.get_method(&service, method).unwrap();
            assert!(function.args.is_empty());
            assert_eq!(function.rets.len(), 1);
        }
    }
    #[test]
    fn cycles_provider_codec_preserves_nat64_instead_of_narrowing_standard_value() {
        use crate::candid_codec::decoder::{Limits, decode_one};
        let bytes = value("cycles_nat64");
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
