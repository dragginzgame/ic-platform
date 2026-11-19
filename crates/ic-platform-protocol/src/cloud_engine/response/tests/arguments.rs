use super::hex;
use crate::{
    candid::Nat,
    cloud_engine::{
        EncodeError, ListAvailableNodesPageArgs, encode_list_available_nodes_page_args as encode,
    },
};
use ic_platform_types::Principal;

// One record and three option table entries, labels independently hashed from
// limit/start/rejoining. These fixtures never use the production declaration.
const HEADER: &str = "44 49 44 4c 04 6c 03 bb be 84 a8 07 01 e2 e8 ad a0 08 02 e5 df 81 c0 0d 03 6e 7d 6e 68 6e 71 01 00";

fn check(args: &ListAvailableNodesPageArgs, payload: &[u8]) {
    let mut expected = hex(HEADER);
    expected.extend_from_slice(payload);
    assert_eq!(encode(args, expected.len()).unwrap(), expected);
    assert_eq!(
        encode(args, expected.len() - 1),
        Err(EncodeError::OutputLimit {
            max_bytes: expected.len() - 1
        })
    );
}

#[test]
fn option_presence_has_exact_request_bytes() {
    for mask in 0..8 {
        let args = ListAvailableNodesPageArgs {
            limit: (mask & 1 != 0).then(|| Nat::from(0)),
            start: (mask & 2 != 0).then(Principal::management_canister),
            rejoining: (mask & 4 != 0).then(String::new),
        };
        let mut payload = Vec::new();
        if args.limit.is_some() {
            payload.extend_from_slice(&[1, 0]);
        } else {
            payload.push(0);
        }
        if args.start.is_some() {
            payload.extend_from_slice(&[1, 1, 0]);
        } else {
            payload.push(0);
        }
        if args.rejoining.is_some() {
            payload.extend_from_slice(&[1, 0]);
        } else {
            payload.push(0);
        }
        check(&args, &payload);
    }
    for max_bytes in 0..36 {
        assert_eq!(
            encode(&ListAvailableNodesPageArgs::default(), max_bytes),
            Err(EncodeError::OutputLimit { max_bytes })
        );
    }
}

#[test]
fn natural_leb_boundaries_and_arbitrary_magnitude() {
    for (text, bytes) in [
        ("0", vec![0]),
        ("127", vec![127]),
        ("128", vec![128, 1]),
        ("18446744073709551615", [vec![255; 9], vec![1]].concat()),
        ("18446744073709551616", [vec![128; 9], vec![2]].concat()),
        (
            "1393796574908163946345982392040522594123776",
            [vec![128; 20], vec![1]].concat(),
        ),
    ] {
        let args = ListAvailableNodesPageArgs {
            limit: Some(text.parse().unwrap()),
            ..Default::default()
        };
        let mut payload = vec![1];
        payload.extend_from_slice(&bytes);
        payload.extend_from_slice(&[0, 0]);
        check(&args, &payload);
    }
}

#[test]
fn cursor_and_opaque_utf8_are_not_normalized() {
    for principal in [Principal::anonymous(), Principal::from_slice(&[5; 29])] {
        for text in [" \0é ".to_owned(), "x".repeat(128)] {
            let args = ListAvailableNodesPageArgs {
                limit: None,
                start: Some(principal),
                rejoining: Some(text.clone()),
            };
            let mut payload = vec![0, 1, 1, principal.as_slice().len() as u8];
            payload.extend_from_slice(principal.as_slice());
            payload.push(1);
            if text.len() == 128 {
                payload.extend_from_slice(&[128, 1]);
            } else {
                payload.push(text.len() as u8);
            }
            payload.extend_from_slice(text.as_bytes());
            check(&args, &payload);
        }
    }
}
