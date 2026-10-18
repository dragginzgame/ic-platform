#[test]
fn original_client_and_dto_identities() {
    macro_rules! identity {
        ($($path:ident)::+) => {
            let original: Option<pocket_ic::$($path)::+> = None;
            let boundary: Option<crate::pocket_ic::$($path)::+> = original;
            let _: Option<pocket_ic::$($path)::+> = boundary;
        };
    }
    identity!(PocketIc);
    identity!(PocketIcBuilder);
    identity!(nonblocking::PocketIc);
    identity!(RejectResponse);
    identity!(RejectCode);
    identity!(ErrorCode);
    identity!(CanisterSettings);
    identity!(CanisterStatusResult);
    identity!(CanisterLogRecord);
    identity!(Snapshot);
    identity!(CreateCanisterParams);
    identity!(Time);
    identity!(common::rest::BlobCompression);
    identity!(common::rest::CreateInstanceResponse);
    identity!(common::rest::IcpFeatures);
    identity!(common::rest::IcpFeaturesConfig);
    identity!(common::rest::RawCanisterId);
    identity!(common::rest::Topology);
}
