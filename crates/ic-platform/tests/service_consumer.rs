//! Service generation is selected independently from external DTO encoding.
use ic_platform::candid::contract;

#[contract::candid_method(query, rename = "value")]
fn declared_value() -> u64 {
    1
}

contract::export_service!(ic_platform::candid::contract);

#[test]
fn service_generation_preserves_declarations() {
    assert_eq!(declared_value(), 1);
    assert_eq!(
        __export_service(),
        "service : { value : () -> (nat64) query }"
    );
}
