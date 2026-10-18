//! Platform-only consumer of management HTTP/history DTOs and external Candid formats.
use ic_platform::Principal;
use ic_platform::candid::{DecodeError, DecodeLimits, contract, decode_exactly_one};
use ic_platform::management_canister_types::{
    CanisterInfoArgs, CanisterInfoResult, ChangeDetails, CodeDeploymentMode, HttpHeader,
    HttpMethod, HttpRequestArgs, HttpRequestResult, TransformArgs, TransformContext, TransformFunc,
};

/// Preserve the probe's request bytes, transform references and caller admission.
pub fn roundtrip() {
    let limits = DecodeLimits {
        reply_bytes: 4096,
        decoding_quota: 1_000_000,
        skipping_quota: 10_000,
        type_entries: 64,
        header_bytes: 2048,
    };
    let mut request = HttpRequestArgs {
        url: "https://perf-context.invalid/0".into(),
        max_response_bytes: Some(256),
        method: HttpMethod::GET,
        headers: vec![],
        body: None,
        transform: None,
        is_replicated: None,
        pricing_version: Some(1),
    };
    // Original 0.11.0 owner, before Platform adaptation: the existing Canic probe
    // request. Captured independently under target/management-upstream/.
    let original_hex = concat!(
        "4449444c0e6c08efd6e40271e1edeb4a01e8d6d8930102a2f5ed880403ecdaccac0405c6",
        "a4a198060a90f8f6fc090c9fc593ca0f0d6b069681ba027fcfc5d5027fa0d2aca8047fe0",
        "88f2d2047fab80e3d6067fc88ddcea0b7f6e786e046d7b6e066c0298d6caa20107efabde",
        "cb01046a0108010901016c02efabdecb010481ddb2900a096c03b2ceef2f7da2f5ed8804",
        "04c6a4a198060a6d0b6c02f1fee18d0371cbe4fdc704716e7e6e7901001e68747470733a",
        "2f2f706572662d636f6e746578742e696e76616c69642f30000100010000000000000000",
        "00000101000000",
    );
    let original: Vec<u8> = original_hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    assert_eq!(contract::encode_one(&request).unwrap(), original);
    assert_eq!(
        decode_exactly_one::<HttpRequestArgs>(&original, limits).unwrap(),
        request
    );
    request.headers.push(HttpHeader {
        name: "accept".into(),
        value: "application/octet-stream".into(),
    });
    request.body = Some(vec![0, 128, 255]);
    request.is_replicated = Some(false);
    request.transform = Some(TransformContext {
        function: TransformFunc::new(Principal::anonymous(), "transform".into()),
        context: vec![255, 0],
    });
    for method in [
        HttpMethod::GET,
        HttpMethod::HEAD,
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::DELETE,
        HttpMethod::PATCH,
    ] {
        request.method = method;
        let bytes = contract::encode_one(&request).unwrap();
        assert_eq!(
            decode_exactly_one::<HttpRequestArgs>(&bytes, limits).unwrap(),
            request
        );
    }
    let result = HttpRequestResult {
        status: contract::Nat::from(200_u16),
        headers: request.headers,
        body: vec![0, 128, 255],
    };
    let bytes = contract::encode_one(&result).unwrap();
    assert_eq!(
        decode_exactly_one::<HttpRequestResult>(&bytes, limits).unwrap(),
        result
    );
    assert!(matches!(
        decode_exactly_one::<HttpRequestResult>(
            &bytes,
            DecodeLimits {
                reply_bytes: bytes.len() - 1,
                ..limits
            }
        ),
        Err(DecodeError::ReplyTooLarge { .. })
    ));
    let transform = TransformArgs {
        response: result,
        context: vec![255, 0],
    };
    let bytes = contract::encode_one(&transform).unwrap();
    assert_eq!(
        decode_exactly_one::<TransformArgs>(&bytes, limits).unwrap(),
        transform
    );
}

#[test]
fn management_http_wire_and_transform_roundtrips() {
    roundtrip();
}

fn wire_hex(text: &str) -> Vec<u8> {
    let (pairs, remainder) = text.trim().as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

/// Preserve Blob's bounded history request and the original owner's change records.
pub fn history_roundtrip() {
    let limits = DecodeLimits {
        reply_bytes: 65_536,
        decoding_quota: 2_000_000,
        skipping_quota: 1024,
        type_entries: 128,
        header_bytes: 4096,
    };
    // Encoded once by upstream 0.11.0, without Platform or copied DTO schemas.
    let request_wire = wire_hex(include_str!("fixtures/management-info-args.hex"));
    let request = CanisterInfoArgs {
        canister_id: Principal::anonymous(),
        num_requested_changes: Some(20),
    };
    assert_eq!(contract::encode_one(&request).unwrap(), request_wire);
    assert_eq!(
        decode_exactly_one::<CanisterInfoArgs>(&request_wire, limits).unwrap(),
        request
    );
    let history_wire = wire_hex(include_str!("fixtures/management-info-result.hex"));
    let history: CanisterInfoResult = decode_exactly_one(&history_wire, limits).unwrap();
    assert_eq!(contract::encode_one(&history).unwrap(), history_wire);
    assert_eq!(history.total_num_changes, 9);
    assert_eq!(history.recent_changes.len(), 9);
    assert_eq!(history.controllers, [Principal::anonymous()]);
    assert_eq!(history.module_hash, Some(vec![0xab; 32]));
    for (i, change) in history.recent_changes.iter().enumerate() {
        assert_eq!(change.canister_version, i as u64 + 1);
        assert_eq!(change.timestamp_nanos, i as u64 + 100);
    }
    let changes = &history.recent_changes;
    assert!(matches!(
        changes[0].details,
        Some(ChangeDetails::Creation(_))
    ));
    assert!(matches!(
        changes[1].details,
        Some(ChangeDetails::CodeUninstall)
    ));
    for (index, mode) in [
        CodeDeploymentMode::Install,
        CodeDeploymentMode::Reinstall,
        CodeDeploymentMode::Upgrade,
    ]
    .into_iter()
    .enumerate()
    {
        match &changes[index + 2].details {
            Some(ChangeDetails::CodeDeployment(deployment)) => {
                assert_eq!(deployment.mode, mode);
                assert_eq!(deployment.module_hash, [0xab; 32]);
            }
            other => panic!("expected deployment, got {other:?}"),
        }
    }
    assert!(matches!(
        changes[5].details,
        Some(ChangeDetails::ControllersChange(_))
    ));
    assert!(matches!(
        changes[6].details,
        Some(ChangeDetails::LoadSnapshot(_))
    ));
    assert!(matches!(
        changes[7].details,
        Some(ChangeDetails::RenameCanister(_))
    ));
    assert!(changes[8].details.is_none());
    assert!(
        decode_exactly_one::<CanisterInfoResult>(&history_wire[..history_wire.len() - 1], limits)
            .is_err()
    );
    let extra = contract::encode_args((&history, 0_u8)).unwrap();
    assert!(decode_exactly_one::<CanisterInfoResult>(&extra, limits).is_err());
    assert!(matches!(
        decode_exactly_one::<CanisterInfoResult>(
            &history_wire,
            DecodeLimits {
                reply_bytes: history_wire.len() - 1,
                ..limits
            }
        ),
        Err(DecodeError::ReplyTooLarge { .. })
    ));
}

#[test]
fn management_history_preserves_wire_and_change_variants() {
    history_roundtrip();
}
