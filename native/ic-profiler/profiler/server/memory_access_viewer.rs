//! The memory access viewer: a web app and a JSON API, including an MCP
//! endpoint for agents, to explore the Wasm memory accesses recorded by memory
//! access tracing in all PocketIC instances of the server. It is served on a
//! separate port and reads the recorded executions directly from the
//! recorders, so it shows them while the instances keep executing.

use super::rendering::merge_ranges as union;
use crate::InstanceId;
use crate::memory_access_tracing::{ExecutionFilter, server_identity};
use crate::pocket_ic::{StartInstructionProfiling, StartMemoryAccessTracing};
use crate::state_api::routes::{AppState, run_operation};
use crate::state_api::state::InstanceRecorder;
use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, Query, Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use ic_replicated_state::canister_state::WASM_PAGE_SIZE_IN_BYTES;
use ic_types::CanisterId;
use pocket_ic::common::rest::{
    ApiResponse, MemoryAccessViewerConfig, MemoryAccessViewerInfo, TracedExecution,
    TracedExecutionOutcome,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const VIEWER_HTML: &str = include_str!("../viewer/viewer.html");
const AGENT_GUIDE: &str = include_str!("../viewer/agent_guide.md");
// The web app reads its data from this element, which is replaced to make it
// load the traces from the API.
const DATA_PLACEHOLDER: &str = r#"<script type="application/json" id="trace-data">null</script>"#;
const LIVE_DATA: &str = r#"<script type="application/json" id="trace-data">{"live":{"api":"api","system_profiling":true}}</script>"#;
const PAGE_SIZE: u64 = WASM_PAGE_SIZE_IN_BYTES as u64;
const DEFAULT_LIMIT: usize = 1_000;
const DEFAULT_TOOL_LIMIT: usize = 100;
const MAX_LIMIT: usize = 50_000;
const DEFAULT_TOP: usize = 20;
const MAX_TOP: usize = 1_000;
const MAX_MCP_BATCH: usize = 64;
const MAX_TOOL_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const MAX_MCP_BATCH_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const FORMAT_VERSION: u64 = 2;
const START_TRACING_TIMEOUT: Duration = Duration::from_secs(10);
const MCP_PROTOCOL_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

#[derive(Clone)]
struct Viewer {
    app: AppState,
    // Whether the viewer is only reachable from this machine, in which case
    // requests must address it by `localhost` or an IP
    // address (and not by a domain name) to prevent DNS rebinding attacks.
    loopback: bool,
}

type ApiResult = Result<Value, (StatusCode, String)>;

/// Serves the memory access viewer at the given address unless it is served
/// already, and returns the address at which it is served. Must be called
/// within a Tokio runtime.
pub fn start_memory_access_viewer(
    app: AppState,
    ip_addr: &str,
    port: u16,
) -> Result<SocketAddr, String> {
    let mut viewer_addr = app.api_state.memory_access_viewer().lock().unwrap();
    if let Some(addr) = *viewer_addr {
        return Ok(addr);
    }
    let bind_error = |e: std::io::Error| {
        format!("Failed to serve the memory access viewer at {ip_addr}:{port}: {e}")
    };
    let listener = std::net::TcpListener::bind((ip_addr, port)).map_err(bind_error)?;
    listener.set_nonblocking(true).map_err(bind_error)?;
    let addr = listener.local_addr().map_err(bind_error)?;
    let listener = tokio::net::TcpListener::from_std(listener).map_err(bind_error)?;
    *viewer_addr = Some(addr);
    drop(viewer_addr);
    let router = router(Viewer {
        app,
        loopback: addr.ip().is_loopback(),
    });
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            tracing::error!("The memory access viewer failed: {e}");
        }
    });
    Ok(addr)
}

/// Serves the memory access viewer unless it is served already.
pub async fn handler_start_memory_access_viewer(
    State(app): State<AppState>,
    axum::extract::Json(config): axum::extract::Json<MemoryAccessViewerConfig>,
) -> (StatusCode, Json<ApiResponse<MemoryAccessViewerInfo>>) {
    let ip_addr = config.ip_addr.unwrap_or("127.0.0.1".to_string());
    match start_memory_access_viewer(app, &ip_addr, config.port.unwrap_or_default()) {
        Ok(addr) => (
            StatusCode::OK,
            Json(ApiResponse::Success(MemoryAccessViewerInfo {
                ip_addr: addr.ip().to_string(),
                port: addr.port(),
            })),
        ),
        Err(message) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::Error { message }),
        ),
    }
}

fn router(viewer: Viewer) -> Router {
    let canister = "/api/instances/{instance_id}/canisters/{canister_id}";
    Router::new()
        .route("/", get(index))
        .route("/llms.txt", get(agent_guide))
        .route("/api", get(agent_guide))
        .route("/api/traces", get(list_traces))
        .route("/api/charge-rules", get(get_charge_rules))
        .route(
            "/api/computation-catalog",
            get(crate::computation_catalog::handler),
        )
        .route(&format!("{canister}/summary"), get(get_summary))
        .route(&format!("{canister}/executions"), get(get_executions))
        .route(&format!("{canister}/profiles"), get(get_profiles))
        .route(&format!("{canister}/runtime-costs"), get(get_runtime_costs))
        .route(&format!("{canister}/pages/{{page}}"), get(get_page_history))
        .route(&format!("{canister}/start"), post(start_tracing))
        .route(&format!("{canister}/profile"), post(start_profiling))
        .route(&format!("{canister}/stop"), post(stop_tracing))
        .route(&format!("{canister}/clear"), post(clear_trace))
        .route(
            &format!("{canister}/snapshots"),
            post(capture_memory_snapshot),
        )
        .route(
            "/api/memory-snapshots/{snapshot_id}",
            get(memory_snapshot_summary).delete(delete_memory_snapshot),
        )
        .route(
            "/api/memory-snapshots/{snapshot_id}/image.png",
            get(memory_snapshot_png),
        )
        .route("/mcp", post(mcp).get(mcp_get).delete(mcp_get))
        .with_state(viewer.clone())
        .merge(crate::system_profile_api::router(viewer.app.clone()))
        .layer(axum::Extension(
            crate::system_profile_api::ListenerAdmission(viewer.loopback),
        ))
        .layer(middleware::from_fn_with_state(
            viewer.clone(),
            protect_state_changes,
        ))
        .layer(middleware::from_fn_with_state(viewer.clone(), keep_alive))
        .layer(tower_http::compression::CompressionLayer::new())
}

// ----------------------------------------------------------------------------
// Middleware

/// Counts requests to the viewer as requests to the PocketIC server so that
/// the server stays alive while the viewer is used, e.g., to look at the
/// traces of a test that has finished.
async fn keep_alive(State(viewer): State<Viewer>, request: Request, next: Next) -> Response {
    let _guard = PendingRequest::new(&viewer.app);
    next.run(request).await
}

struct PendingRequest(AppState);

impl PendingRequest {
    fn new(app: &AppState) -> Self {
        app.pending_requests.fetch_add(1, Ordering::SeqCst);
        Self(app.clone())
    }
}

impl Drop for PendingRequest {
    fn drop(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.0.min_alive_until.fetch_max(now, Ordering::SeqCst);
        self.0.pending_requests.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Rejects requests from other websites, including reads: a rebound host
/// must not be able to read private trace data from a loopback viewer.
async fn protect_state_changes(
    State(viewer): State<Viewer>,
    request: Request,
    next: Next,
) -> Response {
    crate::system_profile_api::protect_requests(State(viewer.loopback), request, next).await
}

// ----------------------------------------------------------------------------
// Web app and guide

async fn index() -> Html<String> {
    Html(VIEWER_HTML.replacen(DATA_PLACEHOLDER, LIVE_DATA, 1))
}

async fn agent_guide(headers: HeaderMap) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
        AGENT_GUIDE.replace("{base}", &base_url(&headers)),
    )
}

fn base_url(headers: &HeaderMap) -> String {
    let host = headers
        .get(header::HOST)
        .and_then(|host| host.to_str().ok())
        .unwrap_or("127.0.0.1");
    format!("http://{host}")
}

// ----------------------------------------------------------------------------
// JSON API

#[derive(Deserialize)]
struct ExecutionsQuery {
    limit: Option<usize>,
    #[serde(flatten)]
    selection: ExecutionSelection,
}

#[derive(Deserialize)]
struct SummaryQuery {
    top: Option<usize>,
    #[serde(flatten)]
    selection: ExecutionSelection,
}

#[derive(Default, Deserialize)]
struct ExecutionSelection {
    #[serde(default, deserialize_with = "query_u64")]
    after: Option<u64>,
    #[serde(default, deserialize_with = "query_u64")]
    before: Option<u64>,
    function: Option<String>,
    outcome: Option<TracedExecutionOutcome>,
    module_hash: Option<String>,
    #[serde(default, deserialize_with = "query_u64")]
    attempt_id: Option<u64>,
}

// Flattened serde_urlencoded fields arrive through serde's generic value
// representation, which retains query integers as strings.
fn query_u64<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<u64>, D::Error> {
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(Value::String(value)) => value.parse().map(Some).map_err(serde::de::Error::custom),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| serde::de::Error::custom("expected an unsigned 64-bit integer")),
    }
}

impl ExecutionSelection {
    fn filter(&self) -> ExecutionFilter<'_> {
        ExecutionFilter {
            after: self.after,
            before: self.before,
            function: self.function.as_deref(),
            outcome: self.outcome,
            module_hash: self.module_hash.as_deref(),
            attempt_id: self.attempt_id,
        }
    }
}

#[derive(Deserialize)]
struct LimitQuery {
    after: Option<u64>,
    limit: Option<usize>,
}

fn respond(result: ApiResult) -> Response {
    match result {
        Ok(value) => Json(encode_response(value)).into_response(),
        Err((code, message)) => (code, Json(json!({ "error": message }))).into_response(),
    }
}

/// JSON numbers cannot represent wasm64 counters and timestamps exactly in
/// JavaScript. Version the viewer format and use decimal strings for integral
/// data, retaining only the bounded routing/page-size fields as numbers.
fn encode_response(mut value: Value) -> Value {
    fn encode(value: &mut Value, field: Option<&str>) {
        // The immutable explanation dictionary uses its public schema: source
        // line numbers and rule versions are bounded u32 values.
        if field == Some("charge_rules") {
            return;
        }
        match value {
            Value::Object(object) => {
                for (key, value) in object {
                    encode(value, Some(key));
                }
            }
            Value::Array(values) => {
                for value in values {
                    encode(value, field);
                }
            }
            Value::Number(number)
                if (number.is_u64() || number.is_i64())
                    && !matches!(
                        field,
                        Some(
                            "instance_id"
                                | "page_size"
                                | "format_version"
                                | "version"
                                | "instrumentation_version"
                                | "function_index"
                                | "origin_id"
                                | "origin_ids"
                                | "block_id"
                                | "operator_ordinal"
                                | "configuration_sha256"
                                | "instrumentation_source_sha256"
                                | "module_sha256"
                        )
                    ) =>
            {
                *value = Value::String(number.to_string());
            }
            _ => {}
        }
    }
    encode(&mut value, None);
    if let Some(object) = value.as_object_mut() {
        let identity = server_identity();
        object.insert("format_version".into(), json!(FORMAT_VERSION));
        object.insert("integer_encoding".into(), json!("decimal"));
        object.insert("server_generation".into(), json!(identity.generation));
        object.insert("server_binary_hash".into(), json!(identity.binary_hash));
    }
    value
}

async fn list_traces(State(viewer): State<Viewer>, headers: HeaderMap) -> Response {
    respond(Ok(traces(&viewer, &base_url(&headers))))
}

async fn get_charge_rules() -> Response {
    respond(Ok(charge_rule_dictionary()))
}

fn charge_rule_dictionary() -> Value {
    json!({ "charge_rules": crate::instruction_charge_rules::charge_rules() })
}

async fn get_summary(
    State(viewer): State<Viewer>,
    headers: HeaderMap,
    Path((instance_id, canister_id)): Path<(InstanceId, String)>,
    Query(query): Query<SummaryQuery>,
) -> Response {
    let top = query.top.unwrap_or(DEFAULT_TOP);
    let base_url = base_url(&headers);
    respond(summary(
        &viewer,
        &base_url,
        instance_id,
        &canister_id,
        top,
        &query.selection,
    ))
}

async fn get_executions(
    State(viewer): State<Viewer>,
    Path((instance_id, canister_id)): Path<(InstanceId, String)>,
    Query(query): Query<ExecutionsQuery>,
) -> Response {
    respond(executions(
        &viewer,
        instance_id,
        &canister_id,
        query.limit.unwrap_or(DEFAULT_LIMIT),
        &query.selection,
    ))
}

async fn get_profiles(
    State(viewer): State<Viewer>,
    Path((instance_id, canister_id)): Path<(InstanceId, String)>,
    Query(query): Query<SummaryQuery>,
) -> Response {
    respond(profiles(
        &viewer,
        instance_id,
        &canister_id,
        query.top.unwrap_or(DEFAULT_TOP),
        &query.selection,
    ))
}

async fn get_runtime_costs(
    State(viewer): State<Viewer>,
    Path((instance_id, canister_id)): Path<(InstanceId, String)>,
    Query(query): Query<LimitQuery>,
) -> Response {
    respond(runtime_costs(
        &viewer,
        instance_id,
        &canister_id,
        query.after,
        query.limit.unwrap_or(DEFAULT_LIMIT),
    ))
}

async fn get_page_history(
    State(viewer): State<Viewer>,
    Path((instance_id, canister_id, page)): Path<(InstanceId, String, u64)>,
    Query(query): Query<LimitQuery>,
) -> Response {
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
    respond(page_history(
        &viewer,
        instance_id,
        &canister_id,
        page,
        query.after,
        limit,
    ))
}

// The requests that change state take a JSON body so that browsers do not
// send them from other websites without asking the viewer first (CORS).
async fn start_tracing(
    State(viewer): State<Viewer>,
    Path((instance_id, canister_id)): Path<(InstanceId, String)>,
    Json(_): Json<Value>,
) -> Response {
    respond(control(&viewer, instance_id, &canister_id, Control::Start).await)
}

async fn stop_tracing(
    State(viewer): State<Viewer>,
    Path((instance_id, canister_id)): Path<(InstanceId, String)>,
    Json(_): Json<Value>,
) -> Response {
    respond(control(&viewer, instance_id, &canister_id, Control::Stop).await)
}

async fn clear_trace(
    State(viewer): State<Viewer>,
    Path((instance_id, canister_id)): Path<(InstanceId, String)>,
    Json(body): Json<Value>,
) -> Response {
    if let Err(error) = clear_server_generation(&body) {
        return respond(Err(error));
    }
    let expected = match clear_condition(&body) {
        Ok(expected) => expected,
        Err(message) => return respond(Err((StatusCode::BAD_REQUEST, message))),
    };
    respond(control(&viewer, instance_id, &canister_id, Control::Clear(expected)).await)
}

fn clear_server_generation(body: &Value) -> Result<(), (StatusCode, String)> {
    let expected = body
        .get("expected_server_generation")
        .or_else(|| body.get("server_generation"));
    if let Some(expected) = expected {
        let expected = expected.as_str().ok_or((
            StatusCode::BAD_REQUEST,
            "expected_server_generation must be a string.".to_string(),
        ))?;
        if expected != server_identity().generation {
            return Err((StatusCode::CONFLICT, "The server restarted after the snapshot; download the current trace before clearing.".to_string()));
        }
    }
    Ok(())
}

async fn start_profiling(
    State(viewer): State<Viewer>,
    Path((instance_id, canister_id)): Path<(InstanceId, String)>,
    Json(body): Json<Value>,
) -> Response {
    let profile = match ProfileRequest::parse(&body) {
        Ok(profile) => profile,
        Err(message) => return respond(Err((StatusCode::BAD_REQUEST, message))),
    };
    respond(
        control(
            &viewer,
            instance_id,
            &canister_id,
            Control::Profile(profile),
        )
        .await,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProfileRequest {
    max_executions: u64,
    read_mask: bool,
    write_mask: bool,
}

impl ProfileRequest {
    fn parse(body: &Value) -> Result<Self, String> {
        let maximum = pocket_ic::common::rest::MAX_INSTRUCTION_PROFILING_EXECUTIONS;
        let max_executions = integer_argument(body, "max_executions")?.unwrap_or(20);
        if max_executions == 0 {
            return Err("max_executions must be positive.".to_string());
        }
        let mask = |key: &str| match body.get(key) {
            None => Ok(false),
            Some(Value::Bool(value)) => Ok(*value),
            _ => Err(format!("{key} must be a boolean")),
        };
        Ok(Self {
            max_executions: max_executions.min(maximum),
            read_mask: mask("read_mask")?,
            write_mask: mask("write_mask")?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClearCondition {
    generation: u64,
    last_sequence: Option<u64>,
    // None omits this guard; Some(None) requires no runtime-cost events.
    runtime_sequence: Option<Option<u64>>,
}

fn clear_condition(body: &Value) -> Result<Option<ClearCondition>, String> {
    if body.get("trace_generation").is_none()
        && body.get("last_sequence_number").is_none()
        && body
            .get("expected_last_runtime_cost_sequence_number")
            .is_none()
        && body.get("last_runtime_cost_sequence_number").is_none()
    {
        return Ok(None);
    }
    let generation = integer_argument(body, "trace_generation")?.ok_or(
        "Conditional clear requires trace_generation and last_sequence_number.".to_string(),
    )?;
    let last = match body.get("last_sequence_number") {
        Some(Value::Null) => None,
        Some(_) => integer_argument(body, "last_sequence_number")?,
        None => {
            return Err(
                "Conditional clear requires last_sequence_number (null for an empty trace)."
                    .to_string(),
            );
        }
    };
    let runtime_key = if body
        .get("expected_last_runtime_cost_sequence_number")
        .is_some()
    {
        "expected_last_runtime_cost_sequence_number"
    } else {
        "last_runtime_cost_sequence_number"
    };
    let runtime = match body.get(runtime_key) {
        None => None,
        Some(Value::Null) => Some(None),
        Some(_) => Some(integer_argument(body, runtime_key)?),
    };
    Ok(Some(ClearCondition {
        generation,
        last_sequence: last,
        runtime_sequence: runtime,
    }))
}

/// Returns the recorder of a PocketIC instance and the parsed canister id.
fn lookup(
    viewer: &Viewer,
    instance_id: InstanceId,
    canister_id: &str,
) -> Result<(InstanceRecorder, CanisterId), (StatusCode, String)> {
    let canister_id = CanisterId::from_str(canister_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid canister id `{canister_id}`: {e}"),
        )
    })?;
    let recorder = viewer
        .app
        .api_state
        .memory_access_recorders()
        .remove(&instance_id)
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("There is no PocketIC instance {instance_id}."),
        ))?;
    Ok((recorder, canister_id))
}

#[derive(Default, Deserialize)]
struct CaptureSnapshotBody {
    label: Option<String>,
}

async fn capture_memory_snapshot(
    State(viewer): State<Viewer>,
    headers: HeaderMap,
    Path((instance_id, canister_id)): Path<(InstanceId, String)>,
    Json(body): Json<CaptureSnapshotBody>,
) -> Response {
    respond(
        crate::memory_snapshot_api::capture(
            &viewer.app,
            &base_url(&headers),
            instance_id,
            &canister_id,
            body.label,
        )
        .await,
    )
}

async fn render_memory_snapshot(
    viewer: &Viewer,
    base_url: String,
    snapshot_id: String,
    query: crate::memory_snapshot_api::SnapshotImageQuery,
) -> crate::memory_snapshot_api::SnapshotResult<crate::memory_snapshot_image::MemorySnapshotImage> {
    let app = viewer.app.clone();
    tokio::task::spawn_blocking(move || {
        crate::memory_snapshot_api::render(&app, &base_url, &snapshot_id, query)
    })
    .await
    .map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Image rendering failed: {error}"),
        )
    })?
}

async fn memory_snapshot_summary(
    State(viewer): State<Viewer>,
    headers: HeaderMap,
    Path(snapshot_id): Path<String>,
    Query(query): Query<crate::memory_snapshot_api::SnapshotImageQuery>,
) -> Response {
    match render_memory_snapshot(&viewer, base_url(&headers), snapshot_id, query).await {
        Ok(image) => ([(header::CACHE_CONTROL, "no-store")], Json(image.summary)).into_response(),
        Err(error) => respond(Err(error)),
    }
}

async fn memory_snapshot_png(
    State(viewer): State<Viewer>,
    headers: HeaderMap,
    Path(snapshot_id): Path<String>,
    Query(query): Query<crate::memory_snapshot_api::SnapshotImageQuery>,
) -> Response {
    match render_memory_snapshot(&viewer, base_url(&headers), snapshot_id, query).await {
        Ok(image) => (
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            image.png,
        )
            .into_response(),
        Err(error) => respond(Err(error)),
    }
}

async fn delete_memory_snapshot(Path(snapshot_id): Path<String>) -> Response {
    respond(crate::memory_snapshot_api::delete(&snapshot_id))
}

fn canister_urls(base_url: &str, instance_id: InstanceId, canister_id: &str) -> Value {
    let api = format!("{base_url}/api/instances/{instance_id}/canisters/{canister_id}");
    json!({
        "summary": format!("{api}/summary"),
        "executions": format!("{api}/executions"),
        "profiles": format!("{api}/profiles"),
        "runtime_costs": format!("{api}/runtime-costs"),
        "charge_rules": format!("{base_url}/api/charge-rules"),
        "viewer": viewer_url(base_url, instance_id, canister_id),
    })
}

fn viewer_url(base_url: &str, instance_id: InstanceId, canister_id: &str) -> String {
    format!("{base_url}/#{instance_id}:{canister_id}")
}

fn traces(viewer: &Viewer, base_url: &str) -> Value {
    let instances: Vec<Value> = viewer
        .app
        .api_state
        .memory_access_recorders()
        .into_iter()
        .map(|(instance_id, recorder)| {
            let canisters: Vec<Value> = recorder
                .recorder
                .canisters()
                .into_iter()
                .map(|canister| {
                    let canister_id = canister.canister_id.to_string();
                    json!({
                        "canister_id": canister_id,
                        "tracing": canister.tracing && !recorder.deleted,
                        "runtime_costs_recording": canister.runtime_costs_recording && !recorder.deleted,
                        "executions": canister.executions,
                        "first_sequence_number": canister.first_sequence_number,
                        "last_sequence_number": canister.last_sequence_number,
                        "trace_generation": canister.trace_generation,
                        "truncated": canister.truncated,
                        "profile_remaining": canister.profile_remaining,
                        "urls": canister_urls(base_url, instance_id, &canister_id),
                    })
                })
                .collect();
            json!({
                "instance_id": instance_id,
                "deleted": recorder.deleted,
                "canisters": canisters,
            })
        })
        .collect();
    json!({ "page_size": PAGE_SIZE, "instances": instances })
}

fn executions(
    viewer: &Viewer,
    instance_id: InstanceId,
    canister_id: &str,
    limit: usize,
    selection: &ExecutionSelection,
) -> ApiResult {
    let (recorder, canister_id) = lookup(viewer, instance_id, canister_id)?;
    let snapshot = recorder.recorder.executions_snapshot(
        canister_id,
        limit.clamp(1, MAX_LIMIT),
        selection.filter(),
    );
    Ok(json!({
        "instance_id": instance_id,
        "canister_id": canister_id.to_string(),
        "page_size": PAGE_SIZE,
        "tracing": recorder.is_tracing(canister_id),
        "first_sequence_number": snapshot.first_sequence_number,
        "last_sequence_number": snapshot.last_sequence_number,
        "trace_generation": snapshot.trace_generation,
        "truncated": snapshot.truncated,
        "dropped_executions": snapshot.dropped_executions,
        "dropped_instructions": snapshot.dropped_instructions.to_string(),
        "dropped_instructions_incomplete": snapshot.dropped_instructions_incomplete,
        "recorded_bytes": snapshot.recorded_bytes,
        "more": snapshot.more,
        "executions": snapshot.executions,
        "modules": snapshot.modules,
        "logical_executions": snapshot.logical_executions,
        "charge_rules": crate::instruction_charge_rules::charge_rules(),
    }))
}

fn runtime_costs(
    viewer: &Viewer,
    instance_id: InstanceId,
    canister_id: &str,
    after: Option<u64>,
    limit: usize,
) -> ApiResult {
    let (recorder, canister_id) = lookup(viewer, instance_id, canister_id)?;
    let snapshot = recorder
        .recorder
        .runtime_costs_snapshot(canister_id, after, limit);
    Ok(json!({
        "instance_id": instance_id, "canister_id": canister_id.to_string(), "tracing": recorder.is_tracing(canister_id),
        "runtime_costs_recording": snapshot.runtime_costs_recording && !recorder.deleted,
        "runtime_costs": snapshot.runtime_costs, "subnet_runtime_costs": snapshot.subnet_runtime_costs,
        "runtime_cost_totals": snapshot.runtime_cost_totals, "subnet_runtime_cost_totals": snapshot.subnet_runtime_cost_totals,
        "runtime_costs_truncated": snapshot.runtime_costs_truncated, "subnet_runtime_costs_truncated": snapshot.subnet_runtime_costs_truncated,
        "runtime_cost_totals_incomplete": snapshot.runtime_cost_totals_incomplete,
        "subnet_runtime_cost_totals_incomplete": snapshot.subnet_runtime_cost_totals_incomplete,
        "last_runtime_cost_sequence_number": snapshot.last_runtime_cost_sequence_number,
        "next_after": snapshot.next_after, "more": snapshot.more, "trace_generation": snapshot.trace_generation,
        "runtime_configurations": snapshot.runtime_configurations, "charge_rules": crate::instruction_charge_rules::charge_rules(),
    }))
}

fn summary(
    viewer: &Viewer,
    base_url: &str,
    instance_id: InstanceId,
    canister_id: &str,
    top: usize,
    selection: &ExecutionSelection,
) -> ApiResult {
    let (recorder, canister_id) = lookup(viewer, instance_id, canister_id)?;
    let trace = recorder.recorder.get(canister_id);
    let executions: Vec<_> = trace
        .executions
        .into_iter()
        .filter(|execution| selection.filter().matches(execution))
        .collect();
    let mut summary = summarize(&executions, top.clamp(1, MAX_TOP));
    let tracing = recorder.is_tracing(canister_id);
    let canister_id = canister_id.to_string();
    summary["instance_id"] = json!(instance_id);
    summary["canister_id"] = json!(canister_id);
    summary["tracing"] = json!(tracing);
    summary["trace_generation"] = json!(trace.trace_generation);
    summary["truncated"] = json!(trace.truncated);
    summary["dropped_executions"] = json!(trace.dropped_executions);
    summary["dropped_instructions"] = json!(trace.dropped_instructions.to_string());
    summary["dropped_instructions_incomplete"] = json!(trace.dropped_instructions_incomplete);
    summary["recorded_bytes"] = json!(trace.recorded_bytes);
    summary["runtime_costs_recording"] = json!(trace.runtime_costs_recording && !recorder.deleted);
    summary["runtime_cost_totals"] = json!(trace.runtime_cost_totals);
    summary["subnet_runtime_cost_totals"] = json!(trace.subnet_runtime_cost_totals);
    summary["runtime_configurations"] = json!(trace.runtime_configurations);
    summary["runtime_costs_truncated"] = json!(trace.runtime_costs_truncated);
    summary["subnet_runtime_costs_truncated"] = json!(trace.subnet_runtime_costs_truncated);
    summary["runtime_cost_totals_incomplete"] = json!(trace.runtime_cost_totals_incomplete);
    summary["subnet_runtime_cost_totals_incomplete"] =
        json!(trace.subnet_runtime_cost_totals_incomplete);
    summary["urls"] = canister_urls(base_url, instance_id, &canister_id);
    summary["logical_executions"] =
        selected_logical_dictionary(&trace.logical_executions, &executions);
    Ok(summary)
}

fn profiles(
    viewer: &Viewer,
    instance_id: InstanceId,
    canister_id: &str,
    top: usize,
    selection: &ExecutionSelection,
) -> ApiResult {
    let (recorder, canister_id) = lookup(viewer, instance_id, canister_id)?;
    let trace = recorder.recorder.get(canister_id);
    let executions: Vec<_> = trace
        .executions
        .into_iter()
        .filter(|execution| selection.filter().matches(execution))
        .collect();
    let mut profile = aggregate_profiles(&executions, top.clamp(1, MAX_TOP));
    let attribution =
        allocate_profile_functions(&executions, &trace.modules, top.clamp(1, MAX_TOP));
    for flag in [
        "module_origin_map_truncated",
        "module_origin_map_unavailable",
        "stack_details_truncated",
    ] {
        profile[flag] = attribution[flag].clone();
    }
    profile["charge_attribution"] = attribution;
    profile["instance_id"] = json!(instance_id);
    profile["canister_id"] = json!(canister_id.to_string());
    profile["tracing"] = json!(recorder.is_tracing(canister_id));
    profile["trace_generation"] = json!(trace.trace_generation);
    profile["truncated"] =
        json!(trace.truncated || profile["truncated"].as_bool().unwrap_or_default());
    profile["dropped_executions"] = json!(trace.dropped_executions);
    profile["dropped_instructions"] = json!(trace.dropped_instructions.to_string());
    profile["recorded_bytes"] = json!(trace.recorded_bytes);
    profile["charge_rules"] = json!(trace.charge_rules);
    profile["runtime_costs_recording"] = json!(trace.runtime_costs_recording && !recorder.deleted);
    profile["dropped_instructions_incomplete"] = json!(trace.dropped_instructions_incomplete);
    profile["logical_executions"] =
        selected_logical_dictionary(&trace.logical_executions, &executions);
    profile["runtime_cost_totals"] = json!(trace.runtime_cost_totals);
    profile["subnet_runtime_cost_totals"] = json!(trace.subnet_runtime_cost_totals);
    profile["runtime_configurations"] = json!(trace.runtime_configurations);
    profile["runtime_costs_truncated"] = json!(trace.runtime_costs_truncated);
    profile["subnet_runtime_costs_truncated"] = json!(trace.subnet_runtime_costs_truncated);
    profile["runtime_cost_totals_incomplete"] = json!(trace.runtime_cost_totals_incomplete);
    profile["subnet_runtime_cost_totals_incomplete"] =
        json!(trace.subnet_runtime_cost_totals_incomplete);

    let referenced: BTreeSet<_> = executions
        .iter()
        .filter_map(|execution| execution.module_hash.as_deref())
        .collect();
    profile["modules"] = json!(
        trace
            .modules
            .into_iter()
            .filter(|(hash, _)| referenced.contains(hash.as_str()))
            .collect::<BTreeMap<_, _>>()
    );
    Ok(profile)
}

/// Exclusive allocation of observed charges. Missing evidence remains a
/// remainder; physical caller stacks are a separate, overlapping projection.
fn allocate_profile_functions(
    executions: &[TracedExecution],
    modules: &BTreeMap<String, pocket_ic::common::rest::WasmInstructionModuleMetadata>,
    top: usize,
) -> Value {
    #[derive(Default)]
    struct Cost {
        amount: u128,
        categories: BTreeMap<String, u128>,
    }
    let mut functions: BTreeMap<(Option<String>, Option<String>, u32), Cost> = BTreeMap::new();
    let mut origin_maps: BTreeMap<(Option<String>, Option<String>), BTreeMap<u32, u32>> =
        BTreeMap::new();
    let mut raw = 0_u128;
    let mut allocated = 0_u128;
    let mut detail_complete = true;
    let mut module_origin_map_truncated = false;
    let mut module_origin_map_unavailable = false;
    let mut stack_details_truncated = false;
    for execution in executions {
        let Some(profile) = &execution.instruction_profile else {
            continue;
        };
        raw += profile.ledger.raw_instructions as u128;
        module_origin_map_truncated |= profile.module_origin_map_truncated;
        module_origin_map_unavailable |= profile.module_origin_map_unavailable;
        stack_details_truncated |= profile.stack_details_truncated;
        detail_complete &= !execution.detail_truncated
            && !profile.truncated
            && !profile.page_details_truncated
            && !profile.stack_details_truncated
            && !profile.module_origin_map_truncated
            && !profile.module_origin_map_unavailable
            && execution
                .attempt
                .as_ref()
                .is_none_or(|attempt| attempt.instructions_available);
        let metadata = execution
            .module_hash
            .as_ref()
            .and_then(|hash| modules.get(hash).or(profile.module.as_ref()))
            .filter(|module| {
                execution.module_hash.as_deref() == Some(hex::encode(module.module_sha256).as_str())
                    && execution.module_hash_kind.as_deref()
                        == Some("installed_uncompressed_sha256")
            });
        module_origin_map_truncated |= metadata.is_some_and(|module| module.origin_map_truncated);
        module_origin_map_unavailable |= metadata.is_none();
        detail_complete &= metadata.is_some_and(|module| !module.origin_map_truncated);
        let origins = origin_maps
            .entry((
                execution.module_hash.clone(),
                execution.module_hash_kind.clone(),
            ))
            .or_insert_with(|| {
                metadata
                    .into_iter()
                    .flat_map(|module| &module.operators)
                    .map(|op| (op.origin_id, op.function_index))
                    .collect()
            });
        let mut add = |function: u32, category: &str, instructions: u64| {
            let row = functions
                .entry((
                    execution.module_hash.clone(),
                    execution.module_hash_kind.clone(),
                    function,
                ))
                .or_default();
            row.amount += instructions as u128;
            *row.categories.entry(category.to_string()).or_default() += instructions as u128;
            allocated += instructions as u128;
        };
        if profile.stack_charges.is_empty() {
            // Legacy profiles have precise static block and dynamic function
            // coordinates, but no recovered API/page operation origins.
            for block in &profile.blocks {
                add(block.function_index, "static_prepaid", block.instructions);
            }
            for charge in &profile.dynamic_charges {
                if let Some(function) = charge.function_index {
                    add(function, &charge.category, charge.instructions);
                }
            }
        } else {
            for charge in &profile.stack_charges {
                // Page records below carry the actual fault or scoped-copy
                // operation. Their aggregate stack rows must not add it twice.
                if matches!(
                    charge.category.as_str(),
                    "heap_access" | "heap_dirty" | "stable_access" | "stable_dirty"
                ) {
                    continue;
                }
                if !matches!(charge.origin_evidence.as_str(), "exact_operation" | "block") {
                    continue;
                }
                if let Some(function) = charge.origin_id.and_then(|id| origins.get(&id)) {
                    add(*function, &charge.category, charge.instructions);
                }
            }
        }
        for charge in &profile.page_charges {
            if !matches!(
                charge.origin_evidence.as_str(),
                "native_fault_pc" | "exact_host_operation"
            ) {
                continue;
            }
            if let Some(function) = charge.origin_id.and_then(|id| origins.get(&id)) {
                add(
                    *function,
                    &format!("{}_{}", charge.memory, charge.kind),
                    charge.instructions,
                );
            }
        }
    }
    let count = functions.len();
    let mut functions: Vec<_> = functions.into_iter().collect();
    functions.sort_by(|(a, x), (b, y)| y.amount.cmp(&x.amount).then(a.cmp(b)));
    let rows: Vec<_> = functions
        .into_iter()
        .take(top)
        .map(|((module_hash, module_hash_kind, function_index), cost)| {
            let categories: BTreeMap<_, _> = cost
                .categories
                .into_iter()
                .map(|(key, amount)| (key, amount.to_string()))
                .collect();
            json!({"module_hash": module_hash, "module_hash_kind": module_hash_kind,
            "function_index": function_index, "instructions": cost.amount.to_string(),
            "categories": categories, "source_status": "original_wasm_coordinates"})
        })
        .collect();
    let difference = if raw >= allocated {
        (raw - allocated).to_string()
    } else {
        format!("-{}", allocated - raw)
    };
    json!({"functions": rows, "function_count": count, "ranking_limited": count > top,
        "raw_instructions": raw.to_string(), "allocated_instructions": allocated.to_string(),
        "raw_minus_allocated_instructions": difference,
        "allocation_valid": allocated <= raw, "detail_complete": detail_complete,
        "module_origin_map_truncated": module_origin_map_truncated,
        "module_origin_map_unavailable": module_origin_map_unavailable,
        "stack_details_truncated": stack_details_truncated,
        "source_coverage": null})
}

/// The additive ledger stays separate from the allocation of prepaid blocks.
/// No source names are inferred without a matching external artifact bundle.
fn aggregate_profiles(executions: &[TracedExecution], top: usize) -> Value {
    #[derive(Default)]
    struct Function {
        instructions: u128,
        block_hits: u128,
        blocks: BTreeSet<u64>,
    }
    let mut functions: BTreeMap<(Option<String>, Option<String>, u32), Function> = BTreeMap::new();
    let mut blocks: BTreeMap<(Option<String>, Option<String>, u64), Value> = BTreeMap::new();
    let mut ledger: BTreeMap<&str, u128> = BTreeMap::new();
    let mut rules: BTreeMap<String, (u128, u128)> = BTreeMap::new();
    let mut limitations = BTreeSet::new();
    let mut adjustment = 0_i128;
    let mut unexplained = 0_i128;
    let mut profiled = 0_u64;
    let mut reconciled = true;
    let mut truncated = false;
    let mut module_origin_map_truncated = false;
    let mut module_origin_map_unavailable = false;
    let mut stack_details_truncated = false;
    for execution in executions {
        let Some(profile) = &execution.instruction_profile else {
            continue;
        };
        profiled += 1;
        truncated |= profile.truncated || execution.detail_truncated;
        module_origin_map_truncated |= profile.module_origin_map_truncated
            || profile
                .module
                .as_ref()
                .is_some_and(|module| module.origin_map_truncated);
        module_origin_map_unavailable |= profile.module_origin_map_unavailable;
        stack_details_truncated |= profile.stack_details_truncated;
        reconciled &= profile.ledger.reconciled;
        adjustment += profile.ledger.adjustment;
        unexplained += profile.ledger.unexplained_instructions;
        limitations.extend(profile.limitations.iter().cloned());
        let charges = &profile.ledger;
        for rule in &profile.rules {
            let aggregate = rules.entry(rule.rule_id.clone()).or_default();
            aggregate.0 += rule.events as u128;
            aggregate.1 += rule.instructions as u128;
        }
        for (category, value) in [
            ("static_instructions", charges.static_instructions),
            ("dynamic_instructions", charges.dynamic_instructions),
            ("api_instructions", charges.api_instructions),
            ("stable_api_instructions", charges.stable_api_instructions),
            ("heap_page_instructions", charges.heap_page_instructions),
            ("stable_page_instructions", charges.stable_page_instructions),
            ("other_known_instructions", charges.other_known_instructions),
            ("raw_instructions", charges.raw_instructions),
            ("reported_instructions", charges.reported_instructions),
        ] {
            *ledger.entry(category).or_default() += value as u128;
        }
        for block in &profile.blocks {
            let function = functions
                .entry((
                    execution.module_hash.clone(),
                    execution.module_hash_kind.clone(),
                    block.function_index,
                ))
                .or_default();
            function.instructions += block.instructions as u128;
            function.block_hits += block.hits as u128;
            function.blocks.insert(block.block_id);
            let key = (
                execution.module_hash.clone(),
                execution.module_hash_kind.clone(),
                block.block_id,
            );
            let entry = blocks.entry(key).or_insert_with(|| {
                json!({
                    "module_hash": execution.module_hash,
                    "module_hash_kind": execution.module_hash_kind,
                    "block_id": block.block_id,
                    "function_index": block.function_index,
                    "body_offset": block.body_offset,
                    "end_body_offset": block.end_body_offset,
                    "operator_ordinal": block.operator_ordinal,
                    "weight": block.weight,
                    "entry_weight": block.entry_weight,
                    "hits": "0",
                    "instructions": "0",
                })
            });
            for (field, value) in [("hits", block.hits), ("instructions", block.instructions)] {
                let sum = entry[field].as_str().unwrap().parse::<u128>().unwrap() + value as u128;
                entry[field] = json!(sum.to_string());
            }
        }
    }
    let allocated_static: u128 = functions
        .values()
        .map(|function| function.instructions)
        .sum();
    let observed_rule_instructions: u128 =
        rules.values().map(|(_, instructions)| instructions).sum();
    let raw_instructions = ledger.get("raw_instructions").copied().unwrap_or_default();
    let rule_difference = if raw_instructions >= observed_rule_instructions {
        (raw_instructions - observed_rule_instructions).to_string()
    } else {
        format!("-{}", observed_rule_instructions - raw_instructions)
    };
    let mut rules: Vec<_> = rules.into_iter().collect();
    rules.sort_by(|(a, x), (b, y)| y.1.cmp(&x.1).then(a.cmp(b)));
    let rules: Vec<_> = rules
        .into_iter()
        .map(|(rule_id, (events, instructions))| {
            json!({
                "rule_id": rule_id,
                "events": events.to_string(),
                "instructions": instructions.to_string(),
            })
        })
        .collect();
    let function_count = functions.len();
    let block_count = blocks.len();
    let mut ranked: Vec<_> = functions.into_iter().collect();
    ranked.sort_by(|(a, x), (b, y)| y.instructions.cmp(&x.instructions).then(a.cmp(b)));
    let functions: Vec<_> = ranked
        .into_iter()
        .take(top)
        .map(
            |((module_hash, module_hash_kind, function_index), function)| {
                json!({
                    "module_hash": module_hash,
                    "module_hash_kind": module_hash_kind,
                    "function_index": function_index,
                    "function": format!("Wasm function {function_index}"),
                    "static_prepaid_instructions": function.instructions.to_string(),
                    "block_hits": function.block_hits.to_string(),
                    "blocks": function.blocks.len(),
                    "source_status": "unmapped",
                })
            },
        )
        .collect();
    let mut blocks: Vec<_> = blocks.into_values().collect();
    blocks.sort_by(|a, b| {
        let cost = |value: &Value| {
            value["instructions"]
                .as_str()
                .unwrap()
                .parse::<u128>()
                .unwrap()
        };
        cost(b).cmp(&cost(a))
    });
    blocks.truncate(top);
    let mut totals: serde_json::Map<String, Value> = ledger
        .iter()
        .map(|(category, value)| (category.to_string(), json!(value.to_string())))
        .collect();
    totals.insert("adjustment".into(), json!(adjustment.to_string()));
    totals.insert(
        "unexplained_instructions".into(),
        json!(unexplained.to_string()),
    );
    totals.insert("reconciled".into(), json!(profiled > 0 && reconciled));
    limitations.insert("Function rows allocate static prepaid IC charges; they do not prove that every operator executed.".to_string());
    limitations.insert("Source and inline attribution require an exact matching artifact bundle and origin map; source coverage is currently unavailable.".to_string());
    json!({
        "executions": executions.len(),
        "profiled_executions": profiled,
        "unprofiled_executions": executions.len() as u64 - profiled,
        "whole_execution_instructions": executions.iter().map(|execution| execution.instructions as u128).sum::<u128>().to_string(),
        "ledger": totals,
        "rules": rules,
        "observed_rule_instructions": observed_rule_instructions.to_string(),
        "raw_minus_rule_instructions": rule_difference,
        "rules_cover_raw_instructions": profiled > 0 && raw_instructions == observed_rule_instructions,
        "allocated_static_instructions": allocated_static.to_string(),
        "unallocated_static_instructions": ledger.get("static_instructions").copied().unwrap_or_default().saturating_sub(allocated_static).to_string(),
        "function_count": function_count,
        "block_count": block_count,
        "functions": functions,
        "blocks": blocks,
        "ranking_limited": function_count > top || block_count > top,
        "truncated": truncated,
        "module_origin_map_truncated": module_origin_map_truncated,
        "module_origin_map_unavailable": module_origin_map_unavailable,
        "stack_details_truncated": stack_details_truncated,
        "source_coverage": null,
        "limitations": limitations,
    })
}

fn page_history(
    viewer: &Viewer,
    instance_id: InstanceId,
    canister_id: &str,
    page: u64,
    after: Option<u64>,
    limit: usize,
) -> ApiResult {
    if page >= 1_u64 << (64 - WASM_PAGE_SIZE_IN_BYTES.trailing_zeros()) {
        return Err((
            StatusCode::BAD_REQUEST,
            "The page index exceeds the wasm64 address space.".to_string(),
        ));
    }
    let (recorder, canister_id) = lookup(viewer, instance_id, canister_id)?;
    let trace = recorder.recorder.get(canister_id);
    let executions = trace.executions;
    let labels = function_labels(&executions);
    let limit = limit.clamp(1, MAX_LIMIT);
    let (mut accessed_by, mut written_by, mut writes_tracked_executions, mut eligible) =
        (0_u64, 0_u64, 0_u64, 0_u64);
    let mut listed = vec![];
    for (e, label) in executions.iter().zip(&labels) {
        if !contains(&e.accessed_pages, page) {
            continue;
        }
        let written = contains(&e.dirty_pages, page);
        accessed_by += 1;
        written_by += written as u64;
        writes_tracked_executions +=
            u64::from(e.dirty_pages_tracked && memory_details_available(e));
        if after.is_some_and(|after| e.sequence_number <= after) {
            continue;
        }
        eligible += 1;
        if listed.len() < limit {
            listed.push(json!({
                "sequence_number": e.sequence_number,
                "api_type": e.api_type,
                "function": label,
                "call_context_id": e.call_context_id,
                "time_nanos": e.time_nanos,
                "written": (e.dirty_pages_tracked && memory_details_available(e)).then_some(written),
                "dirty_pages_tracked": e.dirty_pages_tracked,
            }));
        }
    }
    Ok(json!({
        "instance_id": instance_id,
        "canister_id": canister_id.to_string(),
        "page": page,
        "offset": format!("{:#x}", page.saturating_mul(PAGE_SIZE)),
        "accessed_by": accessed_by,
        "written_by": written_by,
        "writes_tracked_executions": writes_tracked_executions,
        "unknown_access_executions": executions.iter().filter(|e| !memory_details_available(e)).count(),
        "trace_generation": trace.trace_generation,
        "truncated": trace.truncated,
        "more": eligible > listed.len() as u64,
        "next_after": listed.last().map(|e| e["sequence_number"].clone()),
        "executions": listed,
    }))
}

#[derive(Clone, Copy)]
enum Control {
    Start,
    Profile(ProfileRequest),
    Stop,
    Clear(Option<ClearCondition>),
}

async fn control(
    viewer: &Viewer,
    instance_id: InstanceId,
    canister_id: &str,
    control: Control,
) -> ApiResult {
    let (recorder, canister_id) = lookup(viewer, instance_id, canister_id)?;
    match control {
        Control::Start | Control::Profile(_) => {
            if recorder.deleted {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("The PocketIC instance {instance_id} was deleted."),
                ));
            }
            // Tracing is started by the instance, which checks that the canister exists.
            let (code, response) = if let Control::Profile(ProfileRequest {
                max_executions,
                read_mask,
                write_mask,
            }) = control
            {
                run_operation::<()>(
                    viewer.app.api_state.clone(),
                    instance_id,
                    Some(START_TRACING_TIMEOUT),
                    StartInstructionProfiling {
                        canister_id,
                        max_executions,
                        read_mask,
                        write_mask,
                    },
                )
                .await
            } else {
                run_operation::<()>(
                    viewer.app.api_state.clone(),
                    instance_id,
                    Some(START_TRACING_TIMEOUT),
                    StartMemoryAccessTracing { canister_id },
                )
                .await
            };
            match response {
                ApiResponse::Success(()) => {
                    if let Control::Profile(ProfileRequest {
                        max_executions,
                        read_mask,
                        write_mask,
                    }) = control
                    {
                        let remaining = recorder
                            .recorder
                            .canisters()
                            .into_iter()
                            .find(|canister| canister.canister_id == canister_id)
                            .map_or(0, |canister| canister.profile_remaining);
                        Ok(
                            json!({ "tracing": recorder.is_tracing(canister_id), "profile_selected": max_executions, "profile_remaining": remaining, "read_mask": read_mask, "write_mask": write_mask }),
                        )
                    } else {
                        Ok(json!({ "tracing": true }))
                    }
                }
                ApiResponse::Error { message } => Err((code, message)),
                ApiResponse::Busy { .. } | ApiResponse::Started { .. } => Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "The PocketIC instance is busy. Tracing starts once it has finished its \
                     current operation."
                        .to_string(),
                )),
            }
        }
        Control::Stop => {
            recorder.recorder.stop(canister_id);
            Ok(json!({ "tracing": false }))
        }
        Control::Clear(expected) => {
            if let Some(ClearCondition {
                generation,
                last_sequence: last,
                runtime_sequence: runtime,
            }) = expected
            {
                if !recorder
                    .recorder
                    .clear_if_unchanged(canister_id, generation, last, runtime)
                {
                    return Err((
                        StatusCode::CONFLICT,
                        "The trace changed after the snapshot; download it again before clearing."
                            .to_string(),
                    ));
                }
            } else {
                recorder.recorder.clear(canister_id);
            }
            Ok(json!({ "cleared": true }))
        }
    }
}

// ----------------------------------------------------------------------------
// Analysis

fn is_callback(api_type: &str) -> bool {
    api_type.contains("callback") || api_type.contains("cleanup")
}

/// Names response callbacks after the function that made the call, e.g.,
/// "canister_update transfer ↳ reply callback", like the web app does.
fn function_labels(executions: &[TracedExecution]) -> Vec<String> {
    // Composite queries allocate contexts in their own query graph.
    let mut origins: HashMap<(bool, u64), &TracedExecution> = HashMap::new();
    executions
        .iter()
        .map(|e| {
            if e.attempt.is_some() {
                // New traces carry explicit runtime correlation. A restored
                // call-context counter must not select the nearest old caller.
                return e.function.clone();
            }
            if !is_callback(&e.api_type) {
                // Context counters can be restored with a canister snapshot.
                // A newer initiating execution supersedes an older identity.
                if let Some(id) = e.call_context_id {
                    origins.insert((e.api_type.starts_with("composite"), id), e);
                }
                return e.function.clone();
            }
            match e
                .call_context_id
                .and_then(|id| origins.get(&(e.api_type.starts_with("composite"), id)))
            {
                Some(origin) => {
                    format!("{} ↳ {}", origin.function, e.api_type)
                }
                None => format!("{} ({})", e.function, e.api_type),
            }
        })
        .collect()
}

fn pages(ranges: &[(u64, u64)]) -> u64 {
    ranges.iter().map(|(start, end)| end - start).sum()
}

/// Whether sorted and disjoint ranges contain a page.
fn contains(ranges: &[(u64, u64)], page: u64) -> bool {
    let i = ranges.partition_point(|&(_, end)| end <= page);
    i < ranges.len() && ranges[i].0 <= page
}

/// A maximal run of pages `[start, end)` that were accessed and written by
/// the same numbers of executions.
#[derive(Debug, PartialEq)]
struct PageRun {
    start: u64,
    end: u64,
    accessed_by: u64,
    written_by: u64,
}

/// Splits the pages that any of the executions accessed into maximal runs of
/// pages that were accessed and written by the same numbers of executions.
fn page_runs(executions: &[TracedExecution]) -> Vec<PageRun> {
    let mut events: Vec<(u64, i64, i64)> = vec![];
    for e in executions {
        if !memory_details_available(e) {
            continue;
        }
        for &(start, end) in &e.accessed_pages {
            events.extend([(start, 1, 0), (end, -1, 0)]);
        }
        for &(start, end) in &e.dirty_pages {
            events.extend([(start, 0, 1), (end, 0, -1)]);
        }
    }
    events.sort_unstable_by_key(|&(page, _, _)| page);
    let mut runs: Vec<PageRun> = vec![];
    let (mut accessed_by, mut written_by) = (0_i64, 0_i64);
    for (i, &(page, accessed, written)) in events.iter().enumerate() {
        accessed_by += accessed;
        written_by += written;
        let next = events.get(i + 1).map_or(page, |&(next, _, _)| next);
        if next == page || accessed_by == 0 {
            continue;
        }
        let (accessed_by, written_by) = (accessed_by as u64, written_by as u64);
        match runs.last_mut() {
            Some(last)
                if last.end == page
                    && last.accessed_by == accessed_by
                    && last.written_by == written_by =>
            {
                last.end = next
            }
            _ => runs.push(PageRun {
                start: page,
                end: next,
                accessed_by,
                written_by,
            }),
        }
    }
    runs
}

fn ratio(part: u128, total: u128, precision: f64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64 * precision).round() / precision
    }
}

fn share(part: u128, total: u128) -> f64 {
    ratio(part, total, 1000.0)
}

fn mean(sum: u128, count: u64) -> f64 {
    ratio(sum, count as u128, 10.0)
}

fn outcome(e: &TracedExecution) -> String {
    serde_json::to_value(e.outcome)
        .ok()
        .and_then(|outcome| outcome.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn memory_details_available(e: &TracedExecution) -> bool {
    !e.detail_truncated
        && e.attempt
            .as_ref()
            .is_none_or(|attempt| attempt.memory_details_available)
}

fn instructions_available(e: &TracedExecution) -> bool {
    e.attempt
        .as_ref()
        .is_none_or(|attempt| attempt.instructions_available)
}

fn selected_logical_dictionary(
    dictionary: &BTreeMap<String, pocket_ic::common::rest::TracedLogicalExecution>,
    executions: &[TracedExecution],
) -> Value {
    let ids: std::collections::BTreeSet<String> = executions
        .iter()
        .filter_map(|execution| {
            execution
                .disposition
                .as_ref()?
                .logical_execution_id
                .map(|id| id.to_string())
        })
        .collect();
    json!(
        dictionary
            .iter()
            .filter(|(id, _)| ids.contains(*id))
            .collect::<BTreeMap<_, _>>()
    )
}

fn page_range(start: u64, end: u64) -> Value {
    json!({
        "first_page": start,
        "last_page": end - 1,
        "pages": end - start,
        "offset": format!("{:#x}", start.saturating_mul(PAGE_SIZE)),
        "size_bytes": (end - start).saturating_mul(PAGE_SIZE),
    })
}

#[derive(Default)]
struct FunctionStats {
    api_type: String,
    executions: u64,
    outcomes: BTreeMap<String, u64>,
    instructions: u128,
    instruction_executions: u64,
    max_instructions: u64,
    page_access_instructions: u128,
    accessed: u128,
    accessed_executions: u64,
    max_accessed: u64,
    written: u128,
    written_executions: u64,
    max_written: u64,
    accessed_ranges: Vec<(u64, u64)>,
    written_ranges: Vec<(u64, u64)>,
}

#[derive(Default)]
struct Region {
    start: u64,
    end: u64,
    executions: u64,
    page_accesses: u128,
    page_writes: u128,
    written_pages: u64,
}

/// Summarizes executions: totals, statistics per function, the most expensive
/// executions, and where in the heap the activity is.
fn summarize(executions: &[TracedExecution], top: usize) -> Value {
    let labels = function_labels(executions);
    let count = executions.len() as u64;
    let accessed: Vec<u64> = executions
        .iter()
        .map(|e| pages(&e.accessed_pages))
        .collect();
    let written: Vec<u64> = executions.iter().map(|e| pages(&e.dirty_pages)).collect();
    let total_accessed: u128 = executions
        .iter()
        .zip(&accessed)
        .filter(|(e, _)| memory_details_available(e))
        .map(|(_, &n)| n as u128)
        .sum();
    let total_written: u128 = executions
        .iter()
        .zip(&written)
        .filter(|(e, _)| memory_details_available(e) && e.dirty_pages_tracked)
        .map(|(_, &n)| n as u128)
        .sum();
    let instruction_executions = executions
        .iter()
        .filter(|e| instructions_available(e))
        .count() as u64;
    let instructions: u128 = executions
        .iter()
        .filter(|e| instructions_available(e))
        .map(|e| e.instructions as u128)
        .sum();
    let page_access_instructions: u128 = executions
        .iter()
        .map(|e| e.page_access_instructions as u128)
        .sum();
    let accessed_executions = executions
        .iter()
        .filter(|e| memory_details_available(e))
        .count() as u64;
    let written_executions = executions
        .iter()
        .filter(|e| memory_details_available(e) && e.dirty_pages_tracked)
        .count() as u64;
    let (complete_page_fees, complete_page_accesses) = executions
        .iter()
        .enumerate()
        .filter(|(_, e)| memory_details_available(e) && e.dirty_pages_tracked)
        .fold((0_u128, 0_u128), |(fees, page_count), (i, e)| {
            (
                fees + e.page_access_instructions as u128,
                page_count + accessed[i] as u128 + written[i] as u128,
            )
        });
    let mut outcomes: BTreeMap<String, u64> = BTreeMap::new();
    for e in executions {
        *outcomes.entry(outcome(e)).or_default() += 1;
    }

    // Statistics per function, sorted by instructions.
    let mut functions: BTreeMap<&str, FunctionStats> = BTreeMap::new();
    for (i, e) in executions.iter().enumerate() {
        let stats = functions.entry(labels[i].as_str()).or_default();
        stats.api_type.clone_from(&e.api_type);
        stats.executions += 1;
        *stats.outcomes.entry(outcome(e)).or_default() += 1;
        if instructions_available(e) {
            stats.instructions += e.instructions as u128;
            stats.max_instructions = stats.max_instructions.max(e.instructions);
            stats.instruction_executions += 1;
        }
        stats.page_access_instructions += e.page_access_instructions as u128;
        if memory_details_available(e) {
            stats.accessed += accessed[i] as u128;
        }
        stats.accessed_executions += u64::from(memory_details_available(e));
        stats.max_accessed = stats.max_accessed.max(accessed[i]);
        if memory_details_available(e) && e.dirty_pages_tracked {
            stats.written += written[i] as u128;
        }
        stats.written_executions += u64::from(memory_details_available(e) && e.dirty_pages_tracked);
        stats.max_written = stats.max_written.max(written[i]);
        if memory_details_available(e) {
            stats.accessed_ranges.extend(&e.accessed_pages);
            if e.dirty_pages_tracked {
                stats.written_ranges.extend(&e.dirty_pages);
            }
        }
    }
    let function_count = functions.len();
    let mut functions: Vec<_> = functions.into_iter().collect();
    functions.sort_by(|(a, x), (b, y)| y.instructions.cmp(&x.instructions).then(a.cmp(b)));
    let functions: Vec<Value> = functions
        .into_iter()
        .take(top)
        .map(|(label, s)| {
            json!({
                "function": label,
                "api_type": s.api_type,
                "executions": s.executions,
                "outcomes": s.outcomes,
                "instructions": {
                    "total": s.instructions.to_string(),
                    "mean": mean(s.instructions, s.instruction_executions),
                    "detail_complete": s.instruction_executions == s.executions,
                    "available_executions": s.instruction_executions,
                    "max": s.max_instructions,
                    "page_access_total": s.page_access_instructions.to_string(),
                    "page_access_share": share(s.page_access_instructions, s.instructions),
                },
                "pages": {
                    "accessed_mean": mean(s.accessed, s.accessed_executions),
                    "accessed_executions": s.accessed_executions,
                    "accessed_max": s.max_accessed,
                    "written_mean": mean(s.written, s.written_executions),
                    "written_executions": s.written_executions,
                    "written_max": s.max_written,
                    "distinct_accessed": pages(&union(s.accessed_ranges.into_iter())),
                    "distinct_written": pages(&union(s.written_ranges.into_iter())),
                },
            })
        })
        .collect();

    // The most expensive executions.
    let mut by_instructions: Vec<usize> = (0..executions.len())
        .filter(|&i| instructions_available(&executions[i]))
        .collect();
    by_instructions.sort_by(|&a, &b| {
        executions[b]
            .instructions
            .cmp(&executions[a].instructions)
            .then(a.cmp(&b))
    });
    let most_expensive: Vec<Value> = by_instructions
        .into_iter()
        .take(top)
        .map(|i| {
            let e = &executions[i];
            json!({
                "sequence_number": e.sequence_number,
                "function": labels[i],
                "api_type": e.api_type,
                "outcome": outcome(e),
                "instructions": e.instructions,
                "page_access_instructions": e.page_access_instructions,
                "accessed_pages": memory_details_available(e).then_some(accessed[i]),
                "written_pages": (memory_details_available(e) && e.dirty_pages_tracked).then_some(written[i]),
            })
        })
        .collect();

    // The pages accessed by the most executions.
    let runs = page_runs(executions);
    let mut hot: Vec<&PageRun> = runs.iter().collect();
    hot.sort_by(|x, y| {
        (y.accessed_by, y.written_by, y.end - y.start)
            .cmp(&(x.accessed_by, x.written_by, x.end - x.start))
            .then(x.start.cmp(&y.start))
    });
    let hot_pages: Vec<Value> = hot
        .into_iter()
        .take(top)
        .map(|run| {
            let mut value = page_range(run.start, run.end);
            value["accessed_by"] = json!(run.accessed_by);
            value["written_by"] = json!(run.written_by);
            value
        })
        .collect();

    // Regions: maximal runs of pages that any execution accessed, by the
    // number of executions that accessed them.
    let mut regions: Vec<Region> = vec![];
    for run in &runs {
        if regions.last().is_none_or(|last| last.end != run.start) {
            regions.push(Region {
                start: run.start,
                ..Default::default()
            });
        }
        let region = regions.last_mut().unwrap();
        let length = run.end - run.start;
        region.end = run.end;
        region.page_accesses += length as u128 * run.accessed_by as u128;
        region.page_writes += length as u128 * run.written_by as u128;
        if run.written_by > 0 {
            region.written_pages += length;
        }
    }
    let mut last_execution = vec![usize::MAX; regions.len()];
    for (i, e) in executions.iter().enumerate() {
        if !memory_details_available(e) {
            continue;
        }
        for &(start, end) in &e.accessed_pages {
            let r = regions.partition_point(|region| region.end <= start);
            if start < end && last_execution[r] != i {
                last_execution[r] = i;
                regions[r].executions += 1;
            }
        }
    }
    let region_count = regions.len();
    regions.sort_by(|x, y| {
        (y.executions, y.page_accesses)
            .cmp(&(x.executions, x.page_accesses))
            .then(x.start.cmp(&y.start))
    });
    let regions: Vec<Value> = regions
        .into_iter()
        .take(top)
        .map(|region| {
            let mut value = page_range(region.start, region.end);
            value["accessed_by"] = json!(region.executions);
            value["written_pages"] = json!(region.written_pages);
            value["page_accesses"] = json!(region.page_accesses.to_string());
            value["page_writes"] = json!(region.page_writes.to_string());
            value
        })
        .collect();

    let distinct = |runs: &mut dyn Iterator<Item = &PageRun>| -> u64 {
        runs.map(|run| run.end - run.start).sum()
    };
    json!({
        "page_size": PAGE_SIZE,
        "executions": count,
        "first_sequence_number": executions.first().map(|e| e.sequence_number),
        "last_sequence_number": executions.last().map(|e| e.sequence_number),
        "first_time_nanos": executions.first().map(|e| e.time_nanos),
        "last_time_nanos": executions.last().map(|e| e.time_nanos),
        "outcomes": outcomes,
        "wasm_memory_size": executions.last().filter(|e| memory_details_available(e)).map(|e| e.wasm_memory_size),
        "instructions": {
            "total": instructions.to_string(),
            "mean": mean(instructions, instruction_executions),
            "detail_complete": instruction_executions == count,
            "available_executions": instruction_executions,
            "page_access_total": page_access_instructions.to_string(),
            "page_access_share": share(page_access_instructions, instructions),
            "per_page_access": ratio(complete_page_fees, complete_page_accesses, 1.0),
            "per_page_access_available": complete_page_accesses > 0,
        },
        "pages": {
            "distinct_accessed": distinct(&mut runs.iter()),
            "distinct_written": distinct(&mut runs.iter().filter(|run| run.written_by > 0)),
            "detail_complete": accessed_executions == count,
            "accessed_executions": accessed_executions,
            "written_executions": written_executions,
            "accessed_mean": mean(total_accessed, accessed_executions),
            "accessed_max": accessed.iter().max().copied().unwrap_or_default(),
            "written_mean": mean(total_written, written_executions),
            "written_max": written.iter().max().copied().unwrap_or_default(),
        },
        "function_count": function_count,
        "functions": functions,
        "most_expensive_executions": most_expensive,
        "hot_pages": hot_pages,
        "region_count": region_count,
        "regions": regions,
    })
}

// ----------------------------------------------------------------------------
// MCP (the streamable HTTP transport with JSON responses)

async fn mcp_get() -> Response {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        [(header::ALLOW, "POST")],
        "This MCP server only accepts POST requests.",
    )
        .into_response()
}

async fn mcp(State(viewer): State<Viewer>, headers: HeaderMap, body: Bytes) -> Response {
    let base_url = base_url(&headers);
    let message: Value = match serde_json::from_slice(&body) {
        Ok(message) => message,
        Err(e) => {
            let error = rpc_error(Value::Null, -32700, &format!("Parse error: {e}"));
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };
    let response = match message {
        Value::Array(messages) => {
            if messages.is_empty() || messages.len() > MAX_MCP_BATCH {
                let error = rpc_error(
                    Value::Null,
                    -32600,
                    &format!("A batch must contain between 1 and {MAX_MCP_BATCH} messages."),
                );
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
            let mut responses = vec![];
            let mut response_bytes = 0;
            for message in messages {
                let id = message.get("id").cloned();
                if response_bytes >= MAX_MCP_BATCH_RESPONSE_BYTES {
                    if let Some(id) = id {
                        responses.push(rpc_error(id, -32000, "The batch response budget was reached; submit the remaining requests separately."));
                    }
                    continue;
                }
                if let Some(response) = mcp_message(&viewer, &base_url, message).await {
                    let size = response.to_string().len();
                    if response_bytes + size > MAX_MCP_BATCH_RESPONSE_BYTES {
                        responses.push(rpc_error(response["id"].clone(), -32000, "The batch response budget was reached; submit this request separately."));
                        response_bytes = MAX_MCP_BATCH_RESPONSE_BYTES;
                    } else {
                        response_bytes += size;
                        responses.push(response);
                    }
                }
            }
            (!responses.is_empty()).then_some(Value::Array(responses))
        }
        message => mcp_message(&viewer, &base_url, message).await,
    };
    match response {
        Some(response) => Json(response).into_response(),
        // Notifications and responses are only acknowledged.
        None => StatusCode::ACCEPTED.into_response(),
    }
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Handles a JSON-RPC message and returns the response if it is a request.
async fn mcp_message(viewer: &Viewer, base_url: &str, message: Value) -> Option<Value> {
    if let Err(message) = validate_rpc_message(&message) {
        return Some(rpc_error(Value::Null, -32600, message));
    }
    let method = message.get("method").and_then(Value::as_str);
    let id = message.get("id").cloned();
    let (Some(method), Some(id)) = (method, id) else {
        // Notifications and responses (to requests that this server never sends).
        if method.is_some() || message.get("result").is_some() || message.get("error").is_some() {
            return None;
        }
        return Some(rpc_error(Value::Null, -32600, "Invalid request"));
    };
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => {
            let requested = params.get("protocolVersion").and_then(Value::as_str);
            let version = requested
                .filter(|version| MCP_PROTOCOL_VERSIONS.contains(version))
                .unwrap_or(MCP_PROTOCOL_VERSIONS[0]);
            json!({
                "protocolVersion": version,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": {
                    "name": "pocket-ic-memory-access-viewer",
                    "version": env!("CARGO_PKG_VERSION"),
                },
                "instructions": AGENT_GUIDE.replace("{base}", base_url),
            })
        }
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tools() }),
        "tools/call" => call_tool(viewer, base_url, &params).await,
        _ => {
            let message = format!("Method not found: {method}");
            return Some(rpc_error(id, -32601, &message));
        }
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn validate_rpc_message(message: &Value) -> Result<(), &'static str> {
    if !message.is_object() || message.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err("Expected a JSON-RPC 2.0 object.");
    }
    if message.get("id").is_some_and(|id| {
        !matches!(id, Value::Null | Value::String(_))
            && !id.as_u64().is_some()
            && !id.as_i64().is_some()
    }) {
        return Err("A request id must be a string, integer, or null.");
    }
    if let Some(method) = message.get("method") {
        if !method.is_string() {
            return Err("The method must be a string.");
        }
        if message
            .get("params")
            .is_some_and(|params| !params.is_object() && !params.is_array())
        {
            return Err("Parameters must be an object or array.");
        }
    } else if message.get("result").is_none() && message.get("error").is_none() {
        return Err("The method is missing.");
    }
    Ok(())
}

fn tools() -> Value {
    let selection = |mut extra: Value| {
        for (key, value) in json!({
            "after": { "type": ["integer", "string"], "description": "Only executions with a larger sequence number; decimal strings preserve u64 values." },
            "before": { "type": ["integer", "string"], "description": "Only executions with a smaller sequence number." },
            "function": { "type": "string", "description": "Only executions whose entrypoint/callback function contains this text." },
            "outcome": { "type": "string", "enum": ["reply", "reject", "no_response", "error", "aborted"] },
            "module_hash": { "type": "string", "description": "Only executions of this exact installed-module hash." },
            "attempt_id": { "type": ["integer", "string"], "description": "Exact physical attempt ID in this server generation; includes all retained slices of that attempt." },
        }).as_object().unwrap() {
            extra[key] = value.clone();
        }
        extra
    };
    let schema = |extra: Value, required: &[&str]| {
        let mut properties = json!({
            "instance_id": {
                "type": "integer",
                "description": "The PocketIC instance (see `list_traces`).",
            },
            "canister_id": {
                "type": "string",
                "description": "The textual canister id, e.g. `lxzze-o7777-77777-aaaaa-cai`.",
            },
        });
        for (key, value) in extra.as_object().unwrap() {
            properties[key] = value.clone();
        }
        let mut required_properties = vec!["instance_id", "canister_id"];
        required_properties.extend_from_slice(required);
        json!({ "type": "object", "properties": properties, "required": required_properties })
    };
    let canister = || schema(json!({}), &[]);
    let read_only = json!({ "readOnlyHint": true, "openWorldHint": false });
    let control = |destructive: bool| {
        json!({
            "readOnlyHint": false,
            "destructiveHint": destructive,
            "idempotentHint": true,
            "openWorldHint": false,
        })
    };
    let mut definitions = json!([
        {
            "name": "list_traces",
            "description": "Lists the PocketIC instances and their canisters whose Wasm memory accesses are traced or were recorded, with the number of recorded executions.",
            "inputSchema": { "type": "object", "properties": {} },
            "annotations": read_only.clone(),
        },
        {
            "name": "get_runtime_costs",
            "description": "Reads actual host-side cycle flows, scheduler round overhead and management/query budget debits. Units and canister/subnet ownership remain separate; prepayments/refunds and reservations/transfers are not extra Wasm charges. Includes actual subnet configuration and cumulative totals even when bounded event detail is truncated.",
            "inputSchema": schema(json!({
                "after": { "type": ["integer", "string"], "description": "Continue after this runtime-cost event sequence, independent of Wasm execution sequence." },
                "limit": { "type": "integer" }
            }), &[]),
            "annotations": read_only.clone(),
        },
        {
            "name": "get_charge_rules",
            "description": "Returns the immutable IC charge-rule dictionary for this server binary: each rule's condition, formula, explanation, optimization tip, and exact compiled source references. Join each execution's rules by rule_id. Zero-cost decision rows explain a chosen path and do not add charges.",
            "inputSchema": { "type": "object", "properties": {} },
            "annotations": read_only.clone(),
        },
        {
            "name": "get_computation_catalog",
            "description": "Returns the self-contained Game of Compute and Memory reference compiled into this binary: rule dictionary, source excerpts and hashes, API costs, default subnet prices and limits, units, scopes, accounting guidance, timer/round semantics and known profiling gaps. Requires no instance; defaults are distinct from a running subnet's captured configuration.",
            "inputSchema": { "type": "object", "properties": {} },
            "annotations": read_only.clone(),
        },
        {
            "name": "capture_memory_snapshot",
            "description": "Retains an immutable diagnostic image source for both heap and stable memory at the current committed PocketIC operation boundary. No guest or management call runs, and IC time/cycles do not advance. Capture after install, then before/after a workload. Other messages may run between boundaries. Returns snapshot_id and image URLs. Eight snapshots and a retained-delta estimate budget are enforced; delete captures when done.",
            "inputSchema": schema(json!({"label": {"type": "string", "description": "Optional label, at most 1024 UTF-8 bytes."}}), &[]),
            "annotations": {"readOnlyHint": false, "destructiveHint": false, "idempotentHint": false, "openWorldHint": false},
        },
        {
            "name": "get_memory_snapshot_image",
            "description": "Returns a compact PNG contact sheet of all observed touched 64 KiB heap and stable blocks in the selected call/install window, plus exact JSON with the color key. Each block is256x256 byte pixels, separated by a reserved-color thin grid and identified H:page or S:page. No embedded legend or descriptive text. compare_to is before and snapshot_id is after. Optional attempt/logical IDs select one call or installation; explicit pagination is reported for more than256blocks. Changed bytes are not exact written bytes; zero is not allocator-free memory.",
            "inputSchema": {"type": "object", "required": ["snapshot_id"], "properties": {
                "snapshot_id": {"type": "string"},
                "compare_to": {"type": "string", "description": "Earlier snapshot of the same canister/instance; optional."},
                "memory": {"type": "string", "enum": ["both", "heap", "stable"], "default": "both"},
                "view": {"type": "string", "enum": ["data", "reads", "writes"], "default": "data", "description": "Read/write masks use the identical page layout without contents; enable read_mask/write_mask in start_profiling before execution. Write masks include same-value and restored writes."},
                "selection": {"type": "string", "enum": ["touched", "range"], "default": "touched"},
                "start_page": {"type": ["integer", "string"], "description": "First64KiB page for explicit range selection (one memory only)."},
                "page_count": {"type": ["integer", "string"], "description": "Maximum1–256blocks per image; defaults to all observed touched blocks, up to256."},
                "page_offset": {"type": ["integer", "string"], "description": "Continue at this offset in the sorted touched block list; use next_page_offset fromJSON."},
                "attempt_id": {"type": ["integer", "string"], "description": "One physical execution attempt in the captured interval."},
                "logical_execution_id": {"type": ["integer", "string"], "description": "One logical call/install including its retained stages; mutually exclusive with attempt_id."}
            }},
            "annotations": read_only.clone(),
        },
        {
            "name": "delete_memory_snapshot",
            "description": "Releases a retained diagnostic memory snapshot and its quota. Does not change canister memory or delete ordinary profiler traces.",
            "inputSchema": {"type": "object", "required": ["snapshot_id"], "properties": {"snapshot_id": {"type": "string"}}},
            "annotations": control(true),
        },
        {
            "name": "get_trace_summary",
            "description": "Analyzes the recorded executions of a canister: instructions and the share charged for accessing heap pages, statistics per function (by instructions), the most expensive executions, the pages accessed by the most executions, and the most active regions of the heap. Pages are 64 KiB Wasm pages. Start here.",
            "inputSchema": schema(selection(json!({
                "top": { "type": "integer", "description": "How many functions, executions, pages, and regions to list (default 20)." },
            })), &[]),
            "annotations": read_only.clone(),
        },
        {
            "name": "get_executions",
            "description": "Returns recorded executions of a canister in order with the ranges [start, end) of the Wasm pages they accessed and wrote, their function, type, call context, instructions, and outcome. Continue with `after` set to the last sequence number while `more` is true.",
            "inputSchema": schema(selection(json!({
                "limit": { "type": "integer", "description": "The maximum number of executions (default 100)." },
            })), &[]),
            "annotations": read_only.clone(),
        },
        {
            "name": "get_instruction_profile",
            "description": "Returns the charge ledger and ranked internal Wasm functions/blocks for selected profiled executions. Function rows allocate static prepaid charges; the ledger includes dynamic/API/heap/stable fees and explicit unreconciled differences. Source coverage remains unavailable without exact artifact bundles.",
            "inputSchema": schema(selection(json!({ "top": { "type": "integer", "description": "Maximum ranked functions and blocks (default 20)." } })), &[]),
            "annotations": read_only.clone(),
        },
        {
            "name": "start_profiling",
            "description": "Starts tracing and selects the next bounded number of executions for the diagnostic IC charge profiler, using host-owned counters. Defaults to 20 executions, at most 10000. Retained trace data is bounded by a 64 MiB per-instance budget.",
            "inputSchema": schema(json!({ "max_executions": { "type": "integer", "minimum": 1, "maximum": 10000 }, "read_mask": { "type": "boolean", "default": false, "description": "Collect exact logical byte-read ranges/frequencies for read-mask images; slower diagnostic mode." }, "write_mask": { "type": "boolean", "default": false, "description": "Collect completed byte writes and frequencies, including same-value and restored writes." } }), &[]),
            "annotations": control(false),
        },
        {
            "name": "get_page_history",
            "description": "Returns the executions of a canister that accessed a Wasm page (64 KiB) and whether they wrote it.",
            "inputSchema": schema(json!({
                "page": { "type": ["integer", "string"], "description": "The index of the Wasm page (the byte offset divided by 65536)." },
                "after": { "type": ["integer", "string"], "description": "Continue after this sequence number while more is true." },
                "limit": { "type": "integer", "description": "The maximum number of executions to list (default 100)." },
            }), &["page"]),
            "annotations": read_only.clone(),
        },
        {
            "name": "start_tracing",
            "description": "Starts recording the Wasm memory accesses of all executions of a canister.",
            "inputSchema": canister(),
            "annotations": control(false),
        },
        {
            "name": "stop_tracing",
            "description": "Stops recording the Wasm memory accesses of a canister. The recorded executions are kept.",
            "inputSchema": canister(),
            "annotations": control(false),
        },
        {
            "name": "clear_trace",
            "description": "Deletes the recorded executions of a canister.",
            "inputSchema": schema(json!({
                "expected_last_runtime_cost_sequence_number": { "type": ["string", "integer", "null"], "description": "Latest visible canister/subnet runtime-cost cursor in the exported snapshot; null requires no events." },
                "expected_server_generation": { "type": "string", "description": "Server generation from the exported snapshot; rejects a clear after server restart." },
                "trace_generation": { "type": ["integer", "string"], "description": "Optional snapshot generation. Supply with last_sequence_number for a conditional clear that rejects newly recorded data." },
                "last_sequence_number": { "type": ["integer", "string", "null"], "description": "The last retained execution in the exported snapshot, or null if empty." },
            }), &[]),
            "annotations": control(true),
        },
        {
            "name": "get_viewer_url",
            "description": "Returns the URL of the web app showing the trace of a canister (the heap as a bitmap with one square per page, played back execution by execution, a timeline, and statistics) for the user.",
            "inputSchema": canister(),
            "annotations": read_only.clone(),
        },
    ]);
    definitions
        .as_array_mut()
        .unwrap()
        .extend(crate::system_profile_api::tool_definitions());
    definitions
}

enum ToolOutput {
    Trace(Value),
    Native(Value),
}

fn encode_tool_response(output: ToolOutput) -> Value {
    match output {
        ToolOutput::Trace(value) => encode_response(value),
        // Native profile/catalog documents already own their integer format and
        // content hashes; applying trace encoding would invalidate that contract.
        ToolOutput::Native(value) => value,
    }
}

fn image_tool_response(
    metadata: &Value,
    png: &[u8],
    oversize_message: &str,
) -> Result<Value, String> {
    use base64::prelude::*;
    let text = metadata.to_string();
    let png = BASE64_STANDARD.encode(png);
    if text.len().saturating_add(png.len()) > MAX_TOOL_RESPONSE_BYTES {
        return Err(oversize_message.to_string());
    }
    Ok(
        json!({"content":[{"type":"text","text":text},{"type":"image","mimeType":"image/png","data":png}],"isError":false}),
    )
}

async fn call_tool(viewer: &Viewer, base_url: &str, params: &Value) -> Value {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if name == "profile_render" {
        let result = (|| {
            let run_id = arguments
                .get("run_id")
                .and_then(Value::as_str)
                .ok_or("Missing run_id")?;
            arguments.as_object().ok_or("Arguments must be an object")?;
            let parameters = crate::system_profile_api::argument_parameters(&arguments);
            let rendered = crate::system_profile_api::render(run_id, &parameters)?;
            image_tool_response(
                &rendered.metadata,
                &rendered.png,
                "Rendered profile exceeds the tool response budget",
            )
        })();
        return result.unwrap_or_else(
            |error: String| json!({"content":[{"type":"text","text":error}],"isError":true}),
        );
    }
    if name == "get_memory_snapshot_image" {
        let result = async {
            if !arguments.is_object() {
                return Err("Tool arguments must be an object.".to_string());
            }
            let snapshot_id = arguments
                .get("snapshot_id")
                .and_then(Value::as_str)
                .ok_or("Missing string snapshot_id.")?
                .to_string();
            let query = crate::memory_snapshot_api::SnapshotImageQuery::from_arguments(&arguments)?;
            let image = render_memory_snapshot(viewer, base_url.to_string(), snapshot_id, query)
                .await
                .map_err(|(_, message)| message)?;
            image_tool_response(
                &image.summary,
                &image.png,
                "Image exceeds the tool response budget; request fewer pages.",
            )
        }
        .await;
        return result.unwrap_or_else(
            |message| json!({"content": [{"type": "text", "text": message}], "isError": true}),
        );
    }
    let (text, is_error) = match run_tool(viewer, base_url, name, &arguments).await {
        Ok(value) => {
            let text = encode_tool_response(value).to_string();
            if text.len() > MAX_TOOL_RESPONSE_BYTES {
                ("The tool response exceeds 8 MiB; request a smaller execution limit or ranking, or export module dictionaries through the JSON API.".to_string(), true)
            } else {
                (text, false)
            }
        }
        Err(message) => (message, true),
    };
    json!({ "content": [{ "type": "text", "text": text }], "isError": is_error })
}

async fn run_tool(
    viewer: &Viewer,
    base_url: &str,
    name: &str,
    arguments: &Value,
) -> Result<ToolOutput, String> {
    if !arguments.is_object() {
        return Err("Tool arguments must be an object.".to_string());
    }
    if name == "profile_export" {
        let run_id = arguments
            .get("run_id")
            .and_then(Value::as_str)
            .ok_or("Missing run_id")?;
        crate::system_profile_api::query_run(run_id, "status", &HashMap::new())?;
        return Ok(ToolOutput::Native(
            json!({"run_id":run_id,"url":format!("{base_url}/api/profiling/runs/{run_id}/export"),"format":"icprofile","compression":"gzip"}),
        ));
    }
    if name.starts_with("profile_") {
        return crate::system_profile_api::dispatch(
            Some(viewer.app.clone()),
            name,
            arguments.clone(),
        )
        .await
        .map(ToolOutput::Native);
    }
    if name == "get_computation_catalog" {
        return Ok(ToolOutput::Native(crate::computation_catalog::catalog()));
    }
    let number = |key: &str| integer_argument(arguments, key);
    let instance_id = || {
        number("instance_id")?
            .and_then(|instance_id| InstanceId::try_from(instance_id).ok())
            .ok_or("Missing the integer argument `instance_id`.".to_string())
    };
    let canister_id = || {
        arguments
            .get("canister_id")
            .and_then(Value::as_str)
            .ok_or("Missing the string argument `canister_id`.".to_string())
    };
    let limit = || {
        Ok::<_, String>(number("limit")?.map_or(DEFAULT_TOOL_LIMIT, |limit| {
            limit.min(MAX_LIMIT as u64) as usize
        }))
    };
    let api = |result: ApiResult| result.map_err(|(_, message)| message);
    let value = match name {
        "list_traces" => Ok(traces(viewer, base_url)),
        "get_charge_rules" => Ok(charge_rule_dictionary()),
        "capture_memory_snapshot" => {
            let label = arguments
                .get("label")
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .ok_or("label must be a string".to_string())
                })
                .transpose()?;
            api(crate::memory_snapshot_api::capture(
                &viewer.app,
                base_url,
                instance_id()?,
                canister_id()?,
                label,
            )
            .await)
        }
        "delete_memory_snapshot" => {
            let id = arguments
                .get("snapshot_id")
                .and_then(Value::as_str)
                .ok_or("Missing string snapshot_id.")?;
            api(crate::memory_snapshot_api::delete(id))
        }
        "get_runtime_costs" => api(runtime_costs(
            viewer,
            instance_id()?,
            canister_id()?,
            number("after")?,
            limit()?,
        )),
        "get_trace_summary" => {
            let top = number("top")?.map_or(DEFAULT_TOP, |top| top.min(MAX_TOP as u64) as usize);
            api(summary(
                viewer,
                base_url,
                instance_id()?,
                canister_id()?,
                top,
                &tool_selection(arguments)?,
            ))
        }
        "get_executions" => api(executions(
            viewer,
            instance_id()?,
            canister_id()?,
            limit()?,
            &tool_selection(arguments)?,
        )),
        "get_page_history" => {
            let page = number("page")?.ok_or("Missing the integer argument `page`.")?;
            api(page_history(
                viewer,
                instance_id()?,
                canister_id()?,
                page,
                number("after")?,
                limit()?,
            ))
        }
        "start_tracing" => {
            api(control(viewer, instance_id()?, canister_id()?, Control::Start).await)
        }
        "stop_tracing" => api(control(viewer, instance_id()?, canister_id()?, Control::Stop).await),
        "clear_trace" => {
            clear_server_generation(arguments).map_err(|(_, message)| message)?;
            api(control(
                viewer,
                instance_id()?,
                canister_id()?,
                Control::Clear(clear_condition(arguments)?),
            )
            .await)
        }
        "start_profiling" => {
            let profile = ProfileRequest::parse(arguments)?;
            api(control(
                viewer,
                instance_id()?,
                canister_id()?,
                Control::Profile(profile),
            )
            .await)
        }
        "get_instruction_profile" => api(profiles(
            viewer,
            instance_id()?,
            canister_id()?,
            number("top")?
                .unwrap_or(DEFAULT_TOP as u64)
                .min(MAX_TOP as u64) as usize,
            &tool_selection(arguments)?,
        )),
        "get_viewer_url" => {
            let instance_id = instance_id()?;
            let (_, canister_id) =
                lookup(viewer, instance_id, canister_id()?).map_err(|(_, message)| message)?;
            let url = viewer_url(base_url, instance_id, &canister_id.to_string());
            Ok(json!({ "url": url }))
        }
        _ => Err(format!("Unknown tool: {name}")),
    }?;
    Ok(ToolOutput::Trace(value))
}

fn tool_selection(arguments: &Value) -> Result<ExecutionSelection, String> {
    let text = |key: &str| -> Result<Option<String>, String> {
        arguments
            .get(key)
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| format!("The argument `{key}` must be a string."))
            })
            .transpose()
    };
    let outcome = arguments
        .get("outcome")
        .map(|outcome| {
            serde_json::from_value(outcome.clone()).map_err(|_| {
                "The outcome must be reply, reject, no_response, error, or aborted.".to_string()
            })
        })
        .transpose()?;
    Ok(ExecutionSelection {
        after: integer_argument(arguments, "after")?,
        before: integer_argument(arguments, "before")?,
        function: text("function")?,
        module_hash: text("module_hash")?,
        outcome,
        attempt_id: integer_argument(arguments, "attempt_id")?,
    })
}

fn integer_argument(arguments: &Value, key: &str) -> Result<Option<u64>, String> {
    let Some(value) = arguments.get(key) else {
        return Ok(None);
    };
    let number = value.as_u64().or_else(|| {
        let decimal = value.as_str()?;
        (!decimal.is_empty() && decimal.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| decimal.parse().ok())
            .flatten()
    });
    number.map(Some).ok_or_else(|| {
        format!("The argument `{key}` must be an unsigned 64-bit integer or a decimal string.")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_requests_preserve_defaults_limits_masks_and_refusals() {
        let defaults = ProfileRequest {
            max_executions: 20,
            read_mask: false,
            write_mask: false,
        };
        assert_eq!(ProfileRequest::parse(&json!({})).unwrap(), defaults);
        assert_eq!(ProfileRequest::parse(&json!({"max_executions": "18446744073709551615", "read_mask": true, "write_mask": true})).unwrap(),
            ProfileRequest { max_executions: pocket_ic::common::rest::MAX_INSTRUCTION_PROFILING_EXECUTIONS, read_mask: true, write_mask: true });
        for input in [
            json!({"max_executions": 0}),
            json!({"max_executions": -1}),
            json!({"max_executions": "1.5"}),
            json!({"read_mask": "true"}),
            json!({"write_mask": null}),
        ] {
            assert!(ProfileRequest::parse(&input).is_err(), "{input}");
        }
    }
    use pocket_ic::common::rest::TracedExecutionOutcome;

    fn execution(
        sequence_number: u64,
        api_type: &str,
        function: &str,
        call_context_id: Option<u64>,
        accessed_pages: Vec<(u64, u64)>,
        dirty_pages: Vec<(u64, u64)>,
    ) -> TracedExecution {
        let page_accesses = pages(&accessed_pages) + pages(&dirty_pages);
        TracedExecution {
            sequence_number,
            attempt: None,
            disposition: None,
            module_hash: Some("0123456789abcdef".to_string()),
            module_hash_kind: Some("installed_uncompressed_wasm".to_string()),
            instruction_profile: None,
            read_profile: None,
            write_profile: None,
            detail_truncated: false,
            api_type: api_type.to_string(),
            function: function.to_string(),
            call_context_id,
            time_nanos: 1_000 + sequence_number,
            instructions: 1_000 + page_accesses * 80_000,
            page_access_instructions: page_accesses * 80_000,
            outcome: TracedExecutionOutcome::Reply,
            message: None,
            wasm_memory_size: 256,
            stable_memory_size: 0,
            stable_pages_tracked: false,
            stable_accessed_pages: vec![],
            stable_dirty_pages: vec![],
            dirty_pages_tracked: true,
            accessed_pages,
            dirty_pages,
        }
    }

    #[test]
    fn union_merges_overlapping_and_adjacent_ranges() {
        let merged = union([(5, 6), (0, 2), (2, 3), (1, 2), (10, 12)].into_iter());
        assert_eq!(merged, vec![(0, 3), (5, 6), (10, 12)]);
        assert!(contains(&merged, 0));
        assert!(contains(&merged, 2));
        assert!(!contains(&merged, 3));
        assert!(contains(&merged, 11));
        assert!(!contains(&merged, 12));
    }

    #[test]
    fn only_localhost_and_ip_addresses_are_local_hosts() {
        use crate::system_profile_api::is_local_host;
        for host in [
            "localhost",
            "localhost:8080",
            "LOCALHOST:1",
            "app.localhost:1",
            "127.0.0.1:80",
            "10.0.0.1",
            "[::1]:8080",
            "[::1]",
        ] {
            assert!(is_local_host(host), "{host}");
        }
        for host in [
            "example.com",
            "example.com:8080",
            "localhost.example.com",
            "localhost@evil.example",
            "localhost:99999",
            "[::1]evil.example",
            "[::1]:99999",
            "",
        ] {
            assert!(!is_local_host(host), "{host}");
        }
    }

    #[test]
    fn callbacks_are_named_after_their_call() {
        let executions = vec![
            execution(0, "update", "canister_update f", Some(1), vec![], vec![]),
            execution(
                1,
                "reply callback",
                "callback (table index 0)",
                Some(1),
                vec![],
                vec![],
            ),
            execution(
                2,
                "reject callback",
                "callback (table index 3)",
                Some(9),
                vec![],
                vec![],
            ),
        ];
        assert_eq!(
            function_labels(&executions),
            vec![
                "canister_update f",
                "canister_update f ↳ reply callback",
                "callback (table index 3) (reject callback)",
            ]
        );
    }

    #[test]
    fn callback_labels_follow_the_latest_initiating_context() {
        let executions = vec![
            execution(0, "update", "old", Some(1), vec![], vec![]),
            execution(1, "reply callback", "callback", Some(1), vec![], vec![]),
            execution(2, "update", "restored", Some(1), vec![], vec![]),
            execution(3, "reply callback", "callback", Some(1), vec![], vec![]),
        ];
        assert_eq!(
            function_labels(&executions),
            vec![
                "old",
                "old ↳ reply callback",
                "restored",
                "restored ↳ reply callback"
            ]
        );
    }

    #[test]
    fn composite_and_replicated_contexts_have_separate_labels() {
        let executions = vec![
            execution(0, "update", "update", Some(1), vec![], vec![]),
            execution(1, "composite query", "query", Some(1), vec![], vec![]),
            execution(2, "reply callback", "callback", Some(1), vec![], vec![]),
            execution(
                3,
                "composite reply callback",
                "callback",
                Some(1),
                vec![],
                vec![],
            ),
        ];
        assert_eq!(
            function_labels(&executions),
            vec![
                "update",
                "query",
                "update ↳ reply callback",
                "query ↳ composite reply callback"
            ]
        );
    }

    #[test]
    fn page_runs_count_executions_per_page() {
        let executions = vec![
            execution(0, "update", "a", None, vec![(0, 4)], vec![(1, 2)]),
            execution(1, "update", "a", None, vec![(2, 6)], vec![(2, 3)]),
            execution(2, "update", "a", None, vec![(6, 7), (9, 10)], vec![]),
        ];
        let run = |start, end, accessed_by, written_by| PageRun {
            start,
            end,
            accessed_by,
            written_by,
        };
        assert_eq!(
            page_runs(&executions),
            vec![
                run(0, 1, 1, 0),
                run(1, 2, 1, 1),
                run(2, 3, 2, 1),
                run(3, 4, 2, 0),
                run(4, 7, 1, 0),
                run(9, 10, 1, 0),
            ]
        );
    }

    #[test]
    fn summary_lists_functions_pages_and_regions() {
        let executions = vec![
            execution(
                0,
                "update",
                "canister_update put",
                Some(1),
                vec![(0, 1), (10, 12)],
                vec![(10, 12)],
            ),
            execution(
                1,
                "update",
                "canister_update put",
                Some(2),
                vec![(0, 1), (12, 13)],
                vec![(12, 13)],
            ),
            execution(
                2,
                "update",
                "canister_update scan",
                Some(3),
                vec![(0, 1), (100, 200)],
                vec![],
            ),
        ];
        let summary = summarize(&executions, 10);
        assert_eq!(summary["executions"], 3);
        assert_eq!(summary["outcomes"]["reply"], 3);
        assert_eq!(summary["pages"]["distinct_accessed"], 1 + 3 + 100);
        assert_eq!(summary["pages"]["distinct_written"], 3);
        assert_eq!(summary["pages"]["accessed_max"], 101);
        assert_eq!(summary["instructions"]["per_page_access"], 80_000.0);
        assert!(
            summary["instructions"]["page_access_share"]
                .as_f64()
                .unwrap()
                > 0.99
        );

        // The scan takes the most instructions.
        assert_eq!(summary["function_count"], 2);
        let scan = &summary["functions"][0];
        assert_eq!(scan["function"], "canister_update scan");
        assert_eq!(scan["pages"]["accessed_mean"], 101.0);
        let put = &summary["functions"][1];
        assert_eq!(put["function"], "canister_update put");
        assert_eq!(put["executions"], 2);
        assert_eq!(put["pages"]["accessed_mean"], 2.5);
        assert_eq!(put["pages"]["distinct_accessed"], 4);
        assert_eq!(put["pages"]["distinct_written"], 3);
        assert_eq!(
            summary["most_expensive_executions"][0]["sequence_number"],
            2
        );

        // Page 0 is accessed by all executions.
        assert_eq!(summary["hot_pages"][0]["first_page"], 0);
        assert_eq!(summary["hot_pages"][0]["pages"], 1);
        assert_eq!(summary["hot_pages"][0]["accessed_by"], 3);

        // The regions are [0, 1), [10, 13), and [100, 200).
        assert_eq!(summary["region_count"], 3);
        let regions = &summary["regions"];
        assert_eq!(regions[0]["first_page"], 0);
        assert_eq!(regions[0]["accessed_by"], 3);
        assert_eq!(regions[1]["first_page"], 10);
        assert_eq!(regions[1]["last_page"], 12);
        assert_eq!(regions[1]["accessed_by"], 2);
        assert_eq!(regions[1]["written_pages"], 3);
        assert_eq!(regions[1]["offset"], "0xa0000");
        assert_eq!(regions[2]["first_page"], 100);
        assert_eq!(regions[2]["page_accesses"], "100");
    }

    #[test]
    fn summary_of_no_executions_is_empty() {
        let summary = summarize(&[], 10);
        assert_eq!(summary["executions"], 0);
        assert_eq!(summary["pages"]["distinct_accessed"], 0);
        assert_eq!(summary["functions"], json!([]));
        assert_eq!(summary["regions"], json!([]));
        assert_eq!(summary["instructions"]["page_access_share"], 0.0);
    }

    #[test]
    fn unknown_abort_counters_and_pages_are_excluded_from_known_summary_and_attempt_filter_is_exact()
     {
        let known = execution(
            1,
            "update",
            "canister_update known",
            None,
            vec![(0, 1)],
            vec![],
        );
        let mut unavailable = execution(
            2,
            "update",
            "canister_update unavailable",
            None,
            vec![(10, 20)],
            vec![(10, 20)],
        );
        unavailable.instructions = u64::MAX;
        unavailable.attempt = Some(pocket_ic::common::rest::WasmExecutionAttempt {
            attempt_id: 9_007_199_254_740_993,
            status: "aborted".to_string(),
            memory_details_available: false,
            instructions_available: false,
            ..Default::default()
        });
        unavailable.outcome = TracedExecutionOutcome::Aborted;
        let summary = summarize(&[known.clone(), unavailable.clone()], 10);
        assert_eq!(
            summary["instructions"]["total"],
            known.instructions.to_string()
        );
        assert_eq!(summary["instructions"]["detail_complete"], false);
        assert_eq!(summary["instructions"]["available_executions"], 1);
        assert_eq!(summary["pages"]["distinct_accessed"], 1);
        assert_eq!(summary["pages"]["detail_complete"], false);
        assert_eq!(summary["wasm_memory_size"], Value::Null);
        assert_eq!(
            summary["most_expensive_executions"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let selected =
            tool_selection(&json!({"attempt_id": "9007199254740993", "outcome": "aborted"}))
                .unwrap();
        assert!(selected.filter().matches(&unavailable));
        assert!(!selected.filter().matches(&known));
        assert!(tool_selection(&json!({"attempt_id": "-1"})).is_err());
        let parsed = Query::<ExecutionsQuery>::try_from_uri(
            &"/?attempt_id=9007199254740993".parse().unwrap(),
        )
        .unwrap()
        .0;
        assert_eq!(parsed.selection.attempt_id, Some(9_007_199_254_740_993));
    }

    #[test]
    fn shared_image_envelope_preserves_bytes_and_exact_budget() {
        let metadata = json!({"label":"é"});
        let response = image_tool_response(&metadata, &[255], "too large").unwrap();
        assert_eq!(
            response,
            json!({"content":[{"type":"text","text":metadata.to_string()},{"type":"image","mimeType":"image/png","data":"/w=="}],"isError":false})
        );
        let at_limit = Value::String("x".repeat(MAX_TOOL_RESPONSE_BYTES - 6));
        assert!(image_tool_response(&at_limit, &[255], "too large").is_ok());
        let over_limit = Value::String("x".repeat(MAX_TOOL_RESPONSE_BYTES - 5));
        assert_eq!(
            image_tool_response(&over_limit, &[255], "too large").unwrap_err(),
            "too large"
        );
    }

    #[test]
    fn immutable_mcp_catalog_preserves_its_content_hash_and_numeric_contract() {
        let catalog = crate::computation_catalog::catalog();
        let mut encoded = encode_tool_response(ToolOutput::Native(catalog.clone()));
        assert_eq!(encoded, catalog);
        assert!(encoded.get("server_generation").is_none());
        assert_eq!(encoded["catalog_version"], 1);
        let hash = encoded
            .as_object_mut()
            .unwrap()
            .remove("catalog_sha256")
            .unwrap();
        assert_eq!(
            hash,
            hex::encode(ic_crypto_sha2::Sha256::hash(
                &serde_json::to_vec(&encoded).unwrap()
            ))
        );

        // Ordinary trace tools retain their independent process/decimal encoding.
        let trace = encode_tool_response(ToolOutput::Trace(json!({"time_nanos": u64::MAX})));
        assert_eq!(trace["time_nanos"], u64::MAX.to_string());
        assert!(trace["server_generation"].is_string());
    }

    #[test]
    fn json_counters_are_lossless_decimal_strings() {
        let encoded = encode_response(json!({
            "instance_id": 7,
            "page_size": PAGE_SIZE,
            "time_nanos": u64::MAX,
            "executions": [{
                "sequence_number": 9_007_199_254_740_993_u64,
                "call_context_id": u64::MAX,
                "accessed_pages": [[0, 3]],
            }],
            "instructions": { "total": "36893488147419103230", "mean": 1.5 },
        }));
        assert_eq!(encoded["format_version"], 2);
        assert_eq!(encoded["integer_encoding"], "decimal");
        assert_eq!(encoded["instance_id"], 7);
        assert_eq!(encoded["page_size"], PAGE_SIZE);
        assert_eq!(encoded["time_nanos"], u64::MAX.to_string());
        assert_eq!(
            encoded["executions"][0]["sequence_number"],
            "9007199254740993"
        );
        assert_eq!(
            encoded["executions"][0]["call_context_id"],
            u64::MAX.to_string()
        );
        assert_eq!(
            encoded["executions"][0]["accessed_pages"],
            json!([["0", "3"]])
        );
        assert_eq!(encoded["instructions"]["mean"], 1.5);
    }

    #[test]
    fn function_charge_allocation_uses_exact_origins_once_and_preserves_missing_remainder() {
        use pocket_ic::common::rest::{
            WasmInstructionChargeLedger, WasmInstructionModuleMetadata, WasmInstructionOperator,
            WasmInstructionPageCharge, WasmInstructionProfile, WasmInstructionStackCharge,
        };
        let hash = "ab".repeat(32);
        let modules = BTreeMap::from([(
            hash.clone(),
            WasmInstructionModuleMetadata {
                module_sha256: [0xab; 32],
                operators: vec![
                    WasmInstructionOperator {
                        origin_id: 1,
                        function_index: 4,
                        body_offset: 0,
                        operator_ordinal: 0,
                        weight: 100,
                    },
                    WasmInstructionOperator {
                        origin_id: 2,
                        function_index: 5,
                        body_offset: 1,
                        operator_ordinal: 1,
                        weight: 1,
                    },
                ],
                ..Default::default()
            },
        )]);
        let mut observed = execution(0, "update", "f", None, vec![], vec![]);
        observed.module_hash = Some(hash);
        observed.module_hash_kind = Some("installed_uncompressed_sha256".into());
        observed.instruction_profile = Some(WasmInstructionProfile {
            stack_charges: vec![
                WasmInstructionStackCharge {
                    origin_id: Some(1),
                    category: "static_prepaid".into(),
                    instructions: 100,
                    origin_evidence: "block".into(),
                    ..Default::default()
                },
                WasmInstructionStackCharge {
                    origin_id: Some(2),
                    category: "api".into(),
                    instructions: 900,
                    origin_evidence: "exact_operation".into(),
                    ..Default::default()
                },
                WasmInstructionStackCharge {
                    origin_id: None,
                    category: "heap_access".into(),
                    instructions: 160_000,
                    ..Default::default()
                },
            ],
            page_charges: vec![WasmInstructionPageCharge {
                origin_id: Some(1),
                memory: "heap".into(),
                kind: "access".into(),
                instructions: 160_000,
                origin_evidence: "native_fault_pc".into(),
                ..Default::default()
            }],
            ledger: WasmInstructionChargeLedger {
                raw_instructions: 161_000,
                ..Default::default()
            },
            ..Default::default()
        });
        let allocation = allocate_profile_functions(&[observed.clone()], &modules, 10);
        assert_eq!(allocation["allocated_instructions"], "161000");
        assert_eq!(allocation["raw_minus_allocated_instructions"], "0");
        assert_eq!(allocation["functions"][0]["function_index"], 4);
        assert_eq!(allocation["functions"][0]["instructions"], "160100");
        assert_eq!(allocation["detail_complete"], true);
        observed
            .instruction_profile
            .as_mut()
            .unwrap()
            .module_origin_map_truncated = true;
        let module_partial = allocate_profile_functions(&[observed.clone()], &modules, 10);
        assert_eq!(module_partial["allocated_instructions"], "161000");
        assert_eq!(module_partial["detail_complete"], false);
        assert_eq!(module_partial["module_origin_map_truncated"], true);
        assert_eq!(module_partial["module_origin_map_unavailable"], false);
        assert_eq!(module_partial["stack_details_truncated"], false);
        let aggregate = aggregate_profiles(&[observed.clone()], 10);
        assert_eq!(aggregate["module_origin_map_truncated"], true);
        assert_eq!(aggregate["stack_details_truncated"], false);
        assert_eq!(aggregate["truncated"], false);
        let profile = observed.instruction_profile.as_mut().unwrap();
        profile.module_origin_map_truncated = false;
        profile.page_charges.clear();
        profile.page_details_truncated = true;
        let partial = allocate_profile_functions(&[observed.clone()], &modules, 10);
        assert_eq!(partial["raw_minus_allocated_instructions"], "160000");
        assert_eq!(partial["detail_complete"], false);
        observed.module_hash = Some("cd".repeat(32));
        let mismatched = allocate_profile_functions(&[observed], &modules, 10);
        assert_eq!(mismatched["allocated_instructions"], "0");
        assert_eq!(mismatched["module_origin_map_unavailable"], true);
        assert_eq!(mismatched["stack_details_truncated"], false);
    }

    #[test]
    fn aggregate_instructions_do_not_overflow_u64() {
        let mut first = execution(0, "update", "f", None, vec![], vec![]);
        first.instructions = u64::MAX;
        first.page_access_instructions = u64::MAX / 2;
        let mut second = first.clone();
        second.sequence_number = 1;
        let summary = summarize(&[first, second], 10);
        assert_eq!(
            summary["instructions"]["total"],
            (2 * u64::MAX as u128).to_string()
        );
        assert_eq!(
            summary["functions"][0]["instructions"]["total"],
            (2 * u64::MAX as u128).to_string()
        );
        assert_eq!(
            summary["instructions"]["page_access_total"],
            (2 * (u64::MAX / 2) as u128).to_string()
        );
    }

    #[test]
    fn mcp_integer_arguments_accept_exact_decimals_and_reject_invalid_values() {
        assert_eq!(
            integer_argument(&json!({ "after": u64::MAX.to_string() }), "after").unwrap(),
            Some(u64::MAX)
        );
        assert_eq!(
            integer_argument(&json!({ "after": 0 }), "after").unwrap(),
            Some(0)
        );
        assert_eq!(integer_argument(&json!({}), "after").unwrap(), None);
        for invalid in [
            json!(-1),
            json!(1.5),
            json!("18446744073709551616"),
            json!("+1"),
            json!(null),
        ] {
            assert!(integer_argument(&json!({ "after": invalid }), "after").is_err());
        }
    }

    #[test]
    fn mcp_rejects_malformed_json_rpc_envelopes() {
        for invalid in [
            json!({ "method": "ping", "id": 1 }),
            json!({ "jsonrpc": "1.0", "method": "ping", "id": 1 }),
            json!({ "jsonrpc": "2.0", "method": "ping", "id": {} }),
            json!({ "jsonrpc": "2.0", "method": 3, "id": 1 }),
            json!({ "jsonrpc": "2.0", "method": "ping", "params": false }),
            json!([]),
        ] {
            assert!(validate_rpc_message(&invalid).is_err(), "{invalid}");
        }
        assert!(
            validate_rpc_message(&json!({ "jsonrpc": "2.0", "method": "ping", "id": 1 })).is_ok()
        );
        assert!(
            validate_rpc_message(
                &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })
            )
            .is_ok()
        );
    }

    #[test]
    fn profiles_keep_module_versions_and_ledger_categories_separate() {
        use pocket_ic::common::rest::{
            WasmInstructionChargeLedger, WasmInstructionProfile, WasmInstructionProfileBlock,
            WasmInstructionRuleCharge,
        };
        let mut first = execution(0, "update", "f", None, vec![], vec![]);
        first.instructions = 1_000;
        first.instruction_profile = Some(WasmInstructionProfile {
            rules: vec![
                WasmInstructionRuleCharge {
                    rule_id: "static".to_string(),
                    events: 2,
                    instructions: 100,
                },
                WasmInstructionRuleCharge {
                    rule_id: "api".to_string(),
                    events: 1,
                    instructions: 900,
                },
                WasmInstructionRuleCharge {
                    rule_id: "decision".to_string(),
                    events: 1,
                    instructions: 0,
                },
            ],
            blocks: vec![WasmInstructionProfileBlock {
                block_id: 1,
                function_index: 4,
                instructions: 100,
                hits: 2,
                ..Default::default()
            }],
            ledger: WasmInstructionChargeLedger {
                static_instructions: 100,
                api_instructions: 900,
                raw_instructions: 1_000,
                reported_instructions: 1_000,
                reconciled: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let mut upgraded = first.clone();
        upgraded.sequence_number = 1;
        upgraded.module_hash = Some("different-installed-module".to_string());
        let profile = aggregate_profiles(&[first, upgraded], 10);
        assert_eq!(profile["profiled_executions"], 2);
        assert_eq!(profile["function_count"], 2);
        assert_eq!(profile["block_count"], 2);
        assert_eq!(profile["allocated_static_instructions"], "200");
        assert_eq!(profile["ledger"]["static_instructions"], "200");
        assert_eq!(profile["ledger"]["api_instructions"], "1800");
        assert_eq!(profile["ledger"]["reported_instructions"], "2000");
        assert_eq!(profile["ledger"]["reconciled"], true);
        assert_eq!(profile["observed_rule_instructions"], "2000");
        assert_eq!(profile["raw_minus_rule_instructions"], "0");
        assert_eq!(profile["rules_cover_raw_instructions"], true);
        let zero = profile["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|rule| rule["rule_id"] == "decision")
            .unwrap();
        assert_eq!(zero["instructions"], "0");
        assert_eq!(zero["events"], "2");
        assert_eq!(profile["source_coverage"], Value::Null);
    }

    #[test]
    fn missing_page_detail_does_not_become_a_zero_page_execution() {
        let first = execution(0, "update", "f", None, vec![(0, 2)], vec![(1, 2)]);
        let mut truncated = execution(1, "update", "f", None, vec![(10, 15)], vec![(10, 15)]);
        truncated.instructions = first.instructions;
        truncated.page_access_instructions = first.page_access_instructions;
        truncated.detail_truncated = true;
        let summary = summarize(&[first, truncated], 10);
        assert_eq!(summary["pages"]["detail_complete"], false);
        assert_eq!(summary["pages"]["accessed_executions"], 1);
        assert_eq!(summary["pages"]["accessed_mean"], 2.0);
        assert_eq!(summary["pages"]["written_mean"], 1.0);
        assert_eq!(summary["instructions"]["per_page_access"], 80_000.0);
    }

    #[test]
    fn conditional_clear_preserves_runtime_cursor_presence_and_exactness() {
        let request = json!({"trace_generation":"3", "last_sequence_number":null,
            "expected_last_runtime_cost_sequence_number":"18446744073709551615"});
        assert_eq!(
            clear_condition(&request).unwrap(),
            Some(ClearCondition {
                generation: 3,
                last_sequence: None,
                runtime_sequence: Some(Some(u64::MAX))
            })
        );
        assert_eq!(
            clear_condition(&json!({"trace_generation":"3", "last_sequence_number":null,
            "expected_last_runtime_cost_sequence_number":null}))
            .unwrap(),
            Some(ClearCondition {
                generation: 3,
                last_sequence: None,
                runtime_sequence: Some(None)
            })
        );
        assert!(
            clear_condition(&json!({"expected_last_runtime_cost_sequence_number":null})).is_err()
        );
    }

    #[test]
    fn conditional_clear_requires_both_snapshot_fields() {
        assert_eq!(clear_condition(&json!({})).unwrap(), None);
        assert_eq!(
            clear_condition(
                &json!({ "trace_generation": "3", "last_sequence_number": "9007199254740993" })
            )
            .unwrap(),
            Some(ClearCondition {
                generation: 3,
                last_sequence: Some(9_007_199_254_740_993),
                runtime_sequence: None
            })
        );
        assert_eq!(
            clear_condition(&json!({ "trace_generation": "3", "last_sequence_number": null }))
                .unwrap(),
            Some(ClearCondition {
                generation: 3,
                last_sequence: None,
                runtime_sequence: None
            })
        );
        assert!(clear_condition(&json!({ "trace_generation": 3 })).is_err());
        assert!(clear_condition(&json!({ "last_sequence_number": 9 })).is_err());
        assert!(
            clear_server_generation(
                &json!({ "expected_server_generation": server_identity().generation })
            )
            .is_ok()
        );
        assert_eq!(
            clear_server_generation(&json!({ "expected_server_generation": "old-server" }))
                .unwrap_err()
                .0,
            StatusCode::CONFLICT
        );
        assert_eq!(
            clear_server_generation(&json!({ "expected_server_generation": 7 }))
                .unwrap_err()
                .0,
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn execution_selection_accepts_url_query_numbers() {
        let uri = "/?after=0&before=9007199254740993&outcome=reply&limit=2"
            .parse()
            .unwrap();
        let query = Query::<ExecutionsQuery>::try_from_uri(&uri).unwrap().0;
        assert_eq!(query.selection.after, Some(0));
        assert_eq!(query.selection.before, Some(9_007_199_254_740_993));
        assert_eq!(query.selection.outcome, Some(TracedExecutionOutcome::Reply));
        assert_eq!(query.limit, Some(2));
        let summary =
            Query::<SummaryQuery>::try_from_uri(&"/?after=0&before=3&top=5".parse().unwrap())
                .unwrap()
                .0;
        assert_eq!(summary.selection.before, Some(3));
        assert_eq!(summary.top, Some(5));
    }
}
