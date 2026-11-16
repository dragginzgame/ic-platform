//! Installed direct/delegated signed ingress, status recovery and query evidence.
#![forbid(unsafe_code)]
#[cfg(target_family = "wasm")]
fn main() {}
#[cfg(not(target_family = "wasm"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use candidate_platform::{
        certificate::{PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits},
        native::{
            Dispatch, Ed25519Identity, PreparedUpdate, ReadClient, ReadConfig, RequestStatus,
            UpdateArgs,
        },
    };
    use candidate_types::Principal;
    use ic_testkit::pocket_ic::PocketIcBuilder;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    const PEM: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIAkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJ\n-----END PRIVATE KEY-----\n";
    let wasm = wat::parse_str(
        r#"(module
 (import "ic0" "msg_reply_data_append" (func $append (param i32 i32)))
 (import "ic0" "msg_reply" (func $reply))
 (import "ic0" "msg_arg_data_size" (func $size (result i32)))
 (import "ic0" "msg_arg_data_copy" (func $copy (param i32 i32 i32)))
 (memory (export "memory") 1)
 (func $echo (local $n i32) (local.set $n (call $size)) (call $copy (i32.const 0)(i32.const 0)(local.get $n)) (call $append (i32.const 0)(local.get $n))(call $reply))
 (export "canister_update echo" (func $echo)) (func (export "canister_query read_echo") (call $echo)))"#,
    )?;
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async {
  let mut pic=PocketIcBuilder::new().with_nns_subnet().with_application_subnet().with_server_url(std::env::var("POCKET_IC_SERVER_URL")?.parse()?).with_max_request_time_ms(Some(30_000)).build_async().await;
  let topology=pic.topology().await;let root_subnet=topology.get_nns().ok_or("missing NNS")?;
  let root=PreparedRootKey::from_der(&pic.root_key().await.ok_or("missing root")?)?;
  let identity=Ed25519Identity::from_pem(PEM,4096)?;
  let sender=candid::Principal::from_slice(identity.principal().as_slice());
  let mut targets=Vec::new();
  for subnet in [root_subnet,topology.get_app_subnets()[0]] {
   let target=pic.create_canister_on_subnet(Some(sender),None,subnet).await;
   pic.add_cycles(target,1_u128<<80).await;pic.install_canister(target,wasm.clone(),vec![],Some(sender)).await;
   targets.push(Principal::try_from_slice(target.as_slice())?);
  }
  let url=pic.make_live(None).await;
  let client=ReadClient::new(ReadConfig{endpoint:url.to_string(),root,root_subnet:Principal::try_from_slice(root_subnet.as_slice())?,message_bytes:131072,request_timeout:Duration::from_secs(10),verification:VerificationLimits{tree_nodes:4096,tree_depth:64,delegation_bytes:131072,canister_ranges:256}})?;
  let now=||u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()).unwrap();
  let policy=||{let w=TimeWindow{reference_ns:now(),max_age_ns:300_000_000_000,max_future_skew_ns:30_000_000_000};TimePolicy{certificate:w,delegation:TimeWindow{max_age_ns:86_400_000_000_000,..w}}};
  for target in targets {
   let update=identity.prepare_update(UpdateArgs{recipient:target,effective:target,method:"echo",arg:b"exact original reply",expiry:now()+60_000_000_000,nonce:None},4096)?;
   let bytes=update.envelope_bytes().to_vec();let id=*update.request_id();
   // Discard the synchronous delivery result, as a process losing its response
   // would. Recovery gets only durable original bytes, never a replacement call.
   let _=client.dispatch(&update,policy()).await?;
   let recovered=PreparedUpdate::from_bytes(&bytes,target,4096)?;
   let observed=tokio::time::timeout(Duration::from_secs(20),async {
    loop {let observation=client.observe_request(&identity,&recovered,now()+60_000_000_000,policy()).await?;
     match observation.status(){RequestStatus::Replied(_)|RequestStatus::Rejected{..}|RequestStatus::Done=>break Ok::<_,candidate_platform::native::ReadError>(observation),_=>tokio::time::sleep(Duration::from_millis(100)).await}
    }
   }).await??;
   assert_eq!(observed.status(),&RequestStatus::Replied(b"exact original reply".to_vec()));assert_eq!(observed.request_id(),&id);assert_eq!(recovered.envelope_bytes(),bytes);
   assert_eq!(client.signed_query(&identity,target,"read_echo",b"signed query",now()+60_000_000_000,policy()).await?,b"signed query");
   let hash=client.canister_module_hash(target,now()+60_000_000_000,policy()).await?;assert_ne!(hash,[0;32]);
   #[derive(candid::CandidType)]struct Target{canister_id:candid::Principal}
   let arg=candid::encode_one(Target{canister_id:candid::Principal::from_slice(target.as_slice())})?;
   let management=identity.prepare_update(UpdateArgs{recipient:Principal::management_canister(),effective:target,method:"canister_status",arg:&arg,expiry:now()+60_000_000_000,nonce:None},4096)?;
   let dispatch=client.dispatch(&management,policy()).await?;
   let observation=match dispatch{Dispatch::Observed(o)=>o,_=>client.observe_request(&identity,&management,now()+60_000_000_000,policy()).await?};
   assert!(matches!(observation.status(),RequestStatus::Replied(bytes) if bytes.starts_with(b"DIDL")));
   println!("{target}: signed original update/recovery, signed query, module hash and effective-routed management status passed");
  }
  pic.drop().await;Ok::<_,Box<dyn std::error::Error>>(())
 })
}
