# Retired Candid experiments

The reader and codec-packaging prototypes were not adopted. Their active runners,
patches, lockfiles and compiler-formatting selections are retired. Platform uses
registry Candid for required external interfaces.

Historical sources, supplier licenses, pinned inputs and evidence manifests remain
in [the immutable source tree](https://github.com/dragginzgame/ic-platform/tree/41a128751528f8ccd3c926cff550dd8b2ea62926/scripts/benchmarks/0.9).
Reproduction requires that historical checkout and its recorded compiler/caches;
these probes are not current qualification. The identity-preserving codec saved
7,631 bytes in its single-endpoint probe while retaining roughly 3,040 lines of
codec source. Mixed codecs increased Wasm size. See the
[investigation](../../../docs/migration.md#candid-codec-packaging-investigation).
