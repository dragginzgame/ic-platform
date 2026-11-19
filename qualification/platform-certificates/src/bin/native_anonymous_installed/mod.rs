use {
    base64::{Engine, prelude::BASE64_STANDARD},
    candidate_platform::{
        certificate::{PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits},
        native::{
            AuthenticationProfile, PreparedRequestStatus, PreparedUpdate, ReadClient, ReadConfig,
            RequestStatus,
        },
    },
    candidate_types::Principal,
    ic_host_process::{Input, Ownership, Policy, Stderr},
    ic_platform_protocol::{candid::Limits, management as m},
    ic_testkit::pocket_ic::PocketIcBuilder,
    std::{
        path::PathBuf,
        process::Command,
        sync::atomic::AtomicBool,
        time::{Duration, SystemTime, UNIX_EPOCH},
    },
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn now() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    )
    .unwrap()
}

fn time() -> TimePolicy {
    let window = TimeWindow {
        reference_ns: now(),
        max_age_ns: 300_000_000_000,
        max_future_skew_ns: 30_000_000_000,
    };
    TimePolicy {
        certificate: window,
        delegation: TimeWindow {
            max_age_ns: 86_400_000_000_000,
            ..window
        },
    }
}

fn run(command: &mut Command) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let report = ic_host_process::run(
        command,
        Policy {
            ownership: Ownership::Group,
            input: Input::Null,
            stdout_bytes: 1_048_576,
            stderr: Stderr::Inherit,
            elapsed: Duration::from_secs(30),
        },
        &AtomicBool::new(false),
        |_, bytes| {
            output.extend_from_slice(bytes);
            Ok(())
        },
    );
    if report.failure.is_some() || !report.cleanup.is_empty() {
        return Err(format!("ICP CLI failed: {report:?}").into());
    }
    Ok(output)
}
struct Cli {
    binary: PathBuf,
    url: String,
    root: String,
    directory: tempfile::TempDir,
}
impl Cli {
    fn prepare(
        &self,
        recipient: Principal,
        effective: Principal,
        method: &str,
        arg: &[u8],
    ) -> Result<(PreparedUpdate, PreparedRequestStatus)> {
        let args = self.directory.path().join("args.bin");
        let output = self.directory.path().join("prepared.json");
        std::fs::write(&args, arg)?;
        if output.exists() {
            std::fs::remove_file(&output)?;
        }
        run(Command::new(&self.binary)
            .current_dir(self.directory.path())
            .args([
                "canister",
                "call",
                &recipient.to_text(),
                method,
                "--args-file",
            ])
            .arg(&args)
            .args([
                "--args-format",
                "bin",
                "--identity",
                "anonymous",
                "--network",
                &self.url,
                "--root-key",
                &self.root,
                "--sign-only",
            ])
            .arg(&output))?;
        let bytes = std::fs::read(&output)?;
        let json: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(json["format"], "icp-signed-message");
        assert_eq!(json["version"], 1);
        let request = &json["request"];
        let envelope =
            BASE64_STANDARD.decode(request["envelope"].as_str().ok_or("missing envelope")?)?;
        let status =
            BASE64_STANDARD.decode(request["status_check"].as_str().ok_or("missing status")?)?;
        assert!(PreparedUpdate::from_bytes(&envelope, effective, 1_048_576).is_err());
        let update = PreparedUpdate::from_anonymous_bytes(&envelope, effective, 1_048_576)?;
        assert_eq!(update.authentication(), AuthenticationProfile::Anonymous);
        assert_eq!(update.envelope_bytes(), envelope);
        assert_eq!(update.sender(), Principal::anonymous());
        assert_eq!(update.recipient(), recipient);
        let expected = update
            .request_id()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(request["request_id"], expected);
        let read = PreparedRequestStatus::from_anonymous_bytes(&status, &update, 1_048_576)?;
        assert_eq!(read.envelope_bytes(), status);
        Ok((update, read))
    }

    async fn call(
        &self,
        client: &ReadClient,
        recipient: Principal,
        effective: Principal,
        method: &str,
        arg: &[u8],
    ) -> Result<Vec<u8>> {
        let (update, read) = self.prepare(recipient, effective, method, arg)?;
        let original = update.envelope_bytes().to_vec();
        let read_bytes = read.envelope_bytes().to_vec();
        let id = *update.request_id();
        // Lose the synchronous response. Restore only retained bytes and recover
        // using the original CLI status envelope; never prepare another update.
        let _ = client.dispatch(&update, time()).await?;
        drop((update, read));
        let recovered = PreparedUpdate::from_anonymous_bytes(&original, effective, 1_048_576)?;
        let read = PreparedRequestStatus::from_anonymous_bytes(&read_bytes, &recovered, 1_048_576)?;
        let observation = tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                let observation = client
                    .observe_prepared_request(&recovered, &read, time())
                    .await?;
                match observation.status() {
                    RequestStatus::Replied(_)
                    | RequestStatus::Rejected { .. }
                    | RequestStatus::Done => {
                        break Ok::<_, candidate_platform::native::ReadError>(observation);
                    }
                    _ => tokio::time::sleep(Duration::from_millis(100)).await,
                }
            }
        })
        .await??;
        assert_eq!(observation.request_id(), &id);
        assert_eq!(observation.effective_canister(), effective);
        assert_eq!(recovered.envelope_bytes(), original);
        // Retained proof authenticates offline for the original target too.
        assert_eq!(
            client
                .authenticate_request(&recovered, observation.certificate_bytes(), time())?
                .status(),
            observation.status()
        );
        match observation.status() {
            RequestStatus::Replied(bytes) => Ok(bytes.clone()),
            status => Err(format!("{method}: {status:?}").into()),
        }
    }
}

pub(super) fn check() -> Result<()> {
    let binary = std::env::var_os("ICP_CLI").map_or_else(|| PathBuf::from("icp"), PathBuf::from);
    assert_eq!(run(Command::new(&binary).arg("--version"))?, b"icp 1.7.0\n");
    let wasm = wat::parse_str(
        r#"(module
      (import "ic0" "msg_reply_data_append" (func $append (param i32 i32)))
      (import "ic0" "msg_reply" (func $reply))
      (memory (export "memory") 1) (data (i32.const 0) "anonymous installed")
      (func (export "canister_update echo") (call $append (i32.const 0)(i32.const 19))(call $reply)))"#,
    )?;
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async {
        let mut pic=PocketIcBuilder::new().with_nns_subnet().with_application_subnet().with_server_url(std::env::var("POCKET_IC_SERVER_URL")?.parse()?).with_max_request_time_ms(Some(30_000)).build_async().await;
        let root_bytes=pic.root_key().await.ok_or("missing root")?;
        let root=PreparedRootKey::from_der(&root_bytes)?;
        let root_subnet=pic.topology().await.get_nns().ok_or("missing NNS")?;
        // Bootstrap only supplies an explicit routing target. Every journey effect
        // below is prepared by ICP CLI and dispatched by candidate Platform.
        let route=Principal::from_slice(pic.create_canister().await.as_slice());
        let url=pic.make_live(None).await.to_string();
        let cli=Cli{binary,url:url.clone(),root:root_bytes.iter().map(|b|format!("{b:02x}")).collect(),directory:tempfile::tempdir()?};
        let client=ReadClient::new(ReadConfig{endpoint:url,root,root_subnet:Principal::from_slice(root_subnet.as_slice()),message_bytes:1_048_576,request_timeout:Duration::from_secs(15),verification:VerificationLimits{tree_nodes:4096,tree_depth:64,delegation_bytes:131072,canister_ranges:256}})?;
        let limits=Limits{bytes:131072,header:8192,types:128,work:1_000_000,allocation:1_000_000,vector:65536,skipping:131072};
        let management=Principal::management_canister();
        let args=m::ProvisionalCreateArgs{amount:Some("100000000000000".parse().unwrap()),settings:Some(m::CanisterSettings{controllers:Some(vec![Principal::anonymous()]),..Default::default()}),specified_id:None,sender_canister_version:None}.encode(4096)?;
        let reply=cli.call(&client,management,route,"provisional_create_canister_with_cycles",&args).await?;
        let target=m::decode_created(&reply,limits)?.canister_id;
        let install=m::InstallCodeArgs{mode:m::InstallMode::Install,canister_id:target,wasm_module:wasm.clone(),arg:vec![],sender_canister_version:None}.encode(1_048_576)?;
        let reply=cli.call(&client,management,target,"install_code",&install).await?;m::decode_empty_response(&reply,limits)?;
        assert_eq!(cli.call(&client,target,target,"echo",b"").await?,b"anonymous installed");
        let target_args=m::CanisterArgs{canister_id:target}.encode(1024)?;
        let before=m::decode_canister_status(&cli.call(&client,management,target,"canister_status",&target_args).await?,limits)?;
        m::decode_empty_response(&cli.call(&client,management,target,"stop_canister",&target_args).await?,limits)?;
        let stopped=m::decode_canister_status(&cli.call(&client,management,target,"canister_status",&target_args).await?,limits)?;
        assert!(matches!(stopped.status,ic_platform_protocol::management_snapshots::Status::Stopped));
        let upgrade=m::InstallCodeArgs{mode:m::InstallMode::Upgrade(None),canister_id:target,wasm_module:wasm,arg:vec![],sender_canister_version:None}.encode(1_048_576)?;
        m::decode_empty_response(&cli.call(&client,management,target,"install_code",&upgrade).await?,limits)?;
        m::decode_empty_response(&cli.call(&client,management,target,"start_canister",&target_args).await?,limits)?;
        let after=m::decode_canister_status(&cli.call(&client,management,target,"canister_status",&target_args).await?,limits)?;
        assert!(after.version>before.version);
        assert!(matches!(after.status,ic_platform_protocol::management_snapshots::Status::Running));
        assert_eq!(cli.call(&client,target,target,"echo",b"").await?,b"anonymous installed");
        println!("ICP CLI 1.7.0 anonymous create/install/call/stop/Stopped/upgrade/start and original-request recovery passed: {target}");
        pic.drop().await;
        Ok::<_,Box<dyn std::error::Error>>(())
    })
}
