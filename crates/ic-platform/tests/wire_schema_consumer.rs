//! Principal metadata must describe the existing native byte-buffer encoding.
#![cfg(all(test, not(target_family = "wasm")))]
extern crate ic_wire_metadata as ic_wire;

use ic_platform::Principal;
use ic_wire::{Limits, native, typescript};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, ic_wire::Wire)]
struct Request {
    caller: Principal,
}

const LIMITS: Limits = Limits {
    bytes: 512,
    allocation: 65536,
    nodes: 1024,
    depth: 16,
};

#[test]
fn principal_schema_and_frames() {
    assert_eq!(
        typescript::native_schema::<Principal>().unwrap(),
        serde_json::json!({"k": "bytes", "max": 29})
    );
    let schema = typescript::native_schema::<Request>().unwrap();
    let mut vectors = Vec::new();
    for raw in [vec![], vec![4], vec![0; 29], vec![255; 29]] {
        let value = Request {
            caller: Principal::from_slice(&raw),
        };
        let request = native::encode_request(17, &value, LIMITS).unwrap();
        let response = native::encode_response(17, &value, LIMITS).unwrap();
        // A one-field Serde struct has the same payload as its byte buffer.
        assert_eq!(
            request,
            native::encode_request(17, serde_bytes::Bytes::new(&raw), LIMITS).unwrap()
        );
        assert_eq!(
            native::decode_request::<Request>(17, &request, LIMITS).unwrap(),
            value
        );
        assert_eq!(
            native::decode_response::<Request>(17, &response, LIMITS).unwrap(),
            value
        );
        vectors.push(serde_json::json!({"raw": raw, "request": request, "response": response}));
    }
    let mut invalid = Vec::new();
    for length in [30, 31, 255] {
        let raw = vec![0; length];
        assert!(Principal::try_from_slice(&raw).is_err());
        let request = native::encode_request(17, serde_bytes::Bytes::new(&raw), LIMITS).unwrap();
        let response = native::encode_response(17, serde_bytes::Bytes::new(&raw), LIMITS).unwrap();
        assert!(native::decode_request::<Request>(17, &request, LIMITS).is_err());
        assert!(native::decode_response::<Request>(17, &response, LIMITS).is_err());
        invalid.push(serde_json::json!({"raw": raw, "request": request, "response": response}));
    }
    if let Some(directory) = std::env::var_os("PLATFORM_WIRE_SCHEMA_OUTPUT") {
        let directory = std::path::PathBuf::from(directory);
        let method = serde_json::json!({
            "name": "principal", "mode": "query", "id": "17", "request": schema,
            "response": schema, "requestBytes": 512, "responseBytes": 512,
            "requestNodes": 1024, "responseNodes": 1024, "requestDepth": 16, "responseDepth": 16
        });
        std::fs::write(
            directory.join("principal.ts"),
            typescript::generate_registry(&Default::default(), &[method], "./wire-runtime.js")
                .unwrap(),
        )
        .unwrap();
        std::fs::write(directory.join("wire-runtime.ts"), typescript::RUNTIME).unwrap();
        std::fs::write(
            directory.join("vectors.json"),
            serde_json::to_vec(&serde_json::json!({
                "valid": vectors, "invalid": invalid
            }))
            .unwrap(),
        )
        .unwrap();
    }
}
