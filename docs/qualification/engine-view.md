# EngineView resource qualification

## Admission criteria selected before measurement

These are resource-admission thresholds for the current selected Query workload,
not timing, instruction-count or physical-heap performance targets. Query's
reviewed caller supplies 512 KiB input, 64 KiB header, 256 types, 2,000,000 work
units, 4 MiB logical allocation and 128 KiB skipped work. The new decoder also
uses an explicit 1,024-element per-vector cap in this qualification; Query's
separate 128-node and single-result admission remains consumer policy.

The following fixed workloads must decode under all those allowances with exact
selected values and no old/new admission disagreement:

| Workload | Shape |
| --- | --- |
| Empty result | One empty result vector with the full independently declared schema |
| Small selected result | One engine, four nodes in each list, one byte of unknown text |
| Largest selected lists | One engine, 128 distinct 29-byte principals in each list, 256-byte ID and configured-version text |
| Large skipped text | Small selected result with 64 KiB unknown text |

For each accepted frame, find the smallest successful work, allocation and skip
allowance by monotonic bisection, keeping the other Query allowances unchanged.
The exact allowance must accept and one less must refuse. The table records
logical counters separately for each implementation; their units/layouts are
not assumed equivalent. Increasing only skipped text from one byte to 64 KiB
must increase neither decoder's logical allocation charge (zero-byte tolerance).
This proves discarded text is not retained by their accounting, not a bound on
actual allocator traffic. Both work and skip ceilings remain 2,000,000 and
131,072; allocation remains 4,194,304.

Malformed UTF-8 at the end of the 64 KiB unknown text must refuse under those
same budgets; it cannot become a valid projection. A skipped zero-sized vector
of 1,000,000 values must stop at the candidate's explicit vector bound with a
Limit error. The accepted comparison corpus still requires zero mismatched
selected values. Exceeding a criterion requires investigation; do not raise it
from the measured result merely to pass.

This evidence does not close the design's instruction-count, actual allocation,
Wasm-size, native-host or real Query adoption gates. Numerical thresholds for
those separate measurements still need a baseline and launch requirement.

## Observed result — 2026-11-15

All criteria above passed on Linux x86-64 using Rust 1.100.0
(`0ef1bc386059596dada8fb65b50425aab2ea637a`, LLVM 23.1.5), the default test
profile, Protocol 0.1.0 working source and registry Platform 0.33.7. Both use
registry Types 0.20.4. No optimization settings or supplier locks were changed
between the two decoders in the comparison process.

| Workload | Frame bytes | Published work | Candidate work | Published logical allocation | Candidate logical allocation | Published skip | Candidate skip |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Empty | 86 | 175 | 184 | 688 | 608 | 0 | 0 |
| Small | 368 | 497 | 511 | 1,130 | 1,050 | 7 | 5 |
| Largest lists | 8,570 | 9,195 | 9,209 | 9,080 | 9,000 | 7 | 5 |
| Skipped 64 KiB | 65,905 | 66,034 | 66,048 | 1,130 | 1,050 | 65,544 | 65,542 |

The candidate uses more work units in these cases and fewer logical allocation
units. Different traversal charges and type-table representations contribute to
these differences; they do not demonstrate a runtime speedup, physical memory
saving or regression. Every observed value is below its preselected Query
ceiling. Both implementations retain the same logical allocation charge when
only the discarded text grows. The malformed-tail and explicit vector-bound
checks also pass. No criterion was relaxed after observing the results.

Reproduce with `make measure-engine-response` at the repository root. The
implementation is in the private consumer's `tests/comparison/resources.rs`.
The latest local raw output is regenerable under
`target/evidence/engine-view-resources.log` and is not maintained evidence.

Inputs at observation:

- Product lock SHA-256: `bc82b53beb5a552bd71eaec7fb07f2062805485e732b5738da236c9633a0df81`.
- Qualification lock SHA-256: `8591fe8033fa8bebfc418a579bf1cd21d4aede13ca59ae22493b23669c2d2b2c`.
- Product/consumer file-inventory SHA-256: `b95ffe262039ae1c62a6cd48b05882b2b90ff30c74ccc7d10e18179ce910bfd8`.
  This is the digest of sorted `sha256sum` lines for `rg --files
  crates/ic-platform-protocol qualification/engine-view-consumer` from the root;
  it includes their manifests and authored tests, excludes target output, and
  identifies uncommitted working bytes rather than inventing a commit identity.
