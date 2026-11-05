//! Bounded, lossless byte-class images for immutable memory snapshots.
//!
//! Each selected 64 KiB page has a 256 by 256 pixel body: pixel `(x, y)`
//! represents byte `256 * y + x`, without sampling or scaling. Gutters, labels
//! are outside that body. Only page IDs appear in the image. A snapshot comparison
//! describes the whole capture interval, not necessarily one call or a write.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const WASM_PAGE_BYTES: usize = 65_536;
pub(crate) const MAX_IMAGE_PAGES: usize = 256;
pub(crate) const MAX_IMAGE_COLUMNS: usize = 16;
pub(crate) const MAX_PNG_BYTES: usize = 6 * 1024 * 1024;

const BYTE_WIDTH: usize = 256;
const BYTE_HEIGHT: usize = 256;
const LABEL_HEIGHT: usize = 10;
const GRID_WIDTH: usize = 1;
const FIRST_CHANGES_LIMIT: usize = 32;
const DIFFERENCE_RANGES_LIMIT: usize = 16;
const MAX_ID_BYTES: usize = 256;
const READ_FREQUENCY_RANGES_LIMIT: usize = 4096;
const READ_BRIGHTNESS: [u8; 9] = [64, 88, 112, 136, 160, 184, 208, 232, 255];

type Color = [u8; 3];
const BACKGROUND: Color = [255, 255, 255];
const INK: Color = [27, 32, 40];
// Reserved for grid lines; deliberately absent from every byte-class palette.
const GRID_COLOR: Color = [225, 0, 255];

/// The current snapshot is the comparison's "after" side. All page buffers
/// contain exactly 64 KiB, zero-padded outside the corresponding allocation.
#[derive(Clone, Debug)]
pub(crate) struct MemorySnapshotImageInput {
    pub memory: String,
    pub snapshot_id: String,
    pub comparison_snapshot_id: Option<String>,
    pub allocated_bytes: u64,
    pub comparison_allocated_bytes: Option<u64>,
    pub pages: Vec<MemorySnapshotImagePage>,
}

#[derive(Clone, Debug)]
pub(crate) struct MemorySnapshotImagePage {
    pub memory: String,
    pub allocated_bytes: u64,
    pub comparison_allocated_bytes: Option<u64>,
    pub page_index: u64,
    pub bytes: Vec<u8>,
    pub comparison_bytes: Option<Vec<u8>>,
    /// Independently collected page evidence for the capture interval, not
    /// individual byte reads. Snapshot byte values alone cannot establish it.
    pub accessed: Option<bool>,
    /// A dirty page does not prove that all, or any particular, bytes changed.
    pub dirty: Option<bool>,
}

pub(crate) struct MemorySnapshotImage {
    pub png: Vec<u8>,
    pub summary: Value,
}

// The recorder and renderer share the native DTOs without an internal JSON round trip.
pub(crate) use pocket_ic::common::rest::{
    WasmReadRange as MemorySnapshotReadRange, WasmWriteRange as MemorySnapshotWriteRange,
};

fn page_canvas(page_count: usize) -> (Canvas, usize, usize, usize) {
    let columns = (1..=MAX_IMAGE_COLUMNS)
        .find(|count| count * count >= page_count)
        .unwrap_or(MAX_IMAGE_COLUMNS);
    let rows = page_count.div_ceil(columns);
    let width = columns * (BYTE_WIDTH + GRID_WIDTH) + GRID_WIDTH;
    let height = rows * (BYTE_HEIGHT + LABEL_HEIGHT + GRID_WIDTH) + GRID_WIDTH;
    let mut canvas = Canvas::new(width, height);
    for column in 0..=columns {
        canvas.rect(
            column * (BYTE_WIDTH + GRID_WIDTH),
            0,
            GRID_WIDTH,
            height,
            GRID_COLOR,
        );
    }
    for row in 0..=rows {
        canvas.rect(
            0,
            row * (BYTE_HEIGHT + LABEL_HEIGHT + GRID_WIDTH),
            width,
            GRID_WIDTH,
            GRID_COLOR,
        );
    }
    (canvas, columns, rows, width)
}

fn page_position(position: usize, columns: usize) -> (usize, usize, usize) {
    let body_x = GRID_WIDTH + (position % columns) * (BYTE_WIDTH + GRID_WIDTH);
    let label_y = GRID_WIDTH + (position / columns) * (BYTE_HEIGHT + LABEL_HEIGHT + GRID_WIDTH);
    (body_x, label_y, label_y + LABEL_HEIGHT)
}

fn page_id(page: &MemorySnapshotImagePage) -> String {
    format!(
        "{}:{}",
        if page.memory == "heap" { "H" } else { "S" },
        page.page_index
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ByteClass {
    Zero,
    Nonzero,
    UnchangedZero,
    UnchangedNonzero,
    BecameNonzero,
    ChangedNonzero,
    BecameZero,
    NewlyAllocatedZero,
    NewlyAllocatedNonzero,
    OutsideAllocated,
}

impl ByteClass {
    const ALL: [Self; 10] = [
        Self::Zero,
        Self::Nonzero,
        Self::UnchangedZero,
        Self::UnchangedNonzero,
        Self::BecameNonzero,
        Self::ChangedNonzero,
        Self::BecameZero,
        Self::NewlyAllocatedZero,
        Self::NewlyAllocatedNonzero,
        Self::OutsideAllocated,
    ];

    fn index(self) -> usize {
        self as usize
    }

    fn name(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::Nonzero => "nonzero",
            Self::UnchangedZero => "unchanged_zero",
            Self::UnchangedNonzero => "unchanged_nonzero",
            Self::BecameNonzero => "became_nonzero",
            Self::ChangedNonzero => "changed_nonzero",
            Self::BecameZero => "became_zero",
            Self::NewlyAllocatedZero => "newly_allocated_zero",
            Self::NewlyAllocatedNonzero => "newly_allocated_nonzero",
            Self::OutsideAllocated => "outside_allocated",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Zero => "ZERO",
            Self::Nonzero => "NONZERO",
            Self::UnchangedZero => "UNCHANGED ZERO",
            Self::UnchangedNonzero => "UNCHANGED NONZERO",
            Self::BecameNonzero => "ZERO TO NONZERO",
            Self::ChangedNonzero => "CHANGED NONZERO VALUE",
            Self::BecameZero => "NONZERO TO ZERO",
            Self::NewlyAllocatedZero => "NEW ALLOCATION: ZERO",
            Self::NewlyAllocatedNonzero => "NEW ALLOCATION: NONZERO",
            Self::OutsideAllocated => "OUTSIDE AFTER ALLOCATION",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Zero | Self::UnchangedZero => [28, 33, 43],
            Self::Nonzero | Self::UnchangedNonzero => [207, 214, 224],
            Self::BecameNonzero => [245, 158, 11],
            Self::ChangedNonzero => [234, 88, 12],
            Self::BecameZero => [147, 51, 234],
            Self::NewlyAllocatedZero => [187, 247, 208],
            Self::NewlyAllocatedNonzero => [21, 128, 61],
            Self::OutsideAllocated => [210, 214, 220],
        }
    }
}

#[derive(Default)]
struct Counts {
    classes: [u64; 10],
    before_allocated: u64,
    after_allocated: u64,
    changed_existing: u64,
    newly_allocated: u64,
    removed_allocated: u64,
    after_zero: u64,
    after_nonzero: u64,
}

impl Counts {
    fn add(&mut self, other: &Self) {
        for (sum, value) in self.classes.iter_mut().zip(other.classes) {
            *sum += value;
        }
        self.before_allocated += other.before_allocated;
        self.after_allocated += other.after_allocated;
        self.changed_existing += other.changed_existing;
        self.newly_allocated += other.newly_allocated;
        self.removed_allocated += other.removed_allocated;
        self.after_zero += other.after_zero;
        self.after_nonzero += other.after_nonzero;
    }

    fn json(&self, comparison: bool) -> Value {
        let classes: serde_json::Map<String, Value> = ByteClass::ALL
            .iter()
            .filter(|class| relevant(**class, comparison))
            .map(|class| {
                (
                    class.name().to_string(),
                    json!(self.classes[class.index()].to_string()),
                )
            })
            .collect();
        json!({
            "classes": classes,
            "after_allocated_bytes": self.after_allocated.to_string(),
            "after_zero_bytes": self.after_zero.to_string(),
            "after_nonzero_bytes": self.after_nonzero.to_string(),
            "before_allocated_bytes": comparison.then(|| self.before_allocated.to_string()),
            "changed_existing_bytes": comparison.then(|| self.changed_existing.to_string()),
            "newly_allocated_bytes": comparison.then(|| self.newly_allocated.to_string()),
            "removed_allocated_bytes": comparison.then(|| self.removed_allocated.to_string()),
        })
    }
}

fn relevant(class: ByteClass, comparison: bool) -> bool {
    match class {
        ByteClass::Zero | ByteClass::Nonzero => !comparison,
        ByteClass::OutsideAllocated => true,
        _ => comparison,
    }
}

fn classify(after: Option<u8>, before: Option<u8>, comparison: bool) -> ByteClass {
    let Some(after) = after else {
        return ByteClass::OutsideAllocated;
    };
    if !comparison {
        return if after == 0 {
            ByteClass::Zero
        } else {
            ByteClass::Nonzero
        };
    }
    match before {
        None if after == 0 => ByteClass::NewlyAllocatedZero,
        None => ByteClass::NewlyAllocatedNonzero,
        Some(before) if before == after && after == 0 => ByteClass::UnchangedZero,
        Some(before) if before == after => ByteClass::UnchangedNonzero,
        Some(0) => ByteClass::BecameNonzero,
        Some(_) if after == 0 => ByteClass::BecameZero,
        Some(_) => ByteClass::ChangedNonzero,
    }
}

/// Render selected pages in request order. The image encodes exact byte classes,
/// not byte values or write attribution; the summary states its coordinate map.
pub(crate) fn render(input: &MemorySnapshotImageInput) -> Result<MemorySnapshotImage, String> {
    validate(input)?;
    let comparison = input.comparison_snapshot_id.is_some();
    let (mut canvas, columns, rows, width) = page_canvas(input.pages.len());
    let height = canvas.height;
    let mut total = Counts::default();
    let mut memory_totals: BTreeMap<String, Counts> = BTreeMap::new();
    let mut allocations: BTreeMap<String, (u64, Option<u64>)> = BTreeMap::new();
    let mut pages = Vec::with_capacity(input.pages.len());
    let mut first_changes = Vec::new();
    for (position, page) in input.pages.iter().enumerate() {
        let (body_x, label_y, body_y) = page_position(position, columns);
        let page_id = page_id(page);
        canvas.text(body_x + 2, label_y + 1, &page_id, INK);
        let start = page.page_index as u128 * WASM_PAGE_BYTES as u128;
        let mut counts = Counts::default();
        let mut bounds: Option<[usize; 4]> = None;
        let mut first_difference: Option<usize> = None;
        let mut difference_ranges = Vec::new();
        let mut range_start = None;
        let mut ranges_truncated = false;
        for offset in 0..WASM_PAGE_BYTES {
            let address = start + offset as u128;
            let after = (address < page.allocated_bytes as u128).then_some(page.bytes[offset]);
            let before = if comparison && address < page.comparison_allocated_bytes.unwrap() as u128
            {
                Some(page.comparison_bytes.as_ref().unwrap()[offset])
            } else {
                None
            };
            let class = classify(after, before, comparison);
            counts.classes[class.index()] += 1;
            counts.after_allocated += u64::from(after.is_some());
            counts.before_allocated += u64::from(before.is_some());
            counts.after_zero += u64::from(after == Some(0));
            counts.after_nonzero += u64::from(after.is_some_and(|value| value != 0));
            let changed = before
                .zip(after)
                .is_some_and(|(before, after)| before != after);
            counts.changed_existing += u64::from(changed);
            counts.newly_allocated += u64::from(comparison && before.is_none() && after.is_some());
            counts.removed_allocated +=
                u64::from(comparison && before.is_some() && after.is_none());
            if comparison && before != after {
                first_difference.get_or_insert(offset);
                range_start.get_or_insert(offset);
                let (x, y) = (offset % BYTE_WIDTH, offset / BYTE_WIDTH);
                let value = bounds.get_or_insert([x, y, x + 1, y + 1]);
                value[0] = value[0].min(x);
                value[1] = value[1].min(y);
                value[2] = value[2].max(x + 1);
                value[3] = value[3].max(y + 1);
                if first_changes.len() < FIRST_CHANGES_LIMIT {
                    first_changes.push(json!({ "memory": page.memory, "address": address.to_string(),
                        "page_index": page.page_index.to_string(), "page_byte_offset": offset.to_string(),
                        "before": before, "after": after, "class": class.name() }));
                }
            } else if let Some(begin) = range_start.take() {
                append_range(
                    &mut difference_ranges,
                    &mut ranges_truncated,
                    start,
                    begin,
                    offset,
                );
            }
            canvas.pixel(
                body_x + offset % BYTE_WIDTH,
                body_y + offset / BYTE_WIDTH,
                byte_color(class, offset),
            );
        }
        if let Some(begin) = range_start {
            append_range(
                &mut difference_ranges,
                &mut ranges_truncated,
                start,
                begin,
                WASM_PAGE_BYTES,
            );
        }
        pages.push(json!({
            "id": page_id, "memory": page.memory, "page_index": page.page_index.to_string(),
            "allocated_bytes": page.allocated_bytes.to_string(),
            "comparison_allocated_bytes": page.comparison_allocated_bytes.map(|value| value.to_string()),
            "after_sha256": hex::encode(ic_crypto_sha2::Sha256::hash(&page.bytes)),
            "before_sha256": page.comparison_bytes.as_ref().map(|bytes| hex::encode(ic_crypto_sha2::Sha256::hash(bytes))),
            "start_address": start.to_string(), "end_address_exclusive": (start + WASM_PAGE_BYTES as u128).to_string(),
            "byte_pixel_body": { "x": body_x, "y": body_y, "width": BYTE_WIDTH, "height": BYTE_HEIGHT },
            "label_strip": { "x": body_x, "y": label_y, "width": BYTE_WIDTH, "height": LABEL_HEIGHT },
            "counts": counts.json(comparison),
            "first_difference_address": first_difference.map(|offset| (start + offset as u128).to_string()),
            "first_difference_address_hex": first_difference.map(|offset| format!("0x{:x}", start + offset as u128)),
            "difference_pixel_bounds": bounds.map(|[x0, y0, x1, y1]| json!({ "x_min": x0, "y_min": y0, "x_max_exclusive": x1, "y_max_exclusive": y1 })),
            "first_difference_ranges": difference_ranges, "difference_ranges_limit": DIFFERENCE_RANGES_LIMIT,
            "difference_ranges_truncated": ranges_truncated,
            "page_evidence": { "memory": page.memory, "granularity_bytes": WASM_PAGE_BYTES.to_string(),
                "accessed": page.accessed, "dirty": page.dirty, "scope": "snapshot_window_comparison",
                "interpretation": "Independently supplied 64 KiB heap or stable page evidence is separate from byte-value changes. Snapshot bytes alone do not establish reads, writes, or native fee coverage. Evidence appears only in JSON, not the image grid." },
        }));
        allocations
            .entry(page.memory.clone())
            .or_insert((page.allocated_bytes, page.comparison_allocated_bytes));
        memory_totals
            .entry(page.memory.clone())
            .or_default()
            .add(&counts);
        total.add(&counts);
    }
    let png = encode_png(&canvas)?;
    let palette: Vec<_> = ByteClass::ALL.into_iter().filter(|class| relevant(*class, comparison)).map(|class| json!({
        "class": class.name(), "label": class.label(), "rgb": class.color(),
        "alternate_rgb": (class == ByteClass::OutsideAllocated).then_some([230, 233, 238]),
        "pattern": if class == ByteClass::OutsideAllocated { "8x8_checker" } else { "solid" },
    })).collect();
    let per_memory: BTreeMap<_, _> = memory_totals.iter().map(|(kind, counts)| {
        let (allocated, before_allocated) = allocations[kind];
        (kind.clone(), json!({ "allocated_bytes": allocated.to_string(),
            "comparison_allocated_bytes": before_allocated.map(|value| value.to_string()),
            "counts": counts.json(comparison), "entire_memory_selected": counts.after_allocated == allocated }))
    }).collect();
    let allocated_total: u128 = allocations.values().map(|(after, _)| *after as u128).sum();
    let before_allocated_total: u128 = allocations
        .values()
        .filter_map(|(_, before)| before.map(|value| value as u128))
        .sum();
    let summary = json!({
        "format_version": 1, "view": "data", "kind": if comparison { "memory_byte_diff" } else { "memory_byte_snapshot" },
        "scope": "snapshot_window_comparison", "memory": input.memory,
        "snapshot_id": input.snapshot_id, "comparison_snapshot_id": input.comparison_snapshot_id,
        "after_allocated_bytes": allocated_total.to_string(),
        "before_allocated_bytes": comparison.then(|| before_allocated_total.to_string()),
        "selected_pages": input.pages.len().to_string(), "selected_byte_positions": (input.pages.len() * WASM_PAGE_BYTES).to_string(),
        "counts": total.json(comparison), "per_memory": per_memory, "pages": pages, "palette": palette,
        "image_grid_color": GRID_COLOR,
        "count_relationships": "classes are exclusive; changed_existing_bytes sums became_nonzero, changed_nonzero and became_zero. Newly allocated and removed bytes are separate allocation differences, not writes or changes to previously allocated values. Allocation totals cover represented memory kinds; per-page allocation overrides the legacy input-wide values.",
        "first_differences": first_changes, "first_differences_limit": FIRST_CHANGES_LIMIT,
        "first_differences_truncated": comparison && total.changed_existing + total.newly_allocated + total.removed_allocated > FIRST_CHANGES_LIMIT as u64,
        "image": { "mime_type": "image/png", "width": width, "height": height, "layout": "compact_byte_pages",
            "png_bytes": png.len().to_string(), "byte_pixel_width": BYTE_WIDTH, "byte_pixel_height": BYTE_HEIGHT,
            "bytes_per_pixel": "1", "columns": columns, "rows": rows, "resampled": false,
            "grid_width": GRID_WIDTH, "label_strip_height": LABEL_HEIGHT, "labels": "page_id_only",
            "unused_grid_cells": columns * rows - input.pages.len(),
            "coordinate_rule": "For each page body, byte_offset = 256 * (pixel_y - body.y) + (pixel_x - body.x); address = start_address + byte_offset within that page's memory kind. Only page IDs appear in 10-pixel strips. The reserved one-pixel grid is not byte content." },
        "coverage": { "all_selected_byte_positions_classified": true,
            "entire_memory_selected": memory_totals.iter().all(|(kind, counts)| counts.after_allocated == allocations[kind].0),
            "allocation_totals_scope": "represented_memory_kinds", "outside_allocation_is_zero": false,
            "byte_values_encoded_in_png": false, "byte_classification_is_exact": true,
            "individual_byte_reads_or_writes_observed": false, "single_call_attribution": false,
            "note": "The PNG losslessly encodes byte classes at exact byte coordinates. Selection and completeness belong to the caller's touched-page/pagination metadata. Differences compare immutable snapshots across the capture window; unchanged values do not prove no writes. Outside-allocation pixels are unavailable addresses, not zero." },
        "limits": { "max_pages": MAX_IMAGE_PAGES, "max_columns": MAX_IMAGE_COLUMNS, "max_png_bytes": MAX_PNG_BYTES.to_string() },
    });
    Ok(MemorySnapshotImage { png, summary })
}

fn read_brightness(count: u64) -> u8 {
    if count == 0 {
        return 0;
    }
    READ_BRIGHTNESS[(63 - count.leading_zeros() as usize).min(READ_BRIGHTNESS.len() - 1)]
}

fn mask_color(memory: &str, counts: [u64; 3], writes: bool) -> Color {
    let total = counts.into_iter().fold(0_u64, u64::saturating_add);
    let brightness = read_brightness(total);
    if writes && memory == "heap" {
        [brightness, 0, (u16::from(brightness) * 96 / 255) as u8]
    } else if writes {
        [brightness, (u16::from(brightness) * 96 / 255) as u8, 0]
    } else if memory == "heap" {
        [0, brightness, brightness]
    } else if counts[2] > 0 {
        [brightness, (u16::from(brightness) * 2 / 5) as u8, 0]
    } else {
        [0, brightness, 0]
    }
}

/// Render observed reads only, preserving the data sheet's page ordering,
/// dimensions, byte coordinates and grid. No snapshot byte is encoded.
pub(crate) fn render_reads(
    input: &MemorySnapshotImageInput,
    ranges: &[MemorySnapshotReadRange],
    profile: &Value,
) -> Result<MemorySnapshotImage, String> {
    render_mask(
        input,
        ranges.iter().map(|range| MaskRange {
            memory: &range.memory,
            start: range.start,
            end: range.end,
            counts: [
                range.heap_reads,
                range.stable_direct_reads,
                range.stable_native_reads,
            ],
        }),
        profile,
        false,
    )
}

/// Writes use the same bounded range-sweep and layout as reads. Their counters
/// are observations of successful writes, independent of net byte differences.
pub(crate) fn render_writes(
    input: &MemorySnapshotImageInput,
    ranges: &[MemorySnapshotWriteRange],
    profile: &Value,
) -> Result<MemorySnapshotImage, String> {
    render_mask(
        input,
        ranges.iter().map(|range| MaskRange {
            memory: &range.memory,
            start: range.start,
            end: range.end,
            counts: [range.heap_writes, range.stable_writes, 0],
        }),
        profile,
        true,
    )
}

#[derive(Clone, Copy)]
struct MaskRange<'a> {
    memory: &'a str,
    start: u64,
    end: u64,
    counts: [u64; 3],
}

struct MaskFields {
    frequency: &'static [&'static str],
    bytes: &'static [&'static str],
    distinct: &'static [&'static str],
    maximum: &'static [&'static str],
}

fn add_counts(value: &mut Value, fields: &[&str], counts: [u128; 3]) {
    for (field, count) in fields.iter().zip(counts) {
        value[*field] = json!(count.to_string());
    }
}

fn render_mask<'a>(
    input: &MemorySnapshotImageInput,
    ranges: impl Clone + Iterator<Item = MaskRange<'a>>,
    profile: &Value,
    writes: bool,
) -> Result<MemorySnapshotImage, String> {
    validate(input)?;
    for range in ranges.clone() {
        if range.start >= range.end
            || !matches!(range.memory, "heap" | "stable")
            || (range.memory == "heap" && (range.counts[1] != 0 || range.counts[2] != 0))
            || (range.memory == "stable" && range.counts[0] != 0)
        {
            return Err("Byte-access ranges require a nonempty address interval and counters for their own memory kind.".to_string());
        }
    }
    let (mut canvas, columns, rows, width) = page_canvas(input.pages.len());
    let height = canvas.height;
    let mut pages = Vec::with_capacity(input.pages.len());
    let section = if writes { "writes" } else { "reads" };
    let fields = if writes {
        MaskFields {
            frequency: &["heap_writes", "stable_writes"],
            bytes: &["heap_byte_writes", "stable_byte_writes"],
            distinct: &["distinct_heap_bytes", "distinct_stable_bytes"],
            maximum: &[
                "maximum_heap_writes_per_byte",
                "maximum_stable_writes_per_byte",
            ],
        }
    } else {
        MaskFields {
            frequency: &["heap_reads", "stable_direct_reads", "stable_native_reads"],
            bytes: &[
                "heap_byte_reads",
                "stable_direct_byte_reads",
                "stable_native_byte_reads",
            ],
            distinct: &[
                "distinct_heap_bytes",
                "distinct_stable_direct_bytes",
                "distinct_stable_native_bytes",
            ],
            maximum: &[
                "maximum_heap_reads_per_byte",
                "maximum_stable_direct_reads_per_byte",
                "maximum_stable_native_reads_per_byte",
            ],
        }
    };
    let mut total_distinct = 0_u64;
    let mut total_reads = [0_u128; 3];
    let mut frequency_ranges_remaining = READ_FREQUENCY_RANGES_LIMIT;
    let mut frequency_ranges_truncated = false;
    let mut counters_saturated = profile["counters_saturated"] == true;
    for (position, page) in input.pages.iter().enumerate() {
        let (body_x, label_y, body_y) = page_position(position, columns);
        let id = page_id(page);
        canvas.text(body_x + 2, label_y + 1, &id, INK);
        let base = page.page_index as u128 * WASM_PAGE_BYTES as u128;
        let end = base + WASM_PAGE_BYTES as u128;
        let mut events: BTreeMap<usize, [i128; 3]> = BTreeMap::new();
        for range in ranges.clone().filter(|range| range.memory == page.memory) {
            let start = (range.start as u128).max(base);
            let stop = (range.end as u128).min(end);
            if start >= stop {
                continue;
            }
            let counts = range.counts;
            for (at, sign) in [
                ((start - base) as usize, 1_i128),
                ((stop - base) as usize, -1_i128),
            ] {
                let delta = events.entry(at).or_default();
                for (delta, count) in delta.iter_mut().zip(counts) {
                    *delta += sign * i128::from(count);
                }
            }
        }
        let mut running = [0_i128; 3];
        let mut distinct = 0_u64;
        let mut distinct_by_path = [0_u64; 3];
        let mut reads = [0_u128; 3];
        let mut maximum = [0_u64; 3];
        let mut frequency_ranges = Vec::new();
        let mut page_ranges_truncated = false;
        let mut previous = [0_u64; 3];
        let mut run_start = 0;
        let mut events = events.into_iter().peekable();
        for offset in 0..=WASM_PAGE_BYTES {
            if events.peek().is_some_and(|(at, _)| *at == offset) {
                let (_, delta) = events.next().unwrap();
                for (counter, delta) in running.iter_mut().zip(delta) {
                    *counter += delta;
                }
            }
            let counts = running.map(|value| {
                if value > i128::from(u64::MAX) {
                    counters_saturated = true;
                    u64::MAX
                } else {
                    value.max(0) as u64
                }
            });
            if counts != previous || offset == WASM_PAGE_BYTES {
                if previous != [0; 3] && run_start < offset {
                    if frequency_ranges_remaining > 0 {
                        let mut range = json!({
                            "start": (base + run_start as u128).to_string(),
                            "end": (base + offset as u128).to_string(),
                            "start_page_byte_offset": run_start.to_string(),
                            "end_page_byte_offset_exclusive": offset.to_string(),
                        });
                        add_counts(&mut range, fields.frequency, previous.map(u128::from));
                        frequency_ranges.push(range);
                        frequency_ranges_remaining -= 1;
                    } else {
                        page_ranges_truncated = true;
                        frequency_ranges_truncated = true;
                    }
                }
                run_start = offset;
                previous = counts;
            }
            if offset == WASM_PAGE_BYTES {
                break;
            }
            distinct += u64::from(counts != [0; 3]);
            for channel in 0..3 {
                distinct_by_path[channel] += u64::from(counts[channel] != 0);
                reads[channel] += u128::from(counts[channel]);
                maximum[channel] = maximum[channel].max(counts[channel]);
            }
            canvas.pixel(
                body_x + offset % BYTE_WIDTH,
                body_y + offset / BYTE_WIDTH,
                mask_color(&page.memory, counts, writes),
            );
        }
        total_distinct += distinct;
        for (total, count) in total_reads.iter_mut().zip(reads) {
            *total += count;
        }
        let mut counts = json!({
            "distinct_observed_bytes": distinct.to_string(),
            "frequency_ranges": frequency_ranges, "frequency_ranges_truncated": page_ranges_truncated,
        });
        add_counts(&mut counts, fields.bytes, reads);
        add_counts(
            &mut counts,
            fields.distinct,
            distinct_by_path.map(u128::from),
        );
        add_counts(&mut counts, fields.maximum, maximum.map(u128::from));
        pages.push(json!({
            "id": id, "memory": page.memory, "page_index": page.page_index.to_string(),
            "allocated_bytes": page.allocated_bytes.to_string(),
            "comparison_allocated_bytes": page.comparison_allocated_bytes.map(|value| value.to_string()),
            "start_address": base.to_string(), "end_address_exclusive": end.to_string(),
            "byte_pixel_body": {"x": body_x, "y": body_y, "width": BYTE_WIDTH, "height": BYTE_HEIGHT},
            "label_strip": {"x": body_x, "y": label_y, "width": BYTE_WIDTH, "height": LABEL_HEIGHT},
            (section): counts,
            "page_evidence": {"memory": page.memory, "granularity_bytes": WASM_PAGE_BYTES.to_string(), "accessed": page.accessed, "dirty": page.dirty},
        }));
    }
    let png = encode_png(&canvas)?;
    let brightness: Vec<_> = READ_BRIGHTNESS.into_iter().enumerate().map(|(bin, value)| json!({
        "minimum_reads": (1_u64 << bin).to_string(),
        "maximum_reads": (bin + 1 < READ_BRIGHTNESS.len()).then(|| ((1_u64 << (bin + 1)) - 1).to_string()),
        "brightness": value,
        "heap_rgb": [0, value, value], "stable_direct_rgb": [0, value, 0], "stable_native_rgb": [value, (u16::from(value) * 2 / 5) as u8, 0],
    })).collect();
    let mut totals = json!({"distinct_observed_bytes": total_distinct.to_string(), "counters_saturated": counters_saturated});
    add_counts(&mut totals, fields.bytes, total_reads);
    let mut summary = json!({
        "format_version": 1, "view": section, "kind": if writes { "memory_write_mask" } else { "memory_read_mask" }, "scope": "snapshot_window_comparison", "memory": input.memory,
        "snapshot_id": input.snapshot_id, "comparison_snapshot_id": input.comparison_snapshot_id,
        "selected_pages": input.pages.len().to_string(), "selected_byte_positions": (input.pages.len() * WASM_PAGE_BYTES).to_string(),
        "pages": pages,
        (section): totals,
        "palette": {"unobserved_rgb": [0, 0, 0], "brightness": brightness,
            "frequency_rule": "Brightness uses total reads per byte: floor(log2(count)), capped at bin 8 (256 or more). It represents frequency, not instructions or cycle costs.",
            "path_rule": "Heap reads are cyan. Stable direct reads are green; any stable native read gives orange precedence. JSON frequency ranges retain both stable channel counts.",
            "charge_rule": "Direct and native are observed stable execution paths, not a free-versus-paid classification. Direct reads retain API/copy costs. Native repeated reads do not each pay the first-access tax. Actual charge records remain in execution evidence."},
        "image_grid_color": GRID_COLOR,
        "image": {"mime_type": "image/png", "width": width, "height": height, "layout": "compact_byte_pages", "png_bytes": png.len().to_string(),
            "byte_pixel_width": BYTE_WIDTH, "byte_pixel_height": BYTE_HEIGHT, "bytes_per_pixel": "1", "columns": columns, "rows": rows,
            "resampled": false, "grid_width": GRID_WIDTH, "label_strip_height": LABEL_HEIGHT, "labels": "page_id_only", "unused_grid_cells": columns * rows - input.pages.len(),
            "coordinate_rule": "Identical to view=data for the same snapshots, memory, selection, pagination and execution filters. Byte offset = 256 * (pixel_y - body.y) + (pixel_x - body.x). H/S page identifiers and the reserved grid contain no read information."},
        "coverage": {"snapshot_byte_values_encoded_in_png": false, "read_mask_uses_retained_byte_observations": true,
            "black_means": "No retained read observation for this byte. Consult read_profile coverage, truncation, missing attempts and counter saturation before interpreting this as unread.",
            "page_selection": "Same touched-page selection as data; written-only pages remain blank. Reads from discarded attempts or transient allocations can lie outside committed snapshot allocation.",
            "frequency_ranges_limit": READ_FREQUENCY_RANGES_LIMIT, "frequency_ranges_truncated": frequency_ranges_truncated,
            "frequency_ranges_note": "JSON range output is independently bounded. PNG and aggregate counts use all retained input ranges even if the JSON frequency-range list is truncated. Raw execution profiles retain their own bounded ranges.",
            "instruction_charge_attribution_per_byte": false},
        "limits": {"max_pages": MAX_IMAGE_PAGES, "max_columns": MAX_IMAGE_COLUMNS, "max_png_bytes": MAX_PNG_BYTES.to_string()},
    });
    if writes {
        write_metadata(&mut summary);
    }
    Ok(MemorySnapshotImage { png, summary })
}

/// Write-specific palette and explanatory metadata; numeric output is emitted directly.
fn write_metadata(summary: &mut Value) {
    let brightness: Vec<_> = READ_BRIGHTNESS.into_iter().enumerate().map(|(bin, value)| json!({
        "minimum_writes": (1_u64 << bin).to_string(),
        "maximum_writes": (bin + 1 < READ_BRIGHTNESS.len()).then(|| ((1_u64 << (bin + 1)) - 1).to_string()),
        "brightness": value,
        "heap_rgb": [value, 0, (u16::from(value) * 96 / 255) as u8],
        "stable_rgb": [value, (u16::from(value) * 96 / 255) as u8, 0],
    })).collect();
    summary["palette"] = json!({
        "unobserved_rgb": [0, 0, 0], "brightness": brightness,
        "frequency_rule": "Brightness uses total observed writes per byte: floor(log2(count)), capped at bin 8 (256 or more). It represents frequency, not instructions or cycle costs.",
        "memory_rule": "Heap writes are pink; stable writes are orange-red. Both remain distinct from the reserved magenta grid.",
        "write_rule": "Successful logical writes are counted even if the stored value was unchanged, or a later write restored the original value. Snapshot byte differences do not establish individual writes.",
        "charge_rule": "Frequency is independent of first-access and dirty-page taxes. Repeated writes do not each incur the page tax. Actual instruction charges remain in execution evidence."
    });
    summary["image"]["coordinate_rule"] = json!(
        "Identical to view=data and view=reads for the same snapshots, memory, selection, pagination and execution filters. Byte offset = 256 * (pixel_y - body.y) + (pixel_x - body.x). H/S page identifiers and the reserved grid contain no write information."
    );
    summary["coverage"]
        .as_object_mut()
        .unwrap()
        .remove("read_mask_uses_retained_byte_observations");
    summary["coverage"]["write_mask_uses_retained_byte_observations"] = json!(true);
    summary["coverage"]["black_means"] = json!(
        "No retained write observation for this byte. Consult write_profile coverage, truncation, missing attempts and counter saturation before interpreting this as unwritten."
    );
    summary["coverage"]["page_selection"] = json!(
        "Same touched-page selection as data and reads; read-only pages remain blank. Writes from discarded attempts or transient allocations can lie outside committed snapshot allocation."
    );
}

fn byte_color(class: ByteClass, offset: usize) -> Color {
    if class == ByteClass::OutsideAllocated
        && ((offset % BYTE_WIDTH) / 8 + (offset / BYTE_WIDTH) / 8) % 2 == 1
    {
        [230, 233, 238]
    } else {
        class.color()
    }
}

fn append_range(
    ranges: &mut Vec<Value>,
    truncated: &mut bool,
    base: u128,
    start: usize,
    end: usize,
) {
    if ranges.len() < DIFFERENCE_RANGES_LIMIT {
        ranges.push(json!({ "start_address": (base + start as u128).to_string(),
            "end_address_exclusive": (base + end as u128).to_string(),
            "start_page_byte_offset": start.to_string(), "end_page_byte_offset_exclusive": end.to_string() }));
    } else {
        *truncated = true;
    }
}

fn validate(input: &MemorySnapshotImageInput) -> Result<(), String> {
    if !matches!(input.memory.as_str(), "heap" | "stable" | "mixed" | "both") {
        return Err("Memory image kind must be heap, stable, or both.".to_string());
    }
    if input.pages.is_empty() || input.pages.len() > MAX_IMAGE_PAGES {
        return Err(format!(
            "Memory images require 1..={MAX_IMAGE_PAGES} selected pages; paginate larger touched sets explicitly."
        ));
    }
    for id in std::iter::once(&input.snapshot_id).chain(input.comparison_snapshot_id.iter()) {
        if id.is_empty() || id.len() > MAX_ID_BYTES {
            return Err(format!(
                "Snapshot identifiers must contain 1..={MAX_ID_BYTES} bytes."
            ));
        }
    }
    let comparison = input.comparison_snapshot_id.is_some();
    if !comparison && input.comparison_allocated_bytes.is_some() {
        return Err("Comparison allocation metadata requires a comparison snapshot.".to_string());
    }
    let mut pages = BTreeSet::new();
    let mut allocations = BTreeMap::new();
    for page in &input.pages {
        if !matches!(page.memory.as_str(), "heap" | "stable") {
            return Err("Each page must identify heap or stable memory.".to_string());
        }
        if !pages.insert((page.memory.as_str(), page.page_index)) {
            return Err(
                "Selected memory image pages must be distinct within each memory kind.".to_string(),
            );
        }
        if page
            .page_index
            .checked_mul(WASM_PAGE_BYTES as u64)
            .is_none()
        {
            return Err(
                "Memory image page address exceeds the u64 byte address space.".to_string(),
            );
        }
        if page.bytes.len() != WASM_PAGE_BYTES
            || page
                .comparison_bytes
                .as_ref()
                .is_some_and(|bytes| bytes.len() != WASM_PAGE_BYTES)
        {
            return Err(format!(
                "Each memory image page must contain exactly {WASM_PAGE_BYTES} bytes."
            ));
        }
        if comparison != page.comparison_bytes.is_some()
            || comparison != page.comparison_allocated_bytes.is_some()
        {
            return Err("Every selected page requires exact before/after allocation and comparison coverage.".to_string());
        }
        let allocation = (page.allocated_bytes, page.comparison_allocated_bytes);
        if allocations
            .insert(page.memory.as_str(), allocation)
            .is_some_and(|prior| prior != allocation)
        {
            return Err(
                "Pages of one memory kind must share the same snapshot allocation sizes."
                    .to_string(),
            );
        }
        if page.accessed == Some(false) && page.dirty == Some(true) {
            return Err("A dirty page cannot have known-false accessed evidence.".to_string());
        }
    }
    // Kept as a legacy input field; mixed sheets use authoritative per-page sizes.
    let _ = input.allocated_bytes;
    Ok(())
}

struct Canvas {
    width: usize,
    height: usize,
    rgb: Vec<u8>,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        let mut canvas = Self {
            width,
            height,
            rgb: vec![0; width * height * 3],
        };
        canvas.rect(0, 0, width, height, BACKGROUND);
        canvas
    }

    fn pixel(&mut self, x: usize, y: usize, color: Color) {
        if x < self.width && y < self.height {
            let offset = 3 * (y * self.width + x);
            self.rgb[offset..offset + 3].copy_from_slice(&color);
        }
    }

    fn rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: Color) {
        for row in y..y.saturating_add(height).min(self.height) {
            for column in x..x.saturating_add(width).min(self.width) {
                self.pixel(column, row, color);
            }
        }
    }

    fn text(&mut self, x: usize, y: usize, value: &str, color: Color) {
        for (index, ch) in value.chars().enumerate() {
            let left = x + index * 6;
            if left + 5 > self.width {
                break;
            }
            for (row, bits) in glyph(ch.to_ascii_uppercase()).iter().enumerate() {
                for column in 0..5 {
                    if bits & (1 << (4 - column)) != 0 {
                        self.pixel(left + column, y + row, color);
                    }
                }
            }
        }
    }
}

fn encode_png(canvas: &Canvas) -> Result<Vec<u8>, String> {
    let png = super::rendering::encode_png(canvas.width, canvas.height, &canvas.rgb)
        .map_err(|error| format!("PNG compression failed: {error}"))?;
    if png.len() > MAX_PNG_BYTES {
        return Err(format!(
            "Memory image exceeds the {MAX_PNG_BYTES}-byte PNG limit; select fewer pages."
        ));
    }
    Ok(png)
}

/// Small bitmap font avoids system font dependencies and nondeterministic layout.
pub(crate) fn glyph(ch: char) -> [u8; 7] {
    match ch {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '_' => [0, 0, 0, 0, 0, 0, 31],
        '=' => [0, 0, 31, 0, 31, 0, 0],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        '·' => [0, 0, 0, 4, 0, 0, 0],
        '…' => [0, 0, 0, 0, 0, 0, 21],
        '(' => [2, 4, 8, 8, 8, 4, 2],
        ')' => [8, 4, 2, 2, 2, 4, 8],
        ';' => [0, 4, 4, 0, 4, 4, 8],
        '|' => [4, 4, 4, 4, 4, 4, 4],
        '%' => [25, 25, 2, 4, 8, 19, 19],
        ' ' => [0; 7],
        _ => [14, 17, 1, 2, 4, 0, 4],
    }
}

#[cfg(test)]
mod tests;
