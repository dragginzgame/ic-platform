use async_trait::async_trait;
use bytes::Bytes;
use http::{Method, Request, Response, StatusCode};
use ic_agent::{Agent, AgentError, agent::HttpService};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
    },
    time::Instant,
};

struct Counter;
static TRACK: AtomicBool = AtomicBool::new(false);
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if TRACK.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size(), Relaxed);
        }
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        if TRACK.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(n, Relaxed);
        }
        unsafe { System.realloc(p, l, n) }
    }
}
#[global_allocator]
static GLOBAL: Counter = Counter;

#[derive(Debug)]
struct Capture {
    expected: Vec<u8>,
    attempts: usize,
    status: StatusCode,
    verify: bool,
}
#[async_trait]
impl HttpService for Capture {
    async fn call<'a>(
        &'a self,
        req: &'a (dyn Fn() -> Result<Request<Bytes>, AgentError> + Send + Sync),
        retries: usize,
        limit: Option<usize>,
    ) -> Result<Response<Bytes>, AgentError> {
        assert_eq!(retries, 3);
        assert_eq!(limit, Some(4096));
        let mut retained = Vec::new();
        for _ in 0..self.attempts {
            let r = req()?;
            assert_eq!(r.method(), Method::POST);
            assert_eq!(r.uri().path(), "/api/v3/canister/aaaaa-aa/query");
            assert_eq!(r.headers()["content-type"], "application/cbor");
            if self.verify {
                assert_eq!(r.body().as_ref(), self.expected);
            }
            // Keep requests alive across all factory calls, as an async transport may.
            retained.push(r);
        }
        for pair in retained.windows(2) {
            assert_eq!(pair[0].body().as_ptr(), pair[1].body().as_ptr());
        }
        black_box(retained);
        Ok(Response::builder()
            .status(self.status)
            .header("content-type", "application/cbor")
            .body(Bytes::from_static(b"reply"))
            .unwrap())
    }
}
const ENDPOINT: &str = "api/v3/canister/aaaaa-aa/query";
fn agent(expected: Vec<u8>, attempts: usize, status: StatusCode, verify: bool) -> Agent {
    Agent::builder()
        .with_url("http://localhost/")
        .with_max_tcp_error_retries(3)
        .with_max_response_body_size(4096)
        .with_arc_http_middleware(Arc::new(Capture {
            expected,
            attempts,
            status,
            verify,
        }))
        .build()
        .unwrap()
}
fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    for size in [0, 1, 1024, 65536, 1048576, 2097152] {
        let data: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
        for attempts in [1, 4] {
            for status in [
                StatusCode::OK,
                StatusCode::ACCEPTED,
                StatusCode::BAD_REQUEST,
                StatusCode::TOO_MANY_REQUESTS,
                StatusCode::SERVICE_UNAVAILABLE,
                StatusCode::NO_CONTENT,
            ] {
                let a = agent(data.clone(), attempts, status, true);
                let result = rt.block_on(a.execute(Method::POST, ENDPOINT, Some(data.clone())));
                if status == StatusCode::OK || status == StatusCode::ACCEPTED {
                    assert_eq!(result.unwrap(), (status, b"reply".to_vec()));
                } else if status.is_client_error() || status.is_server_error() {
                    let Err(AgentError::HttpError(e)) = result else {
                        panic!("wrong error");
                    };
                    assert_eq!(e.status, status.as_u16());
                    assert_eq!(e.content, b"reply");
                    assert_eq!(e.content_type.as_deref(), Some("application/cbor"));
                } else {
                    assert!(matches!(result, Err(AgentError::InvalidHttpResponse(_))));
                }
            }
            let a = agent(Vec::new(), attempts, StatusCode::OK, false);
            let mut samples = Vec::new();
            let mut counts = (0, 0);
            for round in 0..31 {
                let input = data.clone();
                ALLOCS.store(0, Relaxed);
                BYTES.store(0, Relaxed);
                TRACK.store(true, Relaxed);
                let start = Instant::now();
                black_box(
                    rt.block_on(a.execute(Method::POST, ENDPOINT, Some(input)))
                        .unwrap(),
                );
                let elapsed = start.elapsed().as_nanos();
                TRACK.store(false, Relaxed);
                counts = (ALLOCS.load(Relaxed), BYTES.load(Relaxed));
                if round > 0 {
                    samples.push(elapsed);
                }
            }
            samples.sort_unstable();
            println!(
                "{size}\t{attempts}\t{}\t{}\t{}",
                samples[15], counts.0, counts.1
            );
        }
    }
    eprintln!("PASS: 72 byte/replay/status cases; factory bodies share storage");
}
