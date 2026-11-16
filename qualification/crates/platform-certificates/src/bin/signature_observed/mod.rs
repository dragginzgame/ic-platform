//! Native simulator adapter; no candidate code constructs or signs the witness.
use candidate_platform::certificate::{
    CanisterSignatureKey, CanisterSignatureLimits, CertificateDecodeLimits, PreparedRootKey,
    TimePolicy, TimeWindow, VerificationLimits, authenticate_canister_signature,
    decode_canister_signature,
};
use ic_certification::{labeled, leaf};
use ic_testkit::pocket_ic::PocketIcBuilder;
use serde_cbor::{Value, value::to_value};
use std::{collections::BTreeMap, path::Path};

pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wasm = wat::parse_file(concat!(env!("CARGO_MANIFEST_DIR"), "/signature.wat"))?;
    // Literal SHA-256 results independently obtained with Node crypto, recorded
    // in the fixture provenance. The candidate's hash helpers are never called.
    let seed_hash = [
        0x47, 0xe5, 0x25, 0x00, 0xac, 0xfa, 0x5a, 0x92, 0x6e, 0x4f, 0x26, 0xf1, 0xb9, 0xfb, 0xb4,
        0x55, 0x1e, 0xd1, 0xa8, 0x72, 0xe7, 0xf5, 0x5e, 0x51, 0x75, 0x23, 0x94, 0x4a, 0x6a, 0xab,
        0x47, 0xf6,
    ];
    let message_hash = [
        0xa9, 0x88, 0xb1, 0xe9, 0x3f, 0x81, 0x6e, 0x57, 0x5a, 0x9d, 0xef, 0x84, 0x74, 0x32, 0xd0,
        0x2d, 0x86, 0x1c, 0xc8, 0x62, 0x85, 0x54, 0x87, 0xac, 0xf3, 0xa8, 0x30, 0x3e, 0x1a, 0x37,
        0x3a, 0xce,
    ];
    let auth_hash = [
        0xc0, 0x55, 0xf2, 0xe5, 0x9f, 0x0d, 0x7d, 0x8d, 0x29, 0x73, 0x0b, 0x0d, 0x2b, 0x15, 0xd0,
        0x41, 0xcc, 0xc0, 0x1b, 0x13, 0x57, 0x86, 0xdd, 0xe0, 0x48, 0xf6, 0xdc, 0x10, 0x01, 0xb1,
        0x5d, 0xed,
    ];
    let auth_message = b"\x1bic-auth-portable-credentialBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";
    let output = Path::new("target/platform-signatures");
    std::fs::create_dir_all(output)?;
    std::fs::write(output.join("probe.wasm"), &wasm)?;
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async {
        let owned = PocketIcBuilder::new().with_nns_subnet().with_application_subnet().with_server_url(std::env::var("POCKET_IC_SERVER_URL")?.parse()?).with_max_request_time_ms(Some(30_000)).build_async().await;
        let pic = &owned;
        let topology = pic.topology().await;
        let root_subnet = topology.get_nns().ok_or("NNS subnet missing")?;
        let app_subnet = topology.get_app_subnets()[0];
        let root_der = pic.root_key().await.ok_or("root key missing")?;
        std::fs::write(output.join("root.der"), &root_der)?;
        let root = PreparedRootKey::from_der(&root_der)?;
        for (name, subnet, delegated, message, hash) in [
            ("direct", root_subnet, false, b"ic-platform observed signature".as_slice(), message_hash),
            ("delegated", app_subnet, true, b"ic-platform observed signature".as_slice(), message_hash),
            ("auth-direct", root_subnet, false, auth_message.as_slice(), auth_hash),
            ("auth-delegated", app_subnet, true, auth_message.as_slice(), auth_hash),
        ] {
            let witness = labeled(b"sig".to_vec(), labeled(seed_hash.to_vec(), labeled(hash.to_vec(), leaf(Vec::new()))));
            let canister = pic.create_canister_on_subnet(None, None, subnet).await;
            pic.add_cycles(canister, 1_u128 << 80).await;
            pic.install_canister(canister, wasm.clone(), witness.digest().to_vec(), None).await;
            pic.tick().await;
            let certificate = pic.query_call(canister, candid::Principal::anonymous(), "certificate", vec![]).await.expect("certificate query must succeed");
            let reference = pic.get_time().await.as_nanos_since_unix_epoch();
            let envelope = Value::Map(BTreeMap::from([
                (Value::Text("certificate".into()), Value::Bytes(certificate)),
                (Value::Text("tree".into()), to_value(&witness)?),
            ]));
            let mut signature = vec![0xd9, 0xd9, 0xf7];
            signature.extend(serde_cbor::to_vec(&envelope)?);
            // Literal short-form SPKI framing for a ten-byte issuer and 18-byte
            // seed, following the interface specification's canister-key OID.
            assert_eq!(canister.as_slice().len(), 10);
            let mut key = vec![0x30,0x2e,0x30,0x0c,0x06,0x0a,0x2b,0x06,0x01,0x04,0x01,0x83,0xb8,0x43,0x01,0x02,0x03,0x1e,0x00,0x0a];
            key.extend(canister.as_slice());
            key.extend(b"qualification-seed");
            // Retain the observation even if the candidate refuses it.
            std::fs::write(output.join(format!("{name}.cbor")), &signature)?;
            std::fs::write(output.join(format!("{name}.der")), &key)?;
            std::fs::write(output.join(format!("{name}.txt")), format!("canister={canister}\nsubnet={subnet}\nreference_ns={reference}\n"))?;
            let window = TimeWindow { reference_ns: reference, max_age_ns: 3_600_000_000_000, max_future_skew_ns: 0 };
            let evidence = authenticate_canister_signature(
                decode_canister_signature(&signature, CertificateDecodeLimits { bytes: 131072 })?,
                message, CanisterSignatureKey::from_der(&key)?, &root,
                TimePolicy { certificate: window, delegation: window },
                CanisterSignatureLimits { message_bytes: message.len(), certificate_bytes: 131072,
                    verification: VerificationLimits { tree_nodes: 4096, tree_depth: 64, delegation_bytes: 131072, canister_ranges: 256 } },
            )?;
            assert_eq!(evidence.delegation().is_some(), delegated);
            assert_eq!(evidence.key().issuer().as_slice(), canister.as_slice());
            std::fs::write(output.join(format!("{name}.txt")), format!(
                "canister={canister}\nsubnet={subnet}\nreference_ns={reference}\ntime_ns={}\ndelegation={:?}\n",
                evidence.time_ns(), evidence.delegation()))?;
            println!("{name}: {} replica-signed envelope bytes authenticated", signature.len());
        }
        owned.drop().await;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
