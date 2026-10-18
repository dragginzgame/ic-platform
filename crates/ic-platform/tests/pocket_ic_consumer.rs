//! Standalone host consumer with only Platform declared; no server is started.
#![cfg(not(target_family = "wasm"))]
use ic_platform::pocket_ic::{
    CanisterSettings, CreateCanisterParams, ErrorCode, LATEST_SERVER_VERSION, PocketIc,
    PocketIcBuilder, RejectCode, RejectResponse, Snapshot,
    common::rest::{
        BlobCompression, CreateInstanceResponse, IcpFeatures, IcpFeaturesConfig, RawCanisterId,
        Topology,
    },
    nonblocking,
};

/// Compile the client identities and preserve typed rejection diagnostics.
pub fn client_contract() {
    let _builder = PocketIcBuilder::new().with_application_subnet();
    let _: Option<(
        PocketIc,
        nonblocking::PocketIc,
        CanisterSettings,
        CreateCanisterParams,
        Snapshot,
    )> = None;
    let _: Option<(
        BlobCompression,
        CreateInstanceResponse,
        IcpFeatures,
        IcpFeaturesConfig,
        RawCanisterId,
        Topology,
    )> = None;
    let rejection = RejectResponse {
        reject_code: RejectCode::CanisterReject,
        reject_message: "caller policy".into(),
        error_code: ErrorCode::CanisterRejectedMessage,
        certified: false,
    };
    assert!(rejection.to_string().contains("caller policy"));
    assert_eq!(LATEST_SERVER_VERSION, "16.1.0");
}

#[test]
fn native_client_surface() {
    client_contract();
}
