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

/// A retained byte-read frequency range, independent of snapshot contents.
/// Ranges from different executions may overlap; their counters are added.
#[derive(Clone, Debug)]
pub(crate) struct MemorySnapshotReadRange {
    pub memory: String,
    pub start: u64,
    pub end: u64,
    pub heap_reads: u64,
    pub stable_direct_reads: u64,
    pub stable_native_reads: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct MemorySnapshotWriteRange {
    pub memory: String,
    pub start: u64,
    pub end: u64,
    pub heap_writes: u64,
    pub stable_writes: u64,
}

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
    render_mask(input, ranges, profile, false)
}

/// Writes use the same bounded range-sweep and layout as reads. Their counters
/// are observations of successful writes, independent of net byte differences.
pub(crate) fn render_writes(
    input: &MemorySnapshotImageInput,
    ranges: &[MemorySnapshotWriteRange],
    profile: &Value,
) -> Result<MemorySnapshotImage, String> {
    let ranges: Vec<_> = ranges
        .iter()
        .map(|range| MemorySnapshotReadRange {
            memory: range.memory.clone(),
            start: range.start,
            end: range.end,
            heap_reads: range.heap_writes,
            stable_direct_reads: range.stable_writes,
            stable_native_reads: 0,
        })
        .collect();
    render_mask(input, &ranges, profile, true)
}

fn render_mask(
    input: &MemorySnapshotImageInput,
    ranges: &[MemorySnapshotReadRange],
    profile: &Value,
    writes: bool,
) -> Result<MemorySnapshotImage, String> {
    validate(input)?;
    for range in ranges {
        if range.start >= range.end
            || !matches!(range.memory.as_str(), "heap" | "stable")
            || (range.memory == "heap"
                && (range.stable_direct_reads != 0 || range.stable_native_reads != 0))
            || (range.memory == "stable" && range.heap_reads != 0)
        {
            return Err("Byte-access ranges require a nonempty address interval and counters for their own memory kind.".to_string());
        }
    }
    let (mut canvas, columns, rows, width) = page_canvas(input.pages.len());
    let height = canvas.height;
    let mut pages = Vec::with_capacity(input.pages.len());
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
        for range in ranges.iter().filter(|range| range.memory == page.memory) {
            let start = (range.start as u128).max(base);
            let stop = (range.end as u128).min(end);
            if start >= stop {
                continue;
            }
            let counts = [
                range.heap_reads,
                range.stable_direct_reads,
                range.stable_native_reads,
            ];
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
                        frequency_ranges.push(json!({
                            "start": (base + run_start as u128).to_string(),
                            "end": (base + offset as u128).to_string(),
                            "start_page_byte_offset": run_start.to_string(),
                            "end_page_byte_offset_exclusive": offset.to_string(),
                            "heap_reads": previous[0].to_string(),
                            "stable_direct_reads": previous[1].to_string(),
                            "stable_native_reads": previous[2].to_string(),
                        }));
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
        pages.push(json!({
            "id": id, "memory": page.memory, "page_index": page.page_index.to_string(),
            "allocated_bytes": page.allocated_bytes.to_string(),
            "comparison_allocated_bytes": page.comparison_allocated_bytes.map(|value| value.to_string()),
            "start_address": base.to_string(), "end_address_exclusive": end.to_string(),
            "byte_pixel_body": {"x": body_x, "y": body_y, "width": BYTE_WIDTH, "height": BYTE_HEIGHT},
            "label_strip": {"x": body_x, "y": label_y, "width": BYTE_WIDTH, "height": LABEL_HEIGHT},
            "reads": {
                "distinct_observed_bytes": distinct.to_string(),
                "heap_byte_reads": reads[0].to_string(), "stable_direct_byte_reads": reads[1].to_string(), "stable_native_byte_reads": reads[2].to_string(),
                "distinct_heap_bytes": distinct_by_path[0].to_string(), "distinct_stable_direct_bytes": distinct_by_path[1].to_string(), "distinct_stable_native_bytes": distinct_by_path[2].to_string(),
                "maximum_heap_reads_per_byte": maximum[0].to_string(), "maximum_stable_direct_reads_per_byte": maximum[1].to_string(), "maximum_stable_native_reads_per_byte": maximum[2].to_string(),
                "frequency_ranges": frequency_ranges, "frequency_ranges_truncated": page_ranges_truncated,
            },
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
    let mut summary = json!({
        "format_version": 1, "view": "reads", "kind": "memory_read_mask", "scope": "snapshot_window_comparison", "memory": input.memory,
        "snapshot_id": input.snapshot_id, "comparison_snapshot_id": input.comparison_snapshot_id,
        "selected_pages": input.pages.len().to_string(), "selected_byte_positions": (input.pages.len() * WASM_PAGE_BYTES).to_string(),
        "pages": pages,
        "reads": {"distinct_observed_bytes": total_distinct.to_string(), "heap_byte_reads": total_reads[0].to_string(), "stable_direct_byte_reads": total_reads[1].to_string(), "stable_native_byte_reads": total_reads[2].to_string(), "counters_saturated": counters_saturated},
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
        adapt_write_summary(&mut summary);
    }
    Ok(MemorySnapshotImage { png, summary })
}

fn rename_field(value: &mut Value, old: &str, new: &str) {
    if let Some(object) = value.as_object_mut()
        && let Some(field) = object.remove(old)
    {
        object.insert(new.to_string(), field);
    }
}

/// The shared sweep uses three channels for read-path counts; write masks use
/// its first two channels and expose a separate, correctly named public schema.
fn adapt_write_summary(summary: &mut Value) {
    fn write_counts(value: &mut Value) {
        for (old, new) in [
            ("heap_byte_reads", "heap_byte_writes"),
            ("stable_direct_byte_reads", "stable_byte_writes"),
            ("distinct_stable_direct_bytes", "distinct_stable_bytes"),
            (
                "maximum_heap_reads_per_byte",
                "maximum_heap_writes_per_byte",
            ),
            (
                "maximum_stable_direct_reads_per_byte",
                "maximum_stable_writes_per_byte",
            ),
        ] {
            rename_field(value, old, new);
        }
        if let Some(object) = value.as_object_mut() {
            for field in [
                "stable_native_byte_reads",
                "distinct_stable_native_bytes",
                "maximum_stable_native_reads_per_byte",
            ] {
                object.remove(field);
            }
        }
        if let Some(ranges) = value
            .get_mut("frequency_ranges")
            .and_then(Value::as_array_mut)
        {
            for range in ranges {
                rename_field(range, "heap_reads", "heap_writes");
                rename_field(range, "stable_direct_reads", "stable_writes");
                range.as_object_mut().unwrap().remove("stable_native_reads");
            }
        }
    }
    summary["view"] = json!("writes");
    summary["kind"] = json!("memory_write_mask");
    rename_field(summary, "reads", "writes");
    write_counts(&mut summary["writes"]);
    for page in summary["pages"].as_array_mut().unwrap() {
        rename_field(page, "reads", "writes");
        write_counts(&mut page["writes"]);
    }
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
mod tests {
    use super::*;
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    #[test]
    fn snapshot_png_budget_stays_with_the_snapshot_caller() {
        assert_eq!(MAX_PNG_BYTES, 6 * 1024 * 1024);
        let mut canvas = Canvas::new(2048, 1024);
        let mut pattern = 0x5eed1234u32;
        for byte in &mut canvas.rgb {
            pattern ^= pattern << 13;
            pattern ^= pattern >> 17;
            pattern ^= pattern << 5;
            *byte = pattern as u8;
        }
        let raw =
            super::super::rendering::encode_png(canvas.width, canvas.height, &canvas.rgb).unwrap();
        assert!(raw.len() > MAX_PNG_BYTES);
        assert!(encode_png(&canvas).is_err());
        assert!(encode_png(&Canvas::new(1, 1)).unwrap().len() < MAX_PNG_BYTES);
    }

    fn input() -> MemorySnapshotImageInput {
        MemorySnapshotImageInput {
            memory: "heap".to_string(),
            snapshot_id: "after".to_string(),
            comparison_snapshot_id: None,
            allocated_bytes: WASM_PAGE_BYTES as u64,
            comparison_allocated_bytes: None,
            pages: vec![MemorySnapshotImagePage {
                memory: "heap".to_string(),
                allocated_bytes: WASM_PAGE_BYTES as u64,
                comparison_allocated_bytes: None,
                page_index: 0,
                bytes: vec![0; WASM_PAGE_BYTES],
                comparison_bytes: None,
                accessed: None,
                dirty: None,
            }],
        }
    }

    fn compare(input: &mut MemorySnapshotImageInput, before_allocated: u64) {
        input.comparison_snapshot_id = Some("before".to_string());
        input.comparison_allocated_bytes = Some(before_allocated);
        for page in &mut input.pages {
            page.comparison_allocated_bytes = Some(before_allocated);
            page.comparison_bytes = Some(vec![0; WASM_PAGE_BYTES]);
        }
    }

    fn decoded(image: &MemorySnapshotImage) -> (usize, usize, Vec<u8>) {
        assert_eq!(&image.png[..8], b"\x89PNG\r\n\x1a\n");
        let (mut width, mut height) = (0, 0);
        let mut compressed = Vec::new();
        let mut offset = 8;
        while offset < image.png.len() {
            let size =
                u32::from_be_bytes(image.png[offset..offset + 4].try_into().unwrap()) as usize;
            let kind = &image.png[offset + 4..offset + 8];
            let data = &image.png[offset + 8..offset + 8 + size];
            let expected = u32::from_be_bytes(
                image.png[offset + 8 + size..offset + 12 + size]
                    .try_into()
                    .unwrap(),
            );
            let mut crc = crc32fast::Hasher::new();
            crc.update(kind);
            crc.update(data);
            assert_eq!(crc.finalize(), expected);
            if kind == b"IHDR" {
                width = u32::from_be_bytes(data[..4].try_into().unwrap()) as usize;
                height = u32::from_be_bytes(data[4..8].try_into().unwrap()) as usize;
                assert_eq!(&data[8..], &[8, 2, 0, 0, 0]);
            } else if kind == b"IDAT" {
                compressed.extend_from_slice(data);
            }
            offset += size + 12;
        }
        let mut scanlines = Vec::new();
        ZlibDecoder::new(&compressed[..])
            .read_to_end(&mut scanlines)
            .unwrap();
        assert_eq!(scanlines.len(), height * (width * 3 + 1));
        let mut rgb = Vec::new();
        for row in scanlines.chunks_exact(width * 3 + 1) {
            assert_eq!(row[0], 0);
            rgb.extend_from_slice(&row[1..]);
        }
        (width, height, rgb)
    }

    fn pixel(image: &MemorySnapshotImage, page: usize, offset: usize) -> Color {
        let (width, _, rgb) = decoded(image);
        let body = &image.summary["pages"][page]["byte_pixel_body"];
        let x = body["x"].as_u64().unwrap() as usize + offset % BYTE_WIDTH;
        let y = body["y"].as_u64().unwrap() as usize + offset / BYTE_WIDTH;
        rgb[3 * (y * width + x)..3 * (y * width + x) + 3]
            .try_into()
            .unwrap()
    }

    #[test]
    fn read_mask_preserves_data_layout_and_never_encodes_contents() {
        let mut input = input();
        input.memory = "both".to_string();
        input.pages[0].bytes.fill(255);
        input.pages[0].dirty = Some(true);
        input.pages[0].accessed = Some(true);
        let mut stable = input.pages[0].clone();
        stable.memory = "stable".to_string();
        stable.page_index = 3;
        stable.allocated_bytes = 4 * WASM_PAGE_BYTES as u64;
        input.pages.push(stable);
        let data = render(&input).unwrap();
        let reads = render_reads(&input, &[], &json!({"available": true})).unwrap();
        assert_eq!(
            data.summary["image"]["width"],
            reads.summary["image"]["width"]
        );
        assert_eq!(
            data.summary["image"]["height"],
            reads.summary["image"]["height"]
        );
        for page in 0..2 {
            assert_eq!(
                data.summary["pages"][page]["id"],
                reads.summary["pages"][page]["id"]
            );
            assert_eq!(
                data.summary["pages"][page]["byte_pixel_body"],
                reads.summary["pages"][page]["byte_pixel_body"]
            );
            assert_eq!(pixel(&reads, page, 0), [0; 3]);
            assert_eq!(pixel(&reads, page, WASM_PAGE_BYTES - 1), [0; 3]);
            assert!(reads.summary["pages"][page].get("counts").is_none());
        }
        assert_eq!(reads.summary["reads"]["distinct_observed_bytes"], "0");
    }

    #[test]
    fn read_mask_adds_overlapping_byte_frequencies_and_retains_stable_paths() {
        let mut input = input();
        input.memory = "both".to_string();
        let mut stable = input.pages[0].clone();
        stable.memory = "stable".to_string();
        input.pages.push(stable);
        let ranges = vec![
            MemorySnapshotReadRange {
                memory: "heap".into(),
                start: 255,
                end: 259,
                heap_reads: 1,
                stable_direct_reads: 0,
                stable_native_reads: 0,
            },
            MemorySnapshotReadRange {
                memory: "heap".into(),
                start: 256,
                end: 258,
                heap_reads: 3,
                stable_direct_reads: 0,
                stable_native_reads: 0,
            },
            MemorySnapshotReadRange {
                memory: "stable".into(),
                start: 7,
                end: 10,
                heap_reads: 0,
                stable_direct_reads: 1,
                stable_native_reads: 0,
            },
            MemorySnapshotReadRange {
                memory: "stable".into(),
                start: 8,
                end: 9,
                heap_reads: 0,
                stable_direct_reads: 0,
                stable_native_reads: 3,
            },
        ];
        let image = render_reads(&input, &ranges, &json!({"available": true})).unwrap();
        assert_eq!(pixel(&image, 0, 254), [0, 0, 0]);
        assert_eq!(pixel(&image, 0, 255), [0, 64, 64]);
        assert_eq!(pixel(&image, 0, 256), [0, 112, 112]);
        assert_eq!(pixel(&image, 0, 258), [0, 64, 64]);
        assert_eq!(pixel(&image, 1, 7), [0, 64, 0]);
        assert_eq!(pixel(&image, 1, 8), [112, 44, 0]);
        assert_eq!(pixel(&image, 1, 9), [0, 64, 0]);
        assert_eq!(image.summary["reads"]["distinct_observed_bytes"], "7");
        assert_eq!(image.summary["reads"]["heap_byte_reads"], "10");
        assert_eq!(image.summary["reads"]["stable_direct_byte_reads"], "3");
        assert_eq!(image.summary["reads"]["stable_native_byte_reads"], "3");
        let overlap = &image.summary["pages"][1]["reads"]["frequency_ranges"][1];
        assert_eq!(overlap["start"], "8");
        assert_eq!(overlap["end"], "9");
        assert_eq!(overlap["stable_direct_reads"], "1");
        assert_eq!(overlap["stable_native_reads"], "3");
    }

    #[test]
    fn read_mask_uses_fixed_log_brightness_and_reports_counter_saturation() {
        let input = input();
        let mut ranges = vec![MemorySnapshotReadRange {
            memory: "heap".into(),
            start: 0,
            end: 1,
            heap_reads: u64::MAX,
            stable_direct_reads: 0,
            stable_native_reads: 0,
        }];
        ranges.push(ranges[0].clone());
        let image = render_reads(&input, &ranges, &json!({"available": true})).unwrap();
        assert_eq!(pixel(&image, 0, 0), [0, 255, 255]);
        assert_eq!(image.summary["reads"]["counters_saturated"], true);
        assert_eq!(read_brightness(1), 64);
        assert_eq!(read_brightness(3), 88);
        assert_eq!(read_brightness(255), 232);
        assert_eq!(read_brightness(256), 255);
        assert_eq!(read_brightness(u64::MAX), 255);
        ranges[0].stable_native_reads = 1;
        assert!(render_reads(&input, &ranges, &json!({})).is_err());
    }

    #[test]
    fn write_mask_shows_same_value_and_restored_writes_with_read_only_pages_blank() {
        let mut input = input();
        input.memory = "both".to_string();
        input.allocated_bytes = 2 * WASM_PAGE_BYTES as u64;
        input.pages[0].allocated_bytes = input.allocated_bytes;
        input.pages[0].accessed = Some(true);
        input.pages[0].dirty = Some(true);
        let mut read_only = input.pages[0].clone();
        read_only.page_index = 1;
        read_only.dirty = Some(false);
        let mut stable = input.pages[0].clone();
        stable.memory = "stable".to_string();
        input.pages.extend([read_only, stable]);
        compare(&mut input, 2 * WASM_PAGE_BYTES as u64);
        let ranges = vec![
            MemorySnapshotWriteRange {
                memory: "heap".into(),
                start: 32,
                end: 40,
                heap_writes: 3,
                stable_writes: 0,
            },
            MemorySnapshotWriteRange {
                memory: "heap".into(),
                start: 34,
                end: 36,
                heap_writes: 1,
                stable_writes: 0,
            },
            MemorySnapshotWriteRange {
                memory: "stable".into(),
                start: 10,
                end: 11,
                heap_writes: 0,
                stable_writes: 2,
            },
        ];
        let data = render(&input).unwrap();
        let reads = render_reads(&input, &[], &json!({"available": true})).unwrap();
        let writes = render_writes(&input, &ranges, &json!({"available": true})).unwrap();
        assert_eq!(data.summary["counts"]["changed_existing_bytes"], "0");
        assert_eq!(writes.summary["view"], "writes");
        assert_eq!(writes.summary["kind"], "memory_write_mask");
        for page in 0..3 {
            assert_eq!(
                writes.summary["pages"][page]["id"],
                data.summary["pages"][page]["id"]
            );
            assert_eq!(
                writes.summary["pages"][page]["byte_pixel_body"],
                data.summary["pages"][page]["byte_pixel_body"]
            );
            assert_eq!(
                writes.summary["pages"][page]["byte_pixel_body"],
                reads.summary["pages"][page]["byte_pixel_body"]
            );
        }
        assert_eq!(pixel(&writes, 0, 31), [0; 3]);
        assert_eq!(pixel(&writes, 0, 32), [88, 0, 33]);
        assert_eq!(pixel(&writes, 0, 34), [112, 0, 42]);
        assert_eq!(pixel(&writes, 1, 32), [0; 3]);
        assert_eq!(pixel(&writes, 2, 10), [88, 33, 0]);
        assert_eq!(writes.summary["writes"]["distinct_observed_bytes"], "9");
        assert_eq!(writes.summary["writes"]["heap_byte_writes"], "26");
        assert_eq!(writes.summary["writes"]["stable_byte_writes"], "2");
        let overlap = &writes.summary["pages"][0]["writes"]["frequency_ranges"][1];
        assert_eq!(overlap["start"], "34");
        assert_eq!(overlap["heap_writes"], "4");
        assert!(overlap.get("heap_reads").is_none());
        assert!(writes.summary.get("reads").is_none());
        assert!(writes.summary["pages"][0].get("reads").is_none());
    }

    #[test]
    fn snapshot_png_preserves_exact_byte_coordinates_and_counts() {
        let mut input = input();
        input.pages[0].bytes[255] = 1;
        input.pages[0].bytes[256] = 255;
        let image = render(&input).unwrap();
        let (width, height, _) = decoded(&image);
        assert_eq!((width, height), (258, 268));
        assert_eq!(
            image.summary["pages"][0]["byte_pixel_body"],
            json!({ "x": 1, "y": 11, "width": 256, "height": 256 })
        );
        assert_eq!(pixel(&image, 0, 0), ByteClass::Zero.color());
        assert_eq!(pixel(&image, 0, 255), ByteClass::Nonzero.color());
        assert_eq!(pixel(&image, 0, 256), ByteClass::Nonzero.color());
        assert_eq!(image.summary["counts"]["after_nonzero_bytes"], "2");
        assert_eq!(image.summary["counts"]["after_zero_bytes"], "65534");
        assert_eq!(image.summary["coverage"]["entire_memory_selected"], true);
        assert_eq!(
            image.summary["coverage"]["byte_values_encoded_in_png"],
            false
        );
    }

    #[test]
    fn one_changed_byte_is_not_a_64_kib_dirty_byte_claim() {
        let mut input = input();
        compare(&mut input, WASM_PAGE_BYTES as u64);
        input.pages[0].bytes[4096] = 7;
        input.pages[0].accessed = Some(true);
        input.pages[0].dirty = Some(true);
        let image = render(&input).unwrap();
        assert_eq!(image.summary["counts"]["changed_existing_bytes"], "1");
        assert_eq!(
            image.summary["pages"][0]["before_sha256"],
            "de2f256064a0af797747c2b97505dc0b9f3df0de4f489eac731c23ae9ca9cc31"
        );
        assert_ne!(
            image.summary["pages"][0]["after_sha256"],
            image.summary["pages"][0]["before_sha256"]
        );
        assert_eq!(
            image.summary["counts"]["classes"]["unchanged_zero"],
            "65535"
        );
        assert_eq!(image.summary["pages"][0]["page_evidence"]["dirty"], true);
        assert_eq!(
            image.summary["pages"][0]["page_evidence"]["granularity_bytes"],
            "65536"
        );
        assert_eq!(pixel(&image, 0, 4096), ByteClass::BecameNonzero.color());
        assert_eq!(pixel(&image, 0, 4095), ByteClass::UnchangedZero.color());
        assert_eq!(
            image.summary["first_differences"][0],
            json!({ "memory": "heap", "address": "4096", "page_index": "0",
            "page_byte_offset": "4096", "before": 0, "after": 7, "class": "became_nonzero" })
        );
        assert_eq!(
            image.summary["coverage"]["individual_byte_reads_or_writes_observed"],
            false
        );
        assert_eq!(
            image.summary["pages"][0]["difference_pixel_bounds"],
            json!({ "x_min": 0, "y_min": 16, "x_max_exclusive": 1, "y_max_exclusive": 17 })
        );
        assert_eq!(
            image.summary["pages"][0]["first_difference_address_hex"],
            "0x1000"
        );
        assert_eq!(
            image.summary["pages"][0]["first_difference_ranges"][0]["end_address_exclusive"],
            "4097"
        );
    }

    #[test]
    fn diff_distinguishes_clear_change_growth_and_outside_zero_padding() {
        let mut input = input();
        input.memory = "stable".to_string();
        input.pages[0].memory = "stable".to_string();
        input.pages[0].allocated_bytes = 4;
        compare(&mut input, 3);
        input.pages[0].comparison_bytes.as_mut().unwrap()[..3].copy_from_slice(&[5, 8, 0]);
        input.pages[0].bytes[..4].copy_from_slice(&[0, 9, 0, 0]);
        let image = render(&input).unwrap();
        for (offset, class) in [
            (0, ByteClass::BecameZero),
            (1, ByteClass::ChangedNonzero),
            (2, ByteClass::UnchangedZero),
            (3, ByteClass::NewlyAllocatedZero),
            (4, ByteClass::OutsideAllocated),
        ] {
            assert_eq!(pixel(&image, 0, offset), class.color());
        }
        assert_eq!(image.summary["counts"]["changed_existing_bytes"], "2");
        assert_eq!(image.summary["counts"]["newly_allocated_bytes"], "1");
        assert_eq!(
            image.summary["counts"]["classes"]["outside_allocated"],
            "65532"
        );
        assert!(image.summary["pages"][0]["page_evidence"]["accessed"].is_null());
    }

    #[test]
    fn compact_grid_retains_all_64_pages_with_bounded_exact_change_examples() {
        let mut input = input();
        compare(&mut input, 0);
        let page = input.pages[0].clone();
        input.pages = (0..64)
            .map(|index| MemorySnapshotImagePage {
                page_index: index,
                ..page.clone()
            })
            .collect();
        let image = render(&input).unwrap();
        let (width, height, rgb) = decoded(&image);
        assert_eq!((width, height), (2057, 2137));
        assert_eq!(image.summary["selected_pages"], "64");
        assert_eq!(image.summary["pages"].as_array().unwrap().len(), 64);
        assert!(image.png.len() < MAX_PNG_BYTES);
        assert_eq!(
            image.summary["first_differences"].as_array().unwrap().len(),
            FIRST_CHANGES_LIMIT
        );
        assert_eq!(image.summary["first_differences_truncated"], true);
        assert_eq!(
            image.summary["counts"]["classes"]["outside_allocated"],
            (63 * WASM_PAGE_BYTES).to_string()
        );
        assert_eq!(&rgb[..3], &GRID_COLOR);
        let last_body = &image.summary["pages"][63]["byte_pixel_body"];
        let last = 3
            * (last_body["y"].as_u64().unwrap() as usize * width
                + last_body["x"].as_u64().unwrap() as usize);
        assert_eq!(&rgb[last..last + 3], &ByteClass::OutsideAllocated.color());
        // The bounded renderer also accepts 256 distinct blocks; larger sets
        // require caller-visible pagination rather than silent truncation.
        input.pages = (0..MAX_IMAGE_PAGES)
            .map(|index| MemorySnapshotImagePage {
                page_index: index as u64,
                ..page.clone()
            })
            .collect();
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn invalid_sizes_duplicate_addresses_and_contradictory_evidence_are_rejected() {
        let mut invalid = input();
        invalid.pages.clear();
        assert!(render(&invalid).is_err());
        invalid = input();
        invalid.pages = vec![invalid.pages[0].clone(); MAX_IMAGE_PAGES + 1];
        assert!(render(&invalid).is_err());
        invalid = input();
        invalid.pages.push(invalid.pages[0].clone());
        assert!(render(&invalid).is_err());
        invalid = input();
        invalid.pages[0].page_index = u64::MAX;
        assert!(render(&invalid).is_err());
        invalid = input();
        invalid.pages[0].bytes.pop();
        assert!(render(&invalid).is_err());
        invalid = input();
        invalid.comparison_snapshot_id = Some("before".to_string());
        assert!(render(&invalid).is_err());
        invalid = input();
        invalid.pages[0].memory = "unknown".to_string();
        assert!(render(&invalid).is_err());
        invalid = input();
        invalid.pages[0].dirty = Some(true);
        invalid.pages[0].accessed = Some(false);
        assert!(render(&invalid).is_err());
        invalid = input();
        let mut other = invalid.pages[0].clone();
        other.page_index = 1;
        other.allocated_bytes = 1;
        invalid.pages.push(other);
        assert!(render(&invalid).is_err());
    }

    #[test]
    fn sparse_difference_ranges_are_bounded_and_large_page_addresses_stay_exact() {
        let mut input = input();
        compare(&mut input, u64::MAX);
        input.pages[0].page_index = u64::MAX / WASM_PAGE_BYTES as u64;
        input.pages[0].allocated_bytes = u64::MAX;
        for offset in (0..256).step_by(2) {
            input.pages[0].bytes[offset] = 1;
        }
        let image = render(&input).unwrap();
        assert_eq!(image.summary["counts"]["changed_existing_bytes"], "128");
        assert_eq!(
            image.summary["pages"][0]["first_difference_ranges"]
                .as_array()
                .unwrap()
                .len(),
            DIFFERENCE_RANGES_LIMIT
        );
        assert_eq!(
            image.summary["pages"][0]["difference_ranges_truncated"],
            true
        );
        assert_eq!(
            image.summary["pages"][0]["start_address"],
            "18446744073709486080"
        );
        assert_eq!(
            image.summary["pages"][0]["end_address_exclusive"],
            "18446744073709551616"
        );
        assert_eq!(
            pixel(&image, 0, WASM_PAGE_BYTES - 1),
            byte_color(ByteClass::OutsideAllocated, WASM_PAGE_BYTES - 1)
        );
    }

    #[test]
    fn mixed_memory_uses_page_allocations_only_id_labels_and_reserved_thin_grid() {
        let mut input = input();
        input.memory = "both".to_string();
        input.allocated_bytes = u64::MAX;
        input.pages[0].allocated_bytes = 1;
        let mut stable = input.pages[0].clone();
        stable.memory = "stable".to_string();
        stable.allocated_bytes = 2;
        stable.bytes[1] = 9;
        stable.accessed = Some(true);
        stable.dirty = Some(true);
        input.pages.push(stable);
        let image = render(&input).unwrap();
        let (width, height, rgb) = decoded(&image);
        assert_eq!((width, height), (515, 268));
        assert_eq!(image.summary["pages"][0]["id"], "H:0");
        assert_eq!(image.summary["pages"][1]["id"], "S:0");
        assert_eq!(image.summary["pages"][0]["memory"], "heap");
        assert_eq!(image.summary["pages"][1]["memory"], "stable");
        assert_eq!(image.summary["pages"][1]["page_evidence"]["dirty"], true);
        assert_eq!(image.summary["after_allocated_bytes"], "3");
        assert_eq!(image.summary["counts"]["after_nonzero_bytes"], "1");
        assert_eq!(pixel(&image, 0, 1), ByteClass::OutsideAllocated.color());
        assert_eq!(pixel(&image, 1, 1), ByteClass::Nonzero.color());
        assert_eq!(image.summary["image"]["layout"], "compact_byte_pages");
        assert_eq!(image.summary["image"]["labels"], "page_id_only");
        assert!(image.summary.get("first_difference_zoom").is_none());
        assert_eq!(image.summary["image_grid_color"], json!(GRID_COLOR));
        assert!(
            ByteClass::ALL
                .into_iter()
                .all(|class| class.color() != GRID_COLOR)
        );
        for y in 0..height {
            assert_eq!(
                &rgb[3 * (y * width + 257)..3 * (y * width + 257) + 3],
                &GRID_COLOR
            );
        }
        assert_eq!(
            &rgb[3 * ((height - 1) * width)..3 * ((height - 1) * width) + 3],
            &GRID_COLOR
        );
    }
}
