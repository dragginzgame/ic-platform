//! Independent selected Caffeine wire qualification.
#![cfg(feature = "caffeine")]
#![forbid(unsafe_code)]
use candid::{CandidType, Decode, Encode, Int, Nat, Principal};
use ic_platform_protocol::{
    caffeine as c,
    candid::{Error, Limits},
};
use serde::Deserialize;
fn limits() -> Limits {
    Limits {
        bytes: 65_536,
        header: 8192,
        types: 128,
        work: 262_144,
        allocation: 262_144,
        vector: 4096,
        skipping: 262_144,
    }
}
#[derive(CandidType, Deserialize)]
struct Certificate {
    method: String,
    blob_hash: String,
}
#[derive(CandidType, Deserialize)]
struct TopUp {
    target_balance: Option<Nat>,
    account: Option<Principal>,
}
#[derive(CandidType, Deserialize)]
struct AccountRequest {
    account: Principal,
}
#[derive(CandidType, Deserialize)]
struct Gauges {
    chunk_size_bytes: u64,
    chunk_count: u64,
    blob_count: u64,
    blob_size_bytes: u64,
}
#[derive(CandidType, Deserialize)]
struct Account {
    account_owner_id: Principal,
    cycle_balance: Int,
    storage_usage: Gauges,
    last_charge_ts_ns: u64,
}
#[derive(CandidType, Deserialize)]
enum ResultValue {
    Ok(Account),
    Err(ProviderError),
}
#[derive(CandidType, Deserialize)]
enum ProviderError {
    AccountNotFound,
    NotAuthorized(String),
    Unknown(bool),
}
fn account(balance: &str) -> ResultValue {
    ResultValue::Ok(Account {
        account_owner_id: Principal::from_slice(&[1]),
        cycle_balance: balance.parse().unwrap(),
        storage_usage: Gauges {
            chunk_size_bytes: 200,
            chunk_count: 3,
            blob_count: 2,
            blob_size_bytes: 100,
        },
        last_charge_ts_ns: 42,
    })
}
#[test]
fn independent_arguments_and_exact_output_admission() {
    let root = format!("sha256:{}", "01".repeat(32));
    let encoded = c::encode_root(&root, 1024).unwrap();
    assert_eq!(Decode!(&encoded, String).unwrap(), root);
    assert_eq!(
        c::decode_root(&Encode!(&root).unwrap(), limits()).unwrap(),
        root
    );
    assert_eq!(c::encode_root(&root, encoded.len()).unwrap(), encoded);
    assert!(c::encode_root(&root, encoded.len() - 1).is_err());
    let roots = vec![vec![1; 32], vec![255; 32]];
    let bytes = c::encode_roots(&roots, 4096).unwrap();
    assert_eq!(Decode!(&bytes, Vec<Vec<u8>>).unwrap(), roots);
    assert_eq!(
        c::decode_roots(&Encode!(&roots).unwrap(), limits()).unwrap(),
        roots
    );
    assert!(c::decode_roots(&Encode!(&vec![root.clone()]).unwrap(), limits()).is_err());
    for values in [vec![], vec![true, false]] {
        assert_eq!(
            Decode!(&c::encode_liveness(&values, 4096).unwrap(), Vec<bool>).unwrap(),
            values
        );
        assert_eq!(
            c::decode_liveness(&Encode!(&values).unwrap(), limits()).unwrap(),
            values
        );
    }
    let cert = Certificate {
        method: "upload".into(),
        blob_hash: root.clone(),
    };
    let got = c::decode_certificate(&Encode!(&cert).unwrap(), limits()).unwrap();
    assert_eq!(got.method, "upload");
    assert_eq!(got.blob_hash, root);
    let independent = Decode!(&c::encode_certificate(&got, 1024).unwrap(), Certificate).unwrap();
    assert_eq!(independent.blob_hash, root);
    assert!(c::encode_certificate(&got, 10).is_err());
    assert_eq!(c::encode_empty(6).unwrap(), b"DIDL\0\0");
    c::decode_empty(&Encode!().unwrap(), limits()).unwrap();
    assert!(c::encode_empty(5).is_err());
    assert!(c::decode_empty(&Encode!(&()).unwrap(), limits()).is_err());
    for data in [Encode!().unwrap(), Encode!(&root, &root).unwrap()] {
        assert_eq!(c::decode_root(&data, limits()).unwrap_err(), Error::Arity);
    }
    let owner = candidate_types::Principal::from_slice(&[1]);
    let request = c::AccountRequest { account: owner }.encode(1024).unwrap();
    assert_eq!(
        Decode!(&request, AccountRequest)
            .unwrap()
            .account
            .as_slice(),
        owner.as_slice()
    );
    for balance in [
        None,
        Some("0"),
        Some("128"),
        Some("340282366920938463463374607431768211456000000000000000000000"),
    ] {
        for account in [None, Some(owner)] {
            let args = c::TopUpArgs {
                target_balance: balance.map(|b| b.parse().unwrap()),
                account,
            };
            let bytes = args.encode(1024).unwrap();
            let got = Decode!(&bytes, TopUp).unwrap();
            assert_eq!(
                got.target_balance.map(|b| b.0.to_string()),
                balance.map(str::to_owned)
            );
            assert_eq!(
                got.account.map(|p| p.as_slice().to_vec()),
                account.map(|p| p.as_slice().to_vec())
            );
            assert_eq!(args.encode(bytes.len()).unwrap(), bytes);
            assert!(args.encode(bytes.len() - 1).is_err());
        }
    }
}
#[test]
fn arbitrary_signed_balances_and_known_errors() {
    for number in [
        "0",
        "-1",
        "63",
        "64",
        "127",
        "128",
        "-64",
        "-65",
        "-128",
        "-129",
        "340282366920938463463374607431768211456000000000000000000000",
        "-340282366920938463463374607431768211456000000000000000000000",
    ] {
        let bytes = Encode!(&account(number)).unwrap();
        let c::AccountResult::Ok(decoded) = c::decode_account(&bytes, limits()).unwrap() else {
            panic!()
        };
        assert_eq!(decoded.cycle_balance.to_string(), number);
        assert_eq!(decoded.account_owner_id.as_slice(), &[1]);
        assert_eq!(decoded.storage_usage.blob_count, 2);
        for end in 0..bytes.len() {
            assert!(c::decode_account(&bytes[..end], limits()).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert_eq!(
            c::decode_account(&trailing, limits()).unwrap_err(),
            Error::Trailing
        );
        for restricted in [
            Limits {
                bytes: bytes.len() - 1,
                ..limits()
            },
            Limits {
                header: 4,
                ..limits()
            },
            Limits {
                types: 0,
                ..limits()
            },
            Limits {
                work: 0,
                ..limits()
            },
            Limits {
                allocation: 0,
                ..limits()
            },
        ] {
            assert_eq!(
                c::decode_account(&bytes, restricted).unwrap_err(),
                Error::Limit
            );
        }
    }
    let bytes = Encode!(&ResultValue::Err(ProviderError::NotAuthorized(
        "denied".into()
    )))
    .unwrap();
    assert!(matches!(
        c::decode_account(&bytes, limits()).unwrap(),
        c::AccountResult::Err(c::AccountError::NotAuthorized(_))
    ));
    assert_eq!(
        c::decode_account(
            &bytes,
            Limits {
                skipping: 0,
                ..limits()
            }
        )
        .unwrap_err(),
        Error::Limit
    );
    let mut invalid = bytes;
    *invalid.last_mut().unwrap() = 255;
    assert!(c::decode_account(&invalid, limits()).is_err());
    assert!(matches!(
        c::decode_account(
            &Encode!(&ResultValue::Err(ProviderError::AccountNotFound)).unwrap(),
            limits()
        )
        .unwrap(),
        c::AccountResult::Err(c::AccountError::AccountNotFound)
    ));
    assert!(
        c::decode_account(
            &Encode!(&ResultValue::Err(ProviderError::Unknown(true))).unwrap(),
            limits()
        )
        .is_err()
    );
}
#[test]
fn callback_bounds_and_literals() {
    let bytes = c::encode_roots(&vec![vec![7; 32]; 2], 4096).unwrap();
    assert_eq!(
        c::decode_roots(
            &bytes,
            Limits {
                bytes: bytes.len(),
                ..limits()
            }
        )
        .unwrap()
        .len(),
        2
    );
    assert_eq!(
        c::decode_roots(
            &bytes,
            Limits {
                bytes: bytes.len() - 1,
                ..limits()
            }
        )
        .unwrap_err(),
        Error::Limit
    );
    assert_eq!(
        c::decode_roots(
            &bytes,
            Limits {
                vector: 1,
                ..limits()
            }
        )
        .unwrap_err(),
        Error::Limit
    );
    for end in 0..bytes.len() {
        assert!(c::decode_roots(&bytes[..end], limits()).is_err());
    }
    // Independent literal vec bool, including an invalid selected boolean.
    assert_eq!(
        c::decode_liveness(b"DIDL\x01\x6d\x7e\x01\x00\x02\x01\x00", limits()).unwrap(),
        [true, false]
    );
    assert_eq!(
        c::decode_liveness(b"DIDL\x01\x6d\x7e\x01\x00\x01\x02", limits()).unwrap_err(),
        Error::Malformed
    );
}

#[test]
fn frozen_caffeine_wasm_corpus() {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../engine-view-wasm/fixtures");
    for (name, bytes) in [
        (
            "caffeine-top-up.bin",
            Encode!(&Some(TopUp {
                target_balance: Some("340282366920938463463374607431768211456".parse().unwrap()),
                account: Some(Principal::from_slice(&[42; 29]))
            }))
            .unwrap(),
        ),
        (
            "caffeine-top-up-success.bin",
            Encode!(&TopUpReply::Ok(vec![true, false])).unwrap(),
        ),
        (
            "caffeine-account.bin",
            Encode!(&account(
                "-340282366920938463463374607431768211456000000000000000000000"
            ))
            .unwrap(),
        ),
        (
            "caffeine-roots.bin",
            Encode!(&vec![vec![1_u8; 32], vec![255_u8; 32]]).unwrap(),
        ),
        (
            "caffeine-certificate.bin",
            Encode!(&Certificate {
                method: "upload".into(),
                blob_hash: format!("sha256:{}", "01".repeat(32))
            })
            .unwrap(),
        ),
    ] {
        let path = directory.join(name);
        if std::env::var_os("REGENERATE_CAFFEINE_CORPUS").is_some() {
            std::fs::write(&path, &bytes).unwrap();
        }
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn retained_enrollment_outer_option_bytes() {
    let args = c::TopUpArgs {
        target_balance: Some(c::Nat::from(1_000_000_000)),
        account: Some(candidate_types::Principal::from_slice(&[1])),
    };
    let bytes = c::encode_top_up(Some(&args), 1024).unwrap();
    let independent = Decode!(&bytes, Option<TopUp>).unwrap().unwrap();
    assert_eq!(
        independent.target_balance,
        Some(Nat::from(1_000_000_000_u64))
    );
    assert_eq!(independent.account, Some(Principal::from_slice(&[1])));
    // Literal root-first type table and payload of the retained Blob argument:
    // opt record {target_balance: opt nat; account: opt principal}.
    let literal=b"DIDL\x04\x6e\x01\x6c\x02\xee\xc5\x80\xaa\x09\x02\xad\xf9\xe7\x8a\x0a\x03\x6e\x7d\x6e\x68\x01\x00\x01\x01\x80\x94\xeb\xdc\x03\x01\x01\x01\x01";
    assert_eq!(bytes, literal);
    assert_eq!(c::encode_top_up(Some(&args), bytes.len()).unwrap(), bytes);
    assert!(c::encode_top_up(Some(&args), bytes.len() - 1).is_err());
    assert!(
        Decode!(&c::encode_top_up(None, 1024).unwrap(), Option<TopUp>)
            .unwrap()
            .is_none()
    );
}

#[test]
fn blob_independent_cashier_frames_and_owner_policy() {
    let hex = |text: &str| {
        let text = text.trim();
        (0..text.len())
            .step_by(2)
            .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
            .collect::<Vec<_>>()
    };
    let account = hex(include_str!("../fixtures/caffeine/account.hex"));
    let c::AccountResult::Ok(value) = c::decode_account(&account, limits()).unwrap() else {
        panic!()
    };
    let owner = candidate_types::Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    assert_eq!(value.account_owner_id, owner);
    assert_ne!(
        value.account_owner_id,
        candidate_types::Principal::from_slice(&[1, 1])
    );
    assert_eq!(value.cycle_balance.to_string(), "-123");
    assert_eq!(
        (
            value.storage_usage.blob_count,
            value.storage_usage.blob_size_bytes,
            value.storage_usage.chunk_count,
            value.storage_usage.chunk_size_bytes
        ),
        (2, 100, 3, 200)
    );
    assert_eq!(value.last_charge_ts_ns, 42);
    assert!(matches!(
        c::decode_account(
            &hex(include_str!("../fixtures/caffeine/not-found.hex")),
            limits()
        )
        .unwrap(),
        c::AccountResult::Err(c::AccountError::AccountNotFound)
    ));
    assert!(matches!(
        c::decode_account(
            &hex(include_str!("../fixtures/caffeine/denied.hex")),
            limits()
        )
        .unwrap(),
        c::AccountResult::Err(c::AccountError::NotAuthorized(_))
    ));
    assert!(
        c::decode_account(
            &hex(include_str!("../fixtures/caffeine/missing-storage.hex")),
            limits()
        )
        .is_err()
    );
    let wrong = c::CertificateReply {
        method: "download".into(),
        blob_hash: "sha256:wrong".into(),
    };
    let observed =
        c::decode_certificate(&c::encode_certificate(&wrong, 1024).unwrap(), limits()).unwrap();
    // Successful plain decoding must not rewrite fields into an admitted upload.
    assert_ne!(observed.method, "upload");
    assert_ne!(observed.blob_hash, format!("sha256:{}", "01".repeat(32)));
}

fn top_up_limits() -> Limits {
    Limits {
        bytes: 8192,
        header: 2048,
        types: 32,
        work: 65536,
        allocation: 65536,
        vector: 8192,
        skipping: 65536,
    }
}
fn admission_boundaries<T: std::fmt::Debug>(
    bytes: &[u8],
    limits: Limits,
    decode: fn(&[u8], Limits) -> Result<T, Error>,
) {
    decode(bytes, limits).unwrap();
    for n in 0..bytes.len() {
        assert!(decode(&bytes[..n], limits).is_err(), "truncation at {n}");
    }
    let mut trailing = bytes.to_vec();
    trailing.push(0);
    assert!(decode(&trailing, limits).is_err());
    for dimension in 0..7 {
        let set = |n| {
            let mut l = limits;
            match dimension {
                0 => l.bytes = n,
                1 => l.header = n,
                2 => l.types = n,
                3 => l.work = n,
                4 => l.allocation = n,
                5 => l.vector = n,
                _ => l.skipping = n,
            };
            l
        };
        let mut low = 0;
        let mut high = 65536;
        while low < high {
            let mid = low + (high - low) / 2;
            if decode(bytes, set(mid)).is_ok() {
                high = mid
            } else {
                low = mid + 1
            }
        }
        decode(bytes, set(low)).unwrap();
        if low > 0 {
            assert!(
                decode(bytes, set(low - 1)).is_err(),
                "budget dimension {dimension}"
            );
        }
    }
}
#[test]
fn top_up_arguments_preserve_outer_and_field_options_and_arbitrary_naturals() {
    let limits = Limits {
        bytes: 1024,
        header: 512,
        types: 16,
        work: 4096,
        allocation: 4096,
        vector: 1024,
        skipping: 4096,
    };
    let owner = Principal::from_slice(&[42; 29]);
    for target_balance in [
        None,
        Some(Nat::from(0u8)),
        Some("340282366920938463463374607431768211456".parse().unwrap()),
    ] {
        for account in [None, Some(owner)] {
            let value = Some(TopUp {
                target_balance: target_balance.clone(),
                account,
            });
            let bytes = Encode!(&value).unwrap();
            let got = c::decode_top_up(&bytes, limits).unwrap().unwrap();
            assert_eq!(
                got.target_balance.map(|n| n.0.to_string()),
                target_balance.as_ref().map(|n| n.0.to_string())
            );
            assert_eq!(
                got.account.map(|p| p.as_slice().to_vec()),
                account.map(|p| p.as_slice().to_vec())
            );
            admission_boundaries(&bytes, limits, c::decode_top_up);
        }
    }
    assert!(
        c::decode_top_up(
            &Encode!(&TopUp {
                target_balance: None,
                account: None
            })
            .unwrap(),
            limits
        )
        .is_err()
    );
    assert!(
        c::decode_top_up(&Encode!(&None::<TopUp>).unwrap(), limits)
            .unwrap()
            .is_none()
    );
    assert!(c::decode_top_up(&Encode!().unwrap(), limits).is_err());
    assert!(c::decode_top_up(&Encode!(&None::<TopUp>, &None::<TopUp>).unwrap(), limits).is_err());
}
#[derive(CandidType)]
enum TopUpFailure {
    NotAuthorized(Principal),
    AccountBalanceOverflow,
    InternalError(String),
    TopUpWithoutCycles,
}
#[derive(CandidType)]
enum TopUpReply<T: CandidType> {
    Ok(T),
    Err(TopUpFailure),
}
#[test]
fn top_up_outcome_validates_opaque_success_and_typed_errors() {
    let limits = top_up_limits();
    for payload in [vec![], vec![true, false, true]] {
        let bytes = Encode!(&TopUpReply::Ok(payload)).unwrap();
        assert!(matches!(
            c::decode_top_up_result(&bytes, limits).unwrap(),
            c::TopUpResult::Ok(_)
        ));
        admission_boundaries(&bytes, limits, c::decode_top_up_result);
    }
    // A nested record is also valid opaque success; no schema interpretation escapes.
    let bytes = Encode!(&TopUpReply::Ok(TopUp {
        target_balance: Some(Nat::from(1u8)),
        account: Some(Principal::anonymous())
    }))
    .unwrap();
    admission_boundaries(&bytes, limits, c::decode_top_up_result);
    let owner = Principal::from_slice(&[43; 29]);
    for error in [
        TopUpFailure::NotAuthorized(owner),
        TopUpFailure::AccountBalanceOverflow,
        TopUpFailure::InternalError("exact diagnostic".into()),
        TopUpFailure::TopUpWithoutCycles,
    ] {
        let bytes = Encode!(&TopUpReply::<bool>::Err(error)).unwrap();
        let got = c::decode_top_up_result(&bytes, limits).unwrap();
        match got {
            c::TopUpResult::Err(c::TopUpError::NotAuthorized(p)) => {
                assert_eq!(p.as_slice(), owner.as_slice())
            }
            c::TopUpResult::Err(c::TopUpError::InternalError(text)) => {
                assert_eq!(text, "exact diagnostic")
            }
            c::TopUpResult::Err(
                c::TopUpError::AccountBalanceOverflow | c::TopUpError::TopUpWithoutCycles,
            ) => {}
            c::TopUpResult::Ok(_) => panic!("error changed into success"),
        }
        admission_boundaries(&bytes, limits, c::decode_top_up_result);
    }
    let mut malformed = Encode!(&TopUpReply::Ok(true)).unwrap();
    *malformed.last_mut().unwrap() = 2; // Invalid bool in the discarded success.
    assert!(c::decode_top_up_result(&malformed, limits).is_err());
    assert!(
        c::decode_top_up_result(
            &Encode!(&ResultValue::Err(ProviderError::NotAuthorized(
                "wrong type".into()
            )))
            .unwrap(),
            limits
        )
        .is_err()
    );
    assert!(
        c::decode_top_up_result(
            &Encode!(&ResultValue::Err(ProviderError::Unknown(true))).unwrap(),
            limits
        )
        .is_err()
    );
    assert!(c::decode_top_up_result(&Encode!().unwrap(), limits).is_err());
    assert!(
        c::decode_top_up_result(&Encode!(&TopUpReply::Ok(true), &false).unwrap(), limits).is_err()
    );
    let literal = include_str!("fixtures/top-up-without-cycles.hex").trim();
    let bytes = (0..literal.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&literal[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    assert!(matches!(
        c::decode_top_up_result(&bytes, limits).unwrap(),
        c::TopUpResult::Err(c::TopUpError::TopUpWithoutCycles)
    ));
    admission_boundaries(&bytes, limits, c::decode_top_up_result);
}
