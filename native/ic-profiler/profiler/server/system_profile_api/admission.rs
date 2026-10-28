//! Shared HTTP admission for profiling routes on either listener.
use axum::{
    extract::{Request, State},
    http::{StatusCode, header, uri::Authority},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{net::IpAddr, str::FromStr};

/// Admission follows the actual bound listener, including explicit remote binds.
#[derive(Clone, Copy)]
pub struct ListenerAdmission(pub bool);

/// Installed only on profiling routes. Missing listener configuration fails closed.
pub async fn protect_bound_requests(request: Request, next: Next) -> Response {
    let Some(listener) = request.extensions().get::<ListenerAdmission>().copied() else {
        return StatusCode::FORBIDDEN.into_response();
    };
    protect_requests(State(listener.0), request, next).await
}

/// The listener's actual bound address selects DNS-rebinding protection.
/// Nonbrowser clients may omit Origin; proxy headers confer no authority.
pub(crate) async fn protect_requests(
    State(loopback): State<bool>,
    request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    if headers.get_all(header::HOST).iter().count() > 1
        || headers.get_all(header::ORIGIN).iter().count() > 1
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    let host = match headers.get(header::HOST) {
        Some(value) => value.to_str().unwrap_or_default(),
        None => request
            .uri()
            .authority()
            .map(Authority::as_str)
            .unwrap_or_default(),
    };
    if !valid_authority(host) || (loopback && !is_local_host(host)) {
        return (
            StatusCode::FORBIDDEN,
            "Profiling request host is not admitted.",
        )
            .into_response();
    }
    if let Some(origin) = headers.get(header::ORIGIN) {
        // A browser Origin is a scheme and authority, never a path, credentials,
        // opaque origin, or list. Preserve the existing exact host/port policy.
        let allowed = origin.to_str().ok().is_some_and(|origin| {
            ["http://", "https://"].iter().any(|scheme| {
                origin
                    .strip_prefix(scheme)
                    .is_some_and(|authority| authority == host)
            })
        });
        if !allowed {
            return (
                StatusCode::FORBIDDEN,
                "Cross-origin profiling requests are not admitted.",
            )
                .into_response();
        }
    }
    next.run(request).await
}

fn valid_authority(host: &str) -> bool {
    let Ok(authority) = host.parse::<Authority>() else {
        return false;
    };
    let suffix = authority
        .as_str()
        .strip_prefix(authority.host())
        .unwrap_or_default();
    !host.contains('@')
        && (suffix.is_empty()
            || suffix
                .strip_prefix(':')
                .and_then(|port| port.parse::<u16>().ok())
                .is_some())
}

pub(crate) fn is_local_host(host: &str) -> bool {
    if !valid_authority(host) {
        return false;
    }
    let Ok(authority) = host.parse::<Authority>() else {
        return false;
    };
    let name = authority.host();
    let name = name
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(name);
    let name = name.to_ascii_lowercase();
    name == "localhost" || name.ends_with(".localhost") || IpAddr::from_str(&name).is_ok()
}

#[cfg(test)]
mod tests {
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
}
