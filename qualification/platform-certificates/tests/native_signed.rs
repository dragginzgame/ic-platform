//! Fault-injected single-attempt signed transport and original-request recovery.
#![forbid(unsafe_code)]
#![cfg(all(feature = "native-signed", not(target_family = "wasm")))]
use candidate_platform::{
    certificate::{PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits},
    ingress,
    native::{Dispatch, Ed25519Identity, PreparedUpdate, ReadClient, ReadConfig, UpdateArgs},
};
use candidate_types::Principal;
use std::{
    io::{Read, Write},
    net::TcpListener,
    time::Duration,
};
const PEM: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIAkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJ\n-----END PRIVATE KEY-----\n";
fn client(endpoint: String) -> ReadClient {
    ReadClient::new(ReadConfig {
        endpoint,
        root: PreparedRootKey::from_der(include_bytes!("../fixtures/mainnet-root.der")).unwrap(),
        root_subnet: Principal::anonymous(),
        message_bytes: 4096,
        request_timeout: Duration::from_millis(300),
        verification: VerificationLimits {
            tree_nodes: 128,
            tree_depth: 32,
            delegation_bytes: 4096,
            canister_ranges: 32,
        },
    })
    .unwrap()
}
fn time() -> TimePolicy {
    let w = TimeWindow {
        reference_ns: 100,
        max_age_ns: 0,
        max_future_skew_ns: 0,
    };
    TimePolicy {
        certificate: w,
        delegation: w,
    }
}
fn original() -> (Ed25519Identity, PreparedUpdate) {
    let id = Ed25519Identity::from_pem(PEM, 4096).unwrap();
    let update = id
        .prepare_update(
            UpdateArgs {
                recipient: Principal::management_canister(),
                effective: Principal::from_slice(&[1]),
                method: "canister_status",
                arg: b"retained",
                expiry: 1,
                nonce: Some(b"nonce"),
            },
            4096,
        )
        .unwrap();
    (id, update)
}
fn serve(reply: Vec<u8>) -> (String, std::thread::JoinHandle<(String, Vec<u8>)>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let task = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buf = [0; 8192];
        let (header, end, length) = loop {
            let n = stream.read(&mut buf).unwrap();
            assert_ne!(n, 0);
            bytes.extend_from_slice(&buf[..n]);
            if let Some(end) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
                let header = String::from_utf8(bytes[..end].to_vec()).unwrap();
                let length = header
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length: ")
                            .map(|v| v.parse::<usize>().unwrap())
                    })
                    .unwrap();
                break (header, end + 4, length);
            }
        };
        while bytes.len() < end + length {
            let n = stream.read(&mut buf).unwrap();
            assert_ne!(n, 0);
            bytes.extend_from_slice(&buf[..n]);
        }
        let _ = stream.write_all(&reply);
        drop(stream);
        listener.set_nonblocking(true).unwrap();
        std::thread::sleep(Duration::from_millis(450));
        assert!(
            matches!(listener.accept(),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock),
            "transport must not retry or follow redirects"
        );
        (header, bytes[end..].to_vec())
    });
    (format!("http://{address}"), task)
}
#[test]
fn exact_dispatch_no_retry_and_lost_response_stays_uncertain() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (_, update) = original();
    for (reply,accepted) in [(b"HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\n\r\n".to_vec(),true),(vec![],false),(b"HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\n\r\n".to_vec(),false),(b"HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\n\r\n".to_vec(),false),(b"HTTP/1.1 307 Temporary Redirect\r\nLocation: /replacement\r\nContent-Length: 0\r\n\r\n".to_vec(),false),(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".to_vec(),false),(b"HTTP/1.1 200 OK\r\nContent-Length: 4097\r\n\r\n".to_vec(),false)] {
  let (endpoint,task)=serve(reply);let result=rt.block_on(client(endpoint).dispatch(&update,time()));
  if accepted{assert!(matches!(result,Ok(Dispatch::Accepted)))}else{assert!(result.is_err())}
  let(header,body)=task.join().unwrap();assert!(header.starts_with(&format!("POST /api/v3/canister/{}/call ",update.effective_canister())));assert_eq!(body,update.envelope_bytes());
 }
}
#[test]
fn post_expiry_recovery_only_sends_read_for_original_id() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (identity, update) = original();
    let (endpoint, task) = serve(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n\xa0".to_vec());
    assert!(
        rt.block_on(client(endpoint).observe_request(&identity, &update, 200, time()))
            .is_err()
    );
    let (header, body) = task.join().unwrap();
    assert!(header.starts_with(&format!(
        "POST /api/v3/canister/{}/read_state ",
        update.effective_canister()
    )));
    let decoded = ingress::decode_envelope(&body, 4096).unwrap();
    let ingress::Content::ReadState {
        sender,
        ingress_expiry,
        paths,
    } = decoded.value.content
    else {
        panic!()
    };
    assert_eq!(sender, identity.principal());
    assert_eq!(ingress_expiry, 200);
    assert_eq!(
        paths[0],
        [b"request_status".to_vec(), update.request_id().to_vec()]
    );
    assert_eq!(update.expiry(), 1);
}
