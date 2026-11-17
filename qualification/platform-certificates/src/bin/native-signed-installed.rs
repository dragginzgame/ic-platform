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
 (import "ic0" "msg_caller_size" (func $caller_size (result i32)))
 (import "ic0" "msg_caller_copy" (func $caller_copy (param i32 i32 i32)))
 (memory (export "memory") 1)
 (func (export "canister_query caller") (local $n i32) (local.set $n (call $caller_size)) (call $caller_copy (i32.const 0)(i32.const 0)(local.get $n)) (call $append (i32.const 0)(local.get $n))(call $reply))
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
   #[cfg(feature = "native-signed-secp256k1")]
   {
    let secp=candidate_platform::native::Secp256k1Identity::from_pem("-----BEGIN EC PRIVATE KEY-----\nMC4CAQEEIAcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHoAcGBSuBBAAK\n-----END EC PRIVATE KEY-----\n",4096)?;
    let caller=tokio::time::timeout(Duration::from_secs(20),client.signed_query_secp256k1(&secp,target,"caller",&[],now()+60_000_000_000,policy())).await??;
    assert_eq!(caller,secp.principal().as_slice());
    assert_ne!(caller,Principal::anonymous().as_slice());
    assert_eq!(client.signed_query_secp256k1(&secp,target,"read_echo",b"secp256k1",now()+60_000_000_000,policy()).await?,b"secp256k1");
   }
   // Separately prepare status authorization after the original update expires.
   // Neither algorithm may renew or resubmit the retained update.
   for algorithm in 0..2 {
    #[cfg(not(feature = "native-signed-secp256k1"))]
    if algorithm == 1 { continue; }
    #[cfg(feature = "native-signed-secp256k1")]
    let secp=candidate_platform::native::Secp256k1Identity::from_pem("-----BEGIN EC PRIVATE KEY-----\nMC4CAQEEIAcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHoAcGBSuBBAAK\n-----END EC PRIVATE KEY-----\n",4096)?;
    let original_expiry=now()+3_000_000_123;
    let args=UpdateArgs{recipient:target,effective:target,method:"echo",arg:b"offline original",expiry:original_expiry,nonce:Some(b"offline-status")};
    let original=if algorithm==0 { identity.prepare_update(args,4096)? } else {
     #[cfg(feature = "native-signed-secp256k1")]
     {secp.prepare_update(args,4096)?}
     #[cfg(not(feature = "native-signed-secp256k1"))]
     {unreachable!()}
    };
    let retained=original.envelope_bytes().to_vec();
    client.dispatch(&original,policy()).await?;
    tokio::time::sleep(Duration::from_nanos(original_expiry.saturating_sub(now())+100_000_000)).await;
    assert!(now()>original.expiry());
    let expiry=now()+60_000_000_123;
    let status=if algorithm==0 {identity.prepare_request_status(*original.request_id(),target,expiry,4096)?} else {
     #[cfg(feature = "native-signed-secp256k1")]
     {secp.prepare_request_status(*original.request_id(),target,expiry,4096)?}
     #[cfg(not(feature = "native-signed-secp256k1"))]
     {unreachable!()}
    };
    let status=candidate_platform::native::PreparedRequestStatus::from_signed_bytes(status.envelope_bytes(),*original.request_id(),original.sender(),target,4096)?;
    assert_eq!(status.expiry(),expiry);
    let decoded=candidate_platform::ingress::decode_envelope(status.envelope_bytes(),4096)?.value;
    let candidate_platform::ingress::Content::ReadState{paths,..}=decoded.content else {panic!("not read state")};
    assert_eq!(paths,vec![vec![b"request_status".to_vec(),original.request_id().to_vec()]]);
    let observation=client.observe_prepared_request(&original,&status,policy()).await?;
    assert_eq!(observation.status(),&RequestStatus::Replied(b"offline original".to_vec()));
    assert_eq!(original.envelope_bytes(),retained);
    println!("{target}: algorithm {algorithm} exact offline status lookup after update expiry passed");
   }
   let metadata=client.read_canister(target,now()+60_000_000_000,policy(),10).await?;
   assert_eq!(metadata.canister(),target);assert_eq!(metadata.controllers(),[identity.principal()]);
   let expected=pic.get_subnet(candid::Principal::from_slice(target.as_slice())).await.ok_or("missing subnet")?;
   assert_eq!(metadata.subnet().as_slice(),expected.as_slice());assert!(!metadata.certificate_bytes().is_empty());
   let hash=client.canister_module_hash(target,now()+60_000_000_000,policy()).await?;assert_ne!(hash,[0;32]);
   #[derive(candid::CandidType)]struct Target{canister_id:candid::Principal}
   let arg=candid::encode_one(Target{canister_id:candid::Principal::from_slice(target.as_slice())})?;
   let management=identity.prepare_update(UpdateArgs{recipient:Principal::management_canister(),effective:target,method:"canister_status",arg:&arg,expiry:now()+60_000_000_000,nonce:None},4096)?;
   let dispatch=client.dispatch(&management,policy()).await?;
   let observation=match dispatch{Dispatch::Observed(o)=>o,_=>client.observe_request(&identity,&management,now()+60_000_000_000,policy()).await?};

   let limits=ic_platform_protocol::candid::Limits{bytes:131072,header:8192,types:128,work:1_000_000,allocation:1_000_000,vector:65536,skipping:131072};
   let RequestStatus::Replied(status_bytes)=observation.status() else {panic!("management status refused")};
   let before=ic_platform_protocol::management::decode_canister_status(status_bytes,limits)?;
   let id=candid::Principal::from_slice(target.as_slice());
   let chunk=pic.upload_chunk(id,Some(sender),wasm.clone()).await.unwrap();
   pic.stop_canister(id,Some(sender)).await.unwrap();
   let chunked=ic_platform_protocol::management::InstallChunkedCodeArgs{
       mode:ic_platform_protocol::management::InstallMode::Upgrade(None),target_canister:target,store_canister:None,
       chunk_hashes_list:vec![ic_platform_protocol::management_chunks::ChunkHash{hash:chunk}],wasm_module_hash:hash.to_vec(),arg:vec![],sender_canister_version:None,
   }.encode(4096)?;
   let prepared=identity.prepare_update(UpdateArgs{recipient:Principal::management_canister(),effective:target,method:"install_chunked_code",arg:&chunked,expiry:now()+60_000_000_000,nonce:None},8192)?;
   let exact=prepared.envelope_bytes().to_vec();
   let result=match client.dispatch(&prepared,policy()).await?{Dispatch::Observed(o)=>o,_=>client.observe_request(&identity,&prepared,now()+60_000_000_000,policy()).await?};
   assert_eq!(exact,prepared.envelope_bytes());
   let RequestStatus::Replied(reply)=result.status() else {panic!("chunked upgrade refused: {:?}",result.status())};
   ic_platform_protocol::management::decode_empty_response(reply,limits)?;
   pic.start_canister(id,Some(sender)).await.unwrap();
   let status_request=identity.prepare_update(UpdateArgs{recipient:Principal::management_canister(),effective:target,method:"canister_status",arg:&arg,expiry:now()+60_000_000_000,nonce:None},4096)?;
   let result=match client.dispatch(&status_request,policy()).await?{Dispatch::Observed(o)=>o,_=>client.observe_request(&identity,&status_request,now()+60_000_000_000,policy()).await?};
   let RequestStatus::Replied(reply)=result.status() else {panic!("post-upgrade status refused")};
   let after=ic_platform_protocol::management::decode_canister_status(reply,limits)?;
   assert!(after.version>before.version);assert!(matches!(after.status,ic_platform_protocol::management_snapshots::Status::Running));
   assert_eq!(after.version,pic.canister_status(id,Some(sender)).await.unwrap().version);

   println!("{target}: signed original update/recovery, signed query, certified controllers/subnet, module hash, chunked upgrade and status-version checks passed");
  }
  pic.drop().await;Ok::<_,Box<dyn std::error::Error>>(())
 })
}
