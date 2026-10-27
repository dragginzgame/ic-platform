//! Profile a real preloaded ICP ledger without enabling instruction-detail or
//! byte-mask tracing. The NNS system-subnet cycle schedule is intentionally
//! distinguished from application-subnet pricing in the capture metadata.
//!
//! cargo run -p pocket-ic --example system_profile_ledger -- run.icprofile [SERVER_URL]
//! Set POCKET_IC_PROFILE_KEEP_ALIVE=1 to keep the canister available in the viewer.

use candid::{CandidType, Deserialize, Nat, Principal, decode_args, encode_args};
use pocket_ic::{
    PocketIcBuilder, SystemProfileConfig,
    common::rest::{IcpFeatures, IcpFeaturesConfig},
};
use serde_json::json;

#[derive(CandidType, Deserialize)]
struct Account {
    owner: Principal,
    subaccount: Option<Vec<u8>>,
}

#[derive(CandidType)]
struct Transfer {
    from_subaccount: Option<Vec<u8>>,
    to: Account,
    amount: Nat,
    fee: Option<Nat>,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let output = arguments
        .next()
        .unwrap_or_else(|| "ledger.icprofile".into());
    let mut builder = PocketIcBuilder::new().with_ram_storage();
    if let Some(server) = arguments.next() {
        builder = builder.with_server_url(server.parse().expect("valid PocketIC server URL"));
    }
    let pic = builder
        .with_icp_features(IcpFeatures {
            icp_token: Some(IcpFeaturesConfig::DefaultConfig),
            ..Default::default()
        })
        .build();
    let ledger = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    let recipient = Principal::from_slice(&[42; 29]);
    let payload = encode_args((Transfer {
        from_subaccount: None,
        to: Account {
            owner: recipient,
            subaccount: None,
        },
        amount: Nat::from(100_000_000_u64),
        fee: Some(Nat::from(10_000_u64)),
        memo: None,
        created_at_time: None,
    },))
    .unwrap();
    let run = pic.start_system_profile(SystemProfileConfig {
        name: "Real ICP ledger: 30 transfers and a balance query".into(),
        sample_interval_ms: 20,
        metadata: json!({
            "workload":"pinned ICP ledger icrc1_transfer",
            "successful_transfers":30,"amount_e8s_each":100_000_000,
            "ledger_token_fee_e8s_each":10_000,
            "cycle_schedule":"preloaded NNS system subnet; cycle fees differ from application subnets",
            "setup":"preload/compile before measured interval",
            "detailed_instruction_tracing":false,"byte_masks":false
        }),
        ..Default::default()
    });
    let id = run["run_id"].as_str().unwrap();
    for _ in 0..30 {
        let reply = pic
            .update_call(
                ledger,
                Principal::anonymous(),
                "icrc1_transfer",
                payload.clone(),
            )
            .unwrap();
        assert!(reply.starts_with(b"DIDL"));
    }
    let reply = pic
        .query_call(
            ledger,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((Account {
                owner: recipient,
                subaccount: None,
            },))
            .unwrap(),
        )
        .unwrap();
    let (balance,): (Nat,) = decode_args(&reply).unwrap();
    assert_eq!(balance, Nat::from(3_000_000_000_u64));
    let summary = pic.stop_system_profile(id);
    std::fs::write(&output, pic.export_system_profile(id)).unwrap();
    let viewer = pic.start_memory_access_viewer(None);
    let context = json!({"run_id":id,"instance_id":pic.instance_id(),"canister_id":ledger.to_text(),
        "server":pic.get_server_url().as_str(),"viewer":viewer.as_str(),"bundle":output,
        "recipient_balance_e8s":balance.to_string(),"summary":summary["summary"]});
    if let Ok(path) = std::env::var("POCKET_IC_PROFILE_CONTEXT") {
        std::fs::write(path, serde_json::to_vec_pretty(&context).unwrap()).unwrap();
    }
    println!("{}", serde_json::to_string_pretty(&context).unwrap());
    while std::env::var_os("POCKET_IC_PROFILE_KEEP_ALIVE").is_some() {
        std::thread::sleep(std::time::Duration::from_secs(30));
        std::hint::black_box(&pic);
    }
}
