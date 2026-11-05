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
        let size = u32::from_be_bytes(image.png[offset..offset + 4].try_into().unwrap()) as usize;
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
