//! Installed stable checks qualification scenarios.
use super::{api, codec, protocol};

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::StableEdit as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::StableEdit as u64 }>"
)]
fn stable_edit(value: u64, grow: bool, fail: bool) -> (u64, u64) {
    let before = ic_platform_runtime::ic0::stable64_size();
    if before == 0 || grow {
        assert_eq!(ic_platform_runtime::ic0::stable64_grow(1), before);
    }
    ic_platform_runtime::ic0::stable64_write(&value.to_le_bytes(), 0);
    if fail {
        api::trap("fixture stable rollback");
    }
    (before, ic_platform_runtime::ic0::stable64_size())
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::StableValue as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::StableValue as u64 }>"
)]
fn stable_value() -> (u64, u64) {
    let mut bytes = [0; 8];
    ic_platform_runtime::ic0::stable64_read(&mut bytes, 0);
    (
        ic_platform_runtime::ic0::stable64_size(),
        u64::from_le_bytes(bytes),
    )
}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::StableRefusedGrowth as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::StableRefusedGrowth as u64 }>"
)]
fn stable_refused_growth() -> (u64, u64) {
    let result = ic_platform_runtime::ic0::stable64_grow(u64::MAX);
    (result, ic_platform_runtime::ic0::stable64_size())
}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::StableBoundsTrap as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::StableBoundsTrap as u64 }>"
)]
fn stable_bounds_trap(write: bool) {
    // A subsequent out-of-bounds call must roll back this valid write too.
    ic_platform_runtime::ic0::stable64_write(&[255], 0);
    let end = ic_platform_runtime::ic0::stable64_size() * 65_536;
    if write {
        ic_platform_runtime::ic0::stable64_write(&[1], end);
    } else {
        ic_platform_runtime::ic0::stable64_read(&mut [0], end);
    }
}
