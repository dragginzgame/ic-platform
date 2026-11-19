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

#[cfg(feature = "native-signed-secp256k1")]
#[test]
fn retained_icp_status_recovers_in_fresh_process() {
    use candidate_platform::{
        certificate::construction::{Certificate, encode_certificate, fork, labeled, leaf},
        fixture_signing::PrivateKey,
        native::{PreparedRequestStatus, RequestStatus},
    };
    if std::env::var_os("PLATFORM_EXTERNAL_STATUS_CHILD").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "retained_icp_status_recovers_in_fresh_process",
                "--nocapture",
            ])
            .env("PLATFORM_EXTERNAL_STATUS_CHILD", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    for (update_bytes, status_bytes, expected_id) in [
        (
            include_bytes!("../fixtures/icp-signing/ed25519-update.cbor").as_slice(),
            include_bytes!("../fixtures/icp-signing/ed25519-status.cbor").as_slice(),
            "c235bf9e696faf28625726376d7953ca8d138f96bfd06176c7663b9479e0c871",
        ),
        (
            include_bytes!("../fixtures/icp-signing/secp256k1-update.cbor").as_slice(),
            include_bytes!("../fixtures/icp-signing/secp256k1-status.cbor").as_slice(),
            "509a16df30427bc7eead86e69ce3bc0e3d3949f74607aab062891dccf1cbbd5d",
        ),
    ] {
        let effective = Principal::from_slice(&[1]);
        let original = PreparedUpdate::from_bytes(update_bytes, effective, 4096).unwrap();
        let request_id: String = original
            .request_id()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(request_id, expected_id);
        let status = PreparedRequestStatus::from_bytes(status_bytes, &original, 4096).unwrap();
        assert_eq!(status.expiry(), original.expiry());
        assert_eq!(original.envelope_bytes(), update_bytes);
        let mut scalar = [0; 32];
        scalar[31] = 1;
        let key = PrivateKey::deserialize(&scalar).unwrap();
        let tree = fork(
            labeled(
                b"request_status".to_vec(),
                labeled(
                    original.request_id().to_vec(),
                    fork(
                        labeled(b"reply".to_vec(), leaf(b"retained result".to_vec())),
                        labeled(b"status".to_vec(), leaf(b"replied".to_vec())),
                    ),
                ),
            ),
            labeled(b"time".to_vec(), leaf(vec![100])),
        );
        let signature = key
            .sign(&[b"\x0dic-state-root".as_slice(), &tree.digest()].concat())
            .serialize()
            .to_vec();
        let certificate = encode_certificate(
            &Certificate {
                tree,
                signature,
                delegation: None,
            },
            4096,
        )
        .unwrap();
        let body = serde_cbor::to_vec(&serde_cbor::Value::Map(
            [(
                serde_cbor::Value::Text("certificate".into()),
                serde_cbor::Value::Bytes(certificate.clone()),
            )]
            .into(),
        ))
        .unwrap();
        let reply = [
            format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).as_bytes(),
            &body,
        ]
        .concat();
        let (endpoint, server) = serve(reply);
        let client = ReadClient::new(ReadConfig {
            endpoint,
            root: PreparedRootKey::from_raw(&key.public_key().serialize()).unwrap(),
            root_subnet: Principal::from_slice(&[2]),
            message_bytes: 4096,
            request_timeout: Duration::from_secs(2),
            verification: VerificationLimits {
                tree_nodes: 128,
                tree_depth: 32,
                delegation_bytes: 4096,
                canister_ranges: 16,
            },
        })
        .unwrap();
        let observed = rt
            .block_on(client.observe_prepared_request(&original, &status, time()))
            .unwrap();
        assert_eq!(
            observed.status(),
            &RequestStatus::Replied(b"retained result".to_vec())
        );
        assert_eq!(observed.certificate_bytes(), certificate);
        let (header, sent) = server.join().unwrap();
        assert!(header.starts_with(&format!("POST /api/v3/canister/{effective}/read_state ")));
        assert_eq!(sent, status_bytes);
        let wrong_route =
            PreparedUpdate::from_bytes(update_bytes, Principal::from_slice(&[2]), 4096).unwrap();
        assert!(
            rt.block_on(client.observe_prepared_request(&wrong_route, &status, time()))
                .is_err()
        );
        assert!(
            client
                .authenticate_request(
                    &original,
                    &certificate,
                    TimePolicy {
                        certificate: TimeWindow {
                            reference_ns: 200,
                            max_age_ns: 0,
                            max_future_skew_ns: 0
                        },
                        ..time()
                    }
                )
                .is_err()
        );
        assert!(superfluous_read_path(status_bytes, &original));
    }
}

#[cfg(feature = "native-signed-secp256k1")]
fn superfluous_read_path(bytes: &[u8], update: &PreparedUpdate) -> bool {
    let mut envelope = ingress::decode_envelope(bytes, 4096).unwrap().value;
    if let ingress::Content::ReadState { paths, .. } = &mut envelope.content {
        paths.push(vec![b"time".to_vec()]);
    }
    candidate_platform::native::PreparedRequestStatus::from_bytes(
        &serde_cbor::to_vec(&envelope).unwrap(),
        update,
        4096,
    )
    .is_err()
}

#[test]
fn anonymous_admission_is_explicit_exact_and_bound_to_original() {
    use candidate_platform::native::{AuthenticationProfile, PreparedRequestStatus};
    let target = Principal::from_slice(&[9]);
    let envelope = ingress::Envelope {
        content: ingress::Content::Call {
            nonce: Some(vec![]),
            ingress_expiry: 1,
            sender: Principal::anonymous(),
            canister_id: target,
            method_name: "echo".into(),
            arg: vec![42],
            sender_info: None,
        },
        sender_pubkey: None,
        sender_sig: None,
        sender_delegation: None,
    };
    let bytes = ingress::encode_envelope(&envelope, 4096).unwrap();
    let update = PreparedUpdate::from_anonymous_bytes(&bytes, target, bytes.len()).unwrap();
    assert_eq!(update.authentication(), AuthenticationProfile::Anonymous);
    assert_eq!(update.envelope_bytes(), bytes);
    assert_eq!(update.expiry(), 1);
    assert_eq!(update.recipient(), target);
    assert_eq!(update.request_id(), &envelope.content.request_id().unwrap());
    assert!(PreparedUpdate::from_bytes(&bytes, target, 4096).is_err());
    assert!(PreparedUpdate::from_anonymous_bytes(&bytes, target, bytes.len() - 1).is_err());
    let status = ingress::Envelope {
        content: ingress::Content::ReadState {
            sender: Principal::anonymous(),
            ingress_expiry: 2,
            paths: vec![vec![
                b"request_status".to_vec(),
                update.request_id().to_vec(),
            ]],
        },
        sender_pubkey: None,
        sender_sig: None,
        sender_delegation: None,
    };
    let read = ingress::encode_envelope(&status, 4096).unwrap();
    let prepared = PreparedRequestStatus::from_anonymous_bytes(&read, &update, read.len()).unwrap();
    assert_eq!(prepared.envelope_bytes(), read);
    assert_eq!(prepared.expiry(), 2);
    assert!(PreparedRequestStatus::from_bytes(&read, &update, 4096).is_err());
    assert!(PreparedRequestStatus::from_anonymous_bytes(&read, &update, read.len() - 1).is_err());
    let (_, signed) = original();
    assert!(PreparedUpdate::from_anonymous_bytes(signed.envelope_bytes(), target, 4096).is_err());
    assert!(PreparedRequestStatus::from_anonymous_bytes(&read, &signed, 4096).is_err());
    for index in 0..6 {
        let mut bad = status.clone();
        let ingress::Content::ReadState { sender, paths, .. } = &mut bad.content else {
            unreachable!()
        };
        match index {
            0 => *sender = target,
            1 => paths.push(paths[0].clone()),
            2 => paths[0][1][0] ^= 1,
            3 => paths[0][0] = b"time".to_vec(),
            4 => paths[0].push(vec![]),
            _ => paths.clear(),
        }
        assert!(
            PreparedRequestStatus::from_anonymous_bytes(
                &ingress::encode_envelope(&bad, 4096).unwrap(),
                &update,
                4096
            )
            .is_err()
        );
    }
    let mut wrong_sender = envelope.clone();
    let ingress::Content::Call { sender, .. } = &mut wrong_sender.content else {
        unreachable!()
    };
    *sender = target;
    assert!(
        PreparedUpdate::from_anonymous_bytes(
            &ingress::encode_envelope(&wrong_sender, 4096).unwrap(),
            target,
            4096
        )
        .is_err()
    );
    for source in [envelope, status] {
        // Present null is not absence for any authentication field.
        for field in ["sender_pubkey", "sender_sig", "sender_delegation"] {
            let serde_cbor::Value::Map(mut value) = serde_cbor::value::to_value(&source).unwrap()
            else {
                unreachable!()
            };
            value.insert(
                serde_cbor::Value::Text(field.into()),
                serde_cbor::Value::Null,
            );
            let null = serde_cbor::to_vec(&value).unwrap();
            assert!(PreparedUpdate::from_anonymous_bytes(&null, target, 4096).is_err());
            assert!(PreparedRequestStatus::from_anonymous_bytes(&null, &update, 4096).is_err());
        }

        for index in 0..3 {
            let mut mixed = source.clone();
            match index {
                0 => mixed.sender_pubkey = Some(vec![]),
                1 => mixed.sender_sig = Some(vec![]),
                _ => mixed.sender_delegation = Some(vec![]),
            }
            let mixed = ingress::encode_envelope(&mixed, 4096).unwrap();
            assert!(PreparedUpdate::from_anonymous_bytes(&mixed, target, 4096).is_err());
            assert!(PreparedRequestStatus::from_anonymous_bytes(&mixed, &update, 4096).is_err());
        }
        let original = ingress::encode_envelope(&source, 4096).unwrap();
        for n in 0..original.len() {
            assert!(PreparedUpdate::from_anonymous_bytes(&original[..n], target, 4096).is_err());
            assert!(
                PreparedRequestStatus::from_anonymous_bytes(&original[..n], &update, 4096).is_err()
            );
        }
        let mut trailing = original;
        trailing.push(0);
        assert!(PreparedUpdate::from_anonymous_bytes(&trailing, target, 4096).is_err());
        assert!(PreparedRequestStatus::from_anonymous_bytes(&trailing, &update, 4096).is_err());
    }
}

#[test]
fn anonymous_dispatch_and_recovery_preserve_exact_bytes_without_retry() {
    use candidate_platform::native::PreparedRequestStatus;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let target = Principal::from_slice(&[9]);
    // Independent CBOR framing through serde's value encoder, not the product encoder.
    let content = serde_cbor::value::to_value(ingress::Content::Call {
        nonce: None,
        ingress_expiry: 1,
        sender: Principal::anonymous(),
        canister_id: target,
        method_name: "echo".into(),
        arg: vec![],
        sender_info: None,
    })
    .unwrap();
    let bytes =
        serde_cbor::to_vec(&std::collections::BTreeMap::from([("content", content)])).unwrap();
    let update = PreparedUpdate::from_anonymous_bytes(&bytes, target, 4096).unwrap();
    let (endpoint, task) = serve(b"HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\n\r\n".to_vec());
    assert!(matches!(
        runtime.block_on(client(endpoint).dispatch(&update, time())),
        Ok(Dispatch::Accepted)
    ));
    assert_eq!(task.join().unwrap().1, bytes);
    let content = serde_cbor::value::to_value(ingress::Content::ReadState {
        sender: Principal::anonymous(),
        ingress_expiry: 2,
        paths: vec![vec![
            b"request_status".to_vec(),
            update.request_id().to_vec(),
        ]],
    })
    .unwrap();
    let bytes =
        serde_cbor::to_vec(&std::collections::BTreeMap::from([("content", content)])).unwrap();
    let read = PreparedRequestStatus::from_anonymous_bytes(&bytes, &update, 4096).unwrap();
    let (endpoint, task) = serve(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n\xa0".to_vec());
    assert!(
        runtime
            .block_on(client(endpoint).observe_prepared_request(&update, &read, time()))
            .is_err()
    );
    assert_eq!(task.join().unwrap().1, bytes);
}
