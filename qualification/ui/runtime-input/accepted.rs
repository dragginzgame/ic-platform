//! Owned update inputs and explicit codecs are accepted under a renamed import.
#![forbid(unsafe_code)]

pub fn check() {
    let _ = renamed_runtime::entrypoints::update_manual(
        8,
        || Ok::<_, ()>(()),
        |bytes| Ok::<_, ()>(bytes.to_vec()),
        |input| async move { drop(input) },
    );
}
