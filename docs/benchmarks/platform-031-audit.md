# Platform 0.31 follow-up optimization audit

Measured on 2026-11-11 against `fc55ae1` (published Platform 0.31.0 code).
These are isolated probes, not savings predictions for a complete application.
The pending unused `leb128` dependency cleanup was preserved separately.

## Changes retained

- `Encode::write_slice` defaults to the existing element mapping. Only `u8`
  overrides it with a safe bulk write. Vectors delegate to slices; arrays and
  custom mappings retain their framing, bytes and output ceilings.
- Big integers reuse their radix buffer for continuation bits and append it once.
  Positive signed integers no longer copy their magnitude. Numeric ranges,
  signed boundaries and external bytes are unchanged.
- Schema refinement reuses buffers across rounds; aliases retain final wire
  indices instead of performing a second table search per reference. Recursive
  graph handling, declaration order and structurally merged headers are unchanged.
- Signature verification accepts an existing caller-owned `PreparedPublicKey`
  through `verify_canister_sig_with_prepared_root` and
  `verify_canister_sig_with_time_and_prepared_root`. Both share the existing
  authentication path. Root/subnet signatures, key admission, range authorization,
  witnesses, delegation refusal and signing-time checks still run. There is no
  production trust, certificate or freshness cache. Signing-message framing now
  uses a 46-byte stack buffer instead of a vector.
- Twenty-two management declarations use the declaration macros instead of a
  separately maintained mapping. Only already-compatible field/variant order was
  migrated; the other 27 mapping invocations remain separate. Serde attributes,
  sequence bytes and derived ordering stay unchanged. This removes duplicated
  declarations, not generated validation or public types.

## Matched Wasm sizes

Both arms use Rust 1.99.1, opt-level 3, fat LTO, one codegen unit, abort panics,
stripped symbols and `ic-wasm` 0.11.1 shrink. Identical probe sources, feature
selections and optimization settings select archived baseline/current Platform
sources. Registry inputs come from the same lock input. There are no new
production dependencies. Sizes are uncompressed bytes; code means section payload.

| Probe | Before total / code | After total / code | Total change |
| --- | ---: | ---: | ---: |
| Blob | 53,602 / 50,708 | 53,413 / 50,501 | -189 |
| Words | 53,632 / 50,738 | 53,439 / 50,527 | -193 |
| Map | 69,418 / 65,574 | 69,283 / 65,421 | -135 |
| Records | 60,621 / 57,528 | 60,465 / 57,354 | -156 |
| Nat/Int pair | 63,383 / 59,965 | 63,540 / 60,113 | +157 |
| Management status | 96,743 / 92,278 | 96,623 / 92,149 | -120 |

## Installed codec instructions

Separately instrumented artifacts measure one decode/encode round trip, excluding
ingress copying and reply construction. The host supplies upstream-generated
canonical Candid and checks the complete returned payload against that request.
Four samples per workload produce identical instruction counts. Management has a
size comparison only. Testkit 0.92 and the explicitly prepared PocketIC 16.1 server
are development infrastructure; this does not qualify a newer Testkit release.

| Workload | Before | After | Change |
| --- | ---: | ---: | ---: |
| 4,096-byte blob | 688,240 | 538,753 | -21.7% |
| 1,024 words | 305,558 | 308,647 | +1.01% |
| 128 map entries | 448,474 | 448,752 | +0.06% |
| 128 records | 411,106 | 410,737 | -0.09% |
| 8,192-bit Nat/negative Int pair | 679,924 | 600,120 | -11.7% |

The integer throughput gain is retained despite its small Wasm increase. These
measurements do not establish a universal improvement for every shape.

## Prepared-root certificate verification

The same current Platform implementation and fixture bytes compare ordinary and
prepared-root paths. The fixture owns the prepared key; its first-use preparation
occurs inside the cold instruction window. Each workload gets a fresh canister.
The three warm samples agree. Every response checks expected acceptance, including
wrong messages and identity roots. These figures measure complete verification,
not just a BLS primitive.

| Workload | Ordinary warm | Prepared warm | Change |
| --- | ---: | ---: | ---: |
| Direct certificate | 211,621,562 | 163,971,407 | -22.5% |
| Delegated, legacy ranges | 423,205,358 | 375,556,037 | -11.3% |
| Delegated, sharded ranges | 423,230,875 | 375,742,001 | -11.2% |

Direct cold verification is 225,220,291 versus 225,222,223 instructions: preparation
has no first-use speed advantage. The prepared fixture is 484,363 total / 446,923
code bytes, versus ordinary 483,610 / 446,285 (+753 total / +638 code). This includes
fixture cache plumbing and is not a production API-only size delta. Retaining a
prepared key also retains supplier pairing data; its heap cost was not measured.
Consumers choose whether repeated verification justifies that memory.

## Native diagnostics and rejected experiment

The existing 101-case valid / 16-case discard-admission matrix passed before and
after. Median encoding times in nanoseconds per operation:

| Case | Before | Final |
| --- | ---: | ---: |
| 4,096-byte blob | 2,230.9 | 144.1 |
| 8,192-bit Nat | 2,078.1 | 1,176.1 |
| Positive 8,192-bit Int | 2,138.7 | 1,158.7 |
| Negative 8,192-bit Int | 2,371.4 | 1,369.7 |
| 1,024 words | 1,131.9 | 1,148.1 |
| 128 map entries | 1,005.4 | 1,171.8 |
| 128 records | 1,190.2 | 1,197.7 |

Native runs use the same compiler/release flags as the earlier matrix. Other
host workloads were active; these diagnostic timings are not stable thresholds.
For example, unchanged map decoding varied from 6,700 to 7,451 ns. Installed
instruction counts above provide the deterministic execution comparison.

A direct-to-BTreeMap decoder removed temporary vector storage but made the
128-entry native decode 11,463 ns versus 6,700 ns (about 71% slower). Its map
Wasm was 63,673 bytes versus 69,418 baseline. It was rejected and fully removed:
the existing bulk conversion and its logical allocation/work charges remain.
No new array decoder or generic caching API was added.

## Evidence and reproduction

Retained local evidence:

- `target/evidence/audit-031/`: baseline, experiment and final native logs;
  focused tests, Clippy and graph checks.
- `target/evidence/candid-removal.cEY6Za/`: final size/instruction rows, source
  archives, resolved metadata/locks, original and instrumented Wasms, checksums,
  generated drivers and successful Clippy/source-stability logs.
- `target/evidence/candid-removal.UGE6Oz/`: rejected map experiment size comparison.
- `target/evidence/signature-instructions.QFs9Wp/`: ordinary/prepared installed
  signature evidence, Wasms, graphs, checksums and driver logs.

The first driver invocations could not bind localhost under sandbox restrictions.
Only the already-built drivers were rerun with local-server access; their final
instruction logs contain all passing cases. No hosted/native macOS or fleet
release qualification is claimed.

Reproduce with an explicitly prepared `POCKET_IC_BIN` or Testkit server URL:

```sh
CANDID_BASELINE_CODEC=owned CANDID_MEASURE_INSTRUCTIONS=1 \
CANDID_COMPARISON_PROFILES='blob words map records bigints management' \
bash scripts/benchmarks/candid-removal.sh fc55ae1

SIGNATURE_BENCH_VARIANTS='owned owned-prepared' \
bash scripts/benchmarks/canister-signatures-instructions.sh
```

Affected-package Clippy and 109 focused unit/integration tests pass, with three
timing tests intentionally ignored in the ordinary test run. The timing matrix
was run explicitly. Codec, signature and management Wasm consumers pass Clippy;
normal/build feature graphs exclude upstream Candid, Principal and Agent. The
signature-only graph also excludes IC0, Runtime and retired verifier composition.
Schema/byte/Serde comparisons, recursive types, malformed input, limits and the
16/17 nesting boundary remain covered.
