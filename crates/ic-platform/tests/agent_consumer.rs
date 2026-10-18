//! Standalone native Agent consumer with only Platform declared.
#![cfg(not(target_family = "wasm"))]

#[cfg(test)]
use ic_platform::{
    Principal,
    agent::{
        Agent, AgentBuilder, AgentError, AnonymousIdentity, BasicIdentity, CallResponse,
        DelegatedIdentity, Delegation, EnvelopeContent, HttpErrorPayload, Identity,
        Prime256v1Identity, RejectCode, RejectResponse, ReplyResponse, RequestId,
        RequestStatusResponse, Secp256k1Identity, SignedDelegation, SignedUpdate,
        TransportCallResponse, TransportError, lookup_value, signed_update_inspect, to_request_id,
    },
    certification::{Certificate, SubtreeLookupResult, labeled, labeled_hash, leaf},
};

#[test]
fn signing_preserves_effective_id_payload_and_request_identity() {
    let identity = BasicIdentity::from_raw_key(&[7; 32]);
    let sender: Principal = identity.sender().unwrap();
    let builder: AgentBuilder = Agent::builder()
        .with_url("http://127.0.0.1:1")
        .with_identity(identity)
        .with_max_response_body_size(4096)
        .with_max_tcp_error_retries(0)
        .with_verify_query_signatures(true);
    let agent: Agent = builder.build().unwrap();
    assert_eq!(agent.get_principal().unwrap(), sender);
    let canister = Principal::management_canister();
    let effective = Principal::from_slice(&[1]);
    let arg = vec![0, 1, 2, 255];
    let signed: SignedUpdate = agent
        .update(&canister, "test_method")
        .with_effective_canister_id(effective)
        .with_arg(arg.clone())
        .expire_at(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000))
        .sign()
        .unwrap();
    assert_eq!(signed.effective_canister_id, effective);
    assert_eq!(signed.canister_id, canister);
    assert_eq!(signed.arg, arg);
    let content = EnvelopeContent::Call {
        nonce: signed.nonce.clone(),
        ingress_expiry: signed.ingress_expiry,
        sender,
        canister_id: canister,
        method_name: "test_method".into(),
        arg: arg.clone(),
        sender_info: signed.sender_info.clone(),
    };
    let request_id: RequestId = to_request_id(&content).unwrap();
    assert_eq!(request_id, signed.request_id);
    assert_eq!(RequestId::new(&request_id), request_id);
    signed_update_inspect(
        sender,
        canister,
        "test_method",
        &arg,
        signed.ingress_expiry,
        signed.signed_update.clone(),
    )
    .unwrap();
    assert!(matches!(
        signed_update_inspect(
            sender,
            canister,
            "wrong_method",
            &arg,
            signed.ingress_expiry,
            signed.signed_update
        ),
        Err(AgentError::CallDataMismatch { .. })
    ));
    // Type-check the asynchronous call API without dispatching network traffic.
    let _query = agent.query(&effective, "read").with_arg(arg).call();
    let _status = agent.request_status_raw(&request_id, effective);
}

#[test]
fn anonymous_and_delegated_identities_preserve_sender_and_signed_chain() {
    let anonymous = AnonymousIdentity;
    assert_eq!(anonymous.sender().unwrap(), Principal::anonymous());
    assert!(anonymous.public_key().is_none());
    let unsigned = anonymous.sign_arbitrary(b"fixture").unwrap();
    assert!(unsigned.signature.is_none() && unsigned.delegations.is_none());

    let root = BasicIdentity::from_raw_key(&[7; 32]);
    let session = BasicIdentity::from_raw_key(&[8; 32]);
    let target = Principal::from_slice(&[1]);
    let delegation = Delegation {
        pubkey: session.public_key().unwrap(),
        expiration: 1_000_000,
        targets: Some(vec![target]),
        permissions: None,
    };
    let signature = root.sign_delegation(&delegation).unwrap();
    let from_hex = |hex: &str| -> Vec<u8> {
        let (pairs, remainder) = hex.trim().as_bytes().as_chunks::<2>();
        assert!(remainder.is_empty());
        pairs
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    };
    assert_eq!(
        delegation.signable(),
        from_hex(include_str!("fixtures/agent-delegation-signable.hex"))
    );
    assert_eq!(
        signature.signature.as_ref().unwrap(),
        &from_hex(include_str!("fixtures/agent-delegation-signature.hex"))
    );
    let signed = SignedDelegation {
        delegation,
        signature: signature.signature.unwrap(),
    };
    // No canister signatures occur in this chain, so no network root key is needed.
    let delegated = DelegatedIdentity::new_with_root_key(
        root.public_key().unwrap(),
        Box::new(session),
        vec![signed.clone()],
        &[],
    )
    .unwrap();
    assert_eq!(delegated.sender().unwrap(), root.sender().unwrap());
    let signature = delegated.sign_arbitrary(b"fixture").unwrap();
    assert_eq!(signature.public_key, root.public_key());
    let chain = signature.delegations.unwrap();
    assert_eq!(chain[0].signature, signed.signature);
    assert_eq!(chain[0].delegation.targets, Some(vec![target]));
    assert_eq!(chain[0].delegation.expiration, 1_000_000);

    let mut invalid = signed;
    invalid.delegation.expiration += 1;
    assert!(
        DelegatedIdentity::new_with_root_key(
            root.public_key().unwrap(),
            Box::new(BasicIdentity::from_raw_key(&[8; 32])),
            vec![invalid.clone()],
            &[],
        )
        .is_err()
    );
    // This constructor intentionally leaves replica admission to the caller's fixture.
    let unchecked = DelegatedIdentity::new_unchecked(
        root.public_key().unwrap(),
        Box::new(BasicIdentity::from_raw_key(&[8; 32])),
        vec![invalid],
    );
    assert_eq!(unchecked.sender().unwrap(), root.sender().unwrap());
}

#[test]
fn signed_update_response_variants_do_not_supply_a_retry_policy() {
    let request_id = RequestId::new(&[7; 32]);
    let pending: CallResponse<Vec<u8>> = CallResponse::Poll(request_id);
    assert!(matches!(pending, CallResponse::Poll(id) if id == request_id));
    let immediate: CallResponse<Vec<u8>> = CallResponse::Response(vec![0, 255]);
    assert!(matches!(immediate, CallResponse::Response(bytes) if bytes == [0, 255]));
    let response = TransportCallResponse::Replied {
        certificate: vec![1, 2, 3],
    };
    assert!(
        matches!(response, TransportCallResponse::Replied { certificate } if certificate == [1, 2, 3])
    );
    assert!(matches!(
        TransportCallResponse::Accepted,
        TransportCallResponse::Accepted
    ));
    let rejected = TransportCallResponse::NonReplicatedRejection(RejectResponse {
        reject_code: RejectCode::CanisterReject,
        reject_message: "fixture".into(),
        error_code: None,
    });
    assert!(matches!(
        rejected,
        TransportCallResponse::NonReplicatedRejection(_)
    ));

    let agent = Agent::builder()
        .with_url("http://127.0.0.1:1")
        .build()
        .unwrap();
    let call = agent.update_signed(Principal::anonymous(), vec![]);
    // Check the real return identity without dispatching any request or polling it.
    let _: Box<dyn std::future::Future<Output = Result<CallResponse<Vec<u8>>, AgentError>> + '_> =
        Box::new(call);
}

#[test]
fn certificates_keep_their_shared_identity_and_lookups_do_not_authenticate() {
    let value = leaf(vec![42]);
    let digest = labeled_hash(b"value", &value.digest());
    let tree = labeled(b"value".to_vec(), value);
    assert_eq!(tree.digest(), digest);
    assert!(matches!(
        tree.lookup_subtree([b"value"]),
        SubtreeLookupResult::Found(_)
    ));
    let certificate = Certificate {
        tree,
        signature: vec![],
        delegation: None,
    };
    assert_eq!(
        lookup_value(&certificate, [b"value".as_slice()]),
        Ok([42].as_slice())
    );
    let agent = Agent::builder()
        .with_url("http://127.0.0.1:1")
        .build()
        .unwrap();
    let result: Result<(), AgentError> = agent.verify(&certificate, Principal::anonymous());
    assert!(result.is_err());
}

#[test]
fn pem_identities_and_transport_errors_remain_typed() {
    // Canic's deterministic test-only identity; no authority outside test fixtures.
    let pem = b"-----BEGIN PRIVATE KEY-----\nMIGEAgEAMBAGByqGSM49AgEGBSuBBAAKBG0wawIBAQQgCDLudkRxUeRDhnUp2pvL\nxLDICLIoNCa1sQdMgz5Y14GhRANCAASA7zusnWjPN0y8nJlD4YAEOpTEYu+CcCdO\nVwidXc26G4+/g7dUbMwbN4E3d3bpxHEP31M+2by6jY67MqFKKroR\n-----END PRIVATE KEY-----";
    let identity = Secp256k1Identity::from_pem(pem).unwrap();
    assert_ne!(identity.sender().unwrap(), Principal::anonymous());
    assert!(BasicIdentity::from_pem(b"invalid").is_err());
    assert!(Prime256v1Identity::from_pem(b"invalid").is_err());
    let error = AgentError::HttpError(HttpErrorPayload {
        status: 503,
        content_type: None,
        content: vec![],
    });
    assert!(matches!(error, AgentError::HttpError(payload) if payload.status == 503));
    let error = AgentError::TransportError(TransportError::Generic("fixture".into()));
    assert!(matches!(
        error,
        AgentError::TransportError(TransportError::Generic(_))
    ));
    let status = RequestStatusResponse::Rejected(RejectResponse {
        reject_code: RejectCode::SysTransient,
        reject_message: "fixture".into(),
        error_code: None,
    });
    assert!(matches!(status, RequestStatusResponse::Rejected(_)));
    let status = RequestStatusResponse::Replied(ReplyResponse { arg: vec![42] });
    assert!(matches!(status, RequestStatusResponse::Replied(reply) if reply.arg == [42]));
}
