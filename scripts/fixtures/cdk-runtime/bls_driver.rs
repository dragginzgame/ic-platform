//! Compare unchanged upstream BLS feature variants in installed Wasm.
mod pocketic_support;
use ic_platform::principal::ExternalPrincipal as Principal;
use std::{path::Path, time::Duration};

fn hex(path: &Path) -> Vec<u8> {
    let text = std::fs::read_to_string(path).unwrap();
    text.trim()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn main() {
    pocketic_support::runtime().block_on(run_fixture());
}

async fn run_fixture() {
    let mut args = std::env::args().skip(1);
    let fixtures = args.next().expect("fixture directory");
    let signature = hex(&Path::new(&fixtures).join("bls-signature.hex"));
    let public = hex(&Path::new(&fixtures).join("bls-public.hex"));
    assert_eq!(signature.len(), 48);
    assert_eq!(public.len(), 96);
    let mut request = [signature, public].concat();
    request.extend_from_slice(b"platform-signature");
    let (server, builder) = pocketic_support::builder(Duration::from_secs(60));
    let pic = builder.with_application_subnet().build().await.unwrap();
    println!("variant\tcase\tsample\tinstructions");
    while let Some(variant) = args.next() {
        let wasm = std::fs::read(args.next().expect("Wasm path")).unwrap();
        let id = pic.create_canister().await.unwrap();
        pic.add_cycles(id, 10_000_000_000_000).await.unwrap();
        pic.install_canister(id, wasm, vec![], None).await.unwrap();
        // Updates retain the prepared-generator cache; queries do not commit it.
        for sample in 0..6 {
            let result = pic
                .update_call(
                    id,
                    Principal::anonymous(),
                    "verify_bls_update",
                    request.clone(),
                )
                .await
                .unwrap();
            assert_eq!(result.len(), 9);
            assert_eq!(result[0], 1);
            let instructions = u64::from_le_bytes(result[1..].try_into().unwrap());
            let case = if sample == 0 { "cold" } else { "warm" };
            println!("{variant}\t{case}\t{sample}\t{instructions}");
        }
        let result = pic
            .query_call(id, Principal::anonymous(), "verify_bls", request.clone())
            .await
            .unwrap();
        assert_eq!(result.len(), 9);
        assert_eq!(result[0], 1);
        println!(
            "{variant}\tquery\t0\t{}",
            u64::from_le_bytes(result[1..].try_into().unwrap())
        );
        for (case, offset) in [("wrong-message", 144), ("invalid-signature", 0)] {
            let mut invalid = request.clone();
            if offset == 0 {
                invalid[..48].fill(0);
            } else {
                invalid[offset] ^= 1;
            }
            let result = pic
                .update_call(id, Principal::anonymous(), "verify_bls_update", invalid)
                .await
                .unwrap();
            assert_eq!(result.len(), 9);
            assert_eq!(result[0], 0);
            println!(
                "{variant}\t{case}\t0\t{}",
                u64::from_le_bytes(result[1..].try_into().unwrap())
            );
        }
    }
    pic.drop().await.unwrap();
    drop(server);
}
