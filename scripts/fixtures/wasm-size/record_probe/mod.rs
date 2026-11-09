//! Matched named-record workload with canonical identities and arbitrary Nat.
#[cfg(feature = "codec-generic-record")]
type Principal = ic_platform::Principal;
#[cfg(feature = "codec-generic-record")]
type Nat = num_bigint::BigUint;

#[derive(Debug, PartialEq)]
pub struct AccountState {
    pub owner: Principal,
    pub balance: Nat,
    pub status: Status,
    pub memo: Option<Vec<u8>>,
}
#[derive(Debug, PartialEq)]
pub enum Status {
    Failed(String),
    Ready(Vec<u64>),
}

#[cfg(feature = "codec-generic-record")]
ic_platform::candid_record!(AccountState {
    100394802 => status: Status,
    596483356 => balance: Nat,
    947296307 => owner: Principal,
    1213809850 => memo: Option<Vec<u8>>,
});
#[cfg(feature = "codec-generic-record")]
ic_platform::candid_variant!(Status {
    479410653 => Failed(String),
    2045251651 => Ready(Vec<u64>),
});

#[unsafe(export_name = "canister_query codec")]
pub extern "C" fn roundtrip() {
    let bytes = super::input();
    #[cfg(feature = "codec-generic-record")]
    let output = {
        let limits = crate::codec::decoder::Limits {
            bytes: 1_048_576,
            header: 65_536,
            types: 4096,
            work: 10_000_000,
            allocation: 4_194_304,
        };
        let (value,): (AccountState,) =
            crate::codec::decoder::decode_args(&bytes, limits, crate::codec::decoder::Arity::Exact)
                .unwrap();
        crate::codec::arguments::encode_args(&(value,), 1_048_576).unwrap()
    };
    super::reply(&output);
}
