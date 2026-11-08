//! Opt-in instruction measurements of matched Platform size probes.
use ic_platform::principal::ExternalPrincipal as Principal;
use ic_testkit::pocket_ic::PocketIc;

pub async fn run(pic: &PocketIc, id: Principal, wasm: Vec<u8>, mode: &str) {
    pic.install_canister(id, wasm, vec![], None).await.unwrap();
    match mode {
        "system" => {
            for count in [1_u32, 32] {
                for method in ["system_certificate", "system_root_key"] {
                    let input = count.to_le_bytes().to_vec();
                    let result = if method == "system_root_key" {
                        pic.update_call(id, Principal::anonymous(), method, input)
                            .await
                    } else {
                        pic.query_call(id, Principal::anonymous(), method, input)
                            .await
                    }
                    .unwrap();
                    let (instructions, bytes) = result.split_at(8);
                    let instructions = u64::from_le_bytes(instructions.try_into().unwrap());
                    match method {
                        "system_root_key" => {
                            assert_eq!(bytes, pic.root_key().await.unwrap().unwrap())
                        }
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
                measure_admission(pic, id, "wide", count, bytes).await;
            }
            for depth in [16_u64, 17, 8192] {
                let mut bytes = b"DIDL\x01\x6e\0\x01\0".to_vec();
                bytes.resize(bytes.len() + depth as usize, 1);
                bytes.push(0);
                measure_admission(pic, id, "recursive", depth, bytes).await;
            }
        }
        _ => panic!("unknown explicit codec benchmark mode"),
    }
}

async fn measure_admission(pic: &PocketIc, id: Principal, case: &str, count: u64, bytes: Vec<u8>) {
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
            .await
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

/// Check both copy paths once per size, or explicitly collect instruction medians.
pub async fn buffer_copy(pic: &PocketIc, id: Principal, measure: bool) {
    for size in super::BUFFER_COPY_SIZES {
        let payload: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let mut baseline = Vec::new();
        let mut candidate = Vec::new();
        for round in 0..if measure { 6 } else { 1 } {
            for old in if round % 2 == 0 {
                [true, false]
            } else {
                [false, true]
            } {
                let args = (old, payload.clone());
                let expected = super::request("buffer_copy", &args);
                let (instructions, bytes): (u64, Vec<u8>) =
                    super::query(pic, id, "buffer_copy", args).await;
                assert_eq!(bytes, expected);
                if round != 0 {
                    if old {
                        baseline.push(instructions);
                    } else {
                        candidate.push(instructions);
                    }
                }
            }
        }
        if measure {
            baseline.sort_unstable();
            candidate.sort_unstable();
            println!(
                "buffer-copy payload={size} baseline={} candidate={} instructions",
                baseline[2], candidate[2]
            );
        }
    }
}
