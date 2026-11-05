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
mod tests;
