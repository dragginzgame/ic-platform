//! Installed checks for explicit Wire frames, rejection and callback cleanup.

use crate::wire_protocol::*;
use ic_platform::principal::ExternalPrincipal as Principal;
use ic_testkit::{pic::PocketIc, pocket_ic::RejectCode};

fn observe(pic: &PocketIc, id: Principal) -> (u64, u64) {
    let response = pic
        .query_call(
            id,
            Principal::anonymous(),
            "wire_observe",
            ic_wire::native::encode_request(OBSERVE, &(), REQUEST).unwrap(),
        )
        .unwrap();
    ic_wire::native::decode_response(OBSERVE, &response, RESPONSE).unwrap()
}

pub fn run(pic: &PocketIc, id: Principal) {
    assert_eq!(observe(pic, id), (0, 0));
    let input = vec![0u8, 255, 3];
    for (method, frame_id) in [("wire_set", SET), ("wire_await", AWAIT)] {
        let response = pic
            .update_call(
                id,
                Principal::anonymous(),
                method,
                ic_wire::native::encode_request(frame_id, &input, REQUEST).unwrap(),
            )
            .unwrap();
        assert_eq!(
            response,
            ic_wire::native::encode_response(frame_id, &input, RESPONSE).unwrap()
        );
    }
    assert_eq!(observe(pic, id), (1, 0));
    // Oversized malformed bytes must be refused by the guard, before decode.
    for method in ["wire_set", "wire_await"] {
        let error = pic
            .update_call(
                id,
                Principal::anonymous(),
                method,
                vec![0; REQUEST.bytes + 1],
            )
            .unwrap_err();
        assert_eq!(error.reject_code, RejectCode::CanisterReject);
        assert_eq!(
            error.reject_message,
            "Wire fixture request exceeds byte allowance"
        );
        assert_eq!(observe(pic, id), (1, 0));
    }
    // Wrong endpoint, wrong direction, Candid and truncated frames never reach effects.
    for bytes in [
        ic_wire::native::encode_request(ECHO, &input, REQUEST).unwrap(),
        ic_wire::native::encode_response(SET, &input, RESPONSE).unwrap(),
        ic_platform::candid::contract::encode_one(input.clone()).unwrap(),
        vec![0],
    ] {
        let error = pic
            .update_call(id, Principal::anonymous(), "wire_set", bytes)
            .unwrap_err();
        assert_eq!(error.reject_code, RejectCode::CanisterError);
        assert!(error.reject_message.contains("Wire fixture request:"));
        assert_eq!(observe(pic, id), (1, 0));
    }
    for (method, frame_id) in [
        ("wire_tight", TIGHT),
        ("wire_allocation", ALLOCATION),
        ("wire_depth", DEPTH),
    ] {
        let frame = ic_wire::native::encode_request(frame_id, &input, REQUEST).unwrap();
        assert_eq!(
            ic_wire::native::decode_request::<Vec<u8>>(frame_id, &frame, request_limits(frame_id)),
            Err(ic_wire::Error::Limit)
        );
        let error = pic
            .update_call(id, Principal::anonymous(), method, frame)
            .unwrap_err();
        assert_eq!(error.reject_code, RejectCode::CanisterError);
        assert!(
            error
                .reject_message
                .contains("Wire fixture request: wire Limit")
        );
        assert_eq!(observe(pic, id), (1, 0));
    }
    // Zero Rust arguments must still run the explicit unit decoder.
    assert!(
        pic.query_call(id, Principal::anonymous(), "wire_observe", vec![])
            .is_err()
    );
    let error = pic
        .update_call(
            id,
            Principal::anonymous(),
            "wire_encode_trap",
            ic_wire::native::encode_request(ENCODE_TRAP, &(), REQUEST).unwrap(),
        )
        .unwrap_err();
    assert_eq!(error.reject_code, RejectCode::CanisterError);
    assert!(
        error
            .reject_message
            .contains("Wire fixture response: wire Limit")
    );
    // The post-await effect rolls back, then the original CDK cleanup drops the task.
    assert_eq!(observe(pic, id), (1, 1));
    let response = pic
        .update_call(
            id,
            Principal::anonymous(),
            "wire_await",
            ic_wire::native::encode_request(AWAIT, &input, REQUEST).unwrap(),
        )
        .unwrap();
    assert_eq!(
        response,
        ic_wire::native::encode_response(AWAIT, &input, RESPONSE).unwrap()
    );
    println!("Wire entrypoint admission, identity, raw await and encoding-trap cleanup passed");
}
