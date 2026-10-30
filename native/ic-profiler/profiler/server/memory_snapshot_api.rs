//! Agent-facing PNG exports and exact snapshot comparisons.
use crate::InstanceId;
use crate::memory_access_tracing::SnapshotEvidence;
use crate::memory_snapshot_image::{
    MemorySnapshotImage, MemorySnapshotImageInput, MemorySnapshotImagePage,
};
#[cfg(test)]
use crate::memory_snapshot_image::{MemorySnapshotReadRange, MemorySnapshotWriteRange};
use crate::memory_snapshots::{self, MemoryKind, MemorySnapshotMetadata};
use crate::pocket_ic::CaptureMemorySnapshot;
use crate::state_api::routes::{AppState, run_operation};
use axum::http::StatusCode;
use ic_types::CanisterId;
use pocket_ic::common::rest::ApiResponse;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{str::FromStr, time::Duration};

pub(crate) type SnapshotResult<T> = Result<T, (StatusCode, String)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SnapshotMemory {
    Heap,
    Stable,
    Both,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SnapshotSelection {
    Touched,
    Range,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SnapshotView {
    Data,
    Reads,
    Writes,
}

#[derive(Default, Deserialize)]
pub(crate) struct SnapshotImageQuery {
    pub view: Option<SnapshotView>,
    pub memory: Option<SnapshotMemory>,
    pub selection: Option<SnapshotSelection>,
    pub start_page: Option<String>,
    pub page_offset: Option<String>,
    pub page_count: Option<String>,
    pub compare_to: Option<String>,
    pub attempt_id: Option<String>,
    pub logical_execution_id: Option<String>,
}

impl SnapshotImageQuery {
    pub(crate) fn from_arguments(arguments: &Value) -> Result<Self, String> {
        let text = |key: &str| {
            arguments
                .get(key)
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .ok_or_else(|| format!("{key} must be a string"))
                })
                .transpose()
        };
        let number = |key: &str| {
            arguments
                .get(key)
                .map(|value| {
                    if let Some(number) = value.as_u64() {
                        Ok(number.to_string())
                    } else if let Some(number) = value.as_str() {
                        parse_quantity(number, key).map(|n| n.to_string())
                    } else {
                        Err(format!(
                            "{key} must be an unsigned integer or decimal string"
                        ))
                    }
                })
                .transpose()
        };
        let memory = text("memory")?
            .map(|value| match value.as_str() {
                "heap" => Ok(SnapshotMemory::Heap),
                "stable" => Ok(SnapshotMemory::Stable),
                "both" => Ok(SnapshotMemory::Both),
                _ => Err("memory must be heap, stable or both".to_string()),
            })
            .transpose()?;
        Ok(Self {
            view: text("view")?
                .map(|value| match value.as_str() {
                    "data" => Ok(SnapshotView::Data),
                    "reads" => Ok(SnapshotView::Reads),
                    "writes" => Ok(SnapshotView::Writes),
                    _ => Err("view must be data, reads or writes".to_string()),
                })
                .transpose()?,
            memory,
            selection: text("selection")?
                .map(|value| match value.as_str() {
                    "touched" => Ok(SnapshotSelection::Touched),
                    "range" => Ok(SnapshotSelection::Range),
                    _ => Err("selection must be touched or range".to_string()),
                })
                .transpose()?,
            start_page: number("start_page")?,
            page_offset: number("page_offset")?,
            page_count: number("page_count")?,
            compare_to: text("compare_to")?,
            attempt_id: number("attempt_id")?,
            logical_execution_id: number("logical_execution_id")?,
        })
    }
}

fn parse_quantity(value: &str, field: &str) -> Result<u64, String> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{field} must be an unsigned decimal integer"));
    }
    value.parse().map_err(|_| format!("{field} exceeds u64"))
}

fn snapshot_error(message: String) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, message)
}

pub(crate) fn snapshot_urls(base_url: &str, snapshot_id: &str) -> Value {
    json!({
        "summary": format!("{base_url}/api/memory-snapshots/{snapshot_id}"),
        "image": format!("{base_url}/api/memory-snapshots/{snapshot_id}/image.png"),
        "delete": format!("{base_url}/api/memory-snapshots/{snapshot_id}"),
    })
}

pub(crate) async fn capture(
    app: &AppState,
    base_url: &str,
    instance_id: InstanceId,
    canister_id: &str,
    label: Option<String>,
) -> SnapshotResult<Value> {
    let canister =
        CanisterId::from_str(canister_id).map_err(|error| snapshot_error(error.to_string()))?;
    let op = CaptureMemorySnapshot::new(instance_id, canister, label);
    let (status, response): (_, ApiResponse<MemorySnapshotMetadata>) = run_operation(
        app.api_state.clone(),
        instance_id,
        Some(Duration::from_secs(10)),
        op,
    )
    .await;
    match response {
        ApiResponse::Success(metadata) => {
            let urls = snapshot_urls(base_url, &metadata.snapshot_id);
            let mut value =
                serde_json::to_value(metadata).map_err(|e| snapshot_error(e.to_string()))?;
            value["urls"] = urls;
            Ok(value)
        }
        ApiResponse::Error { message } => Err((status, message)),
        ApiResponse::Started { state_label, op_id } => Err((
            status,
            format!(
                "Capture still running; poll /read_graph/{state_label}/{op_id} on the PocketIC server to retrieve its snapshot ID."
            ),
        )),
        ApiResponse::Busy { .. } => Err((
            StatusCode::CONFLICT,
            "PocketIC instance is busy; retry capture at the intended boundary.".to_string(),
        )),
    }
}

pub(crate) fn delete(snapshot_id: &str) -> SnapshotResult<Value> {
    let deleted = memory_snapshots::delete_snapshot(snapshot_id).map_err(snapshot_error)?;
    Ok(json!({"snapshot_id": snapshot_id, "deleted": deleted}))
}

/// Skip compressed ranges arithmetically: even a 500 GiB stable allocation
/// cannot turn a bounded image request into a dense page scan.
fn touched_window(
    known: &[Vec<(u64, u64)>; 2],
    memory: SnapshotMemory,
    offset: u64,
    limit: u64,
) -> (Vec<(MemoryKind, u64)>, u64) {
    let mut pages = Vec::new();
    let mut skip = offset;
    let mut total = 0_u64;
    for (kind, ranges) in [
        (MemoryKind::Heap, &known[0]),
        (MemoryKind::Stable, &known[1]),
    ] {
        if (memory == SnapshotMemory::Heap && kind != MemoryKind::Heap)
            || (memory == SnapshotMemory::Stable && kind != MemoryKind::Stable)
        {
            continue;
        }
        for &(start, end) in ranges {
            let length = end.saturating_sub(start);
            total = total.saturating_add(length);
            let skipped = skip.min(length);
            skip -= skipped;
            let take = (length - skipped).min(limit.saturating_sub(pages.len() as u64));
            for page in start + skipped..start + skipped + take {
                pages.push((kind, page));
            }
        }
    }
    (pages, total)
}

fn memory_name(memory: MemoryKind) -> &'static str {
    match memory {
        MemoryKind::Heap => "heap",
        MemoryKind::Stable => "stable",
    }
}

pub(crate) fn render(
    app: &AppState,
    base_url: &str,
    snapshot_id: &str,
    query: SnapshotImageQuery,
) -> SnapshotResult<MemorySnapshotImage> {
    let after = memory_snapshots::get_snapshot(snapshot_id)
        .map_err(|message| (StatusCode::NOT_FOUND, message))?;
    let before = query
        .compare_to
        .as_deref()
        .map(memory_snapshots::get_snapshot)
        .transpose()
        .map_err(|message| (StatusCode::NOT_FOUND, message))?;
    let metadata = after.metadata();
    if let Some(before) = &before {
        let old = before.metadata();
        if metadata.server_generation != old.server_generation
            || metadata.instance_generation != old.instance_generation
            || metadata.canister_id != old.canister_id
        {
            return Err(snapshot_error("Comparison snapshots must belong to the same canister, instance and server generation.".to_string()));
        }
        if old.state_height > metadata.state_height
            || old.state_time_nanos > metadata.state_time_nanos
        {
            return Err(snapshot_error(
                "compare_to must be the earlier operation-boundary snapshot.".to_string(),
            ));
        }
    }
    let quantity = |value: &Option<String>, field: &str| {
        value
            .as_deref()
            .map(|v| parse_quantity(v, field))
            .transpose()
            .map_err(snapshot_error)
    };
    let view = query.view.unwrap_or(SnapshotView::Data);
    let memory = query.memory.unwrap_or(SnapshotMemory::Both);
    let selection = query.selection.unwrap_or(SnapshotSelection::Touched);
    let offset = quantity(&query.page_offset, "page_offset")?.unwrap_or(0);
    let attempt = quantity(&query.attempt_id, "attempt_id")?;
    let logical = quantity(&query.logical_execution_id, "logical_execution_id")?;
    if attempt.is_some() && logical.is_some() {
        return Err(snapshot_error(
            "Select attempt_id or logical_execution_id, not both.".to_string(),
        ));
    }
    let count = quantity(&query.page_count, "page_count")?.unwrap_or(match selection {
        SnapshotSelection::Touched => crate::memory_snapshot_image::MAX_IMAGE_PAGES as u64,
        SnapshotSelection::Range => 1,
    });
    if !(1..=crate::memory_snapshot_image::MAX_IMAGE_PAGES as u64).contains(&count) {
        return Err(snapshot_error(format!(
            "page_count must be between 1 and {}",
            crate::memory_snapshot_image::MAX_IMAGE_PAGES
        )));
    }
    let window = match &before {
        Some(before) if before.metadata().trace_generation == metadata.trace_generation => {
            let old = before.metadata();
            let recorder = app
                .api_state
                .memory_access_recorders()
                .remove(&metadata.instance_id);
            if let Some(recorder) = recorder {
                let canister = CanisterId::from_str(&metadata.canister_id)
                    .map_err(|e| snapshot_error(e.to_string()))?;
                let mut evidence = recorder.recorder.snapshot_touched_pages(
                    canister,
                    metadata.trace_generation,
                    old.last_sequence_number,
                    metadata.last_sequence_number,
                    attempt,
                    logical,
                );
                evidence.metadata["tracing_enabled_at_both_boundaries"] =
                    json!(old.tracing_enabled && metadata.tracing_enabled);
                evidence
            } else {
                SnapshotEvidence::unavailable("Execution recorder is unavailable.")
            }
        }
        Some(_) => SnapshotEvidence::unavailable("Trace generation changed between captures."),
        None => SnapshotEvidence::unavailable(
            "Provide compare_to for execution evidence between captures.",
        ),
    };
    let evidence = &window.metadata;
    let start_page = quantity(&query.start_page, "start_page")?.unwrap_or(0);
    if view == SnapshotView::Reads
        && (before.is_none() || evidence["read_profile"]["available"] != true)
    {
        return Err(snapshot_error(
                "view=reads requires matching before/after captures with retained opt-in byte-read profiling. Enable byte-read profiling before the call/install; existing page traces and snapshot bytes cannot reconstruct reads.".to_string(),
            ));
    }
    if view == SnapshotView::Writes
        && (before.is_none() || evidence["write_profile"]["available"] != true)
    {
        return Err(snapshot_error(
                "view=writes requires matching before/after captures with retained opt-in byte-write profiling. Enable byte-write profiling before the call/install; page dirty flags and snapshot differences cannot reconstruct individual writes.".to_string(),
            ));
    }
    let (selected, total) = match selection {
        SnapshotSelection::Touched => {
            if query.start_page.is_some() {
                return Err(snapshot_error("start_page applies only to selection=range; touched selection uses page_offset.".to_string()));
            }
            if before.is_none() || evidence["available"] != true {
                return Err(snapshot_error(format!(
                    "Touched selection requires matching before/after captures with retained trace cursors. {} Enable tracing before the operation and capture both boundaries.",
                    evidence["reason"].as_str().unwrap_or("")
                )));
            }
            let (pages, total) = touched_window(&window.touched, memory, offset, count);
            if total == 0 {
                return Err(snapshot_error("No recorded touched blocks in this selected window. Enable tracing before the call/install, capture both boundaries, and check attempt/logical selection. Missing coverage is not a known empty access set; use selection=range for an explicit byte window.".to_string()));
            }
            if pages.is_empty() {
                return Err(snapshot_error(format!(
                    "page_offset {offset} is outside {total} known touched blocks."
                )));
            }
            (pages, total)
        }
        SnapshotSelection::Range => {
            if query.page_offset.is_some() {
                return Err(snapshot_error(
                    "selection=range uses start_page, not page_offset.".to_string(),
                ));
            }
            let kind = match memory {
                SnapshotMemory::Heap => MemoryKind::Heap,
                SnapshotMemory::Stable => MemoryKind::Stable,
                SnapshotMemory::Both => {
                    return Err(snapshot_error(
                        "selection=range requires memory=heap or memory=stable.".to_string(),
                    ));
                }
            };
            let end = start_page
                .checked_add(count)
                .ok_or_else(|| snapshot_error("Page address overflow.".to_string()))?;
            ((start_page..end).map(|page| (kind, page)).collect(), count)
        }
    };
    let pages = selected
        .iter()
        .map(|&(kind, page)| {
            let name = memory_name(kind);
            let positive = |ranges: &[Vec<(u64, u64)>; 2]| {
                ranges[usize::from(kind == MemoryKind::Stable)]
                    .iter()
                    .any(|&(start, end)| start <= page && page < end)
                    .then_some(true)
            };
            Ok(MemorySnapshotImagePage {
                memory: name.to_string(),
                page_index: page,
                allocated_bytes: metadata.allocated_bytes(kind),
                comparison_allocated_bytes: before
                    .as_ref()
                    .map(|b| b.metadata().allocated_bytes(kind)),
                bytes: if view != SnapshotView::Data {
                    vec![0; crate::memory_snapshot_image::WASM_PAGE_BYTES]
                } else {
                    after.read_pages(kind, page, 1).map_err(snapshot_error)?
                },
                comparison_bytes: if view != SnapshotView::Data {
                    before
                        .as_ref()
                        .map(|_| vec![0; crate::memory_snapshot_image::WASM_PAGE_BYTES])
                } else {
                    before
                        .as_ref()
                        .map(|b| b.read_pages(kind, page, 1))
                        .transpose()
                        .map_err(snapshot_error)?
                },
                accessed: positive(&window.touched),
                dirty: positive(&window.dirty),
            })
        })
        .collect::<SnapshotResult<Vec<_>>>()?;
    let name = match memory {
        SnapshotMemory::Heap => "heap",
        SnapshotMemory::Stable => "stable",
        SnapshotMemory::Both => "both",
    };
    let global_kind = if memory == SnapshotMemory::Stable {
        MemoryKind::Stable
    } else {
        MemoryKind::Heap
    };
    let input = MemorySnapshotImageInput {
        memory: name.to_string(),
        snapshot_id: snapshot_id.to_string(),
        comparison_snapshot_id: query.compare_to.clone(),
        allocated_bytes: metadata.allocated_bytes(global_kind),
        comparison_allocated_bytes: before
            .as_ref()
            .map(|b| b.metadata().allocated_bytes(global_kind)),
        pages,
    };
    let mut rendered = match view {
        SnapshotView::Reads => crate::memory_snapshot_image::render_reads(
            &input,
            &window.reads,
            &evidence["read_profile"],
        ),
        SnapshotView::Writes => crate::memory_snapshot_image::render_writes(
            &input,
            &window.writes,
            &evidence["write_profile"],
        ),
        SnapshotView::Data => crate::memory_snapshot_image::render(&input),
    }
    .map_err(snapshot_error)?;
    rendered.summary["snapshot"] =
        serde_json::to_value(metadata).map_err(|e| snapshot_error(e.to_string()))?;
    rendered.summary["comparison_snapshot"] = before
        .as_ref()
        .map(|b| serde_json::to_value(b.metadata()))
        .transpose()
        .map_err(|e| snapshot_error(e.to_string()))?
        .unwrap_or(Value::Null);
    let next = offset.saturating_add(selected.len() as u64);
    let more = selection == SnapshotSelection::Touched && next < total;
    rendered.summary["selection"] = json!({
        "kind": if selection == SnapshotSelection::Touched { "touched" } else { "range" },
        "memory": name, "known_touched_pages": if selection == SnapshotSelection::Touched { Some(total.to_string()) } else { None },
        "page_offset": offset.to_string(), "page_count": selected.len().to_string(),
        "more": more, "next_page_offset": more.then(|| next.to_string()),
        "attempt_id": attempt.map(|n| n.to_string()), "logical_execution_id": logical.map(|n| n.to_string()),
        "continuous_recording_proven": false,
    });
    let mut evidence = window.into_summary();
    evidence["selected_blocks"] = json!(
        selected
            .iter()
            .map(
                |&(kind, page)| json!({"memory": memory_name(kind), "page_index": page.to_string()})
            )
            .collect::<Vec<_>>()
    );
    for (name, field) in [("reads", "read_profile"), ("writes", "write_profile")] {
        if rendered.summary[name]["counters_saturated"] == true {
            evidence[field]["counters_saturated"] = json!(true);
            evidence[field]["retained_coverage_complete"] = json!(false);
        }
        rendered.summary[field] = evidence[field].clone();
    }
    rendered.summary["execution_evidence"] = evidence;
    rendered.summary["stable_access_coverage"] = json!(
        "Positive 64 KiB coverage from the stable bytemap, including direct first reads when stable_pages_tracked is true. Legacy, aborted, missing or truncated details remain unavailable; no byte-level access history is inferred."
    );
    rendered.summary["comparison_scope"] = json!(
        "Committed operation boundaries. All recorded attempts in the interval are selected by default, possibly multiple messages/install stages/callbacks/timers. Optional attempt_id selects one physical attempt; logical_execution_id selects retained lifecycle stages/retries. Discarded attempts can touch blocks without changing committed bytes."
    );
    rendered.summary["module_changed"] = json!(
        before
            .as_ref()
            .is_some_and(|b| b.metadata().module_hash != metadata.module_hash)
    );
    let mut parameters = url::form_urlencoded::Serializer::new(String::new());
    parameters
        .append_pair(
            "view",
            match view {
                SnapshotView::Data => "data",
                SnapshotView::Reads => "reads",
                SnapshotView::Writes => "writes",
            },
        )
        .append_pair(
            "selection",
            if selection == SnapshotSelection::Touched {
                "touched"
            } else {
                "range"
            },
        )
        .append_pair("memory", name)
        .append_pair("page_count", &count.to_string());
    match selection {
        SnapshotSelection::Touched => {
            parameters.append_pair("page_offset", &offset.to_string());
        }
        SnapshotSelection::Range => {
            parameters.append_pair("start_page", &start_page.to_string());
        }
    }
    if let Some(before) = &before {
        parameters.append_pair("compare_to", &before.metadata().snapshot_id);
    }
    if let Some(attempt) = attempt {
        parameters.append_pair("attempt_id", &attempt.to_string());
    }
    if let Some(logical) = logical {
        parameters.append_pair("logical_execution_id", &logical.to_string());
    }
    let parameters = parameters.finish();
    let mut urls = snapshot_urls(base_url, snapshot_id);
    urls["summary"] = json!(format!(
        "{base_url}/api/memory-snapshots/{snapshot_id}?{parameters}"
    ));
    urls["image"] = json!(format!(
        "{base_url}/api/memory-snapshots/{snapshot_id}/image.png?{parameters}"
    ));
    rendered.summary["urls"] = urls;
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touched_pagination_is_a_sparse_stable_heap_union_with_exact_totals() {
        let evidence = [vec![(0, 2), (9, 10)], vec![(7_000_000, 8_000_000)]];
        let (pages, total) = touched_window(&evidence, SnapshotMemory::Both, 2, 3);
        assert_eq!(total, 1_000_003);
        assert_eq!(
            pages,
            vec![
                (MemoryKind::Heap, 9),
                (MemoryKind::Stable, 7_000_000),
                (MemoryKind::Stable, 7_000_001)
            ]
        );
        let (pages, total) = touched_window(&evidence, SnapshotMemory::Stable, 999_999, 256);
        assert_eq!(total, 1_000_000);
        assert_eq!(pages, vec![(MemoryKind::Stable, 7_999_999)]);
        let (pages, total) = touched_window(&evidence, SnapshotMemory::Both, 1_000_003, 256);
        assert_eq!(total, 1_000_003);
        assert!(pages.is_empty());
    }

    #[test]
    fn mixed_touched_query_preserves_exact_lifecycle_ids_and_pagination() {
        let query = SnapshotImageQuery::from_arguments(&json!({
            "view": "reads", "memory": "both", "selection": "touched", "page_offset": "9007199254740993",
            "page_count": 256, "logical_execution_id": "18446744073709551615"
        }))
        .unwrap();
        assert_eq!(query.memory, Some(SnapshotMemory::Both));
        assert_eq!(query.view, Some(SnapshotView::Reads));
        assert_eq!(query.selection, Some(SnapshotSelection::Touched));
        assert_eq!(query.page_offset.as_deref(), Some("9007199254740993"));
        assert_eq!(
            query.logical_execution_id.as_deref(),
            Some("18446744073709551615")
        );
        for value in [
            json!({"selection":"guessed"}),
            json!({"view":"guessed"}),
            json!({"attempt_id":-1}),
            json!({"page_offset":"1.5"}),
        ] {
            assert!(SnapshotImageQuery::from_arguments(&value).is_err());
        }
    }

    #[test]
    fn byte_read_ranges_preserve_addresses_and_counts_without_rounding() {
        let ranges: Vec<MemorySnapshotReadRange> = serde_json::from_value(json!([{
            "memory":"stable", "start":"9007199254740993", "end":"9007199254741000",
            "heap_reads":"0", "stable_direct_reads":"1", "stable_native_reads":"18446744073709551615"
        }])).unwrap();
        assert_eq!(ranges[0].start, 9_007_199_254_740_993);
        assert_eq!(ranges[0].end, 9_007_199_254_741_000);
        assert_eq!(ranges[0].stable_native_reads, u64::MAX);
        assert!(
            serde_json::from_value::<Vec<MemorySnapshotReadRange>>(
                json!([{"memory": "heap", "start": "1.5"}])
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<Vec<MemorySnapshotReadRange>>(json!({"available": true}))
                .is_err()
        );
    }

    #[test]
    fn write_mask_arguments_and_ranges_preserve_exact_byte_counts() {
        let query = SnapshotImageQuery::from_arguments(&json!({
            "view": "writes", "memory": "both", "compare_to": "before"
        }))
        .unwrap();
        assert_eq!(query.view, Some(SnapshotView::Writes));
        let ranges: Vec<MemorySnapshotWriteRange> = serde_json::from_value(json!([{
            "memory": "heap", "start": "9007199254740993", "end": "9007199254741001",
            "heap_writes": "18446744073709551615", "stable_writes": 0
        }]))
        .unwrap();
        assert_eq!(ranges[0].start, 9_007_199_254_740_993);
        assert_eq!(ranges[0].end, 9_007_199_254_741_001);
        assert_eq!(ranges[0].heap_writes, u64::MAX);
        assert_eq!(ranges[0].stable_writes, 0);
        assert!(
            serde_json::from_value::<Vec<MemorySnapshotWriteRange>>(
                json!([{"memory": "heap", "start": -1}])
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<Vec<MemorySnapshotWriteRange>>(json!({"available": true}))
                .is_err()
        );
    }

    #[test]
    fn image_arguments_preserve_exact_u64_pages_and_reject_ambiguous_inputs() {
        let query = SnapshotImageQuery::from_arguments(&json!({
            "memory": "stable", "start_page": "9007199254740993", "page_count": "16", "compare_to": "before"
        })).unwrap();
        assert_eq!(query.start_page.as_deref(), Some("9007199254740993"));
        assert_eq!(query.memory, Some(SnapshotMemory::Stable));
        assert_eq!(query.compare_to.as_deref(), Some("before"));
        for value in [
            json!({"start_page": -1}),
            json!({"start_page": "1e3"}),
            json!({"start_page": "18446744073709551616"}),
            json!({"memory": "wasm"}),
            json!({"page_count": 1.5}),
            json!({"compare_to": 4}),
        ] {
            assert!(
                SnapshotImageQuery::from_arguments(&value).is_err(),
                "{value}"
            );
        }
        assert_eq!(
            parse_quantity("18446744073709551615", "page").unwrap(),
            u64::MAX
        );
    }
}
