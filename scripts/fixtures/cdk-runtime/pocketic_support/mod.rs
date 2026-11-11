//! Test-only runtime/server plumbing. Drivers own topology and awaited cleanup.
use ic_testkit::{
    pic::{PocketIcManagedServer, PocketIcStartupConfig},
    pocket_ic::PocketIcBuilder,
};
use std::time::Duration;

/// Construct the driver's sole caller-owned async runtime.
pub fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

/// Select an explicitly configured server with the driver's finite timeout.
/// Return managed-server custody; the driver closes its client before dropping it.
pub fn builder(timeout: Duration) -> (Option<PocketIcManagedServer>, PocketIcBuilder) {
    let config = PocketIcStartupConfig::from_env(timeout).unwrap();
    let (server, url) = match config {
        PocketIcStartupConfig::Connect { server_url, .. } => (None, server_url),
        spawn => {
            let server = spawn.start_managed_server().unwrap();
            let url = server.url().to_owned();
            (Some(server), url)
        }
    };
    (
        server,
        PocketIcBuilder::new()
            .with_max_request_time_ms(Some(timeout.as_millis().try_into().unwrap()))
            .with_server_url(url.parse().unwrap()),
    )
}
