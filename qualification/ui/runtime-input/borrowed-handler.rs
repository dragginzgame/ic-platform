//! Handler construction cannot retain a borrow of entry-local state.
#![forbid(unsafe_code)]

pub fn check() {
    let local = vec![1_u8];
    let _ = renamed_runtime::entrypoints::update_manual(
        8,
        || Ok::<_, ()>(()),
        |bytes| Ok::<_, ()>(bytes.to_vec()),
        |_input| {
            assert_eq!(local.len(), 1);
            async {}
        },
    );
}
