//! Compare RustCrypto's existing backends in installed Wasm.
mod pocketic_support;
use ic_platform::ecdsa::{PrehashSigner, Signature, SigningKey};
use std::time::Duration;

fn main() {
    pocketic_support::runtime().block_on(run_fixture());
}

async fn run_fixture() {
    let key = SigningKey::from_slice(&[7; 32]).unwrap();
    let prehash = [42; 32];
    let signature: Signature = key.sign_prehash(&prehash).unwrap();
    let public = key.verifying_key().to_sec1_point(true);
    let request = [public.as_bytes(), &prehash, &signature.to_bytes()].concat();
    let mut cases = vec![("valid", request.clone(), 1)];
    for (name, offset) in [("wrong-prehash", 33), ("wrong-signature", 65)] {
        let mut invalid = request.clone();
        invalid[offset] ^= 1;
        cases.push((name, invalid, 0));
    }
    let mut invalid = request.clone();
    invalid[..33].fill(0);
    cases.push(("invalid-key", invalid, 0));
    let mut invalid = request.clone();
    invalid[65..].fill(0);
    cases.push(("invalid-scalars", invalid, 0));
    let (r, s) = signature.split_scalars();
    let high = Signature::from_scalars(r.to_bytes(), (-s).to_bytes()).unwrap();
    cases.push((
        "high-s",
        [public.as_bytes(), &prehash, &high.to_bytes()].concat(),
        0,
    ));
    let mut truncated = request.clone();
    truncated.pop();
    cases.push(("short-signature", truncated, 0));
    let mut extended = request;
    extended.push(0);
    cases.push(("long-signature", extended, 0));

    let (server, builder) = pocketic_support::builder(Duration::from_secs(60));
    let pic = builder.with_application_subnet().build().await.unwrap();
    println!("backend\tcase\tsample\tinstructions");
    let mut args = std::env::args().skip(1);
    while let Some(backend) = args.next() {
        let wasm = std::fs::read(args.next().expect("Wasm path")).unwrap();
        let id = pic.create_canister().await.unwrap();
        pic.add_cycles(id, 10_000_000_000_000).await.unwrap();
        pic.install_canister(id, wasm, vec![], None).await.unwrap();
        for (name, request, expected) in &cases {
            for sample in 0..3 {
                let result = pic
                    .query_call(id, Principal::anonymous(), "verify_ecdsa", request.clone())
                    .await
                    .unwrap();
                assert_eq!(result.len(), 9);
                assert_eq!(result[0], *expected, "{backend}/{name}");
                println!(
                    "{backend}\t{name}\t{sample}\t{}",
                    u64::from_le_bytes(result[1..].try_into().unwrap())
                );
            }
        }
    }
    pic.drop().await.unwrap();
    drop(server);
}

use ic_testkit::pocket_ic::CanisterId as Principal;
