//! Local installed ingress acceptance; Testkit owns server startup/admission.
use ic_platform::agent::{
    Agent, BasicIdentity, DelegatedIdentity, Delegation, Identity, Prime256v1Identity,
    Secp256k1Identity, SignedDelegation,
};
use ic_platform_pocket_ic::PocketIcBuilder;
use ic_testkit::pic::PocketIcStartupConfig;
use std::{sync::Arc, time::Duration};

fn main() {
    let startup = PocketIcStartupConfig::from_env(Duration::from_secs(60)).unwrap();
    let (server, url) = match startup {
        PocketIcStartupConfig::Connect { server_url, .. } => (None, server_url),
        spawn => {
            let server = spawn.start_managed_server().unwrap();
            let url = server.url().to_owned();
            (Some(server), url)
        }
    };
    let mut pic = PocketIcBuilder::new()
        .with_server_url(url.parse().unwrap())
        .with_nns_subnet()
        .with_application_subnet()
        .build()
        .unwrap();
    let id = pic.create_canister().unwrap();
    pic.add_cycles(id, 2_000_000_000_000).unwrap();
    let wasm = wat::parse_str(
        r#"(module
      (import "ic0" "msg_arg_data_size" (func $size (result i32)))
      (import "ic0" "msg_arg_data_copy" (func $copy (param i32 i32 i32)))
      (import "ic0" "msg_reply_data_append" (func $append (param i32 i32)))
      (import "ic0" "msg_reply" (func $reply))
      (memory (export "memory") 1)
      (func $echo
        (call $copy (i32.const 0) (i32.const 0) (call $size))
        (call $append (i32.const 0) (call $size))
        (call $reply))
      (export "canister_query echo_query" (func $echo))
      (export "canister_update echo_update" (func $echo))
    )"#,
    )
    .unwrap();
    pic.install_canister(id, wasm, vec![], None).unwrap();
    let root = pic.root_key().unwrap().unwrap();
    let endpoint = pic.make_live(None).unwrap();
    let root_identity = BasicIdentity::from_raw_key(&[7; 32]);
    let session = BasicIdentity::from_raw_key(&[8; 32]);
    let delegation = Delegation {
        pubkey: session.public_key().unwrap(),
        expiration: (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            + Duration::from_secs(300))
        .as_nanos()
        .try_into()
        .unwrap(),
        targets: Some(vec![id]),
        permissions: None,
    };
    let signed = SignedDelegation {
        signature: root_identity
            .sign_delegation(&delegation)
            .unwrap()
            .signature
            .unwrap(),
        delegation,
    };
    let delegated = DelegatedIdentity::new(
        root_identity.public_key().unwrap(),
        Box::new(session),
        vec![signed],
    )
    .unwrap();
    let identities: Vec<Arc<dyn Identity>> = vec![
        Arc::new(BasicIdentity::from_raw_key(&[9; 32])),
        Arc::new(delegated),
        Arc::new(Prime256v1Identity::from_pem(include_bytes!("prime256v1.pem")).unwrap()),
        Arc::new(Secp256k1Identity::from_pem(include_bytes!("secp256k1.pem")).unwrap()),
    ];
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        for identity in identities {
            let agent = Agent::builder()
                .with_url(endpoint.as_str())
                .with_arc_identity(identity)
                .with_max_response_body_size(1_000_000)
                .with_max_polling_time(Duration::from_secs(30))
                .build()
                .unwrap();
            agent.set_root_key(root.clone());
            let payload = vec![0, 255, 7, 128];
            assert_eq!(
                agent
                    .query(&id, "echo_query")
                    .with_arg(payload.clone())
                    .call()
                    .await
                    .unwrap(),
                payload
            );
            assert_eq!(
                agent
                    .update(&id, "echo_update")
                    .with_arg(payload.clone())
                    .call_and_wait()
                    .await
                    .unwrap(),
                payload
            );
            let certificate = agent
                .read_state_raw(vec![vec!["time".into()]], id)
                .await
                .unwrap();
            agent.verify(&certificate, id).unwrap();
            agent.set_root_key(vec![0; 133]);
            assert!(agent.verify(&certificate, id).is_err());
        }
    });
    drop(pic);
    drop(server);
    println!(
        "Installed Agent: verified queries, update-and-wait, read-state and wrong-root refusal passed for Ed25519, delegated, P-256 and secp256k1 identities."
    );
}
