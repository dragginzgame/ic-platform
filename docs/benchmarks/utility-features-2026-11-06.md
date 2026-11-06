# Utility dependency features — 2026-11-06

Compared committed baseline `244e08570de164f814743dfdacb1fcce13b8c23f`
with the working-tree utility selection. Both arms use the same current lock,
including the maintainer's Candid 0.10.40 and cc 1.11.1 draft. Its SHA-256 is
`18f3ea369b2d85b9ba2c1a83cbf5e22ce10c501a51efd9f18f4ee0a1d479552a`.
The draft was neither altered nor included in delivery.

Keep `default-features = false, features = ["alloc"]` for `data-encoding` and
`serde_bytes`. Standalone Types' normal Wasm graph loses `data-encoding/std`,
`serde_bytes/std` and the latter's `serde_core/std` edge. Package counts and
public Principal/Account behavior remain unchanged. External Principal, Candid
and CBOR suppliers can still re-enable standard-library features when selected.
This is narrower dependency selection, not a `no_std` promise for Platform.

Also tried disabling defaults for hex, num-bigint and num-traits. The ICRC graph
still enables their defaults through Candid, so those edits were discarded.
No hash, cryptographic implementation, resource policy or upstream source changed.

Same probe sources, Rust 1.99.1, Wasm target, `opt-level=3`, fat LTO, one codegen
unit, aborting panics and symbol stripping; optimiser: ic-wasm 0.11.1 `shrink`.
Both total uncompressed and code-section sizes are shown after shrinking:

| Probe | Module before → after | Code before → after |
| --- | ---: | ---: |
| Principal text/JSON | 64,553 → 64,549 B | 38,351 → 38,351 B |
| Binary Principal | 13,987 → 13,987 B | 12,589 → 12,589 B |
| Runtime | 39,088 → 39,088 B | 33,123 → 33,123 B |
| External Candid | 609,098 → 609,092 B | 556,114 → 556,108 B |

The linked-Wasm benefit is negligible; keep only the two simpler feature
selections. This does not remove any upstream packages or establish application
performance gains. Raw module/code deltas respectively were -4/0, 0/0, 0/0 and
-6/-6 bytes in the same row order.

Focused native Clippy passes for the affected Types/Runtime/facade selections.
Types' fourteen tests and Platform's forty selected unit tests plus five ICRC
consumer tests pass, covering identity formats, Serde, ledger bytes/hashes and
numeric conversions. Actual standalone graphs are captured separately from
test graphs, whose development dependencies can re-enable `std`.

Raw evidence: `target/evidence/utility-features/` and
`target/evidence/wasm-regression.2b8vgO/`. No full release gate, sibling migration
or package publication was performed for this experiment.
