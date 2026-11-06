//! Public ICP account derivation contract, compiled on native and Wasm targets.
use ic_platform::{Principal, icp_ledger_types::account_identifier};

#[test]
fn canic_transfer_destination_keeps_its_exact_bytes() {
    let owner =
        Principal::from_text("iooej-vlrze-c5tme-tn7qt-vqe7z-7bsj5-ebxlc-hlzgs-lueo3-3yast-pae")
            .unwrap();
    let actual: [u8; 32] = account_identifier(owner, &[0; 32]);
    assert_eq!(
        actual,
        [
            189, 196, 238, 5, 212, 44, 208, 102, 151, 134, 137, 159, 37, 108, 143, 215, 33, 127,
            167, 17, 119, 189, 31, 167, 185, 83, 79, 86, 134, 128, 169, 56,
        ]
    );
}
