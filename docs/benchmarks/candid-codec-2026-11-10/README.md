# Owned Candid codec audit — 2026-11-10

The follow-up [expanded comparison matrix](matrix.md) covers 101 valid cases,
16 skip/admission cases and seven Wasm shapes, with shared correctness/timing
inputs and explicit performance/admission differences.

Source: Platform 0.29.2 (`f687d96`), with the working-tree array admission
change described below. The unrelated Cargo.lock updates to arc-swap 1.9.5 and
crc32fast 1.5.5 were preserved and used in both size arms. Compiler: Rust
1.99.1 (`58cf8f9c3`), Linux x86_64, AMD Threadripper 7970X.

## Coverage and correction

The focused codec selection passes 55 tests. Existing independent upstream
oracles cover scalar/float bits, Unicode, arbitrary integers through 8,192 bits,
records/variants/options, recursive reference subtyping, arity, every truncated
prefix in selected corpora, deterministic mutations, and exact 16/17 nesting
admission including skipped and unused types. Allocation, header, input, type,
work and separate skipping budgets have refusal cases. These are finite corpora,
not fuzzing or a proof of universal Candid compatibility.

Fixed-size arrays previously decoded a complete temporary vector before checking
its length. They now reuse the same vector machinery with an exact-length check
before allocation or element decoding. Correct lengths retain their bytes and
values. Incorrect lengths return `Type` earlier; when several faults coexist,
this can take precedence over a later malformed-element or resource error.
Optional fallback still rolls back and validates discarded elements.

New checks cover zero/correct/short/long array lengths, a huge zero-sized vector,
no reservation or element reads on mismatch, upstream successful-array parity,
the independent dynamic schema/byte oracle, malformed optional fallback and
retained work/allocation/skipping charges after failed optional attempts.

An existing semantic difference is preserved: a wrong-length array requested
through `Option<[T; N]>` becomes `None` under Platform's type-fallback rule;
upstream Serde rejects that length mismatch. Malformed discarded elements and
exhausted budgets still refuse. No new API or dependency is introduced.

## Wasm measurements

All values are uncompressed bytes. Recipe: opt-level 3, fat LTO, one codegen
unit, abort panics, stripped symbols, ic-wasm 0.11.1 shrink. Both arms of each
pair use the same transport, fixture, compiler, root lock input and flags.

The array pair isolates this correction against published 0.29.2 source:

| Array decoder | Raw total | Raw code | Shrunk total | Shrunk code |
|---|---:|---:|---:|---:|
| 0.29.2 | 35,739 | 33,441 | 33,767 | 31,469 |
| Working tree | 35,618 | 33,320 | 33,650 | 31,352 |

The reduction is 117 shrunk total/code bytes. A wrong-length 4,096-element
u8 array additionally avoids its former 4,096-byte temporary value buffer and
all element reads; header allocations remain. This is a logical-storage proof
from the regression test, not an allocator-level peak-heap measurement.

The existing management comparison archives Platform 0.27.1 (`97fe286`) for
its former upstream decoder adapter, using current Candid 0.10.40, versus the
current owned codec. Both decode/re-encode `CanisterStatusResult`. Each source
resolves its own dependency graph from that captured lock; the comparison
includes the migration from upstream Nat's bigint 0.4 to owned bigint 0.5:

| Management round trip | Raw total | Raw code | Shrunk total | Shrunk code |
|---|---:|---:|---:|---:|
| Former upstream adapter | 639,345 | 586,333 | 591,241 | 538,219 |
| Current owned codec | 102,711 | 98,245 | 96,829 | 92,363 |

The current isolated codec saves 494,412 shrunk total bytes (83.6%) and 445,856
code bytes. This is the whole migration difference, not the array fix. The two
decoder budget models and error diagnostics differ. This is neither a
whole-canister result nor an installed instruction measurement.

Retained evidence: target/evidence/candid-removal.jqj6TE,
target/evidence/platform-wasm-size.xOGI5a (array before), and
target/evidence/platform-wasm-size.F8gjnD (array after). They contain source and
fixture checksums, resolved metadata/locks and raw/shrunk Wasms; checksum checks
passed. An earlier management attempt, candid-removal.Mhoht1, failed its source
stability check when this session added a regression during compilation; its
numbers are excluded. The repeated immutable-source run supplies this table.

Reproduce management with `bash scripts/benchmarks/candid-removal.sh`. For the
array pair, archive the 0.29.2 source under evidence, use the same root lockfile
and prepared host tools, then run `scripts/benchmarks/wasm-size.sh` for that
source and the working source with `WASM_BENCH_PROFILES=codec-array-decode` and
`WASM_BENCH_RECIPES=3-fat-1`.

## Native throughput

The existing explicit ignored text/composite experiments run in release mode
with opt-level 3, fat LTO and one codegen unit. Five hot-cache samples alternate
implementation order, validate bytes/values outside timing, black-box results,
and include fresh allocation/drop. Other host workloads were active: these are
observations, not stable performance thresholds or isolated CPU acceptance.
Median nanoseconds per operation:

| Text bytes | Owned encode | Upstream encode | Owned decode | Upstream decode |
|---|---:|---:|---:|---:|
| 0 | 120.1 | 220.0 | 50.5 | 213.2 |
| 32 | 141.3 | 263.8 | 62.1 | 213.0 |
| 4,096 | 231.6 | 402.7 | 172.4 | 317.1 |
| 65,536 | 1,134.8 | 2,624.8 | 2,000.5 | 2,161.3 |

The composite workload is `(Option<Result<Vec<u64>, String>>, (Nat, Int))`.
Owned bigint uses 0.5.1; the upstream oracle uses its own Nat/Int carriers.

| Rows / integer bits | Owned encode | Upstream encode | Owned decode | Upstream decode |
|---|---:|---:|---:|---:|
| 0 / 256 | 683.3 | 1,992.4 | 477.0 | 2,503.0 |
| 16 / 1,024 | 1,172.0 | 2,762.4 | 754.9 | 3,159.6 |
| 1,024 / 8,192 | 4,881.5 | 8,432.1 | 4,764.9 | 9,955.9 |

Raw samples: [text](text.csv), [composite](composite.csv). Implementation 0 is
owned; 1 is upstream. These compare complete codec paths with their distinct
admission budgets, not equal-cost policies. No IC cycles/instructions, application
latency, cryptographic behavior or native macOS acceptance is inferred.

Run `cargo test --locked --offline --release -p ic-platform --lib --features
candid-codec native_text_throughput -- --ignored --nocapture`, replacing the
filter with `native_composite_throughput` for composite samples, with the release
profile environment above. Focused native/Wasm Clippy and decoder/consumer tests
qualify the correction; full release and installed-runtime gates remain
release-owned.

## Header allocation follow-through

The next local slice borrows the function-mode field instead of copying its
zero/one bytes into a temporary vector. Its existing logical allocation charge,
malformed/oneway checks and reader budgets remain unchanged. The encoder also
skips structural merging when registration finds at most one constructed type:
there are no distinct declarations to merge, even with a self-recursive entry.
Registration still validates every declaration first. This removes temporary
signature/classification vectors for the common single-entry case.

Matched comparison against the immediately preceding working tree, including
the array correction, using the same recipe and root lock as above:

| Probe | Before shrunk total/code | After shrunk total/code | Total/code reduction |
|---|---:|---:|---:|
| Text encode | 39,009 / 36,280 | 38,930 / 36,202 | 79 / 78 |
| Text decode | 32,791 / 30,495 | 32,692 / 30,396 | 99 / 99 |
| Composite round trip | 69,160 / 65,513 | 69,099 / 65,453 | 61 / 60 |
| Management round trip | 96,837 / 92,371 | 96,760 / 92,295 | 77 / 76 |

The before fixture is an evidence source archive, not another checkout.
Evidence: target/evidence/codec-header-baseline/source,
target/evidence/platform-wasm-size.XEbakc (before), and
target/evidence/platform-wasm-size.w0XbhJ (after). Both source/fixture checksum
checks pass. Within each row both builds use identical sources apart from this
slice, lock input, profile, compiler and fixture. Cross-run byte differences
from the preceding management table are not attributed to this slice; the
matched pair supplies its actual deltas.

Native text samples repeat the existing experiment, with the same limitations
about concurrent host load. Median ns/op for owned operations:

| Bytes | Encode before | Encode after | Decode before | Decode after |
|---|---:|---:|---:|---:|
| 0 | 123.9 | 38.4 | 50.4 | 73.9 |
| 32 | 138.3 | 61.5 | 59.6 | 65.6 |
| 4,096 | 245.4 | 153.6 | 164.7 | 177.7 |
| 65,536 | 1,131.3 | 1,074.7 | 1,997.9 | 1,994.8 |

Encoding improves in this workload, while small native text decoding is slower.
A repeat measured empty decode at 74.5 ns and 32-byte decode at 67.9 ns, so the
slowdown is not dismissed as a single outlier. The changes are retained for
smaller Wasms and removed temporary allocations, with this native timing tradeoff
explicit; these results establish no instruction/latency claim on the IC.
Raw samples, including upstream controls and min/max information:
[before](header-text-before.csv), [after](header-text-after.csv),
[repeat](header-text-repeat.csv).

The core selection passes 57 tests, and the expanded management/ICRC selection
passes 75, with timing cases excluded. New regressions check all function modes,
malformed unused/oneway types, exact logical allocation boundaries, truncation
and a self-recursive singleton schema used by multiple arguments. Both public
codec tests and both native service tests pass. Native/release/Wasm Clippy,
inventory, boundary, documentation and formatting checks pass. Cryptographic
code, public APIs, schema identity and external bytes are unchanged; no full
release gate or installed IC/native macOS performance acceptance is claimed.
