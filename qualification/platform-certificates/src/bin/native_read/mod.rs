//! Live read-only checks against authenticated mainnet data.

use candidate_platform::{
    certificate::{PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits},
    native::{ReadClient, ReadConfig},
};
use candidate_types::Principal;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(super) async fn run() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        std::env::args().nth(1).as_deref(),
        Some("--live"),
        "explicit --live is required"
    );
    let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos())?;
    let root_subnet =
        Principal::from_text("tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe")?;
    let client = ReadClient::new(ReadConfig {
        endpoint: "https://ic0.app".into(),
        root: PreparedRootKey::from_der(include_bytes!("../../../fixtures/mainnet-root.der"))?,
        root_subnet,
        message_bytes: 1024 * 1024,
        request_timeout: Duration::from_secs(20),
        verification: VerificationLimits {
            tree_nodes: 16384,
            tree_depth: 64,
            delegation_bytes: 131072,
            canister_ranges: 4096,
        },
    })?;
    let time = TimePolicy {
        certificate: TimeWindow {
            reference_ns: now,
            max_age_ns: 300_000_000_000,
            max_future_skew_ns: 30_000_000_000,
        },
        delegation: TimeWindow {
            reference_ns: now,
            max_age_ns: 86_400_000_000_000,
            max_future_skew_ns: 30_000_000_000,
        },
    };
    let expiry = now + 300_000_000_000;

    let subnet = client.read_subnet(root_subnet, expiry, time).await?;
    assert!(subnet.node_keys().count() > 0);
    let metrics = subnet.metrics().ok_or("missing metrics")?;
    println!(
        "root subnet={} time={} type={:?} nodes={} canisters={} state_bytes={} cycles={} updates={}",
        subnet.subnet(),
        subnet.time_ns(),
        subnet.subnet_type(),
        subnet.node_keys().count(),
        metrics.num_canisters,
        metrics.canister_state_bytes,
        metrics.consumed_cycles_total,
        metrics.update_transactions_total
    );
    let registry = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai")?;
    let reply = client
        .query(registry, "get_latest_version", &[], expiry, time)
        .await?;
    assert!(!reply.is_empty());
    println!("Registry authenticated latest-version reply: {reply:02x?}");
    let engine = Principal::from_text("q6cfj-fyaaa-aaaar-qb77q-cai")?;
    let fee = client
        .query(engine, "getNetworkFee", b"DIDL\0\0", expiry, time)
        .await?;
    assert!(fee.starts_with(b"DIDL"));
    println!("Cloud Engine authenticated fee reply: {fee:02x?}");

    Ok(())
}
