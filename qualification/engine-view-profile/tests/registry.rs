#![forbid(unsafe_code)]

//! Isolated process measuring only Registry decoding, not fixture construction.
#![cfg(all(feature = "registry-settings", not(target_family = "wasm")))]

use ic_platform_protocol::registry_settings::{Error, Limits, decode_subnet_settings};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[test]
fn registry_success_and_refusal_allocate_nothing() {
    let complete = vec![
        0xba, 1, 4, 0x18, 1, 0x48, 0, 0xc0, 1, 7, 0x92, 2, 8, 8, 0, 16, 2, 24, 3, 32, 4,
    ];
    // Field 1, opaque 65,536-byte payload. All fixture allocation precedes DHAT.
    let mut unknown = vec![10, 0x80, 0x80, 4];
    unknown.resize(65_540, 255);
    unknown.extend_from_slice(&complete);
    let mut duplicate = complete.clone();
    duplicate.extend_from_slice(&[0xc0, 1, 0]);
    let mut malformed = complete.clone();
    malformed.extend_from_slice(&[10, 2, 0]);
    for (name, bytes, fields, byte_limit, error) in [
        ("empty", &[][..], 0, 0, None),
        ("selected", complete.as_slice(), 9, complete.len(), None),
        ("large unknown", unknown.as_slice(), 10, unknown.len(), None),
        (
            "duplicate after selected",
            duplicate.as_slice(),
            10,
            duplicate.len(),
            Some(Error::DuplicateField),
        ),
        (
            "malformed after selected",
            malformed.as_slice(),
            10,
            malformed.len(),
            Some(Error::Malformed),
        ),
        (
            "field limit after selected",
            complete.as_slice(),
            8,
            complete.len(),
            Some(Error::FieldLimit),
        ),
        (
            "byte limit",
            complete.as_slice(),
            9,
            complete.len() - 1,
            Some(Error::ByteLimit),
        ),
    ] {
        let profiler = dhat::Profiler::builder().testing().build();
        let result = std::hint::black_box(decode_subnet_settings(
            std::hint::black_box(bytes),
            Limits {
                bytes: byte_limit,
                fields,
            },
        ));
        let stats = dhat::HeapStats::get();
        drop(profiler);
        assert_eq!(result.err(), error, "{name}");
        assert_eq!(stats.total_blocks, 0, "{name}");
        assert_eq!(stats.total_bytes, 0, "{name}");
        println!("Registry {name}: {stats:?}");
    }
}
