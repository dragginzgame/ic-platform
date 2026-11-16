//! Independent local HTTP fault injection for the anonymous native client.
#![forbid(unsafe_code)]
#![cfg(all(feature = "native-read", not(target_family = "wasm")))]
use candidate_platform::{
    certificate::{PreparedRootKey, TimePolicy, TimeWindow, VerificationLimits},
    native::{ReadClient, ReadConfig, ReadError},
};
use candidate_types::Principal;
use std::{
    io::{Read, Write},
    net::TcpListener,
    time::Duration,
};
fn config(endpoint: String) -> ReadConfig {
    ReadConfig {
        endpoint,
        root: PreparedRootKey::from_der(include_bytes!("../fixtures/mainnet-root.der")).unwrap(),
        root_subnet: Principal::anonymous(),
        message_bytes: 4096,
        request_timeout: Duration::from_secs(1),
        verification: VerificationLimits {
            tree_nodes: 128,
            tree_depth: 32,
            delegation_bytes: 4096,
            canister_ranges: 32,
        },
    }
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
fn serve(reply: Vec<u8>) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let task = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = [0; 8192];
        let n = stream.read(&mut request).unwrap();
        assert!(request[..n].starts_with(b"POST /api/v3/subnet/"));
        let _ = stream.write_all(&reply);
    });
    (format!("http://{address}"), task)
}
#[test]
fn bounded_http_refuses_declared_and_streamed_oversize_before_certificate_admission() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    for reply in [
        b"HTTP/1.1 200 OK\r\nContent-Length: 4097\r\n\r\n".to_vec(),
        [
            b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n".as_slice(),
            &[0; 4097],
        ]
        .concat(),
    ] {
        let (endpoint, task) = serve(reply);
        let client = ReadClient::new(config(endpoint)).unwrap();
        assert!(matches!(
            runtime.block_on(client.read_subnet(Principal::anonymous(), 200, time())),
            Err(ReadError::ByteLimit)
        ));
        task.join().unwrap();
    }
    let (endpoint, task) = serve(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n\xa0".to_vec());
    let client = ReadClient::new(config(endpoint)).unwrap();
    assert!(matches!(
        runtime.block_on(client.read_subnet(Principal::anonymous(), 200, time())),
        Err(ReadError::Packet)
    ));
    task.join().unwrap();
}
#[test]
fn explicit_configuration_rejects_hidden_endpoint_components() {
    for endpoint in [
        "ftp://example.invalid",
        "https://user:secret@example.invalid",
        "https://example.invalid/path",
        "https://example.invalid/?x=1",
        "https://example.invalid/#fragment",
    ] {
        assert!(matches!(
            ReadClient::new(config(endpoint.into())),
            Err(ReadError::Configuration)
        ));
    }
}
