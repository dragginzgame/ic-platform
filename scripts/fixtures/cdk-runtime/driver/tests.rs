use super::*;

#[test]
#[should_panic]
fn missing_entrypoint_is_not_guard_rejection() {
    assert_guard_rejection(&RejectResponse {
        reject_code: RejectCode::CanisterError,
        reject_message: "fixture guard refused".into(),
        error_code: ic_testkit::pocket_ic::ErrorCode::CanisterMethodNotFound,
        certified: false,
    });
}
#[test]
#[should_panic]
fn unrelated_rejection_is_not_guard_rejection() {
    assert_guard_rejection(&RejectResponse {
        reject_code: RejectCode::CanisterReject,
        reject_message: "unrelated rejection".into(),
        error_code: ic_testkit::pocket_ic::ErrorCode::CanisterRejectedMessage,
        certified: false,
    });
}

#[test]
fn buffer_copy_frames_fit_fixture_budget() {
    for size in BUFFER_COPY_SIZES {
        let bytes = request("buffer_copy", (false, vec![255u8; size]));
        let reply = ic_wire::native::encode_response(
            method_id("buffer_copy"),
            &(0u64, bytes.clone()),
            LIMITS,
        )
        .unwrap();
        let decoded: (u64, Vec<u8>) = response("buffer_copy", &reply);
        assert_eq!(decoded.1, bytes);
    }
}
