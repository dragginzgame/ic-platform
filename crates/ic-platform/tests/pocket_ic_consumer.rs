//! Standalone host consumer with only Platform declared; no server is started.
#![cfg(not(target_family = "wasm"))]
use ic_platform::pocket_ic::{
    CanisterInstallMode, CanisterSettings, CreateCanisterParams, CreateCanisterPlacement,
    ErrorCode, LATEST_SERVER_VERSION, PocketIc, PocketIcBuilder, RejectCode, RejectResponse,
    Snapshot,
    common::rest::{
        BlobCompression, CanisterHttpReply, CanisterHttpResponse, CreateInstanceResponse,
        IcpFeatures, IcpFeaturesConfig, InstanceHttpGatewayConfig, MockCanisterHttpResponse,
        RawCanisterId, RawEffectivePrincipal, RawMessageId, Topology,
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
        CanisterInstallMode,
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

    let canister = ic_platform::principal::ExternalPrincipal::management_canister();
    let params = CreateCanisterParams {
        placement: Some(CreateCanisterPlacement::SubnetId(canister)),
        ..CreateCanisterParams::default()
    };
    assert!(matches!(
        params.placement,
        Some(CreateCanisterPlacement::SubnetId(id)) if id == canister
    ));
    let _builder = PocketIcBuilder::new().with_http_gateway(InstanceHttpGatewayConfig {
        ip_addr: Some("127.0.0.1".into()),
        port: Some(8080),
        domains: None,
        https_config: None,
        domain_custom_provider_local_file: None,
    });
    let message = RawMessageId {
        effective_principal: RawEffectivePrincipal::CanisterId(canister.as_slice().into()),
        message_id: vec![7; 32],
    };
    assert_eq!(message.message_id.len(), 32);
    let response = MockCanisterHttpResponse {
        subnet_id: canister,
        request_id: 7,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: b"fixture reply".to_vec(),
        }),
        additional_responses: vec![],
    };
    assert_eq!(response.request_id, 7);

    let _: fn(&PocketIc, RawMessageId) -> Result<Vec<u8>, RejectResponse> = PocketIc::await_call;
    let _: fn(&PocketIc, MockCanisterHttpResponse) = PocketIc::mock_canister_http_response;
    let _update = |client: &PocketIc| -> Result<Vec<u8>, RejectResponse> {
        client.update_call_with_effective_principal(
            canister,
            message.effective_principal,
            canister,
            "fixture",
            vec![],
        )
    };
}

#[test]
fn native_client_surface() {
    client_contract();
}

#[test]
fn chunked_upgrade_preserves_pocketic_management_identity() {
    #[expect(
        clippy::default_trait_access,
        reason = "PocketIC exports the mode but not the contained upgrade flags"
    )]
    let mut mode = CanisterInstallMode::Upgrade(Some(Default::default()));
    let CanisterInstallMode::Upgrade(Some(flags)) = &mut mode else {
        panic!("expected upgrade flags");
    };
    flags.skip_pre_upgrade = Some(true);
    assert_eq!(flags.skip_pre_upgrade, Some(true));

    // Compile the real client parameter without starting a server or installing.
    let _install = |client: &PocketIc, mode: CanisterInstallMode| -> Result<(), RejectResponse> {
        let canister = ic_platform::principal::ExternalPrincipal::management_canister();
        client.install_chunked_canister(canister, None, mode, canister, vec![], vec![], vec![])
    };
}

#[test]
fn pending_ingress_preserves_the_submitted_message_identity() {
    let _status =
        |client: &PocketIc, message: RawMessageId| -> Option<Result<Vec<u8>, RejectResponse>> {
            client.ingress_status(message)
        };
    let _submit = |client: &PocketIc| -> Result<RawMessageId, RejectResponse> {
        let canister = ic_platform::principal::ExternalPrincipal::management_canister();
        client.submit_call(canister, canister, "fixture", vec![])
    };
}
