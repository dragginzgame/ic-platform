//! Explicit external Candid endpoints, without direct Candid/CDK dependencies.
use ic_platform::candid::{
    contract,
    endpoint::{self, ArgumentLimits},
};

const LIMITS: ArgumentLimits = ArgumentLimits {
    max_bytes: 1024,
    decoding_quota: 100000,
    skipping_quota: 1000,
    max_type_len: 32,
    max_header_len: 512,
};
const NESTING: ic_platform::candid::nesting::NestingLimits =
    ic_platform::candid::nesting::NestingLimits {
        depth: 64,
        work: 100_000,
    };

#[derive(contract::CandidType, contract::serde::Deserialize)]
#[candid_path("ic_platform::candid::contract")]
#[serde(crate = "ic_platform::candid::contract::serde")]
struct Standard {
    name: String,
    url: String,
}

fn permit() -> Result<(), String> {
    Ok(())
}
#[endpoint::query(
    name = "icrc10_supported_standards",
    limits = "LIMITS",
    nesting = "NESTING",
    empty_args = "reject",
    guard = "permit"
)]
fn standards() -> Vec<Standard> {
    vec![Standard {
        name: "ICRC-10".into(),
        url: "https://github.com/dfinity/ICRC-1/tree/main/standards/ICRC-10".into(),
    }]
}
#[endpoint::update(limits = "LIMITS", empty_args = "unit")]
fn tuple(value: u64, flag: bool) -> (u64, bool) {
    (value, flag)
}
#[endpoint::update(limits = "LIMITS", empty_args = "reject")]
fn zero() {}
#[endpoint::query(limits = "LIMITS", empty_args = "reject", composite)]
async fn composite(value: u64) -> u64 {
    value + 1
}

mod boundary {
    pub use ic_platform as platform;
}
#[endpoint::query(
    limits = "LIMITS",
    empty_args = "reject",
    platform = "crate::boundary::platform"
)]
fn renamed() -> u64 {
    3
}

type Null = ();
#[endpoint::query(limits = "LIMITS", empty_args = "reject")]
fn one_null() -> Null {}

contract::export_service!(ic_platform::candid::contract);

/// Exercise native external-service registration independently of IC execution.
pub fn native_contract() -> String {
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_platform::candid::endpoint::EmptyArgs;
    #[test]
    fn external_service_metadata_preserves_zero_plain_tuple_and_composite_modes() {
        let service = native_contract();
        assert!(
            service.contains("one_null : () -> (null) query"),
            "{service}"
        );
        assert!(
            service.contains("icrc10_supported_standards : () -> (vec Standard) query"),
            "{service}"
        );
        assert!(
            service.contains("tuple : (nat64, bool) -> (nat64, bool)"),
            "{service}"
        );
        assert!(service.contains("zero : () -> ()"), "{service}");
        assert!(
            service.contains("composite : (nat64) -> (nat64) composite_query"),
            "{service}"
        );
    }
    #[test]
    fn empty_policy_is_explicit_and_argument_limits_are_caller_owned() {
        assert!(LIMITS.decode::<()>(b"", EmptyArgs::Reject).is_err());
        assert!(LIMITS.decode::<()>(b"", EmptyArgs::Unit).is_ok());
        assert!(LIMITS.decode::<()>(b"DIDL\0\0", EmptyArgs::Reject).is_ok());
        let bytes = contract::encode_args((7u64, true)).unwrap();
        assert_eq!(
            LIMITS
                .decode::<(u64, bool)>(&bytes, EmptyArgs::Reject)
                .unwrap(),
            (7, true)
        );
        assert!(matches!(
            ArgumentLimits {
                max_bytes: 1,
                ..LIMITS
            }
            .decode::<(u64, bool)>(&bytes, EmptyArgs::Reject),
            Err(endpoint::ArgumentDecodeError::TooLarge { .. })
        ));
        assert!(
            ArgumentLimits {
                decoding_quota: 0,
                ..LIMITS
            }
            .decode::<(u64, bool)>(&bytes, EmptyArgs::Reject)
            .is_err()
        );
    }
}
