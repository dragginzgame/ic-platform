//! Independent Candid views of the ICP ledger and CMC top-up interfaces.
#![forbid(unsafe_code)]
#![cfg(all(feature = "ledger", not(target_family = "wasm")))]

use candid::{CandidType, Nat, Principal};
use candidate_types::{Account, Principal as CandidatePrincipal};
use ic_platform_protocol::{
    candid::{Error, Limits},
    ledger::{
        self,
        top_up::{
            self, Icrc1TransferArgs, Icrc1TransferError as TransferError,
            Icrc1TransferResult as TransferResult, NotifyTopUpArgs,
            NotifyTopUpError as NotifyError, NotifyTopUpResult as NotifyResult, TOP_UP_MEMO,
        },
    },
};
use serde::Deserialize;

const LIMITS: Limits = Limits {
    bytes: 4096,
    header: 2048,
    types: 64,
    work: 16384,
    allocation: 16384,
    vector: 4096,
    skipping: 4096,
};

///
/// AccountWire
/// The reference ledger account schema with explicit subaccount presence.
///

#[derive(CandidType, Deserialize, Debug, Eq, PartialEq)]
struct AccountWire {
    owner: Principal,
    subaccount: Option<Vec<u8>>,
}

///
/// TransferWire
/// The reference request shape, including the ledger's optional fields.
///

#[derive(CandidType, Deserialize, Debug, Eq, PartialEq)]
struct TransferWire {
    from_subaccount: Option<Vec<u8>>,
    to: AccountWire,
    amount: Nat,
    fee: Option<Nat>,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
}

///
/// NotifyWire
/// The reference notification with a fixed-width payment block.
///

#[derive(CandidType, Deserialize, Debug, Eq, PartialEq)]
struct NotifyWire {
    block_index: u64,
    canister_id: Principal,
}

///
/// TransferErrorWire
/// The published ledger error roster plus an unrecognized fixture case.
///

#[derive(CandidType)]
enum TransferErrorWire {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
    FutureVariant(Vec<u8>),
}

///
/// NotifyErrorWire
/// The published CMC error roster plus an unrecognized fixture case.
///

#[derive(CandidType)]
enum NotifyErrorWire {
    Refunded {
        reason: String,
        block_index: Option<u64>,
    },
    Processing,
    TransactionTooOld(u64),
    InvalidTransaction(String),
    Other {
        error_code: u64,
        error_message: String,
    },
    FutureVariant(Vec<u8>),
}

///
/// FutureResult
/// A top-level label outside both published reply schemas.
///

#[derive(CandidType)]
enum FutureResult {
    Deferred(Vec<u8>),
}

#[test]
fn exact_transfer_and_notification_arguments() {
    let cmc: CandidatePrincipal = "rkp4c-7iaaa-aaaaa-aaaca-cai".parse().unwrap();
    let beneficiary: CandidatePrincipal = "ryjl3-tyaaa-aaaaa-aaaba-cai".parse().unwrap();
    let subaccount = top_up::top_up_subaccount(&beneficiary);
    let expected_subaccount = [
        10, 0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    assert_eq!(subaccount, expected_subaccount);
    assert_eq!(TOP_UP_MEMO, *b"TPUP\0\0\0\0");
    let mut max_subaccount = [0; 32];
    max_subaccount[0] = 29;
    max_subaccount[1..30].fill(255);
    assert_eq!(
        top_up::top_up_subaccount(&CandidatePrincipal::from_slice(&[255; 29])),
        max_subaccount
    );

    for source in [None, Some([0; 32])] {
        let args = Icrc1TransferArgs {
            from_subaccount: source,
            to: Account {
                owner: cmc,
                subaccount: Some(subaccount),
            },
            amount: u64::MAX.into(),
            fee: 10_000.into(),
            memo: TOP_UP_MEMO,
            created_at_time: u64::MAX - 1,
        };
        let bytes = args.encode(512).unwrap();
        assert_eq!(args.encode(bytes.len()).unwrap(), bytes);
        assert_eq!(
            args.encode(bytes.len() - 1),
            Err(ledger::EncodeError::OutputLimit)
        );
        assert_eq!(
            candid::decode_one::<TransferWire>(&bytes).unwrap(),
            TransferWire {
                from_subaccount: source.map(|bytes| bytes.to_vec()),
                to: AccountWire {
                    owner: Principal::from_text("rkp4c-7iaaa-aaaaa-aaaca-cai").unwrap(),
                    subaccount: Some(expected_subaccount.to_vec()),
                },
                amount: u64::MAX.into(),
                fee: Some(10_000u64.into()),
                memo: Some(b"TPUP\0\0\0\0".to_vec()),
                created_at_time: Some(u64::MAX - 1),
            }
        );
    }

    let args = NotifyTopUpArgs {
        block_index: u64::MAX,
        canister_id: beneficiary,
    };
    let bytes = args.encode(128).unwrap();
    assert_eq!(args.encode(bytes.len()).unwrap(), bytes);
    assert_eq!(
        args.encode(bytes.len() - 1),
        Err(ledger::EncodeError::OutputLimit)
    );
    assert_eq!(
        candid::decode_one::<NotifyWire>(&bytes).unwrap(),
        NotifyWire {
            block_index: u64::MAX,
            canister_id: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
        }
    );
}

fn assert_reply<T: std::fmt::Debug + PartialEq>(
    input: impl CandidType,
    decode: fn(&[u8], Limits) -> Result<T, Error>,
    expected: T,
) {
    let bytes = candid::encode_one(input).unwrap();
    let exact = Limits {
        bytes: bytes.len(),
        ..LIMITS
    };
    assert_eq!(decode(&bytes, exact), Ok(expected));
    assert_eq!(
        decode(
            &bytes,
            Limits {
                bytes: bytes.len() - 1,
                ..LIMITS
            }
        ),
        Err(Error::Limit)
    );
    for end in 0..bytes.len() {
        assert!(decode(&bytes[..end], LIMITS).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(decode(&trailing, LIMITS), Err(Error::Trailing));
}

#[test]
fn complete_icrc1_outcomes() {
    use top_up::decode_icrc1_transfer as decode;
    let large = "340282366920938463463374607431768211456";
    assert_reply(
        Ok::<_, TransferErrorWire>(large.parse::<Nat>().unwrap()),
        decode,
        TransferResult::Ok(large.parse().unwrap()),
    );
    for (wire, projected) in [
        (
            TransferErrorWire::BadFee {
                expected_fee: 12u64.into(),
            },
            TransferError::BadFee(top_up::BadFee {
                expected_fee: 12.into(),
            }),
        ),
        (
            TransferErrorWire::BadBurn {
                min_burn_amount: 98u64.into(),
            },
            TransferError::BadBurn(top_up::BadBurn {
                min_burn_amount: 98.into(),
            }),
        ),
        (
            TransferErrorWire::InsufficientFunds {
                balance: 44u64.into(),
            },
            TransferError::InsufficientFunds(ledger::InsufficientFunds { balance: 44.into() }),
        ),
        (TransferErrorWire::TooOld, TransferError::TooOld),
        (
            TransferErrorWire::CreatedInFuture {
                ledger_time: u64::MAX,
            },
            TransferError::CreatedInFuture(ledger::Future {
                ledger_time: u64::MAX,
            }),
        ),
        (
            TransferErrorWire::TemporarilyUnavailable,
            TransferError::TemporarilyUnavailable,
        ),
        (
            TransferErrorWire::Duplicate {
                duplicate_of: large.parse().unwrap(),
            },
            TransferError::Duplicate(top_up::TransferDuplicate {
                duplicate_of: large.parse().unwrap(),
            }),
        ),
        (
            TransferErrorWire::GenericError {
                error_code: large.parse().unwrap(),
                message: "ledger detail".into(),
            },
            TransferError::GenericError(ledger::GenericError {
                error_code: large.parse().unwrap(),
                message: "ledger detail".into(),
            }),
        ),
        (
            TransferErrorWire::FutureVariant(vec![1, 2]),
            TransferError::Unknown(candid::idl_hash("FutureVariant")),
        ),
    ] {
        assert_reply(Err::<Nat, _>(wire), decode, TransferResult::Err(projected));
    }
}

#[test]
fn complete_notify_outcomes() {
    use top_up::decode_notify_top_up as decode;
    let large = "340282366920938463463374607431768211456";
    assert_reply(
        Ok::<_, NotifyErrorWire>(large.parse::<Nat>().unwrap()),
        decode,
        NotifyResult::Ok(large.parse().unwrap()),
    );
    for block_index in [None, Some(0), Some(u64::MAX)] {
        assert_reply(
            Err::<Nat, _>(NotifyErrorWire::Refunded {
                block_index,
                reason: "payment returned".into(),
            }),
            decode,
            NotifyResult::Err(NotifyError::Refunded(top_up::Refunded {
                block_index,
                reason: "payment returned".into(),
            })),
        );
    }
    for (wire, projected) in [
        (NotifyErrorWire::Processing, NotifyError::Processing),
        (
            NotifyErrorWire::TransactionTooOld(u64::MAX),
            NotifyError::TransactionTooOld(u64::MAX),
        ),
        (
            NotifyErrorWire::InvalidTransaction("wrong destination".into()),
            NotifyError::InvalidTransaction("wrong destination".into()),
        ),
        (
            NotifyErrorWire::Other {
                error_code: u64::MAX,
                error_message: "unknown failure".into(),
            },
            NotifyError::Other(top_up::NotifyOther {
                error_code: u64::MAX,
                error_message: "unknown failure".into(),
            }),
        ),
        (
            NotifyErrorWire::FutureVariant(vec![1, 2]),
            NotifyError::Unknown(candid::idl_hash("FutureVariant")),
        ),
    ] {
        assert_reply(Err::<Nat, _>(wire), decode, NotifyResult::Err(projected));
    }
}

#[test]
fn unknown_result_labels_require_complete_bounded_payloads() {
    let label = candid::idl_hash("Deferred");
    assert_reply(
        FutureResult::Deferred(vec![1, 2, 3]),
        top_up::decode_icrc1_transfer,
        TransferResult::Unknown(label),
    );
    assert_reply(
        FutureResult::Deferred(vec![1, 2, 3]),
        top_up::decode_notify_top_up,
        NotifyResult::Unknown(label),
    );
    let bytes = candid::encode_one(FutureResult::Deferred(vec![1, 2, 3])).unwrap();
    for limits in [
        Limits {
            skipping: 0,
            ..LIMITS
        },
        Limits {
            vector: 2,
            ..LIMITS
        },
        Limits { work: 0, ..LIMITS },
        Limits {
            allocation: 0,
            ..LIMITS
        },
        Limits {
            header: 0,
            ..LIMITS
        },
        Limits { types: 0, ..LIMITS },
    ] {
        assert_eq!(
            top_up::decode_icrc1_transfer(&bytes, limits),
            Err(Error::Limit)
        );
        assert_eq!(
            top_up::decode_notify_top_up(&bytes, limits),
            Err(Error::Limit)
        );
    }
    let mistyped = candid::encode_one(Ok::<_, NotifyErrorWire>("not cycles")).unwrap();
    assert_eq!(
        top_up::decode_icrc1_transfer(&mistyped, LIMITS),
        Err(Error::Type)
    );
    assert_eq!(
        top_up::decode_notify_top_up(&mistyped, LIMITS),
        Err(Error::Type)
    );
}

#[test]
fn frozen_wasm_replies_match_independent_candid_encoding() {
    let large: Nat = "340282366920938463463374607431768211456".parse().unwrap();
    let transfer = Err::<Nat, _>(TransferErrorWire::Duplicate {
        duplicate_of: large.clone(),
    });
    let credited = Ok::<_, NotifyErrorWire>(large);
    let refund = Err::<Nat, _>(NotifyErrorWire::Refunded {
        reason: "payment returned".into(),
        block_index: Some(u64::MAX),
    });
    assert_eq!(
        candid::encode_one(transfer).unwrap(),
        include_bytes!("../../engine-view-wasm/fixtures/top-up-transfer-duplicate.bin"),
    );
    assert_eq!(
        candid::encode_one(credited).unwrap(),
        include_bytes!("../../engine-view-wasm/fixtures/top-up-notify-credited.bin"),
    );
    assert_eq!(
        candid::encode_one(refund).unwrap(),
        include_bytes!("../../engine-view-wasm/fixtures/top-up-notify-refunded.bin"),
    );
}
