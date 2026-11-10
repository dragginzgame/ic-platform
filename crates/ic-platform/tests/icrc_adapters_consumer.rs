//! Public ICRC adapter observations derived from the reviewed contract.

use ic_platform::candid_codec::decoder::{Limits, decode_one};
use ic_platform::candid_codec::references::Func;
use ic_platform::candid_codec::{Encode, Error, Type, Writer, encode_one};
use ic_platform::icrc_ledger_types::{Account, GetArchivesArgs, QueryArchiveFn};
use ic_platform::principal::Principal;
use serde::{Deserialize, Deserializer};
use std::collections::hash_map::DefaultHasher;
use std::error::Error as _;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

fn digest(account: &Account) -> u64 {
    let mut hasher = DefaultHasher::new();
    account.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn effective_zero_identity_preserves_serialized_field_distinctions() {
    let owner = Principal::anonymous();
    let absent = Account::from(owner);
    let explicit = Account {
        owner,
        subaccount: Some([0; 32]),
    };
    assert_eq!(absent, explicit);
    assert_eq!(absent.cmp(&explicit), std::cmp::Ordering::Equal);
    assert_eq!(digest(&absent), digest(&explicit));
    assert_eq!(absent.to_string(), owner.to_string());
    assert_eq!(explicit.to_string(), owner.to_string());
    assert_eq!(absent.effective_subaccount(), &[0; 32]);
    assert!(std::ptr::eq(
        explicit.effective_subaccount(),
        explicit.subaccount.as_ref().unwrap()
    ));

    for account in [absent, explicit] {
        let canonical: ic_platform::Account = account.into();
        assert_eq!(canonical.to_string(), account.to_string());
        let restored: Account = canonical.into();
        assert_eq!(restored.owner, owner);
        assert_eq!(restored.subaccount, account.subaccount);
        let json = serde_json::to_value(account).unwrap();
        assert_eq!(json["owner"], serde_json::to_value(owner).unwrap());
        assert_eq!(json.as_object().unwrap().len(), 2);
        let restored: Account = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(restored.subaccount, account.subaccount);
        match account.subaccount {
            None => assert!(json["subaccount"].is_null()),
            Some(_) => assert_eq!(json["subaccount"], serde_json::to_value([0u8; 32]).unwrap()),
        }
    }
    let cbor = serde_cbor::value::to_value(explicit).unwrap();
    let serde_cbor::Value::Map(fields) = cbor else {
        panic!("Account must serialize as a struct map")
    };
    assert!(
        matches!(fields.get(&serde_cbor::Value::Text("subaccount".to_owned())), Some(serde_cbor::Value::Array(values)) if values.len() == 32)
    );
    let missing_optional: Account =
        serde_json::from_value(serde_json::json!({"owner": owner, "unknown": true})).unwrap();
    assert_eq!(missing_optional.subaccount, None);
    assert!(serde_json::from_str::<Account>("{}").is_err());
    let owner_json = serde_json::to_string(&owner).unwrap();
    assert!(
        serde_json::from_str::<Account>(&format!(
            "{{\"owner\":{owner_json},\"owner\":{owner_json}}}"
        ))
        .is_err()
    );
    assert!(
        serde_json::from_str::<Account>(&format!(
            "{{\"owner\":{owner_json},\"subaccount\":null,\"subaccount\":null}}"
        ))
        .is_err()
    );
    assert!(format!("{absent:?}").contains("subaccount: None"));
    assert!(format!("{explicit:?}").contains("subaccount: Some"));
}

#[test]
fn account_ordering_and_text_roundtrips_follow_canonical_fields() {
    let short = Principal::try_from_slice(&[255]).unwrap();
    let longer = Principal::try_from_slice(&[0, 0]).unwrap();
    assert!(Account::from(short) < Account::from(longer));
    let mut one = [0; 32];
    one[31] = 1;
    let mut two = one;
    two[31] = 2;
    let one = Account {
        owner: short,
        subaccount: Some(one),
    };
    let two = Account {
        owner: short,
        subaccount: Some(two),
    };
    assert!(Account::from(short) < one);
    assert!(one < two);
    for owner in [
        Principal::management_canister(),
        Principal::anonymous(),
        short,
        longer,
    ] {
        for subaccount in [
            None,
            Some([0; 32]),
            Some([255; 32]),
            one.subaccount,
            two.subaccount,
        ] {
            let account = Account { owner, subaccount };
            let text = account.to_string();
            let restored: Account = text.parse().unwrap();
            assert_eq!(restored, account);
            if account.effective_subaccount() != &[0; 32] {
                assert_eq!(restored.subaccount, account.subaccount);
                let (prefix, suffix) = text.split_once('.').unwrap();
                let uppercase: Account = format!("{prefix}.{}", suffix.to_ascii_uppercase())
                    .parse()
                    .unwrap();
                assert_eq!(uppercase.subaccount, account.subaccount);
            }
        }
    }
}

#[test]
fn deterministic_account_text_observations() {
    let owner = Principal::anonymous();
    let mut one = [0; 32];
    one[31] = 1;
    let valid = Account {
        owner,
        subaccount: Some(one),
    }
    .to_string();
    let (prefix, _) = valid.split_once('.').unwrap();
    let owner_text = owner.to_string();
    let mut cases = vec![
        ("valid-default".to_owned(), owner_text.clone()),
        (
            "valid-management".to_owned(),
            Principal::management_canister().to_string(),
        ),
        ("valid-one".to_owned(), valid.clone()),
        ("empty-input".to_owned(), String::new()),
        ("owner-leading-space".to_owned(), format!(" {owner_text}")),
        ("owner-trailing-space".to_owned(), format!("{owner_text} ")),
        ("owner-uppercase".to_owned(), owner_text.to_uppercase()),
        ("oversized-principal".to_owned(), "x".repeat(4096)),
        ("missing-checksum".to_owned(), format!("{owner_text}.1")),
        (
            "short-checksum".to_owned(),
            format!("{owner_text}-abcdef.1"),
        ),
        (
            "long-checksum".to_owned(),
            format!("{owner_text}-abcdefgh.1"),
        ),
        (
            "checksum-seven-utf8-bytes".to_owned(),
            format!("{owner_text}-éééa.1"),
        ),
        (
            "checksum-eight-utf8-bytes".to_owned(),
            format!("{owner_text}-éééé.1"),
        ),
        (
            "missing-checksum-before-zero".to_owned(),
            "bad-abcdef.0g".to_owned(),
        ),
        ("zero-before-owner".to_owned(), "bad-abcdefg.0g".to_owned()),
        ("owner-before-hex".to_owned(), "bad-abcdefg.g".to_owned()),
    ];
    let (checksum_owner, checksum) = prefix.rsplit_once('-').unwrap();
    cases.push((
        "checksum-uppercase".to_owned(),
        format!("{checksum_owner}-{}.1", checksum.to_uppercase()),
    ));
    for (label, suffix) in [
        ("empty-suffix", ""),
        ("leading-zero", "01"),
        ("hex-prefix", "0x1"),
        ("ascii-invalid", "g"),
        ("ascii-invalid-upper", "G"),
        ("plus", "+1"),
        ("minus", "-1"),
        ("space", " 1"),
        ("newline", "1\n"),
        ("first-period", "1.2"),
        ("unicode-two-byte", "é"),
        ("unicode-three-byte", "☃"),
        ("unicode-four-byte", "😀"),
        ("two-unicode-scalars", "éé"),
        ("ascii-then-two-byte", "aé"),
        ("ascii-then-three-byte", "a☃"),
        ("wrong-checksum-valid-hex", "2"),
    ] {
        cases.push((label.to_owned(), format!("{prefix}.{suffix}")));
    }
    for (label, suffix) in [
        ("64-valid-hex", "a".repeat(64)),
        ("65-valid-hex", "a".repeat(65)),
        ("66-valid-hex", "a".repeat(66)),
        ("64-invalid-last", format!("{}g", "a".repeat(63))),
        ("65-invalid-last", format!("{}g", "a".repeat(64))),
        ("66-invalid-before-length", format!("{}gg", "a".repeat(64))),
        ("64-scalars-65-bytes", format!("é{}", "a".repeat(63))),
        ("64-scalars-66-bytes", format!("☃{}", "a".repeat(63))),
        ("huge-even-valid", "a".repeat(65536)),
        ("huge-odd-valid", "a".repeat(65537)),
        ("huge-even-invalid-first", format!("g{}", "a".repeat(4095))),
        ("huge-even-invalid-last", format!("{}g", "a".repeat(4095))),
        ("huge-unicode-offset", format!("{}☃", "a".repeat(4095))),
        ("huge-leading-zero", format!("0{}", "g".repeat(4095))),
    ] {
        cases.push((label.to_owned(), format!("{prefix}.{suffix}")));
    }
    let observations: Vec<_> = cases
        .into_iter()
        .map(|(label, text)| {
            let result = match text.parse::<Account>() {
                Ok(account) => format!("ok:{account};debug={account:?}"),
                Err(error) => {
                    assert!(error.source().is_none());
                    format!("error:{error};debug={error:?}")
                }
            };
            (label, text.len(), result)
        })
        .collect();
    println!(
        "ICRC_ADAPTER_OBSERVATIONS={}",
        serde_json::to_string(&observations).unwrap()
    );
    // Captured through these public inputs before the adapter replacement.
    let baseline: Vec<(String, usize, String)> =
        serde_json::from_str(include_str!("fixtures/icrc-adapter-observations.json")).unwrap();
    assert_eq!(observations, baseline);
}

struct NoTraits;

#[derive(PartialEq, Eq)]
struct EqualityOnly;

#[test]
fn callback_construction_clone_order_and_moves_need_only_documented_bounds() {
    let short = Principal::try_from_slice(&[255]).unwrap();
    let long = Principal::try_from_slice(&[0, 0]).unwrap();
    for owner in [
        Principal::anonymous(),
        Principal::management_canister(),
        short,
    ] {
        for method in ["", "query", "任意.λ\0method"] {
            let method = method.to_owned();
            let allocation = method.as_ptr();
            let callback = QueryArchiveFn::<NoTraits, NoTraits>::new(owner, method);
            assert_eq!(callback.method.as_ptr(), allocation);
            let cloned = callback.clone();
            assert_eq!(cloned.canister_id, owner);
            assert_eq!(cloned.method, callback.method);
            let function: Func = callback.into();
            assert_eq!(function.principal.as_slice(), owner.as_slice());
            assert_eq!(function.method.as_ptr(), allocation);
            let result: Result<QueryArchiveFn<NoTraits, NoTraits>, String> = function.try_into();
            let restored = result.unwrap();
            assert_eq!(restored.canister_id, owner);
            assert_eq!(restored.method, cloned.method);
            assert_eq!(restored.method.as_ptr(), allocation);
        }
    }
    let first = QueryArchiveFn::<EqualityOnly, EqualityOnly>::new(short, "z");
    let second = QueryArchiveFn::<EqualityOnly, EqualityOnly>::new(long, "a");
    assert_eq!(first.cmp(&second), std::cmp::Ordering::Less);
    let third = QueryArchiveFn::<EqualityOnly, EqualityOnly>::new(short, "zz");
    assert_eq!(first.partial_cmp(&third), Some(std::cmp::Ordering::Less));
}

struct EncodeOnly;

impl Encode for EncodeOnly {
    const TYPE: &'static Type = <u8 as Encode>::TYPE;

    fn write(&self, writer: &mut Writer) -> Result<(), Error> {
        0u8.write(writer)
    }
}

fn limits(length: usize) -> Limits {
    Limits {
        bytes: length,
        header: 4096,
        types: 256,
        work: 100_000,
        allocation: 100_000,
    }
}

#[test]
fn owned_callback_codec_requires_encode_but_not_decode_on_parameters() {
    let callback =
        QueryArchiveFn::<EncodeOnly, EncodeOnly>::new(Principal::anonymous(), "λ\0archive");
    let bytes = encode_one(&callback, 4096).unwrap();
    let decoded: QueryArchiveFn<EncodeOnly, EncodeOnly> =
        decode_one(&bytes, limits(bytes.len())).unwrap();
    assert_eq!(decoded.canister_id, callback.canister_id);
    assert_eq!(decoded.method, callback.method);
    let Type::Function {
        args,
        results,
        annotations,
    } = <QueryArchiveFn<EncodeOnly, EncodeOnly> as Encode>::TYPE
    else {
        panic!("callback must declare a function type")
    };
    assert_eq!(args.len(), 1);
    assert_eq!(results.len(), 1);
    assert_eq!(*annotations, [1]);
    assert!(decode_one::<QueryArchiveFn<u16, u8>>(&bytes, limits(bytes.len())).is_err());
    assert!(decode_one::<QueryArchiveFn<u8, u16>>(&bytes, limits(bytes.len())).is_err());
    assert!(decode_one::<QueryArchiveFn<u8, u8>>(&bytes, limits(bytes.len() - 1)).is_err());
    assert!(encode_one(&callback, 1).is_err());
}

struct FunctionBytes(Vec<u8>);

impl<'de> Deserializer<'de> for FunctionBytes {
    type Error = serde::de::value::Error;

    fn deserialize_any<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_byte_buf(self.0)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum
        identifier ignored_any
    }
}

#[test]
fn callback_serde_preserves_struct_output_and_function_buffer_input() {
    for owner in [Principal::anonymous(), Principal::management_canister()] {
        for method in ["", "λ.🦀"] {
            let callback = QueryArchiveFn::<u8, u8> {
                canister_id: owner,
                method: method.to_owned(),
                _marker: PhantomData,
            };
            let json = serde_json::to_value(&callback).unwrap();
            assert_eq!(json.as_object().unwrap().len(), 3);
            assert_eq!(json["canister_id"], serde_json::to_value(owner).unwrap());
            assert_eq!(json["method"], method);
            assert!(json["_marker"].is_null());
            assert!(serde_json::from_value::<QueryArchiveFn<u8, u8>>(json).is_err());
            let mut bytes = vec![5, u8::try_from(method.len()).unwrap()];
            bytes.extend_from_slice(method.as_bytes());
            bytes.extend_from_slice(owner.as_slice());
            let restored = QueryArchiveFn::<u8, u8>::deserialize(FunctionBytes(bytes)).unwrap();
            assert_eq!(restored, callback);
        }
    }
    for bytes in [vec![], vec![4, 0], vec![5, 3, b'a'], vec![5, 1, 255]] {
        assert!(QueryArchiveFn::<u8, u8>::deserialize(FunctionBytes(bytes)).is_err());
    }
    let args = GetArchivesArgs {
        from: Some(Principal::anonymous()),
    };
    let restored: GetArchivesArgs =
        serde_json::from_value(serde_json::to_value(&args).unwrap()).unwrap();
    assert_eq!(restored, args);
}
