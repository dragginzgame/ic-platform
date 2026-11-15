//! Decoded input cannot borrow entry-local storage across task suspension.
#![forbid(unsafe_code)]

pub fn check() {
    let local = vec![1_u8];
    let _ = renamed_runtime::entrypoints::update_manual(
        8,
        || Ok::<_, ()>(()),
        |_| Ok::<_, ()>(&local),
        |_input| async {},
    );
}
