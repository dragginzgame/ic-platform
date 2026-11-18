use ic_platform::{certificate::{PreparedRootKey,TimePolicy,TimeWindow,VerificationLimits},native::{ReadClient,ReadConfig,Ed25519Identity,UpdateArgs,RequestStatus,Dispatch,ReadError}};
struct Client { transport: ReadClient, identity: Ed25519Identity }
fn identity(seed:u8)->Ed25519Identity {
    let pem=match seed {
1 => "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIAEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEB\n-----END PRIVATE KEY-----\n",
2 => "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIAICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgIC\n-----END PRIVATE KEY-----\n",
3 => "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIAMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMD\n-----END PRIVATE KEY-----\n",
9 => "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIAkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJ\n-----END PRIVATE KEY-----\n",
        _=>panic!("unselected fixture seed"),
    };
    Ed25519Identity::from_pem(pem,4096).unwrap()
}

fn principal(seed:u8)->Principal { identity(seed).principal() }
fn client(url:&str,root:&[u8],root_subnet:Principal,seed:u8)->Client {
    Client{identity:identity(seed),transport:ReadClient::new(ReadConfig{endpoint:url.into(),root:PreparedRootKey::from_der(root).unwrap(),root_subnet,message_bytes:2*1024*1024,request_timeout:Duration::from_secs(10),verification:VerificationLimits{tree_nodes:4096,tree_depth:64,delegation_bytes:131072,canister_ranges:256}}).unwrap()}
}

fn now()->u64 {std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos().try_into().unwrap()}
fn policy()->TimePolicy{let w=TimeWindow{reference_ns:now(),max_age_ns:300_000_000_000,max_future_skew_ns:30_000_000_000};TimePolicy{certificate:w,delegation:TimeWindow{max_age_ns:86_400_000_000_000,..w}}}
impl Client {
 async fn update(&self,target:Principal,method:&str,args:Vec<u8>)->Vec<u8>{
  let update=self.identity.prepare_update(UpdateArgs{recipient:target,effective:target,method,arg:&args,expiry:now()+60_000_000_000,nonce:Some(&now().to_le_bytes())},2*1024*1024).unwrap();
  let first=self.transport.dispatch(&update,policy()).await.unwrap();
  tokio::time::timeout(Duration::from_secs(30),async {
   let mut observed=match first{Dispatch::Observed(o)=>Some(o),Dispatch::Accepted=>None,Dispatch::UncertifiedRejection{..}=>panic!("fixture dispatch rejected")};
   loop {let observation=match observed.take(){Some(o)=>o,None=>self.transport.observe_request(&self.identity,&update,now()+60_000_000_000,policy()).await.unwrap()};
    match observation.status(){RequestStatus::Replied(reply)=>return reply.clone(),RequestStatus::Rejected{..}|RequestStatus::Done=>panic!("fixture update did not reply: {:?}",observation.status()),_=>tokio::time::sleep(Duration::from_millis(100)).await}
   }
  }).await.unwrap()
 }

 async fn query(&self,target:Principal,method:&str,args:Vec<u8>)->Result<Vec<u8>,ReadError>{self.transport.signed_query(&self.identity,target,method,&args,now()+60_000_000_000,policy()).await}
}
