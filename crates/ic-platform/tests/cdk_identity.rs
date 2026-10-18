//! Compile original-owner identity in both directions; never execute host IC calls.
#[test]
fn upstream_error_identities() {
    macro_rules! identity {
        ($platform:ty, $upstream:ty) => {{
            let _: fn($platform) -> $upstream = |value| value;
            let _: fn($upstream) -> $platform = |value| value;
        }};
    }
    use ic_platform::cdk::{api, call, stable};
    identity!(call::Error, ic_cdk::call::Error);
    identity!(call::CandidDecodeFailed, ic_cdk::call::CandidDecodeFailed);
    identity!(call::CallPerformFailed, ic_cdk::call::CallPerformFailed);
    identity!(call::CallRejected, ic_cdk::call::CallRejected);
    identity!(call::RejectCode, ic_cdk::call::RejectCode);
    identity!(
        call::InsufficientLiquidCycleBalance,
        ic_cdk::call::InsufficientLiquidCycleBalance
    );
    identity!(api::SignCostError, ic_cdk::api::SignCostError);
    identity!(
        api::PerformanceCounterType,
        ic_cdk::api::PerformanceCounterType
    );
    identity!(stable::StableMemoryError, ic_cdk::stable::StableMemoryError);
}

#[test]
fn upstream_function_identities() {
    use ic_platform::cdk::{api, stable, storage};
    macro_rules! same {
        ($platform:path, $upstream:path, $signature:ty) => {
            assert!(std::ptr::fn_addr_eq(
                $platform as $signature,
                $upstream as $signature
            ));
        };
    }
    same!(api::cycles_burn, ic_cdk::api::cycles_burn, fn(u128) -> u128);
    same!(
        api::certified_data_set::<Vec<u8>>,
        ic_cdk::api::certified_data_set::<Vec<u8>>,
        fn(Vec<u8>)
    );
    same!(
        api::data_certificate,
        ic_cdk::api::data_certificate,
        fn() -> Option<Vec<u8>>
    );
    same!(
        api::cost_sign_with_ecdsa::<String>,
        ic_cdk::api::cost_sign_with_ecdsa::<String>,
        fn(String, u32) -> Result<u128, api::SignCostError>
    );
    same!(
        stable::stable_grow,
        ic_cdk::stable::stable_grow,
        fn(u64) -> Result<u64, stable::StableMemoryError>
    );
    same!(
        stable::stable_read,
        ic_cdk::stable::stable_read,
        fn(u64, &mut [u8])
    );
    same!(
        stable::stable_write,
        ic_cdk::stable::stable_write,
        fn(u64, &[u8])
    );
    same!(
        storage::stable_save::<(bool, bool, u64, u64)>,
        ic_cdk::storage::stable_save::<(bool, bool, u64, u64)>,
        fn((bool, bool, u64, u64)) -> Result<(), ic_platform::candid::contract::Error>
    );
    same!(
        storage::stable_restore::<(bool, bool, u64, u64)>,
        ic_cdk::storage::stable_restore::<(bool, bool, u64, u64)>,
        fn() -> Result<(bool, bool, u64, u64), String>
    );
}
