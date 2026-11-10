//! Public DTO observations from the reviewed passive-declaration contract.

use ic_platform::Principal;
use ic_platform::candid_codec::decoder::{Limits, decode_one};
use ic_platform::candid_codec::{Encode, Int, Nat, encode_one};
use ic_platform::icrc_ledger_types::icrc21::{
    ConsentInfo, ConsentMessage, ConsentMessageMetadata, ConsentMessageRequest, ConsentMessageSpec,
    DisplayMessageType, ErrorInfo, FieldsDisplay, Icrc21Error, Value,
};
use ic_platform::icrc_ledger_types::{
    Account, Allowance, AllowanceArgs, ArchivedBlocks, BlockWithId, GetBlocksRequest,
    GetBlocksResult, ICRC3DataCertificate, ICRC3Value, Memo, MetadataValue, QueryArchiveFn,
    SupportedBlockType, TransferArg, TransferError,
};
use serde::Serialize;
use serde_bytes::ByteBuf;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn huge() -> Nat {
    Nat("340282366920938463463374607431768211456".parse().unwrap())
}

fn owner() -> Account {
    Account {
        owner: Principal::anonymous(),
        subaccount: Some([0; 32]),
    }
}

fn limits(length: usize) -> Limits {
    Limits {
        bytes: length,
        header: 32_768,
        types: 4096,
        work: 1_000_000,
        allocation: 1_000_000,
    }
}

#[test]
fn metadata_conversions_preserve_extrema_and_owned_allocations() {
    for value in [i64::MIN, 0, i64::MAX] {
        let MetadataValue::Int(result) = MetadataValue::from(value) else {
            panic!("signed metadata")
        };
        assert_eq!(result.0.to_string(), value.to_string());
    }
    for value in [i128::MIN, i128::MAX] {
        let MetadataValue::Int(result) = MetadataValue::from(value) else {
            panic!("signed metadata")
        };
        assert_eq!(result.0.to_string(), value.to_string());
    }
    for value in [0u64, u64::MAX] {
        let MetadataValue::Nat(result) = MetadataValue::from(value) else {
            panic!("unsigned metadata")
        };
        assert_eq!(result.0.to_string(), value.to_string());
    }
    let MetadataValue::Nat(result) = MetadataValue::from(u128::MAX) else {
        panic!("unsigned metadata")
    };
    assert_eq!(result.0.to_string(), u128::MAX.to_string());
    let MetadataValue::Nat(result) = MetadataValue::from(huge()) else {
        panic!("owned magnitude")
    };
    assert_eq!(result, huge());

    let text = String::from("owned\0λ");
    let allocation = text.as_ptr();
    let MetadataValue::Text(text) = MetadataValue::from(text) else {
        panic!("owned text")
    };
    assert_eq!(text.as_ptr(), allocation);
    assert_eq!(
        MetadataValue::from(text.as_str()),
        MetadataValue::Text(text)
    );
    let bytes = vec![0, 255, 4];
    let allocation = bytes.as_ptr();
    let MetadataValue::Blob(bytes) = MetadataValue::from(bytes) else {
        panic!("owned blob")
    };
    assert_eq!(bytes.as_slice().as_ptr(), allocation);
    let mut source = [1, 2, 3];
    let copied = MetadataValue::from(source.as_slice());
    source[0] = 9;
    assert_eq!(copied, MetadataValue::Blob(ByteBuf::from(vec![1, 2, 3])));
    assert_ne!(copied, MetadataValue::from(source.as_slice()));
}

#[test]
fn memo_moves_buffers_and_u64_always_uses_eight_big_endian_bytes() {
    for value in [0, 1, 0x0102_0304_0506_0708, u64::MAX] {
        let memo = Memo::from(value);
        assert_eq!(memo.0.as_slice(), value.to_be_bytes());
        assert_eq!(memo.0.len(), 8);
    }
    let bytes = vec![0, 255, 9];
    let allocation = bytes.as_ptr();
    let memo = Memo::from(bytes);
    assert_eq!(memo.0.as_slice().as_ptr(), allocation);
    let bytes: ByteBuf = memo.into();
    assert_eq!(bytes.as_slice().as_ptr(), allocation);
    let memo = Memo::from(bytes);
    assert_eq!(memo.0.as_slice().as_ptr(), allocation);
    assert!(Memo::default().0.is_empty());
    assert!(Memo::from(1u64) < Memo::from(2u64));
    let mut first = DefaultHasher::new();
    let mut second = DefaultHasher::new();
    memo.hash(&mut first);
    memo.clone().hash(&mut second);
    assert_eq!(first.finish(), second.finish());
    assert_eq!(
        serde_cbor::value::to_value(&memo).unwrap(),
        serde_cbor::Value::Bytes(vec![0, 255, 9])
    );
    assert_eq!(
        serde_json::to_value(&memo).unwrap(),
        serde_json::json!([0, 255, 9])
    );
}

fn transfer_errors() -> Vec<TransferError> {
    vec![
        TransferError::BadFee {
            expected_fee: huge(),
        },
        TransferError::BadBurn {
            min_burn_amount: huge(),
        },
        TransferError::InsufficientFunds { balance: huge() },
        TransferError::TooOld,
        TransferError::CreatedInFuture {
            ledger_time: u64::MAX,
        },
        TransferError::TemporarilyUnavailable,
        TransferError::Duplicate {
            duplicate_of: huge(),
        },
        TransferError::GenericError {
            error_code: huge(),
            message: "failure\0λ".to_owned(),
        },
    ]
}

#[test]
fn every_transfer_error_retains_its_exact_display() {
    let amount = huge();
    let expected = [
        format!("transfer fee should be {amount}"),
        format!("the minimum number of tokens to be burned is {amount}"),
        format!(
            "the debit account doesn't have enough funds to complete the transaction, current balance: {amount}"
        ),
        "transaction's created_at_time is too far in the past".to_owned(),
        format!(
            "transaction's created_at_time is in future, current ledger time is {}",
            u64::MAX
        ),
        "the ledger is temporarily unavailable".to_owned(),
        format!("transaction is a duplicate of another transaction in block {amount}"),
        format!("{amount} failure\0λ"),
    ];
    for (error, expected) in transfer_errors().into_iter().zip(expected) {
        assert_eq!(error.to_string(), expected);
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(
            serde_json::from_value::<TransferError>(json).unwrap(),
            error
        );
    }
}

#[test]
fn defaults_and_sequence_serde_are_preserved_without_admission_policy() {
    let transfer: TransferArg =
        serde_json::from_value(serde_json::json!({"to": owner(), "amount": huge()})).unwrap();
    assert_eq!(transfer.from_subaccount, None);
    assert_eq!(transfer.fee, None);
    assert_eq!(transfer.created_at_time, None);
    assert_eq!(transfer.memo, None);
    let json = serde_json::to_value(&transfer).unwrap();
    assert_eq!(json.as_object().unwrap().len(), 6);
    for name in ["from_subaccount", "fee", "created_at_time", "memo"] {
        assert!(json[name].is_null());
    }
    let mut transfer = transfer;
    transfer.from_subaccount = Some([255; 32]);
    let serde_cbor::Value::Map(fields) = serde_cbor::value::to_value(&transfer).unwrap() else {
        panic!("transfer struct")
    };
    assert!(
        matches!(fields.get(&serde_cbor::Value::Text("from_subaccount".to_owned())), Some(serde_cbor::Value::Array(value)) if value.len() == 32)
    );
    let allowance: Allowance =
        serde_json::from_value(serde_json::json!({"allowance": huge()})).unwrap();
    assert_eq!(allowance.expires_at, None);
    assert_eq!(
        serde_json::to_value(&allowance)
            .unwrap()
            .as_object()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        FieldsDisplay::default(),
        FieldsDisplay {
            intent: String::new(),
            fields: Vec::new()
        }
    );
    assert!(serde_json::from_str::<TransferArg>("{}").is_err());
    assert!(serde_json::from_str::<Allowance>("{}").is_err());
}

#[test]
fn block_request_narrowing_preserves_extremes_and_error_precedence() {
    for (start, length) in [(0, 0), (u64::MAX, 0), (0, u64::MAX), (u64::MAX, u64::MAX)] {
        let request = GetBlocksRequest {
            start: Nat::from(start),
            length: Nat::from(length),
        };
        assert_eq!(request.as_start_and_length(), Ok((start, length)));
    }
    let both = GetBlocksRequest {
        start: huge(),
        length: huge(),
    };
    assert_eq!(
        both.as_start_and_length(),
        Err(format!(
            "transaction index {} is too large, max allowed: 18446744073709551615",
            huge()
        ))
    );
    let length = GetBlocksRequest {
        start: Nat::from(0u8),
        length: huge(),
    };
    assert_eq!(
        length.as_start_and_length(),
        Err(format!(
            "requested length {} is too large, max allowed: 18446744073709551615",
            huge()
        ))
    );
    let first = GetBlocksRequest {
        start: Nat::from(1u8),
        length: huge(),
    };
    let second = GetBlocksRequest {
        start: Nat::from(2u8),
        length: Nat::from(0u8),
    };
    assert!(first < second);
    assert!(length < first);
}

fn preferences() -> ConsentMessageSpec {
    ConsentMessageSpec {
        metadata: ConsentMessageMetadata {
            language: "unvalidated\0λ".to_owned(),
            utc_offset_minutes: Some(i16::MIN),
        },
        device_spec: Some(DisplayMessageType::FieldsDisplay),
    }
}

fn field_values() -> Vec<Value> {
    vec![
        Value::TokenAmount {
            decimals: u8::MAX,
            amount: u64::MAX,
            symbol: "\0TOKEN".to_owned(),
        },
        Value::TimestampSeconds { amount: u64::MAX },
        Value::DurationSeconds { amount: u64::MAX },
        Value::Text {
            content: "text\nλ".to_owned(),
        },
    ]
}

fn fields_display() -> FieldsDisplay {
    FieldsDisplay {
        intent: "informational only".to_owned(),
        fields: field_values()
            .into_iter()
            .enumerate()
            .map(|(index, value)| (index.to_string(), value))
            .collect(),
    }
}

#[test]
fn nested_consent_and_archive_codecs_preserve_passive_values() {
    let request = ConsentMessageRequest {
        method: "\0λ.method".to_owned(),
        arg: vec![0, 255, 128],
        user_preferences: preferences(),
    };
    let serde_cbor::Value::Map(fields) = serde_cbor::value::to_value(&request).unwrap() else {
        panic!("consent request struct")
    };
    assert!(
        matches!(fields.get(&serde_cbor::Value::Text("arg".to_owned())), Some(serde_cbor::Value::Array(values)) if values.len() == 3)
    );
    let bytes = encode_one(&request, 65_536).unwrap();
    assert_eq!(
        decode_one::<ConsentMessageRequest>(&bytes, limits(bytes.len())).unwrap(),
        request
    );
    let info = ConsentInfo {
        consent_message: ConsentMessage::FieldsDisplayMessage(fields_display()),
        metadata: ConsentMessageMetadata {
            language: String::new(),
            utc_offset_minutes: Some(i16::MAX),
        },
    };
    let bytes = encode_one(&info, 65_536).unwrap();
    assert_eq!(
        decode_one::<ConsentInfo>(&bytes, limits(bytes.len())).unwrap(),
        info
    );
    let result = blocks_result();
    let bytes = encode_one(&result, 65_536).unwrap();
    assert_eq!(
        decode_one::<GetBlocksResult>(&bytes, limits(bytes.len())).unwrap(),
        result
    );
}

fn blocks_result() -> GetBlocksResult {
    GetBlocksResult {
        log_length: huge(),
        blocks: vec![BlockWithId {
            id: huge(),
            block: ICRC3Value::Text("passive block\0λ".to_owned()),
        }],
        archived_blocks: vec![ArchivedBlocks {
            args: vec![GetBlocksRequest {
                start: huge(),
                length: Nat::from(u64::MAX),
            }],
            callback: QueryArchiveFn::new(Principal::management_canister(), "archive\0λ"),
        }],
    }
}

// One serialization helper keeps the observation format uniform; it adds no
// type registry, snapshot selection, policy, or alternate codec implementation.
fn observation<T: Serialize + Encode>(
    label: &str,
    value: &T,
) -> (String, String, Vec<u8>, Vec<u8>) {
    (
        label.to_owned(),
        serde_json::to_string(value).unwrap(),
        serde_cbor::to_vec(value).unwrap(),
        encode_one(value, 65_536).unwrap(),
    )
}

#[test]
fn deterministic_dto_serialization_observations() {
    let metadata = vec![
        MetadataValue::from(huge()),
        MetadataValue::Int(Int("-340282366920938463463374607431768211457"
            .parse()
            .unwrap())),
        MetadataValue::from("text\0λ"),
        MetadataValue::from(vec![0, 255, 128]),
        MetadataValue::from(i128::MIN),
        MetadataValue::from(u128::MAX),
    ];
    let transfer = TransferArg {
        from_subaccount: Some([255; 32]),
        to: owner(),
        fee: Some(huge()),
        created_at_time: Some(u64::MAX),
        memo: Some(Memo::from(1u64)),
        amount: huge(),
    };
    let absent = TransferArg {
        from_subaccount: None,
        to: Account::from(Principal::management_canister()),
        fee: None,
        created_at_time: None,
        memo: None,
        amount: Nat::from(0u8),
    };
    let messages = vec![
        ConsentMessage::GenericDisplayMessage("generic\0λ".to_owned()),
        ConsentMessage::FieldsDisplayMessage(fields_display()),
    ];
    let errors = vec![
        Icrc21Error::UnsupportedCanisterCall(ErrorInfo {
            description: "unsupported".to_owned(),
        }),
        Icrc21Error::ConsentMessageUnavailable(ErrorInfo {
            description: String::new(),
        }),
        Icrc21Error::InsufficientPayment(ErrorInfo {
            description: "payment\0λ".to_owned(),
        }),
        Icrc21Error::GenericError {
            error_code: huge(),
            description: "generic".to_owned(),
        },
    ];
    let observations = vec![
        observation("metadata-all-variants-and-integer-extremes", &metadata),
        observation("memo-empty", &Memo::default()),
        observation("memo-u64-zero", &Memo::from(0u64)),
        observation("memo-u64-max", &Memo::from(u64::MAX)),
        observation("transfer-all-fields", &transfer),
        observation("transfer-none-fields", &absent),
        observation("transfer-errors-all-variants", &transfer_errors()),
        observation(
            "allowance-args",
            &AllowanceArgs {
                account: owner(),
                spender: Account::from(Principal::management_canister()),
            },
        ),
        observation(
            "allowance-present-expiry",
            &Allowance {
                allowance: huge(),
                expires_at: Some(u64::MAX),
            },
        ),
        observation(
            "allowance-no-expiry",
            &Allowance {
                allowance: Nat::from(0u8),
                expires_at: None,
            },
        ),
        observation("blocks-nested-archive", &blocks_result()),
        observation(
            "blocks-empty",
            &GetBlocksResult {
                log_length: Nat::from(0u8),
                blocks: Vec::new(),
                archived_blocks: Vec::new(),
            },
        ),
        observation(
            "data-certificate",
            &ICRC3DataCertificate {
                certificate: ByteBuf::from(vec![0, 255]),
                hash_tree: ByteBuf::from(vec![128, 1]),
            },
        ),
        observation(
            "supported-block-type",
            &SupportedBlockType {
                block_type: "custom\0λ".to_owned(),
                url: "not a validated URL".to_owned(),
            },
        ),
        observation(
            "display-types-all-variants",
            &vec![
                DisplayMessageType::GenericDisplay,
                DisplayMessageType::FieldsDisplay,
            ],
        ),
        observation("consent-spec-present-device", &preferences()),
        observation(
            "consent-spec-no-device",
            &ConsentMessageSpec {
                metadata: ConsentMessageMetadata {
                    language: String::new(),
                    utc_offset_minutes: None,
                },
                device_spec: None,
            },
        ),
        observation(
            "consent-request",
            &ConsentMessageRequest {
                method: "method\0λ".to_owned(),
                arg: vec![0, 255],
                user_preferences: preferences(),
            },
        ),
        observation("consent-values-all-variants", &field_values()),
        observation("fields-default", &FieldsDisplay::default()),
        observation("consent-messages-all-variants", &messages),
        observation(
            "consent-info",
            &ConsentInfo {
                consent_message: ConsentMessage::FieldsDisplayMessage(fields_display()),
                metadata: ConsentMessageMetadata {
                    language: "\0λ".to_owned(),
                    utc_offset_minutes: Some(i16::MAX),
                },
            },
        ),
        observation("consent-errors-all-variants", &errors),
    ];
    let transcript = serde_json::to_string(&observations).unwrap();
    assert!(
        transcript.len() <= 30_000,
        "keep the finite fixture reviewable: {} bytes",
        transcript.len()
    );
    println!("ICRC_DTO_OBSERVATIONS={transcript}");
    // Captured through these same public inputs before the DTO replacement.
    assert_eq!(
        transcript,
        include_str!("fixtures/icrc-dto-observations.json").trim_end()
    );
}
