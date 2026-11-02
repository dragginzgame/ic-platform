use super::*;

fn assert_fields(actual: Account, expected: Account) {
    assert_eq!(actual.owner, expected.owner);
    // Account's logical equality deliberately cannot distinguish these fields.
    assert_eq!(actual.subaccount, expected.subaccount);
}

#[test]
fn postcard_contract_preserves_owner_and_prefixes_present_subaccount_bytes() {
    for owner in [
        Principal::from_slice(&[]),
        Principal::anonymous(),
        Principal::from_slice(&[255; 29]),
    ] {
        for subaccount in [
            None,
            Some([0; 32]),
            Some([255; 32]),
            Some(std::array::from_fn(|i| i as u8)),
        ] {
            let account = Account { owner, subaccount };
            let mut expected = vec![owner.as_slice().len() as u8];
            expected.extend_from_slice(owner.as_slice());
            match subaccount {
                None => expected.push(0),
                Some(bytes) => {
                    expected.extend_from_slice(&[1, 32]);
                    expected.extend_from_slice(&bytes);
                }
            }
            let encoded = postcard::to_allocvec(&account).unwrap();
            assert_eq!(encoded, expected);
            assert_fields(postcard::from_bytes(&encoded).unwrap(), account);
        }
    }
    for length in [31, 33] {
        let payload = vec![0; length];
        let encoded = postcard::to_allocvec(&(
            Principal::anonymous(),
            Some(serde_bytes::Bytes::new(&payload)),
        ))
        .unwrap();
        assert!(postcard::from_bytes::<Account>(&encoded).is_err());
    }
}

#[test]
fn json_keeps_raw_optional_fields_and_omitted_field_admission() {
    let owner = Principal::anonymous();
    for subaccount in [None, Some([0; 32]), Some([255; 32])] {
        let account = Account { owner, subaccount };
        let value = serde_json::to_value(account).unwrap();
        assert_eq!(value["owner"], owner.to_text());
        assert_eq!(
            value["subaccount"],
            serde_json::to_value(subaccount).unwrap()
        );
        assert_fields(serde_json::from_value(value).unwrap(), account);
    }
    assert_fields(
        serde_json::from_value(serde_json::json!({"owner": owner.to_text()})).unwrap(),
        Account::from(owner),
    );
    for length in [31, 33] {
        assert!(
            serde_json::from_value::<Account>(
                serde_json::json!({"owner": owner.to_text(), "subaccount": vec![0; length]})
            )
            .is_err()
        );
    }
}

#[test]
fn cbor_uses_bytes_and_retains_none_versus_explicit_zero() {
    use serde_cbor::Value;
    for subaccount in [None, Some([0; 32]), Some([255; 32])] {
        let account = Account {
            owner: Principal::anonymous(),
            subaccount,
        };
        let encoded = serde_cbor::to_vec(&account).unwrap();
        let Value::Map(fields) = serde_cbor::from_slice(&encoded).unwrap() else {
            panic!("Account must remain a map")
        };
        assert_eq!(fields[&Value::Text("owner".into())], Value::Bytes(vec![4]));
        assert_eq!(
            fields[&Value::Text("subaccount".into())],
            subaccount.map_or(Value::Null, |bytes| Value::Bytes(bytes.to_vec()))
        );
        assert_fields(serde_cbor::from_slice(&encoded).unwrap(), account);
    }
}

#[test]
fn old_unframed_bytes_can_decode_as_different_values() {
    // Old Account Some([0x20, 0 x31]), followed by blob [0x02, 0xaa, 0xbb].
    // A package-version bump cannot fence this message contract change.
    let mut old = vec![1, 4, 1, 32];
    old.extend_from_slice(&[0; 31]);
    old.extend_from_slice(&[3, 2, 0xaa, 0xbb]);
    let (account, blob): (Account, serde_bytes::ByteBuf) = postcard::from_bytes(&old).unwrap();
    let mut reinterpreted = [0; 32];
    reinterpreted[31] = 3;
    assert_eq!(account.subaccount, Some(reinterpreted));
    assert_eq!(blob.as_ref(), &[0xaa, 0xbb]);
}
