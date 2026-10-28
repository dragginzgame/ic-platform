//! Real consumer of transport-independent external protocol operations.
use ic_platform::{
    candid::{
        DecodeError, DecodeLimits,
        protocols::{self, ReplyArity},
    },
    cmc_types,
    management_canister_types::CanisterInfoArgs,
    principal::ExternalPrincipal,
};

/// Encode the concrete consumer history request independently of submission.
pub fn prepare_history() -> Vec<u8> {
    protocols::management::canister_info::encode(&CanisterInfoArgs {
        canister_id: ExternalPrincipal::anonymous(),
        num_requested_changes: Some(20),
    })
    .unwrap()
}
/// Keep a typed CMC result distinct from wire admission errors.
pub fn decode_notification(
    bytes: &[u8],
    limits: DecodeLimits,
) -> Result<cmc_types::NotifyTopUpResult, DecodeError> {
    protocols::cmc::notify_top_up::decode(bytes, limits, ReplyArity::Exact)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_platform::{candid::contract, management_canister_types::CanisterInfoResult};
    fn limits() -> DecodeLimits {
        DecodeLimits {
            reply_bytes: 65536,
            decoding_quota: 1_000_000,
            skipping_quota: 10000,
            type_entries: 128,
            header_bytes: 8192,
        }
    }
    fn hex(s: &str) -> Vec<u8> {
        s.split_whitespace()
            .collect::<String>()
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }

    #[test]
    fn management_history_uses_independent_upstream_wire_fixtures() {
        assert_eq!(
            prepare_history(),
            hex(include_str!("fixtures/management-info-args.hex"))
        );
        assert_eq!(
            protocols::management::canister_info::METHOD,
            "canister_info"
        );
        let response = hex(include_str!("fixtures/management-info-result.hex"));
        let value: CanisterInfoResult =
            protocols::management::canister_info::decode(&response, limits(), ReplyArity::Exact)
                .unwrap();
        assert_eq!(value.total_num_changes, 9);
        assert_eq!(value.recent_changes.len(), 9);
    }
    #[test]
    fn zero_argument_and_single_blob_forms_are_not_implicit_transport_bytes() {
        assert_eq!(
            protocols::management::raw_rand::encode().unwrap(),
            b"DIDL\0\0"
        );
        assert_eq!(
            protocols::management::raw_rand::decode(
                &hex("4449444c016d7b010003010203"),
                limits(),
                ReplyArity::Exact
            )
            .unwrap(),
            [1, 2, 3]
        );
        assert!(protocols::management::raw_rand::decode(b"", limits(), ReplyArity::Exact).is_err());
        assert_eq!(
            protocols::icrc::fee::decode(&hex("4449444c00017d07"), limits(), ReplyArity::Exact)
                .unwrap(),
            contract::Nat::from(7u64)
        );
        assert_eq!(
            protocols::cmc::get_icp_xdr_conversion_rate::encode().unwrap(),
            b"DIDL\0\0"
        );
        assert_eq!(protocols::icrc::fee::encode().unwrap(), b"DIDL\0\0");
        assert_eq!(
            protocols::icrc::get_tip_certificate::encode().unwrap(),
            b"DIDL\0\0"
        );
    }
    #[test]
    fn caller_arity_keeps_zero_results_distinct_from_one_null_and_missing_optional() {
        use protocols::{icrc::get_tip_certificate, management::update_settings};
        assert!(update_settings::decode(b"DIDL\0\0", limits(), ReplyArity::Exact).is_ok());
        assert!(
            update_settings::decode(&hex("4449444c00017f"), limits(), ReplyArity::Exact).is_err()
        );
        assert!(
            update_settings::decode(&hex("4449444c00017f"), limits(), ReplyArity::Compatible)
                .is_ok()
        );
        assert!(get_tip_certificate::decode(b"DIDL\0\0", limits(), ReplyArity::Exact).is_err());
        assert!(
            get_tip_certificate::decode(b"DIDL\0\0", limits(), ReplyArity::Compatible)
                .unwrap()
                .is_none()
        );
        let extra = contract::encode_args((contract::Nat::from(7u64), 42u64)).unwrap();
        assert!(protocols::icrc::fee::decode(&extra, limits(), ReplyArity::Exact).is_err());
        assert_eq!(
            protocols::icrc::fee::decode(&extra, limits(), ReplyArity::Compatible).unwrap(),
            contract::Nat::from(7u64)
        );
    }
    #[test]
    fn typed_provider_errors_stay_separate_from_codec_errors_and_admission() {
        let response = contract::encode_one(cmc_types::NotifyTopUpResult::Err(
            cmc_types::NotifyError::Processing,
        ))
        .unwrap();
        assert!(matches!(
            decode_notification(&response, limits()).unwrap(),
            Err(cmc_types::NotifyError::Processing)
        ));
        assert!(matches!(
            decode_notification(
                &response,
                DecodeLimits {
                    reply_bytes: 1,
                    ..limits()
                }
            ),
            Err(DecodeError::ReplyTooLarge { .. })
        ));
        assert!(decode_notification(&response[..response.len() - 1], limits()).is_err());
    }
}
