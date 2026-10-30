//! Textual-canister profiling boundary for clients that do not own IC codecs or
//! the PocketIC operation graph. State-changing operations are submitted once;
//! a Started reply is observed to completion rather than resubmitted.
use super::arguments::DEFAULT_PROFILE_EXECUTIONS;
use crate::memory_snapshots::MemorySnapshotMetadata;
use crate::pocket_ic::{
    CaptureMemorySnapshot, ClearMemoryAccessTrace, GetMemoryAccessTrace, StartInstructionProfiling,
    StartMemoryAccessTracing, StopMemoryAccessTracing,
};
use crate::state_api::routes::{AppState, FromOpOut, run_operation};
use crate::state_api::state::StateLabel;
use crate::{InstanceId, OpId, Operation};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::URL_SAFE};
use ic_types::CanisterId;
use pocket_ic::common::rest::{
    ApiResponse, MAX_INSTRUCTION_PROFILING_EXECUTIONS, MemoryAccessTrace, trace_u64,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{str::FromStr, time::Duration};
use tokio::time::Instant;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Action {
    Start,
    Stop,
    Clear,
    Trace,
    Profile,
    Snapshot,
}

#[derive(Default, Deserialize)]
pub(crate) struct Options {
    label: Option<String>,
    max_executions: Option<u64>,
    #[serde(default)]
    read_mask: bool,
    #[serde(default)]
    write_mask: bool,
    expected_server_generation: Option<String>,
    #[serde(default, with = "trace_u64::option")]
    expected_trace_generation: Option<u64>,
    #[serde(default, with = "trace_u64::option")]
    expected_last_sequence_number: Option<u64>,
    #[serde(default, with = "trace_u64::present_option")]
    expected_last_runtime_cost_sequence_number: Option<Option<u64>>,
}

fn error(code: StatusCode, message: impl Into<String>) -> Response {
    (code, Json(json!({"message":message.into()}))).into_response()
}

pub(crate) async fn call(
    State(app): State<AppState>,
    Path((instance, text, action)): Path<(InstanceId, String, Action)>,
    Json(options): Json<Options>,
) -> Response {
    let canister_id = match CanisterId::from_str(&text) {
        Ok(id) => id,
        Err(e) => {
            return error(
                StatusCode::BAD_REQUEST,
                format!("Invalid canister id `{text}`: {e}"),
            );
        }
    };
    match action {
        Action::Start => {
            complete::<(), _>(&app, instance, StartMemoryAccessTracing { canister_id }).await
        }
        Action::Stop => {
            complete::<(), _>(&app, instance, StopMemoryAccessTracing { canister_id }).await
        }
        Action::Trace => {
            complete::<MemoryAccessTrace, _>(&app, instance, GetMemoryAccessTrace { canister_id })
                .await
        }
        Action::Clear => {
            complete::<(), _>(
                &app,
                instance,
                ClearMemoryAccessTrace {
                    canister_id,
                    expected_server_generation: options.expected_server_generation,
                    expected_trace_generation: options.expected_trace_generation,
                    expected_last_sequence_number: options.expected_last_sequence_number,
                    expected_last_runtime_cost_sequence_number: options
                        .expected_last_runtime_cost_sequence_number,
                },
            )
            .await
        }
        Action::Profile => {
            let max_executions = options.max_executions.unwrap_or(DEFAULT_PROFILE_EXECUTIONS);
            if !(1..=MAX_INSTRUCTION_PROFILING_EXECUTIONS).contains(&max_executions) {
                return error(
                    StatusCode::BAD_REQUEST,
                    format!(
                        "max_executions must be between 1 and {MAX_INSTRUCTION_PROFILING_EXECUTIONS}"
                    ),
                );
            }
            complete::<(), _>(
                &app,
                instance,
                StartInstructionProfiling {
                    canister_id,
                    max_executions,
                    read_mask: options.read_mask,
                    write_mask: options.write_mask,
                },
            )
            .await
        }
        Action::Snapshot => {
            complete::<MemorySnapshotMetadata, _>(
                &app,
                instance,
                CaptureMemorySnapshot::new(instance, canister_id, options.label),
            )
            .await
        }
    }
}

async fn complete<T, O>(app: &AppState, instance: InstanceId, op: O) -> Response
where
    T: Serialize + FromOpOut,
    O: Operation + Clone + Send + Sync + 'static,
{
    let deadline = Instant::now() + Duration::from_secs(300);
    let mut delay = Duration::from_millis(10);
    loop {
        let result = tokio::time::timeout_at(
            deadline,
            run_operation::<T>(
                app.api_state.clone(),
                instance,
                Some(Duration::ZERO),
                op.clone(),
            ),
        )
        .await;
        let Ok((code, response)) = result else {
            return error(
                StatusCode::GATEWAY_TIMEOUT,
                "timed out waiting for the operation; it may still complete",
            );
        };
        match response {
            ApiResponse::Success(value) => return Json(value).into_response(),
            ApiResponse::Error { message } => return error(code, message),
            ApiResponse::Busy { .. } => {
                if Instant::now() + delay >= deadline {
                    return error(
                        StatusCode::GATEWAY_TIMEOUT,
                        "the PocketIC instance stayed busy for too long",
                    );
                }
                tokio::time::sleep(delay).await;
                delay = (delay * 2).min(Duration::from_secs(1));
            }
            ApiResponse::Started { state_label, op_id } => {
                let label = URL_SAFE
                    .decode(state_label)
                    .ok()
                    .and_then(|bytes| StateLabel::try_from(bytes).ok());
                let Some(label) = label else {
                    return error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "invalid operation graph state label",
                    );
                };
                let op_id = OpId(op_id);
                loop {
                    if let Some((_, output)) =
                        app.api_state.read_graph(label.clone(), op_id.clone()).await
                    {
                        let (code, response) = <T as FromOpOut>::from(output).await;
                        return match response {
                            ApiResponse::Success(value) => {
                                app.api_state.prune_graph(label, op_id).await;
                                Json(value).into_response()
                            }
                            ApiResponse::Error { message } => error(code, message),
                            _ => error(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "unexpected operation graph result",
                            ),
                        };
                    }
                    if Instant::now() >= deadline {
                        return error(
                            StatusCode::GATEWAY_TIMEOUT,
                            "timed out waiting for the operation; it may still complete",
                        );
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clear_guards_keep_wide_values_and_missing_vs_null_runtime_cursor() {
        let absent: Options = serde_json::from_value(json!({})).unwrap();
        assert_eq!(absent.expected_last_runtime_cost_sequence_number, None);
        let empty: Options =
            serde_json::from_value(json!({"expected_last_runtime_cost_sequence_number":null}))
                .unwrap();
        assert_eq!(empty.expected_last_runtime_cost_sequence_number, Some(None));
        let wide: Options = serde_json::from_value(json!({"expected_trace_generation":u64::MAX.to_string(),"expected_last_sequence_number":"0","expected_last_runtime_cost_sequence_number":u64::MAX.to_string()})).unwrap();
        assert_eq!(wide.expected_trace_generation, Some(u64::MAX));
        assert_eq!(
            wide.expected_last_runtime_cost_sequence_number,
            Some(Some(u64::MAX))
        );
        assert!(
            serde_json::from_value::<Options>(json!({"expected_trace_generation":1.5})).is_err()
        );
    }
}
