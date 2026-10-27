//! Compare recording off, 20 ms and 100 ms on an existing, warmed ICP ledger.
//! Usage: system_profile_overhead SERVER_URL INSTANCE_ID SERVER_PID OUTPUT_DIR
//! COUNT and REPETITIONS default to 30 and 3. No instance is created/deleted.
//! CPU observations are independent /proc boundary snapshots, quantized to OS
//! ticks; keep the server idle apart from this workload for useful comparisons.

use candid::{CandidType, Deserialize, Nat, Principal, decode_args, encode_args};
use pocket_ic::{PocketIc, SystemProfileConfig, SystemProfileQuery};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    time::Instant,
};

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
#[derive(CandidType, Deserialize, Debug)]
enum TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

#[derive(Clone)]
struct Stat {
    name: String,
    start: u64,
    cpu: u64,
    waited: u64,
}
fn stat(path: &Path) -> Option<Stat> {
    let s = fs::read_to_string(path).ok()?;
    let begin = s.find('(')?;
    let end = s.rfind(')')?;
    let f: Vec<_> = s[end + 1..].split_whitespace().collect();
    let n = |i: usize| f.get(i)?.parse::<u64>().ok();
    Some(Stat {
        name: s[begin + 1..end].into(),
        start: n(19)?,
        cpu: n(11)? + n(12)?,
        waited: n(13)? + n(14)?,
    })
}
#[derive(Clone)]
struct Snapshot {
    cpu: u64,
    threads: BTreeMap<(u32, u32, u64), Stat>,
}
fn snapshot(root: u32) -> Snapshot {
    let mut pending = vec![root];
    let mut visited = BTreeSet::new();
    let mut cpu = 0;
    let mut threads = BTreeMap::new();
    while let Some(pid) = pending.pop() {
        if !visited.insert(pid) {
            continue;
        }
        let Some(p) = stat(Path::new(&format!("/proc/{pid}/stat"))) else {
            continue;
        };
        cpu += p.cpu + p.waited;
        let Ok(tasks) = fs::read_dir(format!("/proc/{pid}/task")) else {
            continue;
        };
        for task in tasks.flatten() {
            let Ok(tid) = task.file_name().to_string_lossy().parse::<u32>() else {
                continue;
            };
            if let Ok(children) = fs::read_to_string(task.path().join("children")) {
                pending.extend(
                    children
                        .split_whitespace()
                        .filter_map(|p| p.parse::<u32>().ok()),
                );
            }
            if let Some(t) = stat(&task.path().join("stat")) {
                threads.insert((pid, tid, t.start), t);
            }
        }
    }
    Snapshot { cpu, threads }
}
fn elapsed_ns(start: Instant) -> String {
    start.elapsed().as_nanos().to_string()
}
fn delta(a: &Snapshot, b: &Snapshot, tick_ns: u64) -> Value {
    let mut rows: Vec<_> = b
        .threads
        .iter()
        .filter_map(|(key, s)| {
            let before = a.threads.get(key)?;
            let d = s.cpu.checked_sub(before.cpu)?.saturating_mul(tick_ns);
            (d > 0).then(|| {
                json!({"pid":key.0,"tid":key.1,"start_ticks":key.2,
            "name":s.name,"cpu_ns":d.to_string()})
            })
        })
        .collect();
    rows.sort_by_key(|v| std::cmp::Reverse(v["cpu_ns"].as_str().unwrap().parse::<u64>().unwrap()));
    json!({"cpu_ns":b.cpu.checked_sub(a.cpu).map(|x|x.saturating_mul(tick_ns).to_string()),
        "threads":rows,"semantics":"live process tree plus waited-child ticks; existing-thread deltas; nontransactional boundary snapshots"})
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        5,
        "SERVER_URL INSTANCE_ID SERVER_PID OUTPUT_DIR"
    );
    let pic = PocketIc::new_from_existing_instance(
        args[1].parse().unwrap(),
        args[2].parse().unwrap(),
        None,
    );
    let root: u32 = args[3].parse().unwrap();
    let output = Path::new(&args[4]);
    fs::create_dir_all(output).unwrap();
    let count = std::env::var("COUNT")
        .unwrap_or("30".into())
        .parse::<usize>()
        .unwrap();
    let repetitions = std::env::var("REPETITIONS")
        .unwrap_or("3".into())
        .parse::<usize>()
        .unwrap();
    let metrics_level = std::env::var("METRICS_LEVEL").unwrap_or("standard".into());
    let hz = std::process::Command::new("getconf")
        .arg("CLK_TCK")
        .output()
        .unwrap();
    let tick_ns = 1_000_000_000
        / std::str::from_utf8(&hz.stdout)
            .unwrap()
            .trim()
            .parse::<u64>()
            .unwrap();
    let ledger = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    let recipient = Principal::from_slice(&[43; 29]);
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
    let transfer = || {
        let reply = pic
            .update_call(
                ledger,
                Principal::anonymous(),
                "icrc1_transfer",
                payload.clone(),
            )
            .unwrap();
        let (result,): (Result<Nat, TransferError>,) = decode_args(&reply).unwrap();
        result.expect("ledger transfer rejected");
    };
    for _ in 0..10 {
        transfer();
    }
    let balance = || {
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
        decode_args::<(Nat,)>(&reply).unwrap().0
    };
    let before = balance();
    let mut results = vec![];
    for repetition in 0..repetitions {
        // Rotate order to avoid always favoring the same mode through warmup.
        let modes = [0_u64, 20, 100];
        for index in 0..3 {
            let interval = modes[(index + repetition) % 3];
            let mode = if interval == 0 {
                "off".into()
            } else {
                format!("record_{interval}ms")
            };
            let capture_start = Instant::now();
            let capture_a = snapshot(root);
            let run = if interval == 0 {
                None
            } else {
                Some(pic.start_system_profile(SystemProfileConfig {
                    name: format!("Ledger overhead {mode} repeat {repetition}"),
                    sample_interval_ms: interval,
                    metrics_level: metrics_level.clone(),
                    metadata: json!({"workload":"warm real ICP ledger","successful_transfers":count,
                    "repetition":repetition,"benchmark_mode":mode}),
                    ..Default::default()
                }))
            };
            let id = run.as_ref().map(|r| r["run_id"].as_str().unwrap());
            let work_start = Instant::now();
            let work_a = snapshot(root);
            for _ in 0..count {
                transfer();
            }
            let work_b = snapshot(root);
            let work_wall = elapsed_ns(work_start);
            let summary = id.map(|id| pic.stop_system_profile(id));
            let capture_b = snapshot(root);
            let capture_wall = elapsed_ns(capture_start);
            let mut row = json!({"mode":mode,"repetition":repetition,"transfers":count,
                "metrics_level":metrics_level,
                "workload_wall_ns":work_wall,"capture_wall_ns":capture_wall,
                "workload_cpu":delta(&work_a,&work_b,tick_ns),"capture_cpu":delta(&capture_a,&capture_b,tick_ns),
                "run_id":id,"summary":summary.as_ref().map(|s|&s["summary"]),
                "quality":summary.as_ref().map(|s|&s["quality"])});
            if let Some(id) = id {
                let components =
                    pic.query_system_profile(id, "components", &SystemProfileQuery::default());
                let events = pic.query_system_profile(
                    id,
                    "events",
                    &SystemProfileQuery {
                        limit: 10_000,
                        ..Default::default()
                    },
                );
                for (kind, value) in [("components", &components), ("events", &events)] {
                    let name = format!("{mode}-{repetition}-{kind}.json");
                    fs::write(output.join(&name), serde_json::to_vec(value).unwrap()).unwrap();
                    row[kind] = json!(name);
                }
                // Preserve the final capture as a representative viewer run.
                if repetition + 1 < repetitions || index < 2 {
                    pic.delete_system_profile(id);
                }
            }
            println!("{}", serde_json::to_string(&row).unwrap());
            results.push(row);
            fs::write(
                output.join("results.json"),
                serde_json::to_vec_pretty(&results).unwrap(),
            )
            .unwrap();
        }
    }
    assert_eq!(
        balance(),
        before + Nat::from((repetitions * 3 * count) as u64 * 100_000_000)
    );
}
