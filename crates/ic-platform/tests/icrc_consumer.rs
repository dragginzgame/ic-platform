//! Standalone Platform-only consumer of the explicit external ledger surface.
use ic_platform::candid::{DecodeLimits, contract, decode_exactly_one};
use ic_platform::icrc_ledger_types::{
    Account, Allowance, AllowanceArgs, ArchivedBlocks, BlockWithId, GetArchivesArgs,
    GetBlocksRequest, GetBlocksResult, ICRC3ArchiveInfo, ICRC3DataCertificate, ICRC3Value,
    MetadataValue, QueryArchiveFn, SupportedBlockType, TransferError, Value,
};
use ic_platform::principal::ExternalPrincipal as Principal;

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
    let standard = ic_platform::icrc_ledger_types::icrc10::SupportedStandard {
        name: "ICRC-10".into(),
        url: "https://example.test/standard".into(),
    };
    roundtrip_value(&vec![standard]);
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

#[test]
fn owned_accounts_preserve_external_semantics_and_wire() {
    use ic_platform::account::{Account as OwnedAccount, ExternalAccount};
    for len in 0..=29 {
        for seed in 0..=255u8 {
            let owner = Principal::from_slice(&vec![seed; len]);
            for subaccount in [None, Some([0; 32]), Some([seed; 32])] {
                let external = ExternalAccount { owner, subaccount };
                let owned: OwnedAccount = external.into();
                assert_eq!(owned.to_string(), external.to_string());
                assert_eq!(owned.to_string().parse::<OwnedAccount>().unwrap(), owned);
                let restored: ExternalAccount = owned.into();
                assert_eq!(restored.subaccount, external.subaccount);
                assert_eq!(
                    contract::encode_one(restored).unwrap(),
                    contract::encode_one(external).unwrap()
                );
            }
        }
    }
}

#[test]
fn owned_account_parser_and_order_match_external_owner() {
    use ic_platform::account::{Account as OwnedAccount, ExternalAccount};
    use std::hash::{Hash, Hasher};
    let mut accounts = Vec::new();
    // Every possible trimmed hex length, odd/even nibble offsets and mixed bytes.
    for digits in 1..=64 {
        let mut subaccount = [0; 32];
        for (index, byte) in subaccount.iter_mut().enumerate().skip((64 - digits) / 2) {
            *byte = (index as u8).wrapping_mul(37).wrapping_add(19);
        }
        subaccount[(64 - digits) / 2] = if digits % 2 == 0 { 0xa1 } else { 1 };
        let external = ExternalAccount {
            owner: Principal::from_slice(&[digits as u8; 29]),
            subaccount: Some(subaccount),
        };
        let text = external.to_string();
        assert_eq!(text.split_once('.').unwrap().1.len(), digits);
        assert_eq!(text.len(), 72 + digits);
        let owned: OwnedAccount = text.parse().unwrap();
        assert_eq!(owned.owner.as_slice(), external.owner.as_slice());
        assert_eq!(owned.subaccount, external.subaccount);
        assert_eq!(owned.to_string(), text);
        let mut original_hash = std::collections::hash_map::DefaultHasher::new();
        let mut owned_hash = std::collections::hash_map::DefaultHasher::new();
        external.hash(&mut original_hash);
        owned.hash(&mut owned_hash);
        assert_eq!(original_hash.finish(), owned_hash.finish());
        accounts.push((external, owned));
        // Admission parity, not error spelling: mutate every byte including separators.
        for index in 0..text.len() {
            for replacement in [b'0', b'1', b'a', b'F', b'-', b'.', b' ', 0] {
                let mut bytes = text.as_bytes().to_vec();
                bytes[index] = replacement;
                let mutated = String::from_utf8(bytes).unwrap();
                let original = mutated.parse::<ExternalAccount>();
                let candidate = mutated.parse::<OwnedAccount>();
                assert_eq!(original.is_ok(), candidate.is_ok(), "{mutated:?}");
                if let (Ok(original), Ok(candidate)) = (original, candidate) {
                    assert_eq!(original.owner.as_slice(), candidate.owner.as_slice());
                    assert_eq!(original.subaccount, candidate.subaccount);
                }
            }
        }
        for suffix in ["é", "😀", ".", "\n"] {
            assert!(format!("{text}{suffix}").parse::<OwnedAccount>().is_err());
        }
    }
    for (external, owned) in &accounts {
        for (other_external, other_owned) in &accounts {
            assert_eq!(external.cmp(other_external), owned.cmp(other_owned));
        }
    }
}

/// Construct transfers and consent messages without a direct Candid/ledger dependency.
pub fn transfers_and_consent_contracts() {
    use ic_platform::icrc_ledger_types::{Memo, TransferArg, icrc21};
    let request = TransferArg {
        from_subaccount: Some([7; 32]),
        to: Account {
            owner: Principal::anonymous(),
            subaccount: None,
        },
        fee: Some(10u64.into()),
        created_at_time: Some(42),
        memo: Some(Memo::from(99u64)),
        amount: 100u64.into(),
    };
    roundtrip_value(&request);
    let metadata = icrc21::ConsentMessageMetadata {
        language: "en".into(),
        utc_offset_minutes: Some(-60),
    };
    for device_spec in [
        None,
        Some(icrc21::DisplayMessageType::GenericDisplay),
        Some(icrc21::DisplayMessageType::FieldsDisplay),
    ] {
        roundtrip_value(&icrc21::ConsentMessageRequest {
            method: "icrc1_transfer".into(),
            arg: vec![1, 2, 3],
            user_preferences: icrc21::ConsentMessageSpec {
                metadata: metadata.clone(),
                device_spec,
            },
        });
    }
    for consent_message in [
        icrc21::ConsentMessage::GenericDisplayMessage("transfer".into()),
        icrc21::ConsentMessage::FieldsDisplayMessage(icrc21::FieldsDisplay {
            intent: "transfer".into(),
            fields: vec![
                (
                    "amount".into(),
                    icrc21::Value::TokenAmount {
                        decimals: 8,
                        amount: 100,
                        symbol: "ICP".into(),
                    },
                ),
                (
                    "time".into(),
                    icrc21::Value::TimestampSeconds { amount: 42 },
                ),
                (
                    "duration".into(),
                    icrc21::Value::DurationSeconds { amount: 60 },
                ),
                (
                    "description".into(),
                    icrc21::Value::Text {
                        content: "external consent".into(),
                    },
                ),
            ],
        }),
    ] {
        roundtrip_value(&icrc21::ConsentInfo {
            consent_message,
            metadata: metadata.clone(),
        });
    }
    for error in [
        icrc21::Icrc21Error::UnsupportedCanisterCall(icrc21::ErrorInfo {
            description: "unsupported".into(),
        }),
        icrc21::Icrc21Error::ConsentMessageUnavailable(icrc21::ErrorInfo {
            description: "unavailable".into(),
        }),
        icrc21::Icrc21Error::InsufficientPayment(icrc21::ErrorInfo {
            description: "insufficient".into(),
        }),
        icrc21::Icrc21Error::GenericError {
            error_code: 42u64.into(),
            description: "error".into(),
        },
    ] {
        roundtrip_value(&error);
    }
}

#[test]
fn external_transfer_and_consent_shapes_roundtrip() {
    transfers_and_consent_contracts();
}
