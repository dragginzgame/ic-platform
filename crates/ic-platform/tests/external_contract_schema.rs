//! Independent equality against pinned external DIDs, not only self-roundtrips.
#![cfg(not(target_family = "wasm"))]
#[cfg(test)]
mod tests {
    use ic_platform::{
        candid::contract::{self, CandidType},
        candid_parser as parser, certified_assets_types as assets, cmc_types as cmc,
        cycles_ledger_types as cycles, icp_ledger_types as icp, nns_registry_types as registry,
    };
    fn env(source: &str) -> parser::TypeEnv {
        let program: parser::IDLProg = source.parse().unwrap();
        let mut env = parser::TypeEnv::new();
        parser::check_prog(&mut env, &program).unwrap();
        env
    }
    fn equal<T: CandidType>(env: &parser::TypeEnv, name: &str) {
        let mut env = env.clone();
        let declared = env.find_type(name).unwrap().clone();
        let mut rust = parser::TypeContainer::new();
        let actual = rust.add::<T>();
        let actual = env.merge_type(rust.env, actual);
        parser::type_equal(
            &mut std::collections::HashSet::new(),
            &env,
            &declared,
            &actual,
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    #[test]
    fn cmc_matches_pinned_did_including_certification_and_notify_variants() {
        let env = env(include_str!("fixtures/external-cmc.did"));
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
        let env = env(include_str!("fixtures/external-icp.did"));
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
        let env = env(include_str!("fixtures/external-cycles.did"));
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
        #[candid_path("ic_platform::candid::contract")]
        struct LegacyRequired {
            principal: contract::Principal,
        }
        let principal = contract::Principal::from_slice(&[1]);
        let canonical = registry::GetSubnetForCanisterRequest {
            principal: Some(principal),
        };
        let canonical_bytes = contract::encode_one(&canonical).unwrap();
        let old_bytes = contract::encode_one(LegacyRequired { principal }).unwrap();
        assert_ne!(canonical_bytes, old_bytes);
        // Upstream subtyping admits the old required field; that is not equal wire identity.
        assert_eq!(
            ic_platform::candid::decoder::decode_one::<registry::GetSubnetForCanisterRequest>(
                &old_bytes
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
        let source = include_str!("fixtures/external-assets.did");
        let (_, (mut env, service)) =
            parser::instantiate_candid(parser::CandidSource::Text(source)).unwrap();
        fn method_type<T: CandidType>(
            env: &mut parser::TypeEnv,
            service: &parser::Type,
            method: &str,
            input: bool,
        ) {
            let function = env.get_method(service, method).unwrap();
            assert_eq!(function.modes, [parser::FuncMode::Query]);
            let schema = if input {
                function.args[0].clone()
            } else {
                function.rets[0].clone()
            };
            let mut rust = parser::TypeContainer::new();
            let actual = rust.add::<T>();
            let actual = env.merge_type(rust.env, actual);
            parser::type_equal(&mut std::collections::HashSet::new(), env, &schema, &actual)
                .unwrap();
        }
        method_type::<assets::GetArgs>(&mut env, &service, "get", true);
        method_type::<assets::GetResult>(&mut env, &service, "get", false);
        method_type::<assets::GetChunkArgs>(&mut env, &service, "get_chunk", true);
        method_type::<assets::GetChunkResult>(&mut env, &service, "get_chunk", false);
    }
}
