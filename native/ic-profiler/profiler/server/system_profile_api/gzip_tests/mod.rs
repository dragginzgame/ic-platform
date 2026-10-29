use super::*;
use ic_host_artifacts::artifact::encode_gzip;

fn fixture() -> Vec<u8> {
    let run = ic_system_profiler::start(RunConfig::default()).unwrap();
    let id = run["run_id"].as_str().unwrap();
    ic_system_profiler::stop(id, &StopOptions::default()).unwrap();
    let bytes = export_bytes(id).unwrap();
    assert_eq!(bytes, ic_system_profiler::export_bundle(id).unwrap());
    ic_system_profiler::delete_run(id).unwrap();
    bytes
}

fn imported(result: Result<Value, String>) {
    let value = result.unwrap();
    ic_system_profiler::delete_run(value["run_id"].as_str().unwrap()).unwrap();
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    encode_gzip(bytes, &mut output, 1, u64::MAX).unwrap();
    output
}

#[test]
fn gzip_framing_is_checked_before_importing_a_valid_profile() {
    let bytes = fixture();
    imported(import_bytes(&bytes));
    let raw = decode_gzip(&bytes, MAX_IMPORT_BYTES, MAX_IMPORT_BYTES).unwrap();
    imported(import_bytes(&raw));
    let mut crc = bytes.clone();
    let trailer = crc.len() - 8;
    crc[trailer] ^= 1;
    let mut size = bytes.clone();
    let last = size.len() - 1;
    size[last] ^= 1;
    let mut trailing = bytes.clone();
    trailing.push(0);
    let concatenated = [bytes.as_slice(), bytes.as_slice()].concat();
    for invalid in [
        crc.as_slice(),
        size.as_slice(),
        trailing.as_slice(),
        concatenated.as_slice(),
        &bytes[..3],
        &bytes[..bytes.len() - 1],
    ] {
        assert!(
            import_bytes(invalid)
                .unwrap_err()
                .starts_with("Invalid gzip profile bundle:")
        );
    }
}

#[test]
fn api_and_core_keep_their_distinct_inclusive_import_budgets() {
    let bytes = fixture();
    let mut raw = decode_gzip(&bytes, MAX_IMPORT_BYTES, MAX_IMPORT_BYTES).unwrap();
    raw.resize(MAX_IMPORT_BYTES, b' ');
    imported(import_bytes(&raw));
    imported(import_bytes(&gzip(&raw)));
    raw.push(b' ');
    assert_eq!(
        import_bytes(&raw).unwrap_err(),
        "Profile bundle exceeds the 128 MiB input budget"
    );
    let expanded = gzip(&raw);
    assert_eq!(
        import_bytes(&expanded).unwrap_err(),
        "Decompressed profile bundle exceeds the 128 MiB budget"
    );
    imported(ic_system_profiler::import_bundle(&expanded));
}
