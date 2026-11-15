//! Matched standalone decoder probe; not an IC canister or deployment artifact.

#[cfg(any(
    all(feature = "candidate", feature = "published"),
    not(any(feature = "candidate", feature = "published"))
))]
compile_error!("select exactly one of candidate or published");

#[cfg(feature = "published")]
use ic_platform::{candid_codec::decoder::Limits, cloud_engine_types as engine};
#[cfg(feature = "candidate")]
use ic_platform_protocol::{candid::Limits, cloud_engine as engine};

fn main() {
    // Independent literal: one view with id="x" and an empty node list.
    // black_box prevents the compiler from specializing the decoder to this input.
    let bytes = std::hint::black_box(
        b"DIDL\x03\x6d\x01\x6c\x02\xdb\xb7\x01\x71\xbb\xf8\xfd\xed\x0f\x02\x6d\x68\x01\x00\x01\x01x\x00".as_slice(),
    );
    let limits = Limits {
        bytes: 512 * 1024,
        header: 64 * 1024,
        types: 256,
        work: 2_000_000,
        allocation: 4 * 1024 * 1024,
        #[cfg(feature = "candidate")]
        vector: 1024,
        #[cfg(feature = "candidate")]
        skipping: 128 * 1024,
    };
    #[cfg(feature = "candidate")]
    let views = engine::decode_get_engine_views_response(bytes, limits).unwrap();
    #[cfg(feature = "published")]
    let views = engine::decode_get_engine_views_response_with_skip_limit(bytes, limits, 128 * 1024)
        .unwrap();
    assert_eq!(views.len(), 1);
    let view = &views[0];
    assert_eq!(view.id, "x");
    assert!(view.node_ids.is_empty());
    assert!(view.subnet_id.is_none());
    assert!(view.engine_operator_id.is_none());
    assert!(view.deleted_at.is_none());
    assert!(view.subnet.is_none());
    std::hint::black_box(views);
}
