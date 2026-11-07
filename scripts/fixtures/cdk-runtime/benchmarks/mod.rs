//! Opt-in instruction measurements of matched Platform size probes.
use ic_platform::principal::ExternalPrincipal as Principal;
use ic_testkit::pocket_ic::PocketIc;

pub fn run(pic: &PocketIc, id: Principal, wasm: Vec<u8>, mode: &str) {
    pic.install_canister(id, wasm, vec![], None).unwrap();
    match mode {
        "system" => {
            for count in [1_u32, 32] {
                for method in ["system_certificate", "system_root_key"] {
                    let input = count.to_le_bytes().to_vec();
                    let result = if method == "system_root_key" {
                        pic.update_call(id, Principal::anonymous(), method, input)
                    } else {
                        pic.query_call(id, Principal::anonymous(), method, input)
                    }
                    .unwrap();
                    let (instructions, bytes) = result.split_at(8);
                    let instructions = u64::from_le_bytes(instructions.try_into().unwrap());
                    match method {
                        "system_root_key" => assert_eq!(bytes, pic.root_key().unwrap().unwrap()),
                        _ => {
                            ic_platform::certification::cbor::decode_certificate(
                                bytes,
                                bytes.len(),
                            )
                            .unwrap();
                        }
                    }
                    println!(
                        "system method={method} count={count} bytes={} instructions={instructions}",
                        bytes.len()
                    );
                }
            }
        }
        "candid" => {
            for count in [0_u64, 1, 16, 128, 1024] {
                let mut bytes = b"DIDL".to_vec();
                leb(count, &mut bytes);
                for _ in 0..count {
                    bytes.extend([0x6e, 0x7f]); // independent opt null types
                }
                bytes.push(0); // no arguments; unused types must still be checked
                measure_admission(pic, id, "wide", count, bytes);
            }
            for depth in [16_u64, 17, 8192] {
                let mut bytes = b"DIDL\x01\x6e\0\x01\0".to_vec();
                bytes.resize(bytes.len() + depth as usize, 1);
                bytes.push(0);
                measure_admission(pic, id, "recursive", depth, bytes);
            }
        }
        _ => panic!("unknown explicit codec benchmark mode"),
    }
}

fn measure_admission(pic: &PocketIc, id: Principal, case: &str, count: u64, bytes: Vec<u8>) {
    for quota in [64_u64, 1_000_000] {
        let limits = ic_platform::candid::DecodeLimits {
            reply_bytes: 1_000_000,
            header_bytes: 100_000,
            type_entries: 10_000,
            decoding_quota: quota as usize,
            skipping_quota: 0,
        };
        let expected = ic_platform::candid::nesting::check(&bytes, limits)
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default();
        let mut input = quota.to_le_bytes().to_vec();
        input.extend_from_slice(&bytes);
        let result = pic
            .query_call(id, Principal::anonymous(), "candid_admission", input)
            .unwrap();
        let (instructions, error) = result.split_at(8);
        assert_eq!(error, expected.as_bytes());
        let instructions = u64::from_le_bytes(instructions.try_into().unwrap());
        println!(
            "candid case={case} count={count} quota={quota} instructions={instructions} outcome={expected:?}"
        );
    }
}

fn leb(mut value: u64, bytes: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        bytes.push(byte | if value == 0 { 0 } else { 0x80 });
        if value == 0 {
            return;
        }
    }
}
