//! Run-wide physical profiling API. This deliberately never enables detailed
//! Wasm instrumentation or feeds host observations into replicated accounting.

mod admission;

use super::system_profile_product::MAX_IMPORT_BYTES;
pub(crate) use super::system_profile_product::argument_parameters;
pub use super::system_profile_product::{
    export_bytes, import_bytes, query_parameters, query_run, render, tool_definitions,
};

#[cfg(test)]
pub(crate) use admission::is_local_host;
pub(crate) use admission::protect_requests;
pub use admission::{ListenerAdmission, protect_bound_requests};

use crate::state_api::routes::AppState;
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use ic_system_profiler::{Query as ProfileQuery, RunConfig, StopOptions};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Mutex, OnceLock},
};

static SERVER_STORAGE: OnceLock<pocket_ic::common::storage::StorageInfo> = OnceLock::new();

/// Record the ordinary server's default temporary workspace. Instance imports,
/// caller-chosen persistent directories, logs and exports can use other paths.
pub fn configure_storage(info: pocket_ic::common::storage::StorageInfo) {
    let _ = SERVER_STORAGE.set(info);
}

fn server_storage_metadata() -> Value {
    let Some(info) = SERVER_STORAGE.get() else {
        return json!({"availability":"unavailable","scope":"default_temporary_workspace_only","reason":"server startup storage configuration was not registered"});
    };
    let probe = pocket_ic::common::storage::is_ram_backed(&info.temporary_directory);
    json!({
        "ram_backed":info.ram_backed,
        "temporary_directory":info.temporary_directory,
        "temporary_directory_is_tmpfs":probe.as_ref().ok(),
        "effective_temporary_backing":match &probe{Ok(true)=>"tmpfs",Ok(false)=>"non_tmpfs",Err(_)=>"unknown"},
        "probe":"filesystem_statfs_at_capture_start",
        "probe_error":probe.err(),
        "scope":"default_temporary_workspace_only",
        "note":"The configured RAM-mode flag and actual filesystem probe are distinct. Imported or explicitly located instance state, logs, exports and swap may use different backing. Non-tmpfs does not establish SSD media type."
    })
}

/// These routes are merged into both the ordinary server and protected viewer.
pub fn router(app: AppState) -> Router {
    Router::new()
        .route(
            "/api/profiling/instances/{instance_id}/canisters/{canister_id}/{action}",
            post(super::client_api::call),
        )
        .route("/api/profiling/runs", get(list).post(start))
        .route("/api/profiling/explain", get(explain))
        .route("/api/profiling/import", post(import))
        .route("/api/profiling/compare", get(compare))
        .route("/api/profiling/runs/{run_id}", get(status).delete(delete))
        .route("/api/profiling/runs/{run_id}/stop", post(stop))
        .route("/api/profiling/runs/{run_id}/export", get(export))
        .route("/api/profiling/runs/{run_id}/render.svg", get(render_svg))
        .route("/api/profiling/runs/{run_id}/render.png", get(render_png))
        .route(
            "/api/profiling/runs/{run_id}/render.json",
            get(render_metadata),
        )
        .route("/api/profiling/runs/{run_id}/{view}", get(view))
        .layer(DefaultBodyLimit::max(MAX_IMPORT_BYTES))
        .layer(tower_http::compression::CompressionLayer::new())
        .layer(axum::middleware::from_fn(protect_bound_requests))
        .with_state(app)
}

fn response(result: Result<Value, String>) -> Response {
    let _scope = ic_system_profiler::scope("ic.observability.api");
    match result {
        Ok(value) => Json(value).into_response(),
        Err(message) => {
            let code = if message.contains("not found")
                || message.contains("unknown run")
                || message.contains("unknown profiling run")
            {
                StatusCode::NOT_FOUND
            } else if message.contains("query snapshot") {
                StatusCode::GONE
            } else {
                StatusCode::BAD_REQUEST
            };
            (code, Json(json!({"error": message}))).into_response()
        }
    }
}

async fn list() -> Response {
    response(Ok(ic_system_profiler::list_runs()))
}

async fn explain() -> Response {
    response(Ok(super::backend_description::explain()))
}

async fn start(State(app): State<AppState>, Json(config): Json<Value>) -> Response {
    response(dispatch(Some(app), "profile_start", config).await)
}

async fn stop(
    State(app): State<AppState>,
    Path(run_id): Path<String>,
    Json(mut options): Json<Value>,
) -> Response {
    if !options.is_object() {
        return response(Err("Stop options must be an object".into()));
    }
    options["run_id"] = json!(run_id);
    response(dispatch(Some(app), "profile_stop", options).await)
}

async fn status(Path(run_id): Path<String>) -> Response {
    response(query_run(&run_id, "status", &HashMap::new()))
}

async fn delete(Path(run_id): Path<String>) -> Response {
    response(delete_run(&run_id))
}

fn delete_run(run_id: &str) -> Result<Value, String> {
    ic_system_profiler::delete_run(run_id)?;
    scopes().lock().unwrap().remove(run_id);
    super::system_profile_product::forget_run(run_id);
    Ok(json!({"run_id":run_id,"deleted":true}))
}

async fn view(
    Path((run_id, view)): Path<(String, String)>,
    Query(parameters): Query<HashMap<String, String>>,
) -> Response {
    response(query_run(&run_id, &view, &parameters))
}

async fn compare(Query(parameters): Query<HashMap<String, String>>) -> Response {
    let result = (|| {
        let a = parameters.get("a").ok_or("Missing query parameter a")?;
        let b = parameters.get("b").ok_or("Missing query parameter b")?;
        ic_system_profiler::compare(a, b)
    })();
    response(result)
}

async fn export(Path(run_id): Path<String>) -> Response {
    match export_bytes(&run_id) {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, "application/gzip"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=run.icprofile",
                ),
            ],
            bytes,
        )
            .into_response(),
        Err(error) => response(Err(error)),
    }
}

async fn import(body: Bytes) -> Response {
    response(import_bytes(&body))
}

pub async fn dispatch(
    app: Option<AppState>,
    name: &str,
    arguments: Value,
) -> Result<Value, String> {
    if !arguments.is_object() {
        return Err("Profile arguments must be an object".into());
    }
    let name = name.to_owned();
    tokio::task::spawn_blocking(move || dispatch_sync(app.as_ref(), &name, &arguments))
        .await
        .map_err(|e| format!("Profiler worker failed: {e}"))?
}

pub fn dispatch_sync(
    app: Option<&AppState>,
    name: &str,
    arguments: &Value,
) -> Result<Value, String> {
    let _scope = ic_system_profiler::scope("ic.observability.api");
    let run = || {
        arguments
            .get("run_id")
            .and_then(Value::as_str)
            .ok_or("Missing string run_id")
    };
    match name {
        "profile_explain" if arguments.get("run_id").is_none() => {
            Ok(super::backend_description::explain())
        }
        "profile_start" => {
            let mut config: RunConfig = serde_json::from_value(arguments.clone())
                .map_err(|e| format!("Invalid profile configuration: {e}"))?;
            if config.metadata.to_string().len() > 64 * 1024 {
                return Err("Capture metadata exceeds 64 KiB".into());
            }
            let instance_id = config.instance_id;
            let local_metrics = config.root_pid.is_none_or(|pid| pid == std::process::id());
            let metrics_level =
                crate::system_profile_metrics::MetricsLevel::parse(&config.metrics_level)?;
            if let (Some(app), Some(id)) = (app, instance_id)
                && !app.api_state.memory_access_recorders().contains_key(&id)
            {
                return Err(format!("PocketIC instance {id} not found"));
            }
            if app.is_some() {
                let identity = crate::memory_access_tracing::server_identity();
                let mut metadata = if config.metadata.is_object() {
                    config.metadata.clone()
                } else {
                    json!({})
                };
                metadata["server_generation"] = json!(identity.generation);
                metadata["server_binary_hash"] = json!(identity.binary_hash);
                metadata["server_storage"] = server_storage_metadata();
                if local_metrics {
                    metadata["backend_description"] =
                        super::backend_description::description().clone();
                }
                config.metadata = metadata;
            }
            let result = ic_system_profiler::start(config)?;
            if let (Some(app), Some(run_id)) = (app, result.get("run_id").and_then(Value::as_str)) {
                let initial = pricing_snapshot(app, instance_id);
                ingest_snapshot(
                    run_id,
                    json!({"kind":"pricing_snapshot", "phase":"initial", "snapshot":initial}),
                );
                ingest_snapshot(
                    run_id,
                    ic_metrics_snapshot(app, instance_id, "initial", local_metrics, metrics_level),
                );
                scopes().lock().unwrap().insert(
                    run_id.to_string(),
                    RunScope {
                        instance_id,
                        initial,
                        local_metrics,
                        metrics_level,
                    },
                );
            }
            Ok(result)
        }
        "profile_stop" => {
            let id = run()?;
            let current = ic_system_profiler::query(id, "status", &ProfileQuery::default())?;
            if current.get("status").and_then(Value::as_str) != Some("running") {
                return Ok(current);
            }
            let options: StopOptions =
                serde_json::from_value(arguments.clone()).map_err(|e| e.to_string())?;
            if let Some(app) = app.filter(|_| options.drain_ms > 0) {
                let (instance, local_metrics, metrics_level) = {
                    let scopes = scopes().lock().unwrap();
                    let scope = scopes.get(id);
                    (
                        scope.and_then(|scope| scope.instance_id),
                        scope.is_some_and(|scope| scope.local_metrics),
                        scope.map_or(
                            crate::system_profile_metrics::MetricsLevel::Standard,
                            |scope| scope.metrics_level,
                        ),
                    )
                };
                ingest_snapshot(
                    id,
                    json!({"kind":"pricing_snapshot", "phase":"foreground_end", "snapshot":pricing_snapshot(app, instance)}),
                );
                ingest_snapshot(
                    id,
                    ic_metrics_snapshot(
                        app,
                        instance,
                        "foreground_end",
                        local_metrics,
                        metrics_level,
                    ),
                );
            }
            let result = ic_system_profiler::stop(id, &options)?;
            if let Some(app) = app {
                let saved = scopes().lock().unwrap().get(id).cloned();
                let instance = saved.as_ref().and_then(|scope| scope.instance_id);
                let final_snapshot = pricing_snapshot(app, instance);
                ingest_snapshot(
                    id,
                    json!({"kind":"pricing_snapshot", "phase":"stop", "snapshot":final_snapshot}),
                );
                ingest_snapshot(
                    id,
                    ic_metrics_snapshot(
                        app,
                        instance,
                        "stop",
                        saved.as_ref().is_some_and(|scope| scope.local_metrics),
                        saved.as_ref().map_or(
                            crate::system_profile_metrics::MetricsLevel::Standard,
                            |scope| scope.metrics_level,
                        ),
                    ),
                );
                if let Some(saved) = saved {
                    ingest_snapshot(id, pricing_delta(&saved.initial, &final_snapshot));
                }
            }
            let _ = result;
            ic_system_profiler::query(id, "status", &ProfileQuery::default())
        }
        "profile_delete" => delete_run(run()?),
        name => super::system_profile_product::dispatch_sync(name, arguments),
    }
}

fn ingest_snapshot(run_id: &str, value: Value) {
    let _scope = ic_system_profiler::scope("ic.observability.metrics.ingest");
    // Only the few provenance strings survive a failed move. Do not deep-clone
    // a potentially 768 KiB snapshot merely to construct a fallback warning.
    let kind = value.get("kind").cloned();
    let phase = value.get("phase").cloned();
    let snapshot_id = value.get("snapshot_id").cloned();
    if let Err(error) = ic_system_profiler::ingest_metrics(run_id, value) {
        let fallback = json!({"kind":kind,"phase":phase,"snapshot_id":snapshot_id,"rows":[],"coverage":{"truncated":true,"unavailable":[{"reason":"snapshot_not_retained","detail":error}]}});
        let _ = ic_system_profiler::ingest_metrics(run_id, fallback);
    }
}

fn ic_metrics_snapshot(
    app: &AppState,
    instance: Option<usize>,
    phase: &str,
    local: bool,
    level: crate::system_profile_metrics::MetricsLevel,
) -> Value {
    if local || level == crate::system_profile_metrics::MetricsLevel::None {
        crate::system_profile_metrics::snapshot(app, instance, phase, level)
    } else {
        json!({"kind":"ic_metrics_snapshot","phase":phase,"rows":[],"observation":"unavailable","coverage":{"unavailable":[{"reason":"external_target_registry_not_attached"}],"truncated":false}})
    }
}

#[derive(Clone)]
struct RunScope {
    instance_id: Option<usize>,
    initial: Value,
    local_metrics: bool,
    metrics_level: crate::system_profile_metrics::MetricsLevel,
}

fn scopes() -> &'static Mutex<BTreeMap<String, RunScope>> {
    static SCOPES: OnceLock<Mutex<BTreeMap<String, RunScope>>> = OnceLock::new();
    SCOPES.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// No execution/profile/module dictionaries are cloned here. These cumulative
/// ledger snapshots are explicitly conditional on existing runtime recording.
const MAX_PRICING_SNAPSHOT_BYTES: usize = 768 * 1024;

fn retain_pricing_fragment(values: &mut Vec<Value>, value: Value, bytes: &mut usize) -> bool {
    let size = value.to_string().len();
    // Reserve space for outer metadata, punctuation and omission counters.
    if bytes.saturating_add(size) > MAX_PRICING_SNAPSHOT_BYTES - 4096 {
        return false;
    }
    *bytes += size;
    values.push(value);
    true
}

fn pricing_snapshot(app: &AppState, selected_instance: Option<usize>) -> Value {
    let mut rows = Vec::new();
    let mut subnet_totals = Vec::new();
    let mut retained_bytes = 0usize;
    let mut truncated = false;
    let mut omitted_canisters = 0usize;
    let mut omitted_subnet_snapshots = 0usize;
    for (instance_id, recorder) in app.api_state.memory_access_recorders() {
        if selected_instance.is_some_and(|selected| selected != instance_id) {
            continue;
        }
        let mut included_subnet = false;
        let canisters = recorder.recorder.canisters();
        let capacity = 256usize.saturating_sub(rows.len());
        omitted_canisters += canisters.len().saturating_sub(capacity);
        truncated |= canisters.len() > capacity;
        for canister in canisters.into_iter().take(capacity) {
            let costs =
                recorder
                    .recorder
                    .runtime_costs_snapshot(canister.canister_id, Some(u64::MAX), 1);
            if !included_subnet {
                let fragment = json!({"instance_id":instance_id,"totals":costs.subnet_runtime_cost_totals,"incomplete":costs.subnet_runtime_cost_totals_incomplete,"configurations":costs.runtime_configurations});
                if !retain_pricing_fragment(&mut subnet_totals, fragment, &mut retained_bytes) {
                    truncated = true;
                    omitted_subnet_snapshots += 1;
                }
                included_subnet = true;
            }
            let row = json!({
                "instance_id":instance_id,"canister_id":canister.canister_id.to_string(),
                "trace_generation":costs.trace_generation.to_string(),
                "recording":costs.runtime_costs_recording,"totals":costs.runtime_cost_totals,
                "incomplete":costs.runtime_cost_totals_incomplete,"executions":canister.executions.to_string()
            });
            if !retain_pricing_fragment(&mut rows, row, &mut retained_bytes) {
                truncated = true;
                omitted_canisters += 1;
            }
        }
    }
    let result = json!({
        "canisters":rows,"subnets":subnet_totals,
        "coverage":"only_existing_runtime_cost_recorders","truncated":truncated,
        "omitted_canisters":omitted_canisters,"omitted_subnet_snapshots":omitted_subnet_snapshots,
        "max_serialized_bytes":MAX_PRICING_SNAPSHOT_BYTES,
        "note":"Cumulative cycle/round ledgers are separate from physical resources. Missing recorder coverage is unavailable, not zero. No detailed Wasm instrumentation was enabled."
    });
    if result.to_string().len() > MAX_PRICING_SNAPSHOT_BYTES {
        json!({"canisters":[],"subnets":[],"coverage":"snapshot_byte_limit","truncated":true,"omitted_canisters":omitted_canisters+rows.len(),"omitted_subnet_snapshots":omitted_subnet_snapshots+subnet_totals.len(),"max_serialized_bytes":MAX_PRICING_SNAPSHOT_BYTES})
    } else {
        result
    }
}

fn pricing_delta(initial: &Value, final_snapshot: &Value) -> Value {
    let initial_rows = initial
        .get("canisters")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let final_rows = final_snapshot
        .get("canisters")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let key = |row: &Value| {
        format!(
            "{}:{}",
            row.get("instance_id").unwrap_or(&Value::Null),
            row.get("canister_id").and_then(Value::as_str).unwrap_or("")
        )
    };
    let baseline: BTreeMap<_, _> = initial_rows.iter().map(|row| (key(row), row)).collect();
    let mut rows = Vec::new();
    let mut debit = 0u128;
    let mut refund = 0u128;
    let mut covered = 0usize;
    let mut incomplete = initial["truncated"] == true
        || final_snapshot["truncated"] == true
        || baseline.len() != final_rows.len();
    for end in final_rows {
        let Some(begin) = baseline.get(&key(end)) else {
            incomplete = true;
            continue;
        };
        if begin["trace_generation"] != end["trace_generation"]
            || begin["incomplete"] == true
            || end["incomplete"] == true
            || begin["recording"] != true
            || end["recording"] != true
        {
            incomplete = true;
            continue;
        }
        covered += 1;
        fn totals(row: &Value) -> &[Value] {
            row.get("totals")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default()
        }
        let total_key = |row: &Value| format!("{}:{}:{}", row["rule_id"], row["unit"], row["kind"]);
        let before: BTreeMap<_, _> = totals(begin)
            .iter()
            .map(|row| (total_key(row), row))
            .collect();
        for total in totals(end) {
            let parse = |v: &Value| {
                v.as_str()
                    .and_then(|s| s.parse::<u128>().ok())
                    .or_else(|| v.as_u64().map(u128::from))
            };
            let old = before
                .get(&total_key(total))
                .and_then(|v| parse(&v["amount"]))
                .unwrap_or(0);
            let Some(amount) = parse(&total["amount"]).and_then(|n| n.checked_sub(old)) else {
                incomplete = true;
                continue;
            };
            if total["unit"] == "cycles" {
                if total["kind"] == "debit" {
                    if let Some(sum) = debit.checked_add(amount) {
                        debit = sum;
                    } else {
                        incomplete = true;
                    }
                }
                if total["kind"] == "refund" {
                    if let Some(sum) = refund.checked_add(amount) {
                        refund = sum;
                    } else {
                        incomplete = true;
                    }
                }
            }
            rows.push(json!({"label":total["rule_id"],"rule_id":total["rule_id"],"amount":amount.to_string(),"unit":total["unit"],"kind":total["kind"],"scope":format!("instance {} / {}",end["instance_id"],end["canister_id"]),"attribution":"existing_runtime_cost_ledger","canister_id":end["canister_id"]}));
        }
    }
    let complete = covered > 0 && !incomplete;
    json!({"kind":"pricing_delta","rows":rows,"totals":{"cycles_debited":complete.then(||debit.to_string()),"cycles_refunded":complete.then(||refund.to_string())},"coverage":if complete{"selected_existing_recorders"}else{"partial_or_unavailable"},"covered_canisters":covered,"note":"Only generation-matched, continuously recorded cumulative ledgers are differenced. Partial rows are not complete run totals; transfers/reservations remain distinct from debits/refunds."})
}

async fn render_svg(
    Path(run_id): Path<String>,
    Query(parameters): Query<HashMap<String, String>>,
) -> Response {
    match render(&run_id, &parameters) {
        Ok(image) => (
            [
                (header::CONTENT_TYPE, "image/svg+xml"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            image.svg,
        )
            .into_response(),
        Err(error) => response(Err(error)),
    }
}

async fn render_png(
    Path(run_id): Path<String>,
    Query(parameters): Query<HashMap<String, String>>,
) -> Response {
    match render(&run_id, &parameters) {
        Ok(image) => (
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            image.png,
        )
            .into_response(),
        Err(error) => response(Err(error)),
    }
}

async fn render_metadata(
    Path(run_id): Path<String>,
    Query(parameters): Query<HashMap<String, String>>,
) -> Response {
    response(render(&run_id, &parameters).map(|image| image.metadata))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pricing_ledger_differences_keep_transfers_and_currencies_separate() {
        let make = |generation: &str, amount: &str| json!({"canisters":[{"instance_id":0,"canister_id":"aaaaa-aa","trace_generation":generation,"recording":true,"incomplete":false,"totals":[{"rule_id":"fee","unit":"cycles","kind":"debit","amount":amount},{"rule_id":"transfer","unit":"cycles","kind":"transfer","amount":"900000"},{"rule_id":"round","unit":"round_instructions","kind":"debit","amount":"2000000"}]}],"truncated":false});
        let initial = make("1", "9007199254740993");
        let finished = make("1", "9007199254741013");
        let delta = pricing_delta(&initial, &finished);
        assert_eq!(delta["totals"]["cycles_debited"], "20");
        assert_eq!(delta["rows"].as_array().unwrap().len(), 3);
        assert!(
            pricing_delta(&initial, &make("2", "9007199254741013"))["totals"]["cycles_debited"]
                .is_null()
        );
    }

    #[test]
    fn oversized_single_legacy_row_cannot_escape_snapshot_byte_budget() {
        let mut values = Vec::new();
        let mut bytes = 0;
        assert!(!retain_pricing_fragment(
            &mut values,
            json!({"configuration":"x".repeat(MAX_PRICING_SNAPSHOT_BYTES)}),
            &mut bytes
        ));
        assert!(values.is_empty());
        assert_eq!(bytes, 0);
        assert!(retain_pricing_fragment(
            &mut values,
            json!({"amount":"9"}),
            &mut bytes
        ));
        let before = bytes;
        assert!(!retain_pricing_fragment(
            &mut values,
            json!({"totals":"x".repeat(MAX_PRICING_SNAPSHOT_BYTES)}),
            &mut bytes
        ));
        assert_eq!(bytes, before);
        assert_eq!(values.len(), 1);
    }
}
