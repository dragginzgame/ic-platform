# EngineView native heap comparison

## Current hard-cut execution

The executable legacy decoder has been removed. Independent upstream Candid
schemas construct the same workload shapes; current decoding must fit the
retained Linux x86-64 total and peak bounds below (616, 1058, 9008, 425, 425
bytes). Cleanup, refusal classes, and discarded-text allocation invariance
remain checked. Historical paired results below are retained evidence, not
a claim that the old implementation still runs. Other-host baseline qualification
remains separate.

## Historical criteria selected before measurement

Run `make profile-engine-response` on a native host. The private
`qualification/engine-view-profile` package owns a single serial test
process using dhat 0.3.3's allocator. Product packages and the portable consumer
do not select that allocator or dependency. The new package name was absent
from crates.io at selection on 2026-11-15, so its unpublished version is 0.1.0.

Compare the published Platform 0.33.7 decoder and candidate Protocol on identical
frames under the existing Query budgets. Require equal selected values before
profiling. Construct input and compare Serde outside the measured region; measure
one decode, observe retained output, drop it and require zero live allocations.
Each measurement starts a fresh profiler. Use the release profile for both.

Workloads are an empty full-schema vector; one full record with four nodes per
list and one-byte selected text; one full record with 128 distinct 29-byte nodes
per list and 256-byte selected text; and minimal independent records with one
byte or 64 KiB of discarded text. Published DTOs encode the full-schema frames;
a qualification-owned declaration encodes the discarded-text frames. These are
different frames from the logical-budget corpus and its numbers must not be
substituted here.

For every workload, candidate total requested allocation bytes and peak live
bytes must be no greater than published (zero regression tolerance). For both
decoders, growing only discarded text must leave total bytes, peak bytes,
allocation count and retained bytes unchanged. Fail on a violation; investigate
without relaxing the criteria to fit the result.

This measures allocator requests, including reallocations as accounted by dhat,
not RSS, allocator overhead, stack use or execution time. Profiler bookkeeping is
excluded by dhat. No product allocator or unsafe implementation is introduced.
This focused corpus alone does not close general allocation, instruction count,
Wasm size, macOS, or real Query adoption qualification.

## Observed result — 2026-11-15

All criteria passed on Linux x86-64 with Rust 1.100.0
(`0ef1bc386059596dada8fb65b50425aab2ea637a`, LLVM 23.1.5), the default
release profile, registry Platform 0.33.7 and working Protocol 0.1.0.
Both resolve registry Types 0.20.4. Total and peak bytes happened to be equal
for every measurement; both columns are still asserted independently.

| Workload | Frame bytes | Published total/peak bytes | Candidate total/peak bytes | Published allocations | Candidate allocations | Retained bytes, both |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Empty | 82 | 616 | 544 | 4 | 3 | 0 |
| Small | 361 | 1,058 | 986 | 9 | 8 | 442 |
| Largest lists | 8,563 | 9,008 | 8,936 | 9 | 8 | 8,392 |
| Discarded 1 byte | 31 | 425 | 393 | 5 | 4 | 201 |
| Discarded 64 KiB | 65,568 | 425 | 393 | 5 | 4 | 201 |

Every output released all measured allocations. The candidate performs one
fewer allocation in each selected case, saving 72 requested bytes for the full
schema and 32 for the minimal schema. No criterion was relaxed. Native consumer
tests, Clippy with warnings denied, portable consumer Wasm compilation, and
format checks passed. Locked graph inspection confirms dhat is absent from
the product response and Wasm consumer graphs. macOS remains unmeasured.

Inputs at observation:

- Product lock SHA-256: `bc82b53beb5a552bd71eaec7fb07f2062805485e732b5738da236c9633a0df81`.
- Qualification lock SHA-256: `e67d5bc6c2405a4109f47694828c42c8a98c0a77a070d0ca3c5ae591533f1856`.
- Product/profiler file-inventory SHA-256: `3aeb07fecd9fffe5b3bfd42edca57ed231806a643a460f5fae77e7391972c85c`.
  Reproduce this source identity from the root with
  `rg --files crates/ic-platform-protocol qualification/engine-view-profile | sort | xargs sha256sum | sha256sum`.
  It identifies working manifests and authored source/tests, not a commit.

The profiler is an external development dependency, not imported implementation.
Its registry crate checksum is
`98cd11d84628e233de0ce467de10b8633f4ddaecafadefc86e13b84b8739b827`;
Cargo authenticates it through the qualification lock. No downloaded dependency
source is copied into this repository.

## Refusal cleanup — 2026-11-15

The same serial profiler test now exercises six failure paths after allocation
has begun. Each must return the expected typed error, have performed at least
one allocation, and leave zero live blocks/bytes at return. This is a cleanup
criterion, not a new relative performance threshold. Fixture construction and
printing remain outside the measured region.

| Workload / refusal | Expected error | Published requested bytes | Candidate requested bytes | Live bytes at return, both |
| --- | --- | ---: | ---: | ---: |
| Small / last byte removed | Malformed | 1,058 | 986 | 0 |
| Small / byte appended | Trailing | 1,058 | 986 | 0 |
| Largest lists / last byte removed | Malformed | 9,008 | 8,936 | 0 |
| Largest lists / byte appended | Trailing | 9,008 | 8,936 | 0 |
| Largest lists / work cap 1,000 | Limit | 5,168 | 5,096 | 0 |
| Largest lists / allocation cap 2,000 | Limit | 1,328 | 1,256 | 0 |

All checks passed under the same Linux/compiler/release profile as the successful
corpus; its five successful cases still pass. Clippy with warnings denied passes.
These are ordinary typed error returns, not panic/trap recovery or injected host
allocator failure. No production source changed. Reproduce with
`make profile-engine-response`.

Refusal-run qualification lock SHA-256:
`d3ac9513352fb1abbca5c9b4c00e91cce4fb7994c1a908d1ebceb86c73e5870d`.
Product/profiler inventory SHA-256, using the same inventory command above:
`271088254392ab36ebc50755ec909b97a510691946dc52ea984839d66ae1d6a5`.
The product lock and compiler are unchanged from the earlier observation.

Toko Tooling 0.44.7 adoption moved the qualification package directly under
`qualification/`. Historical inventory hashes above remain bound to their
original source revision and directory layout; current reproduction paths are updated.
