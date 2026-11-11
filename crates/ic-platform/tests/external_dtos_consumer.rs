#![cfg(not(target_family = "wasm"))]
//! Independent public observations of four passive external DTO families.

use ic_platform::Principal;
use ic_platform::candid_codec::decoder::{Decode, Limits, decode_one};
use ic_platform::candid_codec::{Encode, Int, Nat, Type, encode_one};
use ic_platform::icrc_ledger_types::{GetBlocksRequest, QueryArchiveFn, Value};
use ic_platform::{
    certified_assets_types as assets, cmc_types as cmc, cycles_ledger_types as cycles,
    icp_ledger_types as icp,
};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use serde_json::{Value as Json, json};
use std::collections::BTreeMap;
use std::fmt::Debug;

const HUGE: &str = "340282366920938463463374607431768211456";
fn huge() -> Nat {
    HUGE.parse().unwrap()
}
fn id() -> Principal {
    Principal::anonymous()
}
fn limits(length: usize) -> Limits {
    Limits {
        bytes: length,
        header: 65_536,
        types: 8192,
        work: 2_000_000,
        allocation: 2_000_000,
    }
}

// Fixture construction only: occurrences of this fixture's Principal text are
// identity fields; no ordinary string sample equals that text. Keep all byte
// arrays as sequences while supplying Principal's binary CBOR representation.
fn cbor_input(input: &Json) -> serde_cbor::Value {
    fn principal_bytes(value: &mut serde_cbor::Value, text: &str) {
        match value {
            serde_cbor::Value::Text(value_text) if value_text == text => {
                *value = serde_cbor::Value::Bytes(id().as_slice().to_vec());
            }
            serde_cbor::Value::Array(values) => {
                for value in values {
                    principal_bytes(value, text);
                }
            }
            serde_cbor::Value::Map(fields) => {
                for value in fields.values_mut() {
                    principal_bytes(value, text);
                }
            }
            _ => {}
        }
    }
    let mut result = serde_cbor::value::to_value(input).unwrap();
    principal_bytes(&mut result, &id().to_text());
    result
}

fn observe<T>(label: &str, input: Json) -> (String, String, Vec<u8>, String, Vec<u8>)
where
    T: for<'a> Deserialize<'a> + Encode + for<'a> Decode<'a> + Debug + Eq + Clone,
{
    let value: T = serde_json::from_value(input.clone()).unwrap();
    let cbor = serde_cbor::to_vec(&cbor_input(&input)).unwrap();
    let from_cbor: T = serde_cbor::from_slice(&cbor).unwrap();
    assert_eq!(from_cbor, value);
    assert_eq!(value.clone(), value);
    let candid = encode_one(&value, 131_072).unwrap();
    let decoded: T = decode_one(&candid, limits(candid.len())).unwrap();
    assert_eq!(decoded, value);
    (
        label.to_owned(),
        input.to_string(),
        cbor,
        format!("{value:?}"),
        candid,
    )
}

fn notify_errors() -> Json {
    json!([
        {"Refunded":{"reason":"\0λ","block_index":u64::MAX}},
        {"Refunded":{"reason":"","block_index":null}},
        "Processing", {"TransactionTooOld":u64::MAX}, {"InvalidTransaction":"invalid\0λ"},
        {"Other":{"error_code":u64::MAX,"error_message":"other"}}
    ])
}

fn create_errors() -> Json {
    json!([
        {"InsufficientFunds":{"balance":HUGE}},"TooOld",
        {"CreatedInFuture":{"ledger_time":u64::MAX}},"TemporarilyUnavailable",
        {"Duplicate":{"duplicate_of":HUGE,"canister_id":id().to_text()}},
        {"Duplicate":{"duplicate_of":0,"canister_id":null}},
        {"FailedToCreate":{"fee_block":HUGE,"refund_block":null,"error":"\0λ"}},
        {"FailedToCreate":{"fee_block":null,"refund_block":HUGE,"error":""}},
        {"GenericError":{"message":"generic","error_code":HUGE}}
    ])
}

fn withdraw_errors() -> Json {
    json!([
        {"GenericError":{"message":"\0λ","error_code":HUGE}},"TemporarilyUnavailable",
        {"FailedToWithdraw":{"fee_block":HUGE,"rejection_code":"Unknown","rejection_reason":"\0λ"}},
        {"FailedToWithdraw":{"fee_block":null,"rejection_code":"NoError","rejection_reason":""}},
        {"Duplicate":{"duplicate_of":HUGE}}, {"BadFee":{"expected_fee":HUGE}},
        {"InvalidReceiver":{"receiver":id().to_text()}},
        {"CreatedInFuture":{"ledger_time":u64::MAX}},"TooOld",{"InsufficientFunds":{"balance":HUGE}}
    ])
}

fn settings_input() -> Json {
    json!({"controllers":[id().to_text()],"compute_allocation":HUGE,"memory_allocation":HUGE,
        "freezing_threshold":HUGE,"reserved_cycles_limit":HUGE,"wasm_memory_limit":HUGE,
        "wasm_memory_threshold":HUGE,"log_visibility":{"allowed_viewers":[id().to_text()]},
        "environment_variables":[{"name":"\0name","value":"\nλ"}]})
}

#[test]
fn passive_options_and_sequences_do_not_add_admission_policy() {
    let mint: cmc::NotifyMintCyclesArg = serde_json::from_value(json!({"block_index":0})).unwrap();
    assert!(mint.to_subaccount.is_none() && mint.deposit_memo.is_none());
    let create: cycles::CreateCanisterArgs =
        serde_json::from_value(json!({"amount":HUGE})).unwrap();
    assert_eq!(create.amount, huge());
    assert!(
        create.from_subaccount.is_none()
            && create.created_at_time.is_none()
            && create.creation_args.is_none()
    );
    let settings: cycles::CanisterSettings = serde_json::from_str("{}").unwrap();
    assert!(
        settings.controllers.is_none()
            && settings.compute_allocation.is_none()
            && settings.log_visibility.is_none()
            && settings.environment_variables.is_none()
    );
    let transfer: icp::TransferArgs = serde_json::from_value(
        json!({"memo":u64::MAX,"amount":{"e8s":u64::MAX},"fee":{"e8s":0},"to":[0,255]}),
    )
    .unwrap();
    assert_eq!(transfer.to, [0, 255]);
    assert!(transfer.from_subaccount.is_none() && transfer.created_at_time.is_none());
    let asset: assets::GetResult = serde_json::from_value(json!({"content":[],"content_type":"not validated","content_encoding":"\0λ","total_length":HUGE})).unwrap();
    assert!(asset.sha256.is_none());
    assert_eq!(asset.total_length, huge());
    let cbor = cbor_input(&json!({"to":[0,255],"from_subaccount":[0]}));
    let serde_cbor::Value::Map(fields) = cbor else {
        panic!("input record")
    };
    assert!(
        matches!(fields.get(&serde_cbor::Value::Text("to".to_owned())),Some(serde_cbor::Value::Array(values)) if values.len()==2)
    );
    assert!(serde_json::from_str::<cmc::NotifyTopUpArg>("{}").is_err());
    assert!(serde_json::from_str::<cycles::CreateCanisterError>("\"too_old\"").is_err());
    assert!(serde_json::from_value::<icp::Tokens>(json!({"e8s":HUGE})).is_err());
    assert!(serde_json::from_value::<cycles::CreateCanisterArgs>(json!({"amount":-1})).is_err());
}

struct OwnedBytes(Vec<u8>);
impl<'de> Deserializer<'de> for OwnedBytes {
    type Error = serde::de::value::Error;
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_byte_buf(self.0)
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }
    serde::forward_to_deserialize_any! { bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option unit unit_struct seq tuple tuple_struct map struct enum identifier ignored_any }
}

fn callback_input() -> Vec<u8> {
    let mut bytes = vec![5, 3];
    bytes.extend_from_slice("λ\0".as_bytes());
    bytes.extend_from_slice(id().as_slice());
    bytes
}

fn provider() -> cycles::GetBlocksResult {
    cycles::GetBlocksResult {
        log_length: huge(),
        blocks: vec![cycles::BlockWithId {
            id: huge(),
            block: Value::Array(vec![
                Value::Nat64(u64::MAX),
                Value::Nat(huge()),
                Value::Int(Int("-340282366920938463463374607431768211457"
                    .parse()
                    .unwrap())),
                Value::Blob(serde_bytes::ByteBuf::from(vec![0, 255])),
                Value::Text("\0λ".to_owned()),
                Value::Map(BTreeMap::from([("entry".to_owned(), Value::Nat64(0))])),
            ]),
        }],
        archived_blocks: vec![cycles::ArchivedBlocks {
            args: vec![GetBlocksRequest {
                start: huge(),
                length: Nat::from(u64::MAX),
            }],
            callback: QueryArchiveFn::new(id(), "λ\0"),
        }],
    }
}

#[test]
fn callbacks_and_provider_nat64_keep_exact_owned_schemas() {
    let callback = icp::QueryEncodedBlocksFn::deserialize(OwnedBytes(callback_input())).unwrap();
    assert_eq!(
        callback,
        icp::QueryEncodedBlocksFn::new(id(), "λ\0".to_owned())
    );
    let bytes = encode_one(&callback, 65_536).unwrap();
    assert_eq!(
        decode_one::<icp::QueryEncodedBlocksFn>(&bytes, limits(bytes.len())).unwrap(),
        callback
    );
    let Type::Function {
        args,
        results,
        annotations,
    } = <icp::QueryEncodedBlocksFn as Encode>::TYPE
    else {
        panic!("function schema")
    };
    assert_eq!(args.len(), 1);
    assert_eq!(results.len(), 1);
    assert_eq!(*annotations, [1]);
    assert!(
        decode_one::<QueryArchiveFn<Vec<GetBlocksRequest>, cycles::GetBlocksResult>>(
            &bytes,
            limits(bytes.len())
        )
        .is_err()
    );
    let callback = QueryArchiveFn::<Vec<GetBlocksRequest>, cycles::GetBlocksResult>::deserialize(
        OwnedBytes(callback_input()),
    )
    .unwrap();
    assert_eq!(callback.canister_id, id());
    assert_eq!(callback.method, "λ\0");
    let bytes = encode_one(&callback, 65_536).unwrap();
    assert!(
        decode_one::<
            QueryArchiveFn<Vec<GetBlocksRequest>, ic_platform::icrc_ledger_types::GetBlocksResult>,
        >(&bytes, limits(bytes.len()))
        .is_err()
    );
    let original = provider();
    let bytes = encode_one(&original, 131_072).unwrap();
    let decoded: cycles::GetBlocksResult = decode_one(&bytes, limits(bytes.len())).unwrap();
    assert_eq!(decoded.log_length, huge());
    assert_eq!(decoded.blocks[0].id, huge());
    let Value::Array(values) = &decoded.blocks[0].block else {
        panic!("provider value")
    };
    assert!(matches!(&values[0],Value::Nat64(value) if *value==u64::MAX));
    assert!(matches!(&values[1],Value::Nat(value) if *value==huge()));
    assert_eq!(decoded.archived_blocks[0].callback.canister_id, id());
    assert_eq!(decoded.archived_blocks[0].callback.method, "λ\0");
    assert_eq!(encode_one(&decoded, 131_072).unwrap(), bytes);
    assert_eq!(encode_one(&decoded.clone(), 131_072).unwrap(), bytes);
}

#[test]
fn deterministic_external_dto_observations() {
    let observations = vec![
        observe::<Vec<cmc::NotifyTopUpArg>>(
            "cmc-top-up",
            json!([{"block_index":0,"canister_id":id().to_text()},{"block_index":u64::MAX,"canister_id":id().to_text()}]),
        ),
        observe::<Vec<cmc::NotifyError>>("cmc-errors", notify_errors()),
        observe::<Vec<cmc::NotifyTopUpResult>>(
            "cmc-top-up-results",
            json!([{"Ok":HUGE},{"Err":"Processing"}]),
        ),
        observe::<cmc::IcpXdrConversionRateResponse>(
            "cmc-rate-response",
            json!({"data":{"timestamp_seconds":u64::MAX,"xdr_permyriad_per_icp":0},"hash_tree":[],"certificate":[0,255]}),
        ),
        observe::<Vec<cmc::NotifyMintCyclesArg>>(
            "cmc-mint-args",
            json!([{"block_index":0},{"block_index":u64::MAX,"to_subaccount":[],"deposit_memo":[0,255]}]),
        ),
        observe::<Vec<cmc::NotifyMintCyclesResult>>(
            "cmc-mint-results",
            json!([{"Ok":{"block_index":HUGE,"minted":HUGE,"balance":HUGE}},{"Err":"Processing"}]),
        ),
        observe::<Vec<icp::TransferArgs>>(
            "icp-transfer",
            json!([{"memo":0,"amount":{"e8s":0},"fee":{"e8s":0},"to":[]},{"memo":u64::MAX,"amount":{"e8s":u64::MAX},"fee":{"e8s":u64::MAX},"from_subaccount":[0,255],"to":[128,1],"created_at_time":{"timestamp_nanos":u64::MAX}}]),
        ),
        observe::<Vec<icp::TransferError>>(
            "icp-transfer-errors",
            json!([{"BadFee":{"expected_fee":{"e8s":u64::MAX}}},{"InsufficientFunds":{"balance":{"e8s":0}}},{"TxTooOld":{"allowed_window_nanos":u64::MAX}},"TxCreatedInFuture",{"TxDuplicate":{"duplicate_of":u64::MAX}}]),
        ),
        observe::<Vec<icp::TransferResult>>(
            "icp-transfer-results",
            json!([{"Ok":u64::MAX},{"Err":"TxCreatedInFuture"}]),
        ),
        observe::<icp::GetBlocksArgs>(
            "icp-block-args",
            json!({"start":u64::MAX,"length":u64::MAX}),
        ),
        observe::<Vec<icp::QueryArchiveError>>(
            "icp-archive-errors",
            json!([{"BadFirstBlockIndex":{"requested_index":u64::MAX,"first_valid_index":0}},{"Other":{"error_code":u64::MAX,"error_message":"\0λ"}}]),
        ),
        observe::<Vec<icp::QueryEncodedBlocksResult>>(
            "icp-archive-results",
            json!([{"Ok":[[],[0,255]]},{"Err":{"Other":{"error_code":0,"error_message":""}}}]),
        ),
        observe::<Vec<icp::QueryEncodedBlocksResponse>>(
            "icp-block-response",
            json!([{"blocks":[],"chain_length":0,"first_block_index":0,"archived_blocks":[]},{"certificate":[0,255],"blocks":[[],[0,255]],"chain_length":u64::MAX,"first_block_index":u64::MAX,"archived_blocks":[]}]),
        ),
        observe::<Vec<cycles::CanisterSettings>>("cycles-settings", json!([{}, settings_input()])),
        observe::<Vec<cycles::LogVisibility>>(
            "cycles-log-visibility",
            json!(["controllers","public",{"allowed_viewers":[id().to_text()]}]),
        ),
        observe::<Vec<cycles::SubnetSelection>>(
            "cycles-subnet-selection",
            json!([{"Subnet":{"subnet":id().to_text()}},{"Filter":{}},{"Filter":{"subnet_type":"\0λ"}}]),
        ),
        observe::<Vec<cycles::CreateCanisterArgs>>(
            "cycles-create",
            json!([{"amount":0},{"from_subaccount":[0,255],"created_at_time":u64::MAX,"amount":HUGE,"creation_args":{"settings":settings_input(),"subnet_selection":{"Subnet":{"subnet":id().to_text()}}}}]),
        ),
        observe::<Vec<cycles::CreateCanisterError>>("cycles-create-errors", create_errors()),
        observe::<Vec<cycles::CreateCanisterResult>>(
            "cycles-create-results",
            json!([{"Ok":{"block_id":HUGE,"canister_id":id().to_text()}},{"Err":"TooOld"}]),
        ),
        observe::<Vec<cycles::WithdrawArgs>>(
            "cycles-withdraw",
            json!([{"amount":0,"to":id().to_text()},{"amount":HUGE,"from_subaccount":[0,255],"to":id().to_text(),"created_at_time":u64::MAX}]),
        ),
        observe::<Vec<cycles::RejectionCode>>(
            "cycles-rejection-codes",
            json!([
                "NoError",
                "CanisterError",
                "SysTransient",
                "DestinationInvalid",
                "Unknown",
                "SysFatal",
                "CanisterReject"
            ]),
        ),
        observe::<Vec<cycles::WithdrawError>>("cycles-withdraw-errors", withdraw_errors()),
        observe::<Vec<cycles::WithdrawResult>>(
            "cycles-withdraw-results",
            json!([{"Ok":HUGE},{"Err":"TooOld"}]),
        ),
        observe::<Vec<assets::GetArgs>>(
            "assets-get",
            json!([{"key":"","accept_encodings":[]},{"key":"\0λ","accept_encodings":["identity","arbitrary\0"]}]),
        ),
        observe::<Vec<assets::GetResult>>(
            "assets-get-results",
            json!([{"content":[],"content_type":"","content_encoding":"","total_length":0},{"content":[0,255],"content_type":"\0λ","content_encoding":"arbitrary","sha256":[1],"total_length":HUGE}]),
        ),
        observe::<Vec<assets::GetChunkArgs>>(
            "assets-chunk",
            json!([{"key":"","content_encoding":"","index":0},{"key":"\0λ","content_encoding":"arbitrary","index":HUGE,"sha256":[0,255]}]),
        ),
        observe::<Vec<assets::GetChunkResult>>(
            "assets-chunk-results",
            json!([{"content":[]},{"content":[0,255]}]),
        ),
    ];
    let archived = icp::QueryEncodedBlocksResponse {
        certificate: Some(vec![0, 255]),
        blocks: vec![vec![], vec![128, 1]],
        chain_length: u64::MAX,
        first_block_index: u64::MAX,
        archived_blocks: vec![icp::ArchivedEncodedBlocksRange {
            callback: icp::QueryEncodedBlocksFn::new(id(), "λ\0".to_owned()),
            start: u64::MAX,
            length: u64::MAX,
        }],
    };
    let bytes = encode_one(&archived, 131_072).unwrap();
    assert_eq!(
        decode_one::<icp::QueryEncodedBlocksResponse>(&bytes, limits(bytes.len())).unwrap(),
        archived
    );
    let constructed = vec![
        ("icp-callback-response", format!("{archived:?}"), bytes),
        (
            "cycles-provider-nat64",
            format!("{:?}", provider()),
            encode_one(&provider(), 131_072).unwrap(),
        ),
    ];
    let transcript = serde_json::to_string(
        &json!({"serde_inputs":observations,"constructed_callbacks":constructed}),
    )
    .unwrap();
    assert!(
        transcript.len() < 60_000,
        "bounded external fixture: {} bytes",
        transcript.len()
    );
    assert_eq!(
        transcript,
        include_str!("fixtures/external-dtos-observations.json").trim_end()
    );
    println!("EXTERNAL_DTOS_OBSERVATIONS={transcript}");
}
