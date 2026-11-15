//! A decoder must accept raw bytes, not an unrelated typed argument.
#![forbid(unsafe_code)]

fn decode(_: u64) -> Result<Vec<u8>, ()> {
    Ok(vec![])
}

pub fn check() {
    let _ = renamed_runtime::entrypoints::update_manual(
        8,
        || Ok::<_, ()>(()),
        decode,
        |input| async move { drop(input) },
    );
}
