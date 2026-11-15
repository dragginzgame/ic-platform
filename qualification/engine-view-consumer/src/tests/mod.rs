use super::selected_subnet;
use ic_platform_protocol::{
    candid::Limits,
    cloud_engine::{EngineView, decode_get_engine_views_response},
};
use ic_platform_types::Principal;

#[test]
fn public_projection_preserves_principal_identity_and_serde_shape() {
    // Independent selected record: id, optional subnet_id, and node_ids.
    // Its subnet is the anonymous Principal and it contains no node IDs.
    let frame = b"DIDL\x04\x6d\x01\x6c\x03\xdb\xb7\x01\x71\xbd\x86\x9d\x8b\x04\x02\xbb\xf8\xfd\xed\x0f\x03\x6e\x68\x6d\x68\x01\x00\x01\x01x\x01\x01\x01\x04\x00";
    let mut views = decode_get_engine_views_response(
        frame,
        Limits {
            bytes: 1024,
            header: 512,
            types: 16,
            work: 1024,
            allocation: 8192,
            vector: 8,
            skipping: 0,
        },
    )
    .unwrap();
    let view = views.pop().unwrap();
    let principal: Principal = selected_subnet(&view).unwrap();
    assert_eq!(principal, Principal::anonymous());
    let expected = serde_json::json!({
        "id": "x", "subnet_id": "2vxsx-fae", "engine_operator_id": null,
        "subnet": null, "deleted_at": null, "node_ids": []
    });
    assert_eq!(serde_json::to_value(&view).unwrap(), expected);
    let from_json: EngineView = serde_json::from_value(expected).unwrap();
    assert_eq!(from_json, view);
}

#[cfg(not(target_family = "wasm"))]
mod comparison;

mod serde;

#[cfg(feature = "marketplace")]
mod marketplace;
mod observations;

#[cfg(feature = "registry-settings")]
mod registry_settings;
