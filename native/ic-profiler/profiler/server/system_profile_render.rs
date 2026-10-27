//! Deterministic image projections of immutable system-profiler queries.
use flate2::{Compression, write::ZlibEncoder};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
};

pub struct RenderedProfile {
    pub svg: String,
    pub png: Vec<u8>,
    pub metadata: Value,
}

type Color = [u8; 3];
const INK: Color = [31, 41, 55];
const GRID: Color = [226, 232, 240];
const MUTED: Color = [100, 116, 139];
const COLORS: [Color; 8] = [
    [59, 130, 246],
    [20, 184, 166],
    [139, 92, 246],
    [245, 158, 11],
    [14, 165, 233],
    [99, 102, 241],
    [16, 185, 129],
    [236, 72, 153],
];

struct Scene {
    width: usize,
    height: usize,
    rgb: Vec<u8>,
    svg: String,
}

impl Scene {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            rgb: vec![255; width * height * 3],
            svg: format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\"><rect width=\"100%\" height=\"100%\" fill=\"white\"/>"
            ),
        }
    }
    fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        for py in y.min(self.height)..y.saturating_add(h).min(self.height) {
            for px in x.min(self.width)..x.saturating_add(w).min(self.width) {
                self.rgb[(py * self.width + px) * 3..(py * self.width + px) * 3 + 3]
                    .copy_from_slice(&color);
            }
        }
        self.svg.push_str(&format!(
            "<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"rgb({},{},{})\"/>",
            color[0], color[1], color[2]
        ));
    }
    fn line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: Color) {
        let steps = x0.abs_diff(x1).max(y0.abs_diff(y1)).max(1);
        for i in 0..=steps {
            let f = i as f64 / steps as f64;
            let x = (x0 as f64 + (x1 as f64 - x0 as f64) * f).round().max(0.0) as usize;
            let y = (y0 as f64 + (y1 as f64 - y0 as f64) * f).round().max(0.0) as usize;
            if x < self.width && y < self.height {
                self.rgb[(y * self.width + x) * 3..(y * self.width + x) * 3 + 3]
                    .copy_from_slice(&color);
            }
        }
        self.svg.push_str(&format!("<line x1=\"{x0}\" y1=\"{y0}\" x2=\"{x1}\" y2=\"{y1}\" stroke=\"rgb({},{},{})\" stroke-width=\"1.5\"/>",color[0],color[1],color[2]));
    }
    fn text(&mut self, x: usize, y: usize, text: &str, color: Color, scale: usize) {
        for (i, ch) in text.to_ascii_uppercase().chars().enumerate() {
            for (row, bits) in crate::memory_snapshot_image::glyph(ch).iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        let px = x + i * 6 * scale + col * scale;
                        let py = y + row * scale;
                        for dy in 0..scale {
                            for dx in 0..scale {
                                if px + dx < self.width && py + dy < self.height {
                                    let offset = ((py + dy) * self.width + px + dx) * 3;
                                    self.rgb[offset..offset + 3].copy_from_slice(&color);
                                }
                            }
                        }
                    }
                }
            }
        }
        self.svg.push_str(&format!("<text x=\"{x}\" y=\"{}\" font-family=\"ui-monospace,monospace\" font-size=\"{}\" fill=\"rgb({},{},{})\">{}</text>",y+8*scale,10*scale,color[0],color[1],color[2],escape(text)));
    }
    fn png(&self) -> Result<Vec<u8>, String> {
        let mut zip = ZlibEncoder::new(Vec::new(), Compression::fast());
        for row in self.rgb.chunks_exact(self.width * 3) {
            zip.write_all(&[0])
                .and_then(|_| zip.write_all(row))
                .map_err(|e| e.to_string())?;
        }
        let data = zip.finish().map_err(|e| e.to_string())?;
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = Vec::new();
        header.extend_from_slice(&(self.width as u32).to_be_bytes());
        header.extend_from_slice(&(self.height as u32).to_be_bytes());
        header.extend_from_slice(&[8, 2, 0, 0, 0]);
        for (kind, data) in [(*b"IHDR", header), (*b"IDAT", data), (*b"IEND", Vec::new())] {
            png.extend_from_slice(&(data.len() as u32).to_be_bytes());
            png.extend_from_slice(&kind);
            png.extend_from_slice(&data);
            let mut crc = crc32fast::Hasher::new();
            crc.update(&kind);
            crc.update(&data);
            png.extend_from_slice(&crc.finalize().to_be_bytes());
        }
        Ok(png)
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn numeric(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str()?.parse().ok())
}
fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("unavailable")
}
fn color(id: &str) -> Color {
    let hash = id
        .bytes()
        .fold(0usize, |a, b| a.wrapping_mul(31).wrapping_add(b as usize));
    COLORS[hash % COLORS.len()]
}
fn shorten(s: &str, n: usize) -> String {
    if s.chars().count() > n {
        format!(
            "{}…",
            s.chars().take(n.saturating_sub(1)).collect::<String>()
        )
    } else {
        s.into()
    }
}

fn exact_integer(value: &Value) -> Option<i128> {
    if let Some(value) = value.as_str() {
        value.parse().ok()
    } else if let Some(value) = value.as_i64() {
        Some(value.into())
    } else {
        value.as_u64().map(Into::into)
    }
}

/// Round values stay exact in metadata. Floating point is used only to project
/// bounded ratios into image coordinates, never to reconstruct accounting.
fn render_rounds(
    scene: &mut Scene,
    data: &Value,
    parameters: &HashMap<String, String>,
    top: usize,
    bottom: usize,
    start: f64,
    end: f64,
    elements: &mut Vec<Value>,
    legend: &mut BTreeMap<String, Value>,
) -> Result<Value, String> {
    let source = data.get("rounds").unwrap_or(&Value::Null);
    let rows = source.get("rows").and_then(Value::as_array);
    let configurations = source.get("configurations").unwrap_or(&Value::Null);
    let selected: Vec<_> = rows
        .into_iter()
        .flatten()
        .filter(|row| {
            let a = row.get("start_ns").and_then(numeric);
            let b = row.get("end_ns").and_then(numeric);
            match (a, b) {
                (Some(a), Some(b)) => b >= start && a <= end,
                _ => true,
            }
        })
        .collect();
    let selected_id = parameters.get("render_round_id").map(String::as_str);
    let focused = if let Some(id) = selected_id {
        Some(
            *selected
                .iter()
                .find(|row| row.get("id").and_then(Value::as_str) == Some(id))
                .ok_or("render_round_id is absent from the immutable selected round rows")?,
        )
    } else {
        selected.last().copied()
    };
    scene.text(
        20,
        top,
        "Scheduler allowance · logical instructions, not CPU",
        INK,
        1,
    );
    if focused.is_none() {
        scene.text(
            20,
            top + 32,
            "Dedicated round evidence unavailable or no matching rounds retained.",
            MUTED,
            1,
        );
        scene.text(
            20,
            top + 52,
            "Old imports, censored capture boundaries and record loss are not zero work.",
            MUTED,
            1,
        );
        return Ok(
            json!({"coverage":source.get("coverage"),"summary":source.get("summary"),"configurations":configurations,"rendered_rounds":0,"selected_rows":selected.len(),"missing_data_is_zero":false}),
        );
    }
    legend.insert("worker_budget_spent".into(),json!({"rgb":COLORS[0],"encoding":"logical allowance consumed, not CPU; overview uses nominal R as its reference"}));
    legend.insert("heap_delta".into(),json!({"rgb":COLORS[1],"encoding":"known round-cumulative logical heap delta / captured heap-delta limit; not RSS or disk bytes"}));
    legend.insert("heap_delta_threshold_reached".into(),json!({"rgb":COLORS[3],"encoding":"captured heap_delta_limit_reached=true; scheduler checks cumulative dirty-memory proxy between iterations"}));
    legend.insert("heap_delta_unavailable".into(),json!({"rgb":GRID,"encoding":"heap delta unknown; no zero value or threshold outcome inferred"}));
    let plot_x = 160usize;
    let plot_w = scene.width.saturating_sub(plot_x + 30);
    let overview_y = top + 25;
    let overview_h = ((bottom - top) / 4).clamp(50, 130);
    let maximum_rows = (plot_w / 5).clamp(1, 256);
    let shown: Vec<_> = selected
        .iter()
        .rev()
        .take(maximum_rows)
        .copied()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let max_ratio = shown
        .iter()
        .filter_map(|row| {
            let config = &configurations[text(row, "config_id")];
            let nominal = config
                .get("max_instructions_per_round")
                .and_then(exact_integer)?;
            let initial = row.get("initial_instructions").and_then(exact_integer)?;
            let remaining = row.get("remaining_instructions").and_then(exact_integer)?;
            (nominal > 0).then_some((initial - remaining).max(0) as f64 / nominal as f64)
        })
        .fold(1.0_f64, f64::max)
        .max(1.0);
    for fraction in [0.0, 0.5, 1.0] {
        let y = overview_y + overview_h - ((fraction / max_ratio) * overview_h as f64) as usize;
        scene.line(plot_x, y, scene.width - 30, y, GRID);
        scene.text(
            20,
            y.saturating_sub(4),
            &format!("{:.0}% nominal R", fraction * 100.0),
            MUTED,
            1,
        );
    }
    scene.text(
        20,
        overview_y + overview_h + 25,
        "Heap delta / limit",
        MUTED,
        1,
    );
    let col = (plot_w / shown.len().max(1)).max(1);
    for (i, row) in shown.iter().enumerate() {
        let config = &configurations[text(row, "config_id")];
        let nominal = config
            .get("max_instructions_per_round")
            .and_then(exact_integer);
        let initial = row.get("initial_instructions").and_then(exact_integer);
        let remaining = row.get("remaining_instructions").and_then(exact_integer);
        let consumed = initial.zip(remaining).map(|(a, b)| a - b);
        let x = plot_x + i * col;
        let ratio = consumed
            .zip(nominal)
            .filter(|(_, r)| *r > 0)
            .map(|(used, r)| (used.max(0) as f64 / r as f64).min(max_ratio));
        let h = ratio
            .map(|ratio| (ratio / max_ratio * overview_h as f64).round() as usize)
            .unwrap_or(0);
        let selected_round = focused.is_some_and(|f| f["id"] == row["id"]);
        let c = if selected_round { COLORS[2] } else { COLORS[0] };
        if ratio.is_some() {
            scene.rect(
                x,
                overview_y + overview_h - h,
                col.saturating_sub(2).max(1),
                h,
                c,
            )
        } else {
            scene.rect(
                x,
                overview_y + overview_h - 2,
                col.saturating_sub(2).max(1),
                2,
                MUTED,
            )
        }
        if let Some(initial_ratio) = initial
            .zip(nominal)
            .filter(|(_, r)| *r > 0)
            .map(|(initial, r)| initial.max(0) as f64 / r as f64)
        {
            let y = overview_y + overview_h
                - ((initial_ratio.min(max_ratio) / max_ratio) * overview_h as f64).round() as usize;
            scene.line(
                x,
                y,
                (x + col.saturating_sub(2)).min(scene.width - 30),
                y,
                COLORS[3],
            );
        }
        if i % ((shown.len() / 12).max(1)) == 0 || selected_round {
            scene.text(x, overview_y + overview_h + 8, text(row, "round"), MUTED, 1)
        }
        let heap = row.get("heap_delta_bytes").and_then(exact_integer);
        let heap_limit = config
            .get("max_heap_delta_per_iteration_bytes")
            .and_then(exact_integer);
        let heap_y = overview_y + overview_h + 25;
        let heap_width = col.saturating_sub(2).max(1);
        scene.rect(x, heap_y, heap_width, 6, GRID);
        if let Some(ratio) = heap
            .zip(heap_limit)
            .filter(|(_, limit)| *limit > 0)
            .map(|(bytes, limit)| (bytes.max(0) as f64 / limit as f64).clamp(0.0, 1.0))
        {
            let c = if row.get("heap_delta_limit_reached").and_then(Value::as_bool) == Some(true) {
                COLORS[3]
            } else {
                COLORS[1]
            };
            scene.rect(
                x,
                heap_y,
                (ratio * heap_width as f64).round() as usize,
                6,
                c,
            );
        }
        elements.push(json!({"kind":"round_budget","round_id":row.get("id"),"round":row.get("round"),"subnet_id":row.get("subnet_id"),"config_id":row.get("config_id"),"initial_instructions":row.get("initial_instructions"),"remaining_instructions":row.get("remaining_instructions"),"consumed_instructions":consumed.map(|v|v.to_string()),"nominal_per_worker_round_limit":config.get("max_instructions_per_round"),"stop_reasons":row.get("stop_reasons"),"heap_delta_bytes":row.get("heap_delta_bytes"),"max_heap_delta_per_iteration_bytes":config.get("max_heap_delta_per_iteration_bytes"),"heap_delta_limit_reached":row.get("heap_delta_limit_reached"),"heap_delta_gauge_bounds":[x,heap_y,heap_width,6],"heap_delta_semantics":"round-cumulative logical dirty-memory proxy checked between iterations, not RSS or actual disk","reserved_instructions":row.get("reserved_instructions"),"accounting_reconciles":row.get("accounting_reconciles"),"details_complete":row.get("details_complete"),"detail_mode":row.get("detail_mode"),"details_omitted_by_query":row.get("details_omitted_by_query"),"retained_iteration_count":row.get("retained_iteration_count"),"retained_worker_count":row.get("retained_worker_count"),"retained_canister_pass_count":row.get("retained_canister_pass_count"),"bounds":[x,overview_y,col,overview_h],"selected":selected_round}));
    }
    let focused = focused.unwrap();
    if bottom < overview_y + overview_h + 160 {
        scene.text(
            20,
            bottom.saturating_sub(18),
            "Worker detail requires a taller image; use the exact JSON evidence.",
            MUTED,
            1,
        );
        return Ok(
            json!({"coverage":source.get("coverage"),"summary":source.get("summary"),"configurations":configurations,"focused_round_id":focused.get("id"),"rendered_rounds":shown.len(),"selected_rows":selected.len(),"unrendered_round_rows":selected.len().saturating_sub(shown.len()),"detail_mode":source.get("detail_mode"),"details_omitted_by_query":focused.get("details_omitted_by_query"),"worker_detail":"omitted_due_to_image_height","logical_units":"round_instructions","changes_accounting_scope":false}),
        );
    }
    let config = &configurations[text(focused, "config_id")];
    let detail_y = overview_y + overview_h + 42;
    scene.text(
        20,
        detail_y,
        &shorten(
            &format!(
                "Round {} · {} · {}",
                text(focused, "round"),
                text(focused, "round_kind"),
                text(focused, "subnet_id")
            ),
            (scene.width - 40) / 6,
        ),
        INK,
        1,
    );
    scene.text(
        20,
        detail_y + 20,
        &shorten(
            &format!(
                "Nominal R/worker {} · initial {} · message {} · DTS slice {}",
                text(config, "max_instructions_per_round"),
                text(focused, "initial_instructions"),
                text(config, "max_instructions_per_message"),
                text(config, "max_instructions_per_slice")
            ),
            (scene.width - 40) / 6,
        ),
        MUTED,
        1,
    );
    let heap_evidence = json!({"bytes":focused.get("heap_delta_bytes"),"limit_bytes":config.get("max_heap_delta_per_iteration_bytes"),"limit_reached":focused.get("heap_delta_limit_reached"),"observed_stop_reasons":focused.get("stop_reasons"),"measurement":"round-cumulative logical dirty-memory proxy checked between scheduler iterations; not RSS or actual disk I/O"});
    let heap_status = match focused
        .get("heap_delta_limit_reached")
        .and_then(Value::as_bool)
    {
        Some(true) => "threshold reached",
        Some(false) => "below threshold",
        None => "threshold outcome unavailable",
    };
    scene.text(
        20,
        detail_y + 40,
        &shorten(
            &format!(
                "Heap delta {} / {} bytes · {heap_status}",
                text(focused, "heap_delta_bytes"),
                text(config, "max_heap_delta_per_iteration_bytes")
            ),
            (scene.width - 40) / 6,
        ),
        MUTED,
        1,
    );
    let compact = focused
        .get("details_omitted_by_query")
        .and_then(Value::as_bool)
        == Some(true)
        || source.get("detail_mode").and_then(Value::as_str) == Some("summary");
    if compact {
        scene.text(
            20,
            detail_y + 65,
            "Compact overview: worker/pass/slice detail was omitted by this query.",
            MUTED,
            1,
        );
        scene.text(
            20,
            detail_y + 85,
            "Fetch round_id + limit=1 + round_details=true for the selected timeline.",
            MUTED,
            1,
        );
        return Ok(
            json!({"coverage":source.get("coverage"),"summary":source.get("summary"),"configurations":configurations,"pagination":source.get("pagination"),"detail_mode":"summary","details_omitted_by_query":true,"source_details_complete":focused.get("details_complete"),"focused_round_id":focused.get("id"),"rendered_rounds":shown.len(),"selected_rows":selected.len(),"unrendered_round_rows":selected.len().saturating_sub(shown.len()),"worker_detail":"not_requested","heap_delta":heap_evidence,"logical_units":"round_instructions","changes_accounting_scope":false}),
        );
    }
    let initial = focused.get("initial_instructions").and_then(exact_integer);
    let nominal = config
        .get("max_instructions_per_round")
        .and_then(exact_integer);
    let iterations = focused.get("iterations").and_then(Value::as_array);
    let mut workers = BTreeMap::new();
    for iteration in iterations.into_iter().flatten() {
        for worker in iteration
            .get("workers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(id) = worker.get("worker").and_then(exact_integer) {
                workers.entry(id).or_insert(0usize);
            }
        }
    }
    let worker_top = detail_y + 70;
    let max_workers = ((bottom.saturating_sub(worker_top + 44)) / 38).max(1);
    for (index, (_, lane)) in workers.iter_mut().take(max_workers).enumerate() {
        *lane = index;
    }
    let worker_ids: Vec<_> = workers.keys().take(max_workers).copied().collect();
    let axis_max = initial.unwrap_or(0).max(nominal.unwrap_or(0)).max(1);
    let project = |amount: i128| {
        ((amount.max(0) as f64 / axis_max as f64) * plot_w as f64)
            .clamp(0.0, plot_w as f64)
            .round() as usize
    };
    for worker in &worker_ids {
        let index = workers[worker];
        let y = worker_top + index * 38;
        scene.text(20, y + 5, &format!("Worker {worker}"), INK, 1);
        scene.line(plot_x, y + 24, scene.width - 30, y + 24, GRID);
    }
    for iteration in iterations.into_iter().flatten() {
        let offset = initial
            .zip(iteration.get("budget_before").and_then(exact_integer))
            .map(|(a, b)| a - b);
        for worker in iteration
            .get("workers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(id) = worker.get("worker").and_then(exact_integer) else {
                continue;
            };
            if !worker_ids.contains(&id) {
                continue;
            }
            let consumption = worker
                .get("initial_instructions")
                .and_then(exact_integer)
                .zip(worker.get("remaining_instructions").and_then(exact_integer))
                .map(|(a, b)| a - b);
            let (Some(offset), Some(consumed)) = (offset, consumption) else {
                continue;
            };
            let x = plot_x + project(offset);
            let end_x = plot_x + project(offset.saturating_add(consumed));
            let y = worker_top + workers[&id] * 38;
            let c =
                if iteration.get("max_worker_consumed").and_then(exact_integer) == Some(consumed) {
                    COLORS[2]
                } else {
                    COLORS[0]
                };
            scene.rect(x, y, end_x.saturating_sub(x).max(1), 18, c);
            if end_x.saturating_sub(x) > 40 {
                scene.text(
                    x + 3,
                    y + 5,
                    &format!("i{}", text(iteration, "iteration")),
                    [255, 255, 255],
                    1,
                )
            }
            elements.push(json!({"kind":"worker_iteration","round_id":focused.get("id"),"iteration":iteration.get("iteration"),"worker":worker.get("worker"),"offset_instructions":offset.to_string(),"consumed_instructions":consumed.to_string(),"initial_instructions":worker.get("initial_instructions"),"remaining_instructions":worker.get("remaining_instructions"),"guest_instructions":worker.get("guest_instructions"),"heap_delta_bytes":worker.get("heap_delta_bytes"),"iteration_heap_delta_bytes":iteration.get("heap_delta_bytes"),"dirty_page_copy_instructions":worker.get("dirty_page_copy_instructions"),"execution_overhead_instructions":worker.get("execution_overhead_instructions"),"canister_overhead_instructions":worker.get("canister_overhead_instructions"),"completed_messages":worker.get("completed_messages"),"slice_count":worker.get("slice_count"),"stop_reasons":worker.get("stop_reasons"),"bounds":[x,y,end_x.saturating_sub(x),18]}));
        }
    }
    if let Some(nominal) = nominal.filter(|v| *v > 0) {
        let x = plot_x + project(nominal);
        scene.line(
            x,
            worker_top.saturating_sub(6),
            x,
            (worker_top + worker_ids.len() * 38).min(bottom),
            COLORS[3],
        );
    }
    let reason = focused
        .get("stop_reasons")
        .map(Value::to_string)
        .unwrap_or_else(|| "unavailable".into());
    scene.text(
        20,
        bottom.saturating_sub(27),
        &shorten(&format!("Stop: {reason}"), (scene.width - 40) / 6),
        MUTED,
        1,
    );
    scene.text(20,bottom.saturating_sub(9),&shorten(&format!("{}/{} retained rows rendered · parallel workers share the iteration offset; capacities are not additive",shown.len(),selected.len()),(scene.width-40)/6),MUTED,1);
    legend.insert("worker_budget_spent".into(),json!({"rgb":COLORS[0],"encoding":"actual initial minus remaining round instructions; placement follows shared iteration offset"}));
    legend.insert("max_worker_path".into(),json!({"rgb":COLORS[2],"encoding":"worker matches captured max_worker_consumed for that iteration; selected overview round also purple"}));
    legend.insert("nominal_round_limit".into(),json!({"rgb":COLORS[3],"encoding":"detail vertical line: configured nominal per-worker/round allowance; overview ticks: actual reserved initial counters; reserve use or negative remaining is not automatically a violation"}));
    Ok(
        json!({"coverage":source.get("coverage"),"summary":source.get("summary"),"configurations":configurations,"pagination":source.get("pagination").or_else(||data.get("pagination")),"focused_round_id":focused.get("id"),"rendered_rounds":shown.len(),"selected_rows":selected.len(),"unrendered_round_rows":selected.len().saturating_sub(shown.len()),"unrendered_worker_rows":workers.len().saturating_sub(worker_ids.len()),"axis_max_instructions":axis_max.to_string(),"heap_delta":heap_evidence,"logical_units":"round_instructions","placement":"round.initial_instructions minus iteration.budget_before; worker width = worker.initial minus worker.remaining","changes_accounting_scope":false,"note":"Parallel worker sums, shared max-worker decrement, per-message totals and slice caps are distinct. Completed-message totals can include earlier rounds and are not added to current-round guest work. Missing/censored round records and omitted pass/slice details remain explicit; generic event retention does not establish dedicated round completeness."}),
    )
}

pub fn render(
    data: &Value,
    view: &str,
    parameters: &HashMap<String, String>,
) -> Result<RenderedProfile, String> {
    let dimension = |key: &str, default: usize, min: usize, max: usize| -> Result<usize, String> {
        let value = parameters
            .get(key)
            .map(|s| s.parse::<usize>())
            .transpose()
            .map_err(|_| format!("{key} must be an integer"))?
            .unwrap_or(default);
        if !(min..=max).contains(&value) {
            return Err(format!("{key} must be {min}..{max}"));
        }
        Ok(value)
    };
    let width = dimension("width", 1280, 640, 4096)?;
    let height = dimension("height", 720, 360, 4096)?;
    if width * height > 8_388_608 {
        return Err("Image exceeds the 8 megapixel budget".into());
    }
    let mut scene = Scene::new(width, height);
    let id = text(data, "query_id");
    let run = text(data, "run_id");
    scene.text(20, 16, &format!("IC system profile · {view}"), INK, 2);
    scene.text(
        20,
        43,
        &format!("Run {} · {}", shorten(run, 42), shorten(id, 70)),
        MUTED,
        1,
    );
    let scope = data.get("scope").unwrap_or(&Value::Null);
    scene.text(
        20,
        61,
        &format!(
            "{} · {} · clock: host monotonic",
            text(scope, "backend"),
            text(scope, "resource_scope")
        ),
        MUTED,
        1,
    );
    let summary = data.get("summary").unwrap_or(&Value::Null);
    for (i, (label, key, unit)) in [
        ("CPU", "cpu_time_ns", "ns"),
        ("RSS (sum; not unique)", "peak_rss_bytes", "bytes"),
        ("Storage reads", "read_bytes", "bytes"),
        ("Storage writes", "write_bytes", "bytes"),
    ]
    .iter()
    .enumerate()
    {
        let x = 20 + i * (width.saturating_sub(40) / 4);
        let value = summary
            .get(*key)
            .filter(|v| !v.is_null())
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| v.to_string())
            })
            .unwrap_or_else(|| "unavailable".into());
        scene.text(x, 86, label, MUTED, 1);
        scene.text(x, 103, &format!("{value} {unit}"), INK, 1);
    }
    scene.line(20, 128, width - 20, 128, GRID);
    let top = 153;
    let bottom = height.saturating_sub(55);
    let mut elements = Vec::new();
    let mut legend = BTreeMap::new();
    let window = data.get("window").unwrap_or(&Value::Null);
    let data_start = window.get("start_ns").and_then(numeric).unwrap_or(0.0);
    let data_end = window
        .get("end_ns")
        .and_then(numeric)
        .unwrap_or(data_start + 1.0)
        .max(data_start + 1.0);
    let crop = |key: &str, default: f64| {
        parameters
            .get(key)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(default)
    };
    let start = crop("render_start_ns", crop("start_ns", data_start)).max(data_start);
    let end = crop("render_end_ns", crop("end_ns", data_end)).min(data_end);
    if !start.is_finite() || !end.is_finite() || end <= start {
        return Err("Render crop must overlap the immutable query window".into());
    }
    let mut round_evidence = Value::Null;
    match view {
        "rounds" => {
            round_evidence = render_rounds(
                &mut scene,
                data,
                parameters,
                top,
                bottom,
                start,
                end,
                &mut elements,
                &mut legend,
            )?;
        }
        "events" | "flows" => {
            let events = if view == "flows" {
                data.get("flows").and_then(|v| v.get("events"))
            } else {
                data.get("events")
            };
            let events = events
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let max_lanes = ((bottom - top) / 24).max(1);
            let mut lanes = BTreeMap::new();
            for event in &events {
                let lane = format!(
                    "{}:{}",
                    event.get("pid").map(Value::to_string).unwrap_or_default(),
                    event.get("tid").map(Value::to_string).unwrap_or_default()
                );
                if !lanes.contains_key(&lane) && lanes.len() < max_lanes {
                    let index = lanes.len();
                    lanes.insert(lane, index);
                }
            }
            let x0 = 170;
            let plot_width = width - x0 - 30;
            for (lane, index) in &lanes {
                let y = top + index * 24;
                scene.text(20, y + 5, lane, MUTED, 1);
                scene.line(x0, y + 19, width - 25, y + 19, GRID);
            }
            for event in events.iter().take(10_000) {
                let lane = format!(
                    "{}:{}",
                    event.get("pid").map(Value::to_string).unwrap_or_default(),
                    event.get("tid").map(Value::to_string).unwrap_or_default()
                );
                let Some(index) = lanes.get(&lane) else {
                    continue;
                };
                let a = event.get("start_ns").and_then(numeric).unwrap_or(0.0);
                let b = event.get("end_ns").and_then(numeric).unwrap_or(a + 1.0);
                if b < start || a > end {
                    continue;
                }
                let x =
                    x0 + (((a.max(start) - start) / (end - start)) * plot_width as f64) as usize;
                let w = ((((b.min(end) - a.max(start)).max(1.0)) / (end - start))
                    * plot_width as f64)
                    .round()
                    .max(2.0) as usize;
                let y = top + index * 24;
                let component = text(event, "component_id");
                let c = color(component);
                scene.rect(x, y, w, 16, c);
                legend.insert(component.to_string(),json!({"rgb":c,"encoding":"observed event/scope; width is elapsed time, not CPU"}));
                elements.push(json!({"event_id":event.get("id"),"component_id":component,"bounds":[x,y,w,16]}));
            }
            if elements.is_empty() {
                scene.text(
                    20,
                    top + 30,
                    "No retained matching timeline events; see coverage and evidence.",
                    MUTED,
                    1,
                );
            }
            scene.text(
                x0,
                bottom + 10,
                &format!("{start:.0} .. {end:.0} host-monotonic ns"),
                MUTED,
                1,
            );
        }
        "resources" => {
            let metric = parameters
                .get("metric")
                .map(String::as_str)
                .unwrap_or("rss_bytes");
            if ![
                "rss_bytes",
                "pss_bytes",
                "cgroup_memory_bytes",
                "cpu_time_ns",
                "read_bytes",
                "write_bytes",
                "minor_faults",
                "major_faults",
            ]
            .contains(&metric)
            {
                return Err("Unknown resource metric".into());
            }
            let samples = data
                .get("resources")
                .and_then(|v| v.get("samples"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let points: Vec<_> = samples
                .iter()
                .filter_map(|s| {
                    Some((
                        s.get("time_ns").and_then(numeric)?,
                        s.get(metric).and_then(numeric)?,
                    ))
                })
                .filter(|(t, _)| *t >= start && *t <= end)
                .collect();
            let maximum = points
                .iter()
                .map(|(_, v)| *v)
                .fold(0.0_f64, f64::max)
                .max(1.0);
            scene.text(
                20,
                top,
                &format!("{metric} · raw counter/gauge; see metadata for units"),
                INK,
                1,
            );
            let x0 = 90;
            let y0 = top + 35;
            let chart_h = bottom.saturating_sub(y0 + 20);
            for i in 0..=4 {
                let y = y0 + chart_h * i / 4;
                scene.line(x0, y, width - 30, y, GRID);
                scene.text(
                    20,
                    y,
                    &format!("{:.0}", maximum * (1.0 - i as f64 / 4.0)),
                    MUTED,
                    1,
                );
            }
            let mut previous = None;
            for (time, value) in points {
                let x = x0
                    + ((time - start) / (end - start) * (width - x0 - 30) as f64).round() as usize;
                let y = y0 + chart_h - ((value / maximum) * chart_h as f64).round() as usize;
                if let Some((px, py)) = previous {
                    scene.line(px, py, x, y, COLORS[0]);
                }
                scene.rect(x.saturating_sub(1), y.saturating_sub(1), 3, 3, COLORS[0]);
                previous = Some((x, y));
                elements.push(json!({"sample_time_ns":format!("{time:.0}"),"metric":metric,"value":format!("{value:.0}"),"point":[x,y]}));
            }
            if elements.is_empty() {
                scene.text(
                    x0,
                    y0 + 30,
                    "Metric unavailable in selected samples",
                    MUTED,
                    1,
                );
            }
            legend.insert(metric.into(),json!({"rgb":COLORS[0],"encoding":"raw sampled counter/gauge; not a rate estimate"}));
        }
        "pricing" | "limits" => {
            let source = data.get(view).unwrap_or(&Value::Null);
            let rows = source
                .get("rows")
                .or_else(|| source.get("events"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for (i, row) in rows.iter().take(((bottom - top) / 24).max(1)).enumerate() {
                let label = row
                    .get("label")
                    .or_else(|| row.get("operation"))
                    .and_then(Value::as_str)
                    .unwrap_or("Observed logical work");
                scene.text(20, top + i * 24, &shorten(label, 48), INK, 1);
                scene.text(
                    width / 2,
                    top + i * 24,
                    &shorten(&row.to_string(), (width / 2 - 30) / 6),
                    MUTED,
                    1,
                );
                elements.push(json!({"row":i,"evidence":row}));
            }
            if rows.is_empty() {
                scene.text(
                    20,
                    top + 25,
                    "No normalized rows available. Read JSON evidence and coverage.",
                    MUTED,
                    1,
                );
            }
        }
        _ => {
            let mut rows = data
                .get("components")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if !parameters.contains_key("component")
                && !parameters.contains_key("canister_id")
                && !parameters.contains_key("process_id")
            {
                if let Some(amount) = summary.get("unattributed_cpu_ns").and_then(numeric) {
                    if amount > 0.0 {
                        rows.push(json!({"component_id":"unattributed/shared","exclusive_cpu_ns":summary["unattributed_cpu_ns"],"attribution":"independent_host_envelope_residual"}));
                    }
                }
            }
            let mut rows: Vec<_> = rows.iter().collect();
            rows.sort_by(|a, b| {
                b.get("exclusive_cpu_ns")
                    .and_then(numeric)
                    .unwrap_or(0.0)
                    .total_cmp(&a.get("exclusive_cpu_ns").and_then(numeric).unwrap_or(0.0))
            });
            let maximum = rows
                .iter()
                .filter_map(|row| row.get("exclusive_cpu_ns").and_then(numeric))
                .fold(0.0_f64, f64::max)
                .max(1.0);
            let x0 = (width / 3).min(460);
            let plot_width = width - x0 - 165;
            scene.text(20, top - 10, "Component · exclusive measured CPU", INK, 1);
            for (i, row) in rows.iter().take(((bottom - top) / 25).max(1)).enumerate() {
                let y = top + 20 + i * 25;
                let component = text(row, "component_id");
                let amount = row.get("exclusive_cpu_ns").and_then(numeric).unwrap_or(0.0);
                let c = if component == "unattributed/shared" {
                    MUTED
                } else {
                    color(component)
                };
                scene.text(20, y + 5, &shorten(component, (x0 - 35) / 6), INK, 1);
                let w = (amount / maximum * plot_width as f64).round() as usize;
                scene.rect(x0, y, w, 17, c);
                scene.text(
                    x0 + plot_width + 12,
                    y + 5,
                    &format!("{:.3} ms", amount / 1e6),
                    INK,
                    1,
                );
                legend.insert(
                    component.to_string(),
                    json!({"rgb":c,"encoding":"exclusive measured thread CPU"}),
                );
                elements.push(json!({"component_id":component,"exclusive_cpu_ns":row.get("exclusive_cpu_ns"),"bounds":[x0,y,w,17]}));
            }
            if rows.is_empty() {
                scene.text(
                    20,
                    top + 30,
                    "No instrumented component CPU. Host totals remain independent.",
                    MUTED,
                    1,
                );
            }
        }
    }
    scene.line(20, height - 38, width - 20, height - 38, GRID);
    let coverage = data.get("coverage").unwrap_or(&Value::Null);
    let quality = data.get("quality").unwrap_or(&Value::Null);
    let footer = if view == "rounds" {
        let rounds_coverage = data
            .get("rounds")
            .and_then(|r| r.get("coverage"))
            .unwrap_or(&Value::Null);
        format!(
            "Round coverage: {} · dropped {} · rejected {} · boundary {} · generic event loss stays separate",
            text(rounds_coverage, "status"),
            text(rounds_coverage, "dropped_rounds"),
            text(rounds_coverage, "rejected_rounds"),
            text(rounds_coverage, "boundary_crossing_rounds")
        )
    } else {
        format!(
            "Coverage: {} · {} · unknown/shared work is not zero",
            text(coverage, "status"),
            text(data, "status")
        )
    };
    scene.text(
        20,
        height - 25,
        &shorten(&footer, (width - 40) / 6),
        MUTED,
        1,
    );
    let metadata = json!({"schema_version":1,"run_id":run,"query_id":id,"view":view,"width":width,"height":height,"scope":scope,"window":window,"summary":summary,"coverage":coverage,"quality":quality,"legend":legend,"elements":elements,"round_evidence":round_evidence,"render_crop":{"start_ns":format!("{start:.0}"),"end_ns":format!("{end:.0}"),"changes_accounting_scope":false},"note":"Image is a projection of retained evidence. Missing records, unattributed CPU and emulated paths are not synthesized."});
    scene.svg.push_str(&format!(
        "<metadata>{}</metadata></svg>",
        escape(&metadata.to_string())
    ));
    let png = scene.png()?;
    Ok(RenderedProfile {
        svg: scene.svg,
        png,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn image_preserves_query_identity_and_escapes_untrusted_labels() {
        let data = json!({"run_id":"r","query_id":"q","status":"stopped","scope":{"backend":"pocketic","resource_scope":"observed_process_tree"},"summary":{"cpu_time_ns":"1000"},"components":[{"component_id":"<script>","exclusive_cpu_ns":"800"}],"window":{"start_ns":"0","end_ns":"1000"},"coverage":{"status":"partial"}});
        let a = render(&data, "components", &HashMap::new()).unwrap();
        let b = render(&data, "components", &HashMap::new()).unwrap();
        assert_eq!(a.png, b.png);
        assert!(a.png.starts_with(b"\x89PNG"));
        assert!(!a.svg.contains("<script>"));
        assert_eq!(a.metadata["query_id"], "q");
        assert_eq!(a.metadata["elements"][0]["exclusive_cpu_ns"], "800");
    }
    fn round_fixture() -> Value {
        json!({"run_id":"r","query_id":"round-q","status":"stopped","scope":{"backend":"pocketic","resource_scope":"observed_process_tree"},"summary":{"cpu_time_ns":"9007199254740993"},"quality":{"dropped_events":"100"},"coverage":{"status":"partial"},"window":{"start_ns":"0","end_ns":"10000"},"rounds":{"coverage":{"status":"complete","dropped_rounds":"0"},"configurations":{"cfg":{"max_instructions_per_round":"4000000000","max_instructions_per_message":"40000000000","max_instructions_per_slice":"2000000000"}},"rows":[{"id":"round-a","config_id":"cfg","subnet_id":"subnet","round":"7","round_kind":"ordinary<script>","start_ns":"100","end_ns":"9000","initial_instructions":"4000000000","remaining_instructions":"2900000000","stop_reasons":["no_executable_canisters"],"iterations":[{"iteration":"0","budget_before":"4000000000","max_worker_consumed":"600000000","sum_worker_consumed":"800000000","workers":[{"worker":"0","initial_instructions":"4000000000","remaining_instructions":"3400000000","guest_instructions":"599000000","slice_count":"3","completed_messages":"2"},{"worker":"1","initial_instructions":"4000000000","remaining_instructions":"3800000000","guest_instructions":"199000000","slice_count":"1","completed_messages":"1"}]},{"iteration":"1","budget_before":"3400000000","max_worker_consumed":"500000000","sum_worker_consumed":"700000000","workers":[{"worker":"0","initial_instructions":"3400000000","remaining_instructions":"3200000000","guest_instructions":"199000000","slice_count":"1","completed_messages":"1"},{"worker":"1","initial_instructions":"3400000000","remaining_instructions":"2900000000","guest_instructions":"499000000","slice_count":"2","completed_messages":"2"}]}]}]}})
    }

    #[test]
    fn round_images_preserve_logical_offsets_without_summing_parallel_capacity() {
        let image = render(&round_fixture(), "rounds", &HashMap::new()).unwrap();
        let rows = image.metadata["elements"].as_array().unwrap();
        let round = rows
            .iter()
            .find(|row| row["kind"] == "round_budget")
            .unwrap();
        assert_eq!(round["consumed_instructions"], "1100000000");
        let worker = rows
            .iter()
            .find(|row| {
                row["kind"] == "worker_iteration" && row["iteration"] == "1" && row["worker"] == "1"
            })
            .unwrap();
        assert_eq!(worker["offset_instructions"], "600000000");
        assert_eq!(worker["consumed_instructions"], "500000000");
        assert_eq!(image.metadata["summary"]["cpu_time_ns"], "9007199254740993");
        assert_eq!(
            image.metadata["round_evidence"]["coverage"]["dropped_rounds"],
            "0"
        );
        assert_eq!(image.metadata["quality"]["dropped_events"], "100");
        assert_eq!(image.metadata["query_id"], "round-q");
        assert!(image.png.starts_with(b"\x89PNG"));
        assert!(!image.svg.contains("ordinary<script>"));
        assert!(
            render(
                &round_fixture(),
                "rounds",
                &HashMap::from([("render_round_id".into(), "other-round".into())])
            )
            .is_err()
        );
    }

    #[test]
    fn legacy_missing_rounds_stay_unavailable_and_small_images_omit_detail_explicitly() {
        let mut data = round_fixture();
        data["rounds"] = Value::Null;
        let image = render(&data, "rounds", &HashMap::new()).unwrap();
        assert_eq!(image.metadata["round_evidence"]["rendered_rounds"], 0);
        assert_eq!(
            image.metadata["round_evidence"]["missing_data_is_zero"],
            false
        );
        assert!(image.metadata["elements"].as_array().unwrap().is_empty());
        let small = render(
            &round_fixture(),
            "rounds",
            &HashMap::from([("height".into(), "360".into())]),
        )
        .unwrap();
        assert_eq!(
            small.metadata["round_evidence"]["worker_detail"],
            "omitted_due_to_image_height"
        );
    }
    #[test]
    fn compact_round_projection_is_distinct_from_source_loss() {
        let mut data = round_fixture();
        data["rounds"]["detail_mode"] = json!("summary");
        data["rounds"]["rows"][0]["iterations"] = json!([]);
        data["rounds"]["rows"][0]["details_omitted_by_query"] = json!(true);
        data["rounds"]["rows"][0]["details_complete"] = json!(true);
        let image = render(&data, "rounds", &HashMap::new()).unwrap();
        assert_eq!(
            image.metadata["round_evidence"]["worker_detail"],
            "not_requested"
        );
        assert_eq!(
            image.metadata["round_evidence"]["source_details_complete"],
            true
        );
        assert_eq!(
            image.metadata["round_evidence"]["details_omitted_by_query"],
            true
        );
        assert_eq!(image.metadata["elements"].as_array().unwrap().len(), 1);
    }
    #[test]
    fn heap_delta_gauges_preserve_unknowns_and_observed_stop_evidence() {
        let missing = render(&round_fixture(), "rounds", &HashMap::new()).unwrap();
        assert!(missing.metadata["round_evidence"]["heap_delta"]["bytes"].is_null());
        assert!(missing.metadata["round_evidence"]["heap_delta"]["limit_reached"].is_null());
        let mut data = round_fixture();
        data["rounds"]["configurations"]["cfg"]["max_heap_delta_per_iteration_bytes"] =
            json!("200000000");
        data["rounds"]["rows"][0]["heap_delta_bytes"] = json!("212000000");
        data["rounds"]["rows"][0]["heap_delta_limit_reached"] = json!(true);
        data["rounds"]["rows"][0]["stop_reasons"] = json!(["heap_delta_limit"]);
        let image = render(&data, "rounds", &HashMap::new()).unwrap();
        let gauge = &image.metadata["elements"][0];
        assert_eq!(gauge["heap_delta_bytes"], "212000000");
        assert_eq!(gauge["max_heap_delta_per_iteration_bytes"], "200000000");
        assert_eq!(gauge["heap_delta_limit_reached"], true);
        assert_eq!(
            image.metadata["round_evidence"]["heap_delta"]["observed_stop_reasons"][0],
            "heap_delta_limit"
        );
        assert_eq!(image.metadata["summary"]["cpu_time_ns"], "9007199254740993");
    }
}
