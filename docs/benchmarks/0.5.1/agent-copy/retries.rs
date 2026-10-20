use http::Method;
use ic_agent::{Agent, AgentError};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::Relaxed},
    },
    thread,
    time::Duration,
};
fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    for (statuses, limit, expected_status) in [
        (vec![503, 429, 200], 4096, Some(200)),
        (vec![503; 7], 4096, Some(503)),
        (vec![400], 4096, Some(400)),
        (vec![200], 2, None),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let done = Arc::new(AtomicBool::new(false));
        let stop = done.clone();
        let expected = statuses.len();
        let server = thread::spawn(move || {
            let mut count = 0;
            while !stop.load(Relaxed) {
                let (mut stream, _) = match listener.accept() {
                    Ok(s) => s,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(e) => panic!("{e}"),
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut data = Vec::new();
                let mut buf = [0u8; 4096];
                let end = loop {
                    let n = stream.read(&mut buf).unwrap();
                    assert!(n > 0);
                    data.extend_from_slice(&buf[..n]);
                    if let Some(p) = data.windows(4).position(|b| b == b"\r\n\r\n") {
                        break p + 4;
                    }
                };
                let header = String::from_utf8(data[..end].to_vec())
                    .unwrap()
                    .to_lowercase();
                assert!(header.starts_with("post /api/v3/canister/aaaaa-aa/query "));
                assert!(header.contains("content-type: application/cbor"));
                let size: usize = header
                    .lines()
                    .find_map(|s| s.strip_prefix("content-length: "))
                    .unwrap()
                    .parse()
                    .unwrap();
                while data.len() < end + size {
                    let n = stream.read(&mut buf).unwrap();
                    assert!(n > 0);
                    data.extend_from_slice(&buf[..n]);
                }
                assert_eq!(&data[end..], vec![0xA5; 65536]);
                let status = statuses[count];
                count += 1;
                let reply = format!(
                    "HTTP/1.1 {status} Fixture\r\nContent-Length: 5\r\nContent-Type: application/cbor\r\nConnection: close\r\n\r\nreply"
                );
                stream.write_all(reply.as_bytes()).unwrap();
            }
            count
        });
        let agent = Agent::builder()
            .with_url(url)
            .with_max_tcp_error_retries(3)
            .with_max_response_body_size(limit)
            .build()
            .unwrap();
        let result = rt.block_on(agent.execute(
            Method::POST,
            "api/v3/canister/aaaaa-aa/query",
            Some(vec![0xA5; 65536]),
        ));
        done.store(true, Relaxed);
        assert_eq!(server.join().unwrap(), expected);
        match expected_status {
            Some(200) => assert_eq!(result.unwrap().1, b"reply"),
            Some(status) => {
                let Err(AgentError::HttpError(e)) = result else {
                    panic!("wrong error");
                };
                assert_eq!(e.status, status);
                assert_eq!(e.content, b"reply");
            }
            None => assert!(matches!(result, Err(AgentError::TransportError(_)))),
        }
    }
    println!(
        "PASS: default HTTP retries 503/429→200, 7-attempt cap, immediate 400, response limit; exact 64KiB bodies on every attempt"
    );
}
