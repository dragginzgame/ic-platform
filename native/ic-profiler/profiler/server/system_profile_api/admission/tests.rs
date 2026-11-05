use super::*;
use axum::{Router, body::Body, routing::get};
use tower::ServiceExt;

#[tokio::test]
async fn route_admission_covers_reads_writes_and_browser_origins() {
    for path in [
        "/api/profiling/runs",
        "/api/profiling/import",
        "/memory_access_viewer",
        "/instances/0/read/capture_memory_snapshot",
        "/instances/0/read/delete_memory_snapshot",
        "/instances/0/read/get_memory_access_trace",
        "/instances/0/update/start_instruction_profiling",
        "/instances/0/update/start_memory_access_tracing",
        "/instances/0/update/stop_memory_access_tracing",
        "/instances/0/update/clear_memory_access_trace",
    ] {
        for method in ["GET", "POST", "DELETE", "OPTIONS"] {
            for (loopback, host, origin, allowed) in [
                (true, "localhost:8080", None, true),
                (true, "127.0.0.1:8080", Some("http://127.0.0.1:8080"), true),
                (true, "[::1]:8080", Some("http://[::1]:8080"), true),
                (true, "attacker.example:8080", None, false),
                (
                    true,
                    "attacker.example:8080",
                    Some("http://attacker.example:8080"),
                    false,
                ),
                (
                    true,
                    "localhost:8080",
                    Some("http://attacker.example"),
                    false,
                ),
                (true, "localhost:8080", Some("http://localhost:8081"), false),
                (true, "localhost:8080", Some("null"), false),
                (true, "localhost:8080", Some("file://localhost:8080"), false),
                (
                    true,
                    "localhost:8080",
                    Some("http://localhost:8080/path"),
                    false,
                ),
                (
                    true,
                    "localhost:8080",
                    Some("http://localhost:8080, http://evil"),
                    false,
                ),
                (true, "localhost:99999", None, false),
                (true, "", None, false),
                (false, "profiler.example:8080", None, true),
                (
                    false,
                    "profiler.example:8080",
                    Some("http://profiler.example:8080"),
                    true,
                ),
                (
                    false,
                    "profiler.example:8080",
                    Some("http://attacker.example"),
                    false,
                ),
            ] {
                let app = Router::new()
                    .route(
                        path,
                        get(|| async { StatusCode::NO_CONTENT })
                            .post(|| async { StatusCode::NO_CONTENT })
                            .delete(|| async { StatusCode::NO_CONTENT })
                            .options(|| async { StatusCode::NO_CONTENT }),
                    )
                    .layer(axum::middleware::from_fn(protect_bound_requests))
                    .layer(axum::Extension(ListenerAdmission(loopback)));
                let mut request = Request::builder()
                    .method(method)
                    .uri(path)
                    .header(header::HOST, host);
                if let Some(origin) = origin {
                    request = request.header(header::ORIGIN, origin);
                }
                let response = app
                    .oneshot(request.body(Body::empty()).unwrap())
                    .await
                    .unwrap();
                assert_eq!(
                    response.status(),
                    if allowed {
                        StatusCode::NO_CONTENT
                    } else {
                        StatusCode::FORBIDDEN
                    },
                    "{loopback} {host} {origin:?} {method} {path}"
                );
            }
        }
    }
}

#[tokio::test]
async fn duplicate_headers_and_forwarded_hosts_do_not_bypass_admission() {
    for duplicate in [header::HOST, header::ORIGIN] {
        let app = Router::new()
            .route("/", get(|| async { StatusCode::NO_CONTENT }))
            .layer(axum::middleware::from_fn_with_state(true, protect_requests));
        let request = Request::builder()
            .uri("/")
            .header(header::HOST, "localhost")
            .header(header::ORIGIN, "http://localhost")
            .header(duplicate, "evil.example")
            .header("x-forwarded-host", "localhost")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.oneshot(request).await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
    }
}

#[tokio::test]
async fn missing_listener_configuration_fails_closed() {
    let app = Router::new()
        .route("/", get(|| async { StatusCode::NO_CONTENT }))
        .layer(axum::middleware::from_fn(protect_bound_requests));
    let request = Request::builder()
        .uri("/")
        .header(header::HOST, "localhost")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.oneshot(request).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn native_api_method_router_preserves_unrelated_routes() {
    let app = aide::axum::ApiRouter::new()
        .api_route(
            "/instances/0/read/get_memory_access_trace",
            aide::axum::routing::post(|| async { StatusCode::NO_CONTENT })
                .layer(axum::middleware::from_fn(protect_bound_requests)),
        )
        .api_route(
            "/status",
            aide::axum::routing::get(|| async { StatusCode::NO_CONTENT }),
        )
        .finish_api(&mut aide::openapi::OpenApi::default())
        .layer(axum::Extension(ListenerAdmission(true)));
    for (method, path, expected) in [
        (
            "POST",
            "/instances/0/read/get_memory_access_trace",
            StatusCode::FORBIDDEN,
        ),
        ("GET", "/status", StatusCode::NO_CONTENT),
    ] {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, "rebound.example")
            .header(header::ORIGIN, "http://rebound.example")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            expected
        );
    }
}
