//! Installed comparison using identical request bytes and explicitly prepared infrastructure.
use ic_testkit::{
    pic::PocketIcStartupConfig,
    pocket_ic::{CanisterId, PocketIcBuilder},
};
use ic_verify_bls_signature::PrivateKey;
use std::{path::PathBuf, time::Duration};

fn request(value: u8, message: &[u8]) -> Vec<u8> {
    let mut bytes = [0; 32];
    bytes[31] = value;
    let key = PrivateKey::deserialize(&bytes).unwrap();
    [
        key.sign(message).serialize().as_slice(),
        key.public_key().serialize().as_slice(),
        message,
    ]
    .concat()
}

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(measure());
}

async fn measure() {
    let paths: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    assert_eq!(paths.len(), 2, "expected baseline and candidate Wasm paths");
    let timeout = Duration::from_secs(120);
    let config = PocketIcStartupConfig::from_env(timeout).unwrap();
    let (server, url) = match config {
        PocketIcStartupConfig::Connect { server_url, .. } => (None, server_url),
        spawn => {
            let server = spawn.start_managed_server().unwrap();
            let url = server.url().to_owned();
            (Some(server), url)
        }
    };
    let pic = PocketIcBuilder::new(timeout)
        .with_server_url(url.parse().unwrap())
        .with_application_subnet()
        .build()
        .await
        .unwrap();
    println!("variant\tmethod\tcase\tsample\tinstructions");
    for (variant, path) in ["baseline", "candidate"].into_iter().zip(paths) {
        for method in ["verify_once", "verify_cached"] {
            let canister = pic.create_canister().await.unwrap();
            pic.add_cycles(canister, 20_000_000_000_000).await.unwrap();
            pic.install_canister(canister, std::fs::read(&path).unwrap(), vec![], None)
                .await
                .unwrap();
            let input = request(7, b"independent-bls-wrapper-benchmark");
            for sample in 0..6 {
                let output = pic
                    .update_call(canister, CanisterId::anonymous(), method, input.clone())
                    .await
                    .unwrap();
                assert_eq!(output.len(), 9);
                assert_eq!(output[0], 1);
                println!(
                    "{variant}\t{method}\t{}\t{sample}\t{}",
                    if sample == 0 { "cold" } else { "warm" },
                    u64::from_le_bytes(output[1..].try_into().unwrap())
                );
            }
            let mut wrong_message = input.clone();
            *wrong_message.last_mut().unwrap() ^= 1;
            let mut malformed_signature = input.clone();
            malformed_signature[..48].fill(0);
            let mut malformed_key = input.clone();
            malformed_key[48..144].fill(0);
            for (case, bytes, expected) in [
                ("changed-key", request(9, b"different-key"), 1),
                ("empty-message", request(7, b""), 1),
                ("wrong-message", wrong_message, 0),
                ("invalid-signature", malformed_signature, 0),
                ("invalid-key", malformed_key, 0),
                ("identity-key", request(0, b"identity-refusal"), 0),
                ("truncated", vec![0; 143], 0),
                ("valid-after-refusals", input.clone(), 1),
            ] {
                let output = pic
                    .update_call(canister, CanisterId::anonymous(), method, bytes)
                    .await
                    .unwrap();
                assert_eq!(output.len(), 9);
                assert_eq!(output[0], expected, "{variant}/{method}/{case}");
                println!(
                    "{variant}\t{method}\t{case}\t0\t{}",
                    u64::from_le_bytes(output[1..].try_into().unwrap())
                );
            }
        }
    }
    pic.drop().await.unwrap();
    drop(server);
}
