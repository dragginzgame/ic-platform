//! Native profiling handlers, kept separate from the general PocketIC API.
//! A child of state_api::routes; existing registrations and public aliases stay.
use super::{AppState, OpConversionError, OpOut, run_operation, timeout_or_default};
use crate::InstanceId;
use crate::memory_snapshots::MemorySnapshotMetadata;
use crate::pocket_ic::{
    CaptureMemorySnapshot, ClearMemoryAccessTrace, GetMemoryAccessTrace, StartInstructionProfiling,
    StartMemoryAccessTracing, StopMemoryAccessTracing,
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use ic_types::CanisterId;
use pocket_ic::common::rest::{
    ApiResponse, MAX_INSTRUCTION_PROFILING_EXECUTIONS, MemoryAccessTrace, RawCanisterId,
    RawClearMemoryAccessTrace, RawStartInstructionProfiling,
};
use std::sync::Arc;

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CaptureMemorySnapshotRequest {
    #[serde(with = "pocket_ic::common::rest::base64")]
    #[schemars(with = "String")]
    pub canister_id: Vec<u8>,
    pub label: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct DeleteMemorySnapshotRequest {
    pub snapshot_id: String,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct DeleteMemorySnapshotResponse {
    pub deleted: bool,
}

pub(crate) async fn handler_capture_memory_snapshot(
    State(AppState { api_state, .. }): State<AppState>,
    Path(instance_id): Path<InstanceId>,
    headers: HeaderMap,
    Json(request): Json<CaptureMemorySnapshotRequest>,
) -> (StatusCode, Json<ApiResponse<MemorySnapshotMetadata>>) {
    let canister_id = match CanisterId::try_from(request.canister_id) {
        Ok(canister_id) => canister_id,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::Error {
                    message: format!("{error:?}"),
                }),
            );
        }
    };
    let op = CaptureMemorySnapshot::new(instance_id, canister_id, request.label);
    let (status, response) =
        run_operation(api_state, instance_id, timeout_or_default(headers), op).await;
    (status, Json(response))
}

/// Global retained captures remain deletable after their IC instance is gone.
pub async fn handler_delete_memory_snapshot(
    Path(instance_id): Path<InstanceId>,
    Json(request): Json<DeleteMemorySnapshotRequest>,
) -> (StatusCode, Json<ApiResponse<DeleteMemorySnapshotResponse>>) {
    let snapshot = match crate::memory_snapshots::get_snapshot(&request.snapshot_id) {
        Ok(snapshot) if snapshot.metadata().instance_id == instance_id => snapshot,
        Ok(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::Error {
                    message: "snapshot belongs to a different instance".to_string(),
                }),
            );
        }
        Err(error) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::Error { message: error }),
            );
        }
    };
    match crate::memory_snapshots::delete_snapshot(&snapshot.metadata().snapshot_id) {
        Ok(deleted) => (
            StatusCode::OK,
            Json(ApiResponse::Success(DeleteMemorySnapshotResponse {
                deleted,
            })),
        ),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::Error { message: error }),
        ),
    }
}

pub async fn handler_get_memory_access_trace(
    State(AppState { api_state, .. }): State<AppState>,
    Path(instance_id): Path<InstanceId>,
    headers: HeaderMap,
    axum::extract::Json(raw_canister_id): axum::extract::Json<RawCanisterId>,
) -> (StatusCode, Json<ApiResponse<MemoryAccessTrace>>) {
    let timeout = timeout_or_default(headers);
    match CanisterId::try_from(raw_canister_id.canister_id) {
        Ok(canister_id) => {
            let op = GetMemoryAccessTrace { canister_id };
            let (code, response) = run_operation(api_state, instance_id, timeout, op).await;
            (code, Json(response))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::Error {
                message: format!("{e:?}"),
            }),
        ),
    }
}
pub async fn handler_start_instruction_profiling(
    State(AppState { api_state, .. }): State<AppState>,
    Path(instance_id): Path<InstanceId>,
    headers: HeaderMap,
    axum::extract::Json(raw): axum::extract::Json<RawStartInstructionProfiling>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    if !(1..=MAX_INSTRUCTION_PROFILING_EXECUTIONS).contains(&raw.max_executions) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::Error {
                message: format!(
                    "max_executions must be between 1 and {MAX_INSTRUCTION_PROFILING_EXECUTIONS}"
                ),
            }),
        );
    }
    let timeout = timeout_or_default(headers);
    match CanisterId::try_from(raw.canister_id) {
        Ok(canister_id) => {
            let op = StartInstructionProfiling {
                canister_id,
                max_executions: raw.max_executions,
                read_mask: raw.read_mask,
                write_mask: raw.write_mask,
            };
            let (code, response) = run_operation(api_state, instance_id, timeout, op).await;
            (code, Json(response))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::Error {
                message: format!("{e:?}"),
            }),
        ),
    }
}

pub async fn handler_start_memory_access_tracing(
    State(AppState { api_state, .. }): State<AppState>,
    Path(instance_id): Path<InstanceId>,
    headers: HeaderMap,
    axum::extract::Json(raw_canister_id): axum::extract::Json<RawCanisterId>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let timeout = timeout_or_default(headers);
    match CanisterId::try_from(raw_canister_id.canister_id) {
        Ok(canister_id) => {
            let op = StartMemoryAccessTracing { canister_id };
            let (code, response) = run_operation(api_state, instance_id, timeout, op).await;
            (code, Json(response))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::Error {
                message: format!("{e:?}"),
            }),
        ),
    }
}

pub async fn handler_stop_memory_access_tracing(
    State(AppState { api_state, .. }): State<AppState>,
    Path(instance_id): Path<InstanceId>,
    headers: HeaderMap,
    axum::extract::Json(raw_canister_id): axum::extract::Json<RawCanisterId>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let timeout = timeout_or_default(headers);
    match CanisterId::try_from(raw_canister_id.canister_id) {
        Ok(canister_id) => {
            let op = StopMemoryAccessTracing { canister_id };
            let (code, response) = run_operation(api_state, instance_id, timeout, op).await;
            (code, Json(response))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::Error {
                message: format!("{e:?}"),
            }),
        ),
    }
}

pub async fn handler_clear_memory_access_trace(
    State(AppState { api_state, .. }): State<AppState>,
    Path(instance_id): Path<InstanceId>,
    headers: HeaderMap,
    axum::extract::Json(raw): axum::extract::Json<RawClearMemoryAccessTrace>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let timeout = timeout_or_default(headers);
    if raw.expected_trace_generation.is_none()
        && (raw.expected_last_sequence_number.is_some()
            || raw.expected_server_generation.is_some()
            || raw.expected_last_runtime_cost_sequence_number.is_some())
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::Error {
                message: "snapshot clear guards require expected_trace_generation".to_string(),
            }),
        );
    }
    match CanisterId::try_from(raw.canister_id) {
        Ok(canister_id) => {
            let op = ClearMemoryAccessTrace {
                canister_id,
                expected_server_generation: raw.expected_server_generation,
                expected_trace_generation: raw.expected_trace_generation,
                expected_last_sequence_number: raw.expected_last_sequence_number,
                expected_last_runtime_cost_sequence_number: raw
                    .expected_last_runtime_cost_sequence_number,
            };
            let (code, response) = run_operation(api_state, instance_id, timeout, op).await;
            (code, Json(response))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::Error {
                message: format!("{e:?}"),
            }),
        ),
    }
}

impl TryFrom<OpOut> for MemoryAccessTrace {
    type Error = OpConversionError;
    fn try_from(value: OpOut) -> Result<Self, Self::Error> {
        match value {
            OpOut::MemoryAccessTrace(trace) => Ok(Arc::unwrap_or_clone(trace)),
            _ => Err(OpConversionError),
        }
    }
}

impl TryFrom<OpOut> for MemorySnapshotMetadata {
    type Error = OpConversionError;

    fn try_from(value: OpOut) -> Result<Self, Self::Error> {
        match value {
            OpOut::Bytes(bytes) => serde_json::from_slice(&bytes).map_err(|_| OpConversionError),
            _ => Err(OpConversionError),
        }
    }
}
