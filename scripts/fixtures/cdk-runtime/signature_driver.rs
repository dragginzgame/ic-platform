//! Installed instruction windows and whole-update cycle debits for signatures.
mod pocketic_support;
use ic_testkit::pocket_ic::CanisterId;
use std::time::Duration;

fn main() {
    pocketic_support::runtime().block_on(run());
}
async fn run() {
    let (server, builder) = pocketic_support::builder(Duration::from_secs(60));
    let pic = builder.with_application_subnet().build().await.unwrap();
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert!(args.len().is_multiple_of(2));
    println!("variant\tcase\tsample\trepetitions\twindow_instructions\twhole_update_cycles");
    for pair in args.as_chunks::<2>().0 {
        let variant = &pair[0];
        let wasm = std::fs::read(&pair[1]).unwrap();
        for (name, code, repetitions) in [
            ("noop-batch", 0, 1000u32),
            ("raw-encode", 1, 1000),
            ("der-encode", 2, 1000),
            ("raw-decode", 3, 1000),
            ("der-decode", 4, 1000),
            ("sha256-32", 5, 1000),
            ("sha256-4096", 6, 1000),
            ("noop-single", 0, 1),
            ("direct", 10, 1),
            ("delegated-legacy", 11, 1),
            ("delegated-sharded", 12, 1),
            ("wrong-message", 13, 1),
            ("identity-root", 14, 1),
        ] {
            // A fresh installation per workload makes the first sample cold.
            let id = pic.create_canister().await.unwrap();
            pic.add_cycles(id, 100_000_000_000_000).await.unwrap();
            pic.install_canister(id, wasm.clone(), vec![], None)
                .await
                .unwrap();
            let mut request = vec![code];
            request.extend(repetitions.to_le_bytes());
            for sample in 0..4 {
                let before = pic.cycle_balance(id).await.unwrap();
                let response = pic
                    .update_call(id, CanisterId::anonymous(), "measure", request.clone())
                    .await
                    .unwrap();
                let after = pic.cycle_balance(id).await.unwrap();
                assert_eq!(response.len(), 9);
                assert_eq!(
                    response[0],
                    u8::from(code < 13),
                    "{variant} {name} sample {sample}"
                );
                let instructions = u64::from_le_bytes(response[1..].try_into().unwrap());
                println!(
                    "{variant}\t{name}\t{sample}\t{repetitions}\t{instructions}\t{}",
                    before - after
                );
            }
        }
    }
    pic.drop().await.unwrap();
    drop(server);
}
