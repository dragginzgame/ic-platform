use candidate_platform::certificate::{
    CertificateDecodeLimits, PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits,
    VerifyError, authenticate_certificate, authenticate_direct_certificate,
    authenticate_subnet_time, authenticate_subnet_time_from_certificate, decode_certificate,
};
use candidate_types::Principal;
use ic_certification::LookupResult;
use ic_testkit::pocket_ic::PocketIcBuilder;
use std::path::Path;

pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wasm = wat::parse_file(concat!(env!("CARGO_MANIFEST_DIR"), "/certificate.wat"))?;
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async {
        let owned = PocketIcBuilder::new().with_nns_subnet().with_application_subnet().with_server_url(std::env::var("POCKET_IC_SERVER_URL")?.parse()?).with_max_request_time_ms(Some(30_000)).build_async().await;
        let pic = &owned;
        let topology = pic.topology().await;
        let root_subnet = topology.get_nns().ok_or("NNS subnet missing")?;
        let app_subnet = topology.get_app_subnets()[0];
        let root_der = pic.root_key().await.ok_or("root key missing")?;
        let root = PreparedRootKey::from_der(&root_der)?;
        let output = Path::new("target/platform-certificates");
        std::fs::create_dir_all(output)?;
        std::fs::write(output.join("root.der"), &root_der)?;
        for (name, subnet, delegated) in [("direct", root_subnet, false), ("delegated", app_subnet, true)] {
            let canister = pic.create_canister_on_subnet(None, None, subnet).await;
            pic.add_cycles(canister, 1_u128 << 80).await;
            pic.install_canister(canister, wasm.clone(), vec![], None).await;
            pic.tick().await;
            let bytes = pic.query_call(canister, candid::Principal::anonymous(), "certificate", vec![]).await.expect("certificate query must succeed");
            let reference = pic.get_time().await.as_nanos_since_unix_epoch();
            // An explicit byte conversion crosses the native simulator identity boundary.
            let target = Principal::try_from_slice(canister.as_slice())?;
            let window = TimeWindow { reference_ns: reference, max_age_ns: 3_600_000_000_000,
                max_future_skew_ns: 0 };
            let policy = TimePolicy { certificate: window, delegation: window };
            let limits = VerificationLimits { tree_nodes: 4096, tree_depth: 64,
                delegation_bytes: 131072, canister_ranges: 256 };
            let decode = || decode_certificate(&bytes, CertificateDecodeLimits { bytes: 131072 });
            let evidence = authenticate_certificate(decode()?, &root, target, policy, limits)?;
            let trusted_root_subnet = Principal::try_from_slice(root_subnet.as_slice())?;
            let requested_subnet = Principal::try_from_slice(subnet.as_slice())?;
            let subnet_time = authenticate_subnet_time(decode()?, &root, requested_subnet,
                Some(trusted_root_subnet), policy, limits)?;
            let parsed = decode()?;
            let parsed_time = authenticate_subnet_time_from_certificate(parsed.unverified(), &root,
                requested_subnet, Some(trusted_root_subnet), policy, limits)?;
            assert_eq!(parsed_time.time_ns(), evidence.time_ns());
            assert!(parsed_time.original_bytes().is_none());
            assert!(std::ptr::eq(parsed_time.certificate().unwrap(), parsed.unverified()));
            assert_eq!(subnet_time.time_ns(), evidence.time_ns());
            assert_eq!(subnet_time.subnet(), requested_subnet);
            assert_eq!(subnet_time.original_bytes().unwrap().as_ptr(), bytes.as_ptr());
            assert_eq!(subnet_time.root_key_der(), root.as_der());
            let wrong_subnet = if delegated { root_subnet } else { app_subnet };
            assert_eq!(authenticate_subnet_time(decode()?, &root,
                Principal::try_from_slice(wrong_subnet.as_slice())?, Some(trusted_root_subnet),
                policy, limits).unwrap_err(), VerifyError::SubnetMismatch);
            assert_eq!(evidence.original_bytes().as_ptr(), bytes.as_ptr());
            assert_eq!(evidence.root_key_der(), root.as_der());
            assert_eq!(evidence.effective_canister(), target);
            assert_eq!(evidence.time_window(), window);
            assert_eq!(evidence.delegation().is_some(), delegated);
            assert_eq!(evidence.tree().lookup_path([b"canister".as_slice(), target.as_slice(), b"certified_data"]),
                LookupResult::Found(&[0; 32]));
            if delegated {
                assert_eq!(authenticate_direct_certificate(decode()?, &root, target, window, limits).unwrap_err(),
                    VerifyError::DelegationUnsupported);
                let wrong_target = Principal::from_slice(root_subnet.as_slice());
                assert_eq!(authenticate_certificate(decode()?, &root, wrong_target, policy, limits).unwrap_err(),
                    VerifyError::NotAuthorized);
                assert_eq!(evidence.delegation().unwrap().0.as_slice(), subnet.as_slice());
            } else {
                assert!(authenticate_direct_certificate(decode()?, &root, target, window, limits).is_ok());
            }
            let outside = TimeWindow { reference_ns: evidence.time_ns() + 1,
                max_age_ns: 0, max_future_skew_ns: 0 };
            assert_eq!(authenticate_certificate(decode()?, &root, target,
                TimePolicy { certificate: outside, delegation: window }, limits).unwrap_err(),
                VerifyError::TimeOutsideWindow);
            std::fs::write(output.join(format!("{name}.cbor")), &bytes)?;
            std::fs::write(output.join(format!("{name}.txt")), format!(
                "canister={canister}\nsubnet={subnet}\nreference_ns={reference}\ntime_ns={}\ndelegation={:?}\n",
                evidence.time_ns(), evidence.delegation()))?;
            println!("{name}: {} observed bytes; root, target, certified data, selected-subnet time and refusals passed", bytes.len());
        }
        owned.drop().await;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
