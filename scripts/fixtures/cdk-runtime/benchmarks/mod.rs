//! Opt-in instruction measurements of matched Platform size probes.
use ic_testkit::pocket_ic::CanisterId as Principal;
use ic_testkit::pocket_ic::nonblocking::PocketIc;

pub async fn run(pic: &PocketIc, id: Principal, wasm: Vec<u8>, mode: &str) {
    pic.install_canister(id, wasm, vec![], None).await;
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
                            assert_eq!(bytes, pic.root_key().await.unwrap())
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
        _ => panic!("unknown explicit codec benchmark mode"),
    }
}
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
