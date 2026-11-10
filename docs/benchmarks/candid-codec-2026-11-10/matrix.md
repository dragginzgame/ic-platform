# Expanded codec comparison matrix

This development-only catalogue expands the earlier text/composite comparisons.
The same catalogue runs as ordinary correctness tests and as an explicitly
ignored native timing experiment. Production dependencies, codec implementations
and public APIs are unchanged by this test expansion.

## Catalogue and checks

| Valid case family | Cases | Selected boundaries |
|---|---:|---|
| Scalars | 21 | Unit/bool, fixed integer minima/maxima, negative float zero, infinity |
| Text | 14 | ASCII and multi-byte Unicode; 0–65,536 repetitions, up to 393,216 bytes |
| Blobs | 8 | 0, 1, 31, 127, 128, 1,024, 4,096, 65,536 bytes |
| Collections/maps | 20 | Words, booleans, zero-sized elements and ordered maps; 0–1,024 rows |
| Arrays | 3 | Empty, 32 bytes and 16 words |
| Option/result branches | 9 | Absent/present/nested, empty/nonempty success and error |
| Record collections | 4 | Tuples with numeric labels and optional Unicode text; 0–128 rows |
| Arbitrary integers | 18 | Nat and both Int signs, 0–8,192 magnitude bits |
| Principals | 4 | 0, 1, 28 and 29 bytes; owned and registry identities stay separate |

Each of the **101 valid cases** checks exact independently produced bytes,
cross-decoding in both directions, an exact encoding byte ceiling and a
one-byte-short ceiling, selected truncated prefixes, trailing-data rejection,
input-size refusal and zero-work refusal. These supplement the existing full
truncation, mutation, recursive schema and 16/17 depth tests; they do not replace
them. Floats with NaN payloads retain their separate bitwise tests.

The **16 discard/admission cases** comprise four valid skipped argument sequences,
11 mutually rejected frames, and one deliberately different admission result.
Refusals cover magic, bool/option tags, UTF-8, missing values, invalid references,
field order, function mode, variant index, trailing bytes and overflowing lengths.

The selected upstream Candid 0.10.40 accepts an unused oneway function type with
a nonempty result list. Platform rejects it. The test freezes both observations
and labels its timing `admission-difference`, rather than treating acceptance
and rejection as equivalent work or weakening Platform's check. Other known
differences, including optional array-length fallback, retain their earlier
regressions. This finite catalogue is neither fuzzing nor complete conformance.

## Native measurements

[Raw samples](matrix-native.csv) contain 2,180 measurements: five samples for
both implementations, encode/decode for each valid case, plus skip/reject work
for each discard case. Implementation 0 is owned; 1 is upstream.

Both implementations validate expected bytes/values/admission before timing.
Inputs and results pass through `black_box`; order alternates between samples.
Each operation allocates/drops its own result. Iteration counts scale with wire
length (16–1,000 per sample). Decode configuration is reused outside timing.
The unsigned/signed bigint pairs use owned bigint 0.5.1 versus upstream's 0.4
carriers with matching magnitudes and wire bytes.

Rust 1.99.1, opt-level 3, fat LTO, one codegen unit; Linux x86_64 on Threadripper
7970X. Root lock SHA-256:
`34e8d2d9bac796de933532954919229960ab722876205aa7356fc79ca82ed90c`.
Concurrent host workloads remained active; these hot-cache samples are diagnostic,
not stable thresholds or IC instruction/latency measurements. Admission budgets
and quota units differ; matching numeric settings do not equalize policies.

Representative median nanoseconds per operation:

| Case | Owned encode | Upstream encode | Owned decode | Upstream decode |
|---|---:|---:|---:|---:|
| 4,096-byte blob | 2,046.5 | 534.3 | 6,158.1 | 8,398.7 |
| 1,024 words | 995.2 | 699.5 | 1,385.3 | 2,379.0 |
| 128 map entries | 1,009.3 | 1,413.6 | 6,371.8 | 9,841.4 |
| 128 records | 1,083.1 | 1,670.9 | 6,730.0 | 16,215.0 |
| 8,192-bit Nat | 2,024.6 | 1,318.3 | 1,499.1 | 3,033.9 |
| Negative 8,192-bit Int | 2,203.1 | 6,079.1 | 1,559.5 | 3,805.3 |
| Present successful option/result | 253.1 | 922.0 | 178.9 | 971.9 |
| 29-byte Principal | 63.6 | 278.6 | 48.6 | 266.9 |

The expanded matrix exposes cases hidden by text-only results: owned blob,
word-vector and large-Nat encoding are slower in these samples. This expansion
records those observations; it does not change encoding or claim a correction.
The decoded values and bytes pass independently in all these cases.

## Wasm shape matrix

The existing source-archive comparison now accepts named profiles through
`CANDID_COMPARISON_PROFILES`; its default remains management status. It compares
the former Platform 0.27.1 upstream adapter (`97fe286`) with the current owned
codec using matched raw input/reply framing, compiler and optimization flags.
Each arm resolves its own graph from the same root lock input; bigint and DTO
identity migrations are part of this complete implementation comparison.

Uncompressed bytes after ic-wasm 0.11.1 shrink; both use Rust 1.99.1, opt-level 3,
fat LTO, one codegen unit, abort panics and stripped symbols:

| Shape | Upstream total / code | Owned total / code | Total reduction |
|---|---:|---:|---:|
| Management status | 591,239 / 538,217 | 96,752 / 92,287 | 83.6% |
| Blob `Vec<u8>` | 359,536 / 330,102 | 53,602 / 50,708 | 85.1% |
| Words `Vec<u64>` | 359,321 / 329,885 | 53,632 / 50,738 | 85.1% |
| Option/result/vector | 418,385 / 388,302 | 59,482 / 56,404 | 85.8% |
| Map | 252,309 / 230,585 | 69,418 / 65,574 | 72.5% |
| Record collection | 421,934 / 392,002 | 60,621 / 57,528 | 85.6% |
| Nat/Int pair | 471,584 / 441,078 | 63,383 / 59,965 | 86.6% |

[Full raw and shrunk section measurements](matrix-wasm.tsv) retain both totals
and code/custom section sizes. Artifact/source hashes, generated probes, resolved
locks/metadata, Clippy logs and Wasms are at
target/evidence/candid-removal.4eg2MY. All 14 build/Clippy selections and their
source checksum checks passed. This is a reachable-code footprint comparison,
not installed execution or a whole-application saving. It does not imply removal
of Candid from separately selected upstream signature or native Agent graphs.

The native catalogue and Wasm probes share named shape families, but the Wasm
probes take values dynamically from ingress rather than embedding the catalogue's
literal values. No installed execution of these seven Wasm probes is claimed.

## Refreshed complete-removal size check

After signature composition removed the remaining production Wasm Candid edge,
the management comparison was rerun against the current working source with
the same `97fe286` baseline and recipe. Both Clippy/build selections and source
stability checks passed. This refresh includes the independently replaced Types
implementation; it is not an isolated measurement of one dependency edge.

| Measurement (uncompressed bytes) | Former upstream adapter | Current owned codec | Saved |
|---|---:|---:|---:|
| Complete optimized module | 591,238 | 96,743 | 494,495 (83.6%) |
| Optimized code-section payload | 538,216 | 92,278 | 445,938 (82.9%) |

Fresh [raw size rows](removal-refresh-wasm.tsv),
[input/artifact hashes](removal-refresh-inputs.sha256), and
[validation outputs](removal-refresh-validation.txt) retain this result.
Complete generated sources, locks, metadata and Wasms remain reclaimable under
`target/evidence/candid-removal.EXDO1s/`. The other six shape rows above retain
their earlier measurement identity and were not rerun for this question.

The [signature comparison](../canister-signatures-2026-11-10.md#wasm-footprint)
separately measures the last dependency removal. A dependency in Cargo's graph
does not mean its entire implementation survives linking. In particular, the
raw upstream signature implementation's Candid edge does not produce the same
footprint as invoking Candid decoding. There is no single library-wide saving,
and the codec and signature deltas must not be added to predict a consumer's
Wasm: shared retained code and selected entrypoints determine that artifact.

## Reproduction and validation

Correctness runs in the ordinary `candid-codec` unit selection. Expanded
management/ICRC selection passes 77 tests; three timing experiments remain ignored.
Native/release Clippy and source/dependency boundary checks pass. No full release
qualification or macOS/provider acceptance was run for this test-only expansion.

```sh
cargo test --locked --offline -p ic-platform --lib --features candid-codec comparison_matrix

CARGO_PROFILE_RELEASE_OPT_LEVEL=3 CARGO_PROFILE_RELEASE_LTO=fat \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
cargo test --locked --offline --release -p ic-platform --lib \
  --features candid-codec comparison_matrix_native_throughput -- --ignored --nocapture

CANDID_COMPARISON_PROFILES='management blob words options map records bigints' \
  bash scripts/benchmarks/candid-removal.sh
```

`sizes.tsv` now includes a `profile` column after `arm`. The compatibility default
still emits only management rows. Logs: /tmp/platform-codec-matrix-{clippy,tests}.log,
/tmp/platform-codec-matrix-all-tests.log, /tmp/platform-codec-matrix-throughput.log,
/tmp/platform-codec-matrix-wasm-final.log and /tmp/platform-codec-matrix-checks.log.
