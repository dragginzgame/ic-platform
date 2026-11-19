//! Independently authenticate Runtime's actual certification-root publication.

use candid::Principal;
use candidate_platform::certificate::{
    CertificateDecodeLimits, PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits,
    authenticate_certificate, decode_certificate,
};
use ic_certification::LookupResult;
use ic_testkit::pocket_ic::nonblocking::PocketIc;
use published_platform::certificate as published;

fn identity<'a>(bytes: &mut &'a [u8]) -> &'a [u8] {
    let size = usize::from(bytes[0]);
    let (principal, rest) = bytes[1..].split_at(size);
    *bytes = rest;
    principal
}

pub(super) async fn verify(pic: &PocketIc, canister: Principal, root_bytes: [u8; 32]) {
    pic.tick().await;
    let root = PreparedRootKey::from_der(&pic.root_key().await.unwrap()).unwrap();
    let controllers = pic
        .canister_status(canister, None)
        .await
        .unwrap()
        .settings
        .controllers;
    for caller in [Principal::anonymous(), canister] {
        let reply = pic
            .query_call(canister, caller, "system_observe", vec![])
            .await
            .unwrap();
        let mut bytes = reply.as_slice();
        assert_eq!(identity(&mut bytes), canister.as_slice());
        assert_eq!(identity(&mut bytes), caller.as_slice());
        let timestamp = u64::from_le_bytes(bytes[..8].try_into().unwrap());
        assert_eq!(timestamp, pic.get_time().await.as_nanos_since_unix_epoch());
        assert_eq!(bytes[8], u8::from(controllers.contains(&caller)));
        assert_eq!(bytes[9], 1);
        let certificate =
            decode_certificate(&bytes[10..], CertificateDecodeLimits { bytes: 1_048_576 }).unwrap();
        let window = TimeWindow {
            reference_ns: timestamp,
            max_age_ns: 60_000_000_000,
            max_future_skew_ns: 0,
        };
        let target = canister.as_slice().try_into().unwrap();
        let evidence = authenticate_certificate(
            certificate,
            &root,
            target,
            TimePolicy {
                certificate: window,
                delegation: window,
            },
            VerificationLimits {
                tree_nodes: 4096,
                tree_depth: 64,
                delegation_bytes: 131072,
                canister_ranges: 256,
            },
        )
        .unwrap();
        assert_eq!(
            evidence.tree().lookup_path([
                b"canister".as_slice(),
                canister.as_slice(),
                b"certified_data"
            ]),
            LookupResult::Found(root_bytes.as_slice())
        );
        // Independently admit the original bytes through the latest registry
        // release. Each verifier owns its Types source; only bytes cross here.
        let published_root =
            published::PreparedRootKey::from_der(&pic.root_key().await.unwrap()).unwrap();
        let published_window = published::TimeWindow {
            reference_ns: timestamp,
            max_age_ns: 60_000_000_000,
            max_future_skew_ns: 0,
        };
        let published_evidence = published::authenticate_certificate(
            published::decode_certificate(
                &bytes[10..],
                published::CertificateDecodeLimits { bytes: 1_048_576 },
            )
            .unwrap(),
            &published_root,
            canister.as_slice().try_into().unwrap(),
            published::TimePolicy {
                certificate: published_window,
                delegation: published_window,
            },
            published::VerificationLimits {
                tree_nodes: 4096,
                tree_depth: 64,
                delegation_bytes: 131072,
                canister_ranges: 256,
            },
        )
        .unwrap();
        assert_eq!(
            published_evidence.tree().lookup_path([
                b"canister".as_slice(),
                canister.as_slice(),
                b"certified_data"
            ]),
            LookupResult::Found(root_bytes.as_slice())
        );
    }
    // The same query export in replicated execution receives no certificate.
    let reply = pic
        .update_call(canister, Principal::anonymous(), "system_observe", vec![])
        .await
        .unwrap();
    let mut bytes = reply.as_slice();
    assert_eq!(identity(&mut bytes), canister.as_slice());
    assert_eq!(identity(&mut bytes), Principal::anonymous().as_slice());
    assert_eq!(bytes.len(), 10);
    assert_eq!(bytes[9], 0);
    println!(
        "installed identity/time/controller/certificate observation passed, root byte {}",
        root_bytes[0]
    );
}
