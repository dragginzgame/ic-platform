//! Ledger-only consumer: no legacy Candid feature or codec required.
mod support;
use ic_platform::{
    candid_codec::{
        decoder::{Limits, decode_one},
        encode_one,
    },
    cycles_ledger_types as cycles,
    icrc_ledger_types::{Account, GetBlocksRequest, GetBlocksResult, QueryArchiveFn, Value},
    principal::Principal,
};
fn limits() -> Limits {
    Limits {
        bytes: 8192,
        header: 4096,
        types: 128,
        work: 1_000_000,
        allocation: 100_000,
    }
}
#[test]
fn frozen_account_and_recursive_archive_reply_decode_without_registry_codec() {
    let bytes = support::decode_hex(include_str!("fixtures/icrc-account.hex"));
    let account: Account = decode_one(&bytes, limits()).unwrap();
    assert_eq!(account.owner, Principal::anonymous());
    assert_eq!(encode_one(&account, 8192).unwrap(), bytes);
    let bytes = support::decode_hex(include_str!("fixtures/icrc-blocks.hex"));
    let blocks: GetBlocksResult = decode_one(&bytes, limits()).unwrap();
    assert_eq!(blocks.log_length, 2u64);
    assert_eq!(
        blocks.archived_blocks[0].callback.method,
        "icrc3_get_blocks"
    );
    let encoded = encode_one(&blocks, 8192).unwrap();
    assert_eq!(
        decode_one::<GetBlocksResult>(&encoded, limits()).unwrap(),
        blocks
    );
}
#[test]
fn cycles_retains_nat64_and_its_own_recursive_callback_signature() {
    let value = cycles::GetBlocksResult {
        log_length: 1u64.into(),
        blocks: vec![cycles::BlockWithId {
            id: 0u64.into(),
            block: Value::Nat64(u64::MAX),
        }],
        archived_blocks: vec![cycles::ArchivedBlocks {
            args: vec![GetBlocksRequest {
                start: 1u64.into(),
                length: 2u64.into(),
            }],
            callback: QueryArchiveFn::new(Principal::anonymous(), "blocks"),
        }],
    };
    let bytes = encode_one(&value, 8192).unwrap();
    let decoded: cycles::GetBlocksResult = decode_one(&bytes, limits()).unwrap();
    assert_eq!(decoded.blocks[0].block, Value::Nat64(u64::MAX));
    assert_eq!(decoded.archived_blocks[0].callback.method, "blocks");
    for n in 0..bytes.len() {
        assert!(decode_one::<cycles::GetBlocksResult>(&bytes[..n], limits()).is_err());
    }
}
