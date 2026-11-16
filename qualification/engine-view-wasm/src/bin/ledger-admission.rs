//! Independent creation reply replay and resource refusals on wasm32.
#![forbid(unsafe_code)]
use ic_platform_protocol::{candid::Limits, ledger, management};
fn main() {
    let bytes = include_bytes!("../../fixtures/ledger-created.bin");
    let limits = Limits {
        bytes: bytes.len(),
        header: 8192,
        types: 128,
        work: 262144,
        allocation: 262144,
        vector: 1024,
        skipping: 8192,
    };
    let ledger::CreateResult::Ok(value) = ledger::decode_created(bytes, limits).unwrap() else {
        panic!()
    };
    assert_eq!(
        value.block_id.0.to_string(),
        "340282366920938463463374607431768211456"
    );
    assert_eq!(value.canister_id.as_slice(), &[9; 29]);
    for end in 0..bytes.len() {
        assert!(ledger::decode_created(&bytes[..end], limits).is_err());
    }
    for select in 0..5 {
        let mut bounded = limits;
        match select {
            0 => bounded.bytes -= 1,
            1 => bounded.header = 0,
            2 => bounded.types = 0,
            3 => bounded.work = 0,
            _ => bounded.allocation = 0,
        };
        assert!(ledger::decode_created(bytes, bounded).is_err());
    }
    let args = management::InstallCodeArgs {
        mode: management::InstallMode::Upgrade(None),
        canister_id: value.canister_id,
        wasm_module: vec![0, 97, 115, 109],
        arg: vec![1, 2, 3],
        sender_canister_version: Some(u64::MAX),
    };
    let encoded = args.encode(4096).unwrap();
    assert_eq!(args.encode(encoded.len()).unwrap(), encoded);
    assert!(args.encode(encoded.len() - 1).is_err());
}
