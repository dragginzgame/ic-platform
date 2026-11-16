//! Frozen independent Caffeine frames executed on 32-bit Wasm.
#![forbid(unsafe_code)]
use ic_platform_protocol::{
    caffeine as c,
    candid::{Error, Limits},
};
fn main() {
    let callback = Limits {
        bytes: 4096,
        header: 512,
        types: 8,
        work: 65536,
        allocation: 65536,
        vector: 128,
        skipping: 65536,
    };
    let roots = include_bytes!("../../fixtures/caffeine-roots.bin");
    assert_eq!(
        c::decode_roots(roots, callback).unwrap(),
        [vec![1; 32], vec![255; 32]]
    );
    let cert = include_bytes!("../../fixtures/caffeine-certificate.bin");
    let value = c::decode_certificate(
        cert,
        Limits {
            bytes: 1024,
            ..callback
        },
    )
    .unwrap();
    assert_eq!(value.method, "upload");
    assert_eq!(value.blob_hash, format!("sha256:{}", "01".repeat(32)));
    let encoded = c::encode_certificate(&value, 1024).unwrap();
    assert_eq!(c::decode_certificate(&encoded, callback).unwrap(), value);
    let cashier = Limits {
        bytes: 65536,
        header: 8192,
        types: 128,
        work: 262144,
        allocation: 262144,
        vector: 4096,
        skipping: 262144,
    };
    let account = include_bytes!("../../fixtures/caffeine-account.bin");
    let c::AccountResult::Ok(value) = c::decode_account(account, cashier).unwrap() else {
        panic!()
    };
    assert_eq!(
        value.cycle_balance.to_string(),
        "-340282366920938463463374607431768211456000000000000000000000"
    );
    for end in 0..account.len() {
        assert!(c::decode_account(&account[..end], cashier).is_err());
    }
    for select in 0..5 {
        let mut bound = cashier;
        match select {
            0 => bound.bytes = account.len() - 1,
            1 => bound.header = 0,
            2 => bound.types = 0,
            3 => bound.work = 0,
            _ => bound.allocation = 0,
        };
        assert_eq!(c::decode_account(account, bound).unwrap_err(), Error::Limit);
    }
    assert_eq!(
        c::decode_roots(
            roots,
            Limits {
                vector: 1,
                ..callback
            }
        )
        .unwrap_err(),
        Error::Limit
    );
    assert_eq!(c::decode_empty(b"DIDL\0\0", callback), Ok(()));
    assert_eq!(
        c::decode_empty(b"DIDL\0\x01\x7f", callback),
        Err(Error::Arity)
    );
    let args = c::TopUpArgs {
        target_balance: Some(
            "340282366920938463463374607431768211456000000000000000000000"
                .parse()
                .unwrap(),
        ),
        account: None,
    };
    let bytes = args.encode(1024).unwrap();
    assert_eq!(args.encode(bytes.len()).unwrap(), bytes);
    assert!(args.encode(bytes.len() - 1).is_err());
}
