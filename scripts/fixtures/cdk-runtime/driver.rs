//! Installed Wire runtime checks. Testkit owns server lifecycle and transport.
mod benchmarks;
mod driver_scenarios;
mod external_driver;
mod pocketic_support;
mod protocol;
mod wire_driver;
mod wire_protocol;

use ic_testkit::pocket_ic::CanisterId as Principal;
use ic_testkit::pocket_ic::{RejectCode, RejectResponse, nonblocking::PocketIc};
use protocol::{LIMITS, method_id};
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

// Keep the existing copy corpus and its explicit fixture-owned frame allowance.
const BUFFER_COPY_SIZES: [usize; 5] = [0, 1, 63, 1024, 2048];

fn request(method: &str, args: impl Serialize) -> Vec<u8> {
    ic_wire::native::encode_request(method_id(method), &args, LIMITS).unwrap()
}
fn response<R: DeserializeOwned>(method: &str, bytes: &[u8]) -> R {
    ic_wire::native::decode_response(method_id(method), bytes, LIMITS).unwrap()
}
async fn query<R: DeserializeOwned>(
    pic: &PocketIc,
    id: Principal,
    method: &str,
    args: impl Serialize,
) -> R {
    response(
        method,
        &pic.query_call(id, Principal::anonymous(), method, request(method, args))
            .await
            .unwrap(),
    )
}
async fn update<R: DeserializeOwned>(
    pic: &PocketIc,
    id: Principal,
    method: &str,
    args: impl Serialize,
) -> R {
    response(
        method,
        &pic.update_call(id, Principal::anonymous(), method, request(method, args))
            .await
            .unwrap(),
    )
}

fn assert_guard_rejection(error: &RejectResponse) {
    assert_eq!(error.reject_code, RejectCode::CanisterReject);
    assert_eq!(error.reject_message, "fixture guard refused");
}

#[cfg(test)]
mod tests;

fn main() {
    pocketic_support::runtime().block_on(run_fixture());
}

async fn run_fixture() {
    let wasm = std::fs::read(std::env::args().nth(1).expect("canister Wasm path")).unwrap();
    let legacy = std::env::var_os("PLATFORM_EXTERNAL_CANDID").is_some();
    // Testkit owns server startup/teardown; cross the client-owner boundary by URL.
    let (server, builder) = pocketic_support::builder(Duration::from_secs(60));
    let builder = builder.with_application_subnet();
    // The optional root-key measurement compares against Testkit's NNS key.
    // Keep the ordinary owner qualification topology unchanged.
    let builder = if std::env::var("PLATFORM_CODEC_BENCHMARK").as_deref() == Ok("system") {
        builder.with_nns_subnet()
    } else {
        builder
    };
    let instance = builder.build_async().await;
    let pic = &instance;
    let id = pic.create_canister().await;
    pic.add_cycles(id, 10_000_000_000_000).await;
    if let Ok(mode) = std::env::var("PLATFORM_CODEC_BENCHMARK") {
        benchmarks::run(pic, id, wasm, &mode).await;
        instance.drop().await;
        drop(server);
        return;
    }
    pic.install_canister(id, wasm.clone(), request("initialize", 10u64), None)
        .await;
    if std::env::var_os("PLATFORM_EXECUTOR_BENCHMARK").is_some() {
        wire_driver::benchmark_fanout(pic, id).await;
        instance.drop().await;
        drop(server);
        return;
    }
    let measure_copies = std::env::var_os("PLATFORM_BUFFER_COPY_BENCHMARK").is_some();
    benchmarks::buffer_copy(pic, id, measure_copies).await;
    if measure_copies {
        instance.drop().await;
        drop(server);
        return;
    }
    driver_scenarios::execution(pic, id, legacy).await;
    driver_scenarios::upgrade(pic, id, wasm.clone(), legacy).await;
    let receiver = pic.create_canister().await;
    pic.add_cycles(receiver, 10_000_000_000_000).await;
    pic.install_canister(receiver, wasm, request("initialize", 0u64), None)
        .await;
    assert_eq!(update::<u64>(pic, id, "cancel_task", ()).await, 2);
    driver_scenarios::cycles(pic, id, receiver).await;
    driver_scenarios::stable_memory(pic, receiver).await;
    driver_scenarios::system(pic, receiver).await;
    wire_driver::run(pic, receiver).await;
    if legacy && std::env::var_os("PLATFORM_EXTERNAL_ADAPTERS").is_some() {
        external_driver::qualify(pic, id).await;
    }
    println!(
        "Runtime: Wire RPCs, raw awaits, trap cleanup, upgrade, cycles, stable rollback and certificates passed (external Candid: {legacy})"
    );
    instance.drop().await;
    drop(server);
}
