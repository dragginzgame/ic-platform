//! Standalone Platform-only consumer of the explicit external ledger surface.
use ic_platform::Principal;
use ic_platform::candid::{DecodeLimits, contract, decode_exactly_one};
use ic_platform::icrc_ledger_types::{
    Account, Allowance, AllowanceArgs, ArchivedBlocks, BlockWithId, GetArchivesArgs,
    GetBlocksRequest, GetBlocksResult, ICRC3ArchiveInfo, ICRC3DataCertificate, ICRC3Value,
    MetadataValue, QueryArchiveFn, SupportedBlockType, TransferError, Value,
};

fn wire_hex(text: &str) -> Vec<u8> {
    let (pairs, remainder) = text.trim().as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn roundtrip_value<T>(value: &T) -> Vec<u8>
where
    T: contract::CandidType + for<'de> contract::serde::Deserialize<'de>,
{
    let bytes = contract::encode_one(value).unwrap();
    let decoded: T = decode_exactly_one(
        &bytes,
        DecodeLimits {
            reply_bytes: 4096,
            decoding_quota: 1_000_000,
            skipping_quota: 1024,
            type_entries: 128,
            header_bytes: 2048,
        },
    )
    .unwrap();
    assert_eq!(contract::encode_one(&decoded).unwrap(), bytes);
    bytes
}

/// Exercise upstream identities, account semantics, hashes, archive callbacks and bytes.
pub fn roundtrip() {
    let owner = Principal::anonymous();
    let account = Account {
        owner,
        subaccount: None,
    };
    assert_eq!(
        roundtrip_value(&account),
        wire_hex(include_str!("fixtures/icrc-account.hex"))
    );
    let zero_account = Account {
        owner,
        subaccount: Some([0; 32]),
    };
    assert_eq!(account, zero_account);
    assert_ne!(roundtrip_value(&account), roundtrip_value(&zero_account));
    assert_eq!(account.effective_subaccount(), &[0; 32]);
    let named = Account {
        owner,
        subaccount: Some([7; 32]),
    };
    assert_eq!(named.to_string().parse::<Account>().unwrap(), named);
    let big: contract::Nat = "340282366920938463463374607431768211456".parse().unwrap();
    for value in [
        MetadataValue::Nat(big.clone()),
        MetadataValue::Int((-65_i64).into()),
        MetadataValue::Text("token".into()),
        MetadataValue::Blob(vec![0, 255].into()),
    ] {
        roundtrip_value(&value);
    }
    roundtrip_value(&AllowanceArgs {
        account,
        spender: named,
    });
    roundtrip_value(&Allowance {
        allowance: big.clone(),
        expires_at: Some(123),
    });
    roundtrip_value(&TransferError::Duplicate { duplicate_of: big });
    let legacy = Value::Map(
        [
            ("amount".into(), Value::Nat64(42)),
            ("memo".into(), Value::Blob(vec![0, 128, 255].into())),
            ("signed".into(), Value::Int((-65_i64).into())),
        ]
        .into(),
    );
    let modern = ICRC3Value::from(legacy.clone());
    let hash = wire_hex(include_str!("fixtures/icrc-value-hash.hex"));
    assert_eq!(legacy.clone().hash().as_slice(), hash);
    assert_eq!(modern.clone().hash().as_slice(), hash);
    assert_ne!(roundtrip_value(&legacy), roundtrip_value(&modern));
    let request = GetBlocksRequest {
        start: 1_u8.into(),
        length: 1_u8.into(),
    };
    roundtrip_value(&request);
    assert_eq!(request.as_start_and_length().unwrap(), (1, 1));
    let callback: QueryArchiveFn<Vec<GetBlocksRequest>, GetBlocksResult> =
        QueryArchiveFn::new(owner, "icrc3_get_blocks");
    roundtrip_value(&callback);
    let result = GetBlocksResult {
        log_length: 2_u8.into(),
        blocks: vec![BlockWithId {
            id: 0_u8.into(),
            block: modern,
        }],
        archived_blocks: vec![ArchivedBlocks {
            args: vec![request],
            callback,
        }],
    };
    assert_eq!(
        roundtrip_value(&result),
        wire_hex(include_str!("fixtures/icrc-blocks.hex"))
    );
    roundtrip_value(&GetArchivesArgs { from: Some(owner) });
    roundtrip_value(&ICRC3ArchiveInfo {
        canister_id: owner,
        start: 1_u8.into(),
        end: 2_u8.into(),
    });
    roundtrip_value(&ICRC3DataCertificate {
        certificate: vec![0, 255].into(),
        hash_tree: vec![1, 128].into(),
    });
    roundtrip_value(&SupportedBlockType {
        block_type: "1xfer".into(),
        url: "https://example.invalid/spec".into(),
    });
}

#[test]
fn ledger_dtos_preserve_wire_hashes_and_type_identities() {
    roundtrip();
}
