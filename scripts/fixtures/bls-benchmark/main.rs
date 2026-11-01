//! Compare unchanged upstream BLS feature variants in installed Wasm.
use ic_platform::pocket_ic::PocketIcBuilder;
use ic_platform::principal::ExternalPrincipal as Principal;
use ic_testkit::pic::PocketIcStartupConfig;
use std::{path::Path, time::Duration};

fn hex(path: &Path) -> Vec<u8> {
    let text = std::fs::read_to_string(path).unwrap();
    text.trim()
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let fixtures = args.next().expect("fixture directory");
    let signature = hex(&Path::new(&fixtures).join("bls-signature.hex"));
    let public = hex(&Path::new(&fixtures).join("bls-public.hex"));
    assert_eq!(signature.len(), 48);
    assert_eq!(public.len(), 96);
    let mut request = [signature, public].concat();
    request.extend_from_slice(b"platform-signature");
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))
        .unwrap()
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    let pic = owned.instance();
    println!("variant\tcase\tsample\tinstructions");
    while let Some(variant) = args.next() {
        let wasm = std::fs::read(args.next().expect("Wasm path")).unwrap();
        let id = pic.create_canister();
        pic.add_cycles(id, 10_000_000_000_000);
        pic.install_canister(id, wasm, vec![], None);
        // Updates retain the prepared-generator cache; queries do not commit it.
        for sample in 0..6 {
            let result = pic
                .update_call(
                    id,
                    Principal::anonymous(),
                    "verify_bls_update",
                    request.clone(),
                )
                .unwrap();
            assert_eq!(result.len(), 9);
            assert_eq!(result[0], 1);
            let instructions = u64::from_le_bytes(result[1..].try_into().unwrap());
            let case = if sample == 0 { "cold" } else { "warm" };
            println!("{variant}\t{case}\t{sample}\t{instructions}");
        }
        let result = pic
            .query_call(id, Principal::anonymous(), "verify_bls", request.clone())
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
                .unwrap();
            assert_eq!(result.len(), 9);
            assert_eq!(result[0], 0);
            println!(
                "{variant}\t{case}\t0\t{}",
                u64::from_le_bytes(result[1..].try_into().unwrap())
            );
        }
    }
}
