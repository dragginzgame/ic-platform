# Binary Candid codec investigation

This investigation measures the cost of reachable binary codecs before replacing
the general supplier. The initial stages below were isolated prototypes; the
subsequent adoption stage supplies the owned codec and migrates Registry DTOs.
Wire and Storage were inspected read-only; their benchmark suites were not executed.

## First size baseline — 2026-11-09

Run the existing size harness with isolated feature selections:

```sh
WASM_BENCH_PROFILES='codec-empty codec-unused codec-text-owned codec-text-upstream codec-text-decode' \
WASM_BENCH_RECIPES=3-fat-1 bash scripts/benchmarks/wasm-size.sh
```

Rust 1.99.1, optimization 3, fat LTO, one codegen unit, abort panics, stripped
symbols, IC-Wasm 0.11.1 shrink. All rows use the same source, compiler, lock and
IC0 input/reply helpers. Each capability is compiled into a separate Wasm.
This run deliberately uses the current deferred lock selecting Candid 0.10.40;
it is not evidence for the published 0.27.1 lock selecting 0.10.39.

| Reachable operation | Original total bytes | Original code bytes | Optimized total bytes | Optimized code bytes |
| --- | ---: | ---: | ---: | ---: |
| Raw echo, no Candid | 14,836 | 13,449 | 13,779 | 12,392 |
| Same echo, Candid selected but unused | 14,836 | 13,449 | 13,779 | 12,392 |
| Owned single-text encoder | 20,852 | 18,759 | 19,253 | 17,160 |
| Upstream single-text encoder | 137,578 | 123,694 | 126,827 | 112,921 |
| Platform bounded upstream text decoder | 338,292 | 312,359 | 312,396 | 286,407 |

The matched text encoder difference is 107,574 total / 95,761 code bytes.
The decoder adds 298,617 total / 274,015 code bytes over echo. The latter includes
Platform admission, error handling and String construction: it is not an
algorithm-only attribution or a measured replacement saving. No custom derived
type is involved. The dependency-only control proves selection alone adds no
retained code in this fixture. No parser is selected.

Raw artifacts, effective flags, copied fixture, resolved metadata/lock, input
hashes and both original/optimized Wasms are retained under
`target/evidence/platform-wasm-size.Xg7CRH`. The harness verifies source and
fixture hashes again after building. These are footprint measurements, not
throughput, allocation or installed instruction measurements.

## Generic encoder prototype

The qualification-only implementation is in `scripts/fixtures/wasm-size/codec`.
Its `Encode` trait describes a static schema separately from value writing.
One table builder handles primitives, options, vectors, records, variants,
recursive references, function signatures and services. Typed implementations
currently cover fixed-width scalars, bool/unit, text, Option, Vec and pairs.
It writes directly from the input value; no intermediate value tree is built.

`codec-generic` exercises its text encoder with the same input/reply framing as
`codec-text-upstream`. At the same 3-fat-1/shrink settings, the first prototype
measured **20,175 total / 18,012 code bytes**, against upstream's
**126,827 total / 112,921 code bytes**: 106,652 / 94,909 fewer bytes. These are
single-text encoder results, not savings for a general decoder or an application.
First prototype evidence: `target/evidence/platform-wasm-size.clgiYt`.
The expanded seven-test corpus and repeat size run passed under
`target/evidence/platform-wasm-size.kYO7ds`, reproducing both size rows exactly.
Native all-target Clippy with warnings denied, Rustfmt, ShellCheck and local
documentation links passed. No installed instruction or throughput result is
claimed yet. The encoder is 268 physical source lines, plus 163 test lines;
these counts exclude the shared harness and are not a full codec estimate.

The prototype remains outside the public package and production features. It
does not yet implement full Rust mappings, arbitrary argument lists or a derive
macro. Schema registration currently has an experimental 256-node
cap; it is not the final caller-budget contract. Schemas and Encode implementations
are trusted Rust code. This is not suitable for admitting attacker-defined
schemas. In particular, table coverage does not establish value compatibility
for every function/service/recursive schema.

The upstream differential suite checks fixed scalar bytes (including float bit
patterns), UTF-8/length boundaries, nested containers, recursive schema
termination, reference framing and output limits. A deterministic nested corpus
adds empty through 1,024-row workloads. The decoder milestone below adds bounded
typed reading; complete compatibility qualification is still required.

Run a fresh `codec-generic`/`codec-text-upstream` size selection, then Clippy and
tests against its emitted manifest with features
`codec-generic,ic-platform/candid`. The extra Candid feature supplies the native
reference tests only; the generic Wasm row does not select it.

## Typed decoder milestone

The decoder in `codec/decoder` parses compound type tables separately from value
reading. It validates references, field ordering, function annotations, service
entries and unused schemas. Scalar/text/Vec/Option/pair mappings decode directly
into Rust values. Extra record fields are skipped with validation; missing
optional/null fields and optional fallback/promotion are covered by differential
tests. Type/value traversal, including skipped and unused paths, refuses level
17 while admitting level 16. Recursive schemas are allowed with finite values.

Caller budgets cover input/header bytes, table entries, work and charged
allocation. Invalid UTF-8, bool tags, references, overflowing framing integers,
truncation, trailing bytes and zero-sized collection expansion are tested.
The current prototype requires exactly one argument. Its work units and
allocation policy are experimental and do not establish parity with existing
Platform decoding/skipping quotas. It must not replace that policy yet.

The upstream differential tests caught nested optional promotion: the selected
upstream decoder reads a scalar as `Some(Some(value))` for a nested Option target.
The prototype now preserves that observed behaviour rather than assuming the
type-checker's optional-promotion restriction describes all decoder behaviour.
Tuple mappings also retain upstream's tuple-layout admission.

Final evidence: `target/evidence/platform-wasm-size.drCxUC`. Under the same
Rust 1.99.1 / 3-fat-1 / IC-Wasm 0.11.1 shrink recipe:

Each paired cohort captures its own lock. Independent supplier edits advanced
the root lock between cohorts (including Syn 3.0.10); they were preserved. Both
arms of this final pair use the same captured lock and Candid 0.10.40. Use this
pair's upstream row rather than subtracting an earlier cohort's measurement.

| Text decoder | Original total | Original code | Optimized total | Optimized code |
| --- | ---: | ---: | ---: | ---: |
| Prototype | 32,838 | 30,584 | 31,060 | 28,806 |
| Existing bounded upstream adapter | 338,273 | 312,356 | 312,377 | 286,404 |

The paired difference is **281,317 total / 257,598 code bytes**. This is a
single-text workload with the current partial mappings, not a claim that a
complete replacement will retain this footprint. Both schemas and malformed
input admission remain reachable. The prototype's normal Wasm graph contains
only the probe, Platform and Runtime (IC0); it contains neither Candid nor Serde.

The explicit ignored `native_text_throughput` test ran in release mode with
optimization 3, fat LTO and one codegen unit on Linux x86_64 / AMD Threadripper
7970X. Five samples per case alternate implementation order, compare bytes and
decoded values before timing, black-box input/results, and include fresh output
allocation/drop. Data are hot-cache repeated strings, not a representative
application workload. Raw samples and min/median/max are in `throughput.log`
and `native-throughput.tsv` within that evidence directory.

| Text bytes | Prototype encode ns | Upstream encode ns | Prototype decode ns | Upstream decode ns |
| --- | ---: | ---: | ---: | ---: |
| 0 | 29.9 | 226.1 | 26.8 | 222.0 |
| 32 | 61.5 | 264.9 | 37.6 | 241.5 |
| 4,096 | 152.0 | 389.9 | 149.5 | 331.6 |
| 65,536 | 1,094.9 | 2,756.5 | 2,084.3 | 2,295.5 |

These are median native nanoseconds per operation, not IC instructions/cycles.
Seventeen correctness tests and the explicitly selected throughput experiment
passed; all-target Clippy with warnings denied, Rustfmt and ShellCheck passed.
Correctness includes truncation at every position, deterministic byte mutations,
upstream parity for accepted mutations, float payload bits and exact 16/17 cases.
This is a finite corpus, not fuzzing or complete specification conformance.

Remaining at this milestone: arbitrary-size Nat/Int value mappings, general
record/variant APIs and derives, typed principal/function/service references,
complete subtyping/recursive compatibility, argument-sequence semantics,
resource/error contract parity, fuzzing, native allocation and installed IC
instruction measurements, and actual consumer Wasm comparisons. Opaque references
and future constructors currently fail closed. No cryptographic code is changed.

## Arbitrary integers, variants and mixed workloads

The next implementation adds `num_bigint::BigUint`/`BigInt` mappings for Candid
Nat/Int, including Nat-to-Int decoding, signed extension and values beyond u128.
Arithmetic remains supplied by the existing root-selected num-bigint dependency;
the prototype owns only LEB128 framing. Tests cover zero, powers of two and their
neighbours through 8,192 bits, both signs, overlong encodings, truncation and
allocation/output budgets. `Result<T, E>` supplies the standard Ok/Err variants.

Further differential tests exposed value-directed conversions that a strict
schema precheck would reject: upstream can decode an empty vector with a
different declared element type, including inside an optional variant. The
prototype now checks each actual typed value and retains validated skipping.
Optional fallback rolls back only the input cursor on a type mismatch; consumed
work/allocation remain charged. Malformed input and resource exhaustion never
become a successful absent value. The duplicate schema compatibility prewalk was
removed. Unused type validity and all-path wire nesting checks remain in place.

The broader paired workload decodes and re-encodes:

```text
(Option<Result<Vec<u64>, String>>, (Nat, Int))
```

This retains records, variants, options, vectors, text and both arbitrary integer
paths in one Wasm. Both arms receive Candid bytes and return encoded values, using
the same transport and optimization recipe. Cross-decoding tests cover absent,
success/empty-vector, success/nonempty-vector and error branches. They establish
interoperability, not a blanket claim that every possible schema has identical
type-table numbering or encoder bytes. Existing retained-intent byte contracts
still require explicit qualification before consumer migration.

Final matched evidence: `target/evidence/platform-wasm-size.dvCsJQ`.

| Mixed decode + encode | Original total | Original code | Optimized total | Optimized code |
| --- | ---: | ---: | ---: | ---: |
| Prototype | 49,953 | 46,832 | 47,135 | 44,014 |
| Existing bounded upstream adapter | 571,424 | 540,248 | 529,991 | 498,801 |

The difference is **482,856 total / 454,787 code bytes**, about 91% for this
workload. This includes the prototype's reachable num-bigint code. Its normal
Wasm graph contains Platform/Runtime (IC0), num-bigint 0.4.8, num-integer 0.1.47
and num-traits 0.2.19; neither Candid nor Serde is selected. The bigint requirement
is read from the root dependency catalogue, not separately pinned in the harness.
The encoder/decoder implementation is currently 954 physical source lines,
excluding tests/harnesses; this is not a final full-compatibility LOC estimate.

The explicit ignored `native_composite_throughput` experiment uses the same
native release recipe as above, five alternating-order samples and 1,000
operations per sample. Checks occur outside timing; each operation includes
fresh allocation/drop. Both decoders consume the same upstream-encoded frame.
Medians in nanoseconds per operation:

| Vector elements / integer bits | Prototype encode | Upstream encode | Prototype decode | Upstream decode |
| --- | ---: | ---: | ---: | ---: |
| 0 / 256 | 420.4 | 1,978.3 | 436.3 | 2,868.6 |
| 16 / 1,024 | 899.7 | 2,806.3 | 752.5 | 3,891.7 |
| 1,024 / 8,192 | 5,171.3 | 9,123.4 | 5,054.3 | 12,582.3 |

These are hot-cache native results, not IC instructions or application latency.
Raw samples and min/median/max are retained in `throughput.log` and
`native-throughput.tsv`. Twenty-two correctness tests passed, with timing tests
excluded from the ordinary run; the mixed timing experiment passed separately.
Clippy, Rustfmt, ShellCheck and dependency-boundary checks passed. Earlier failed
experiments remain retained, including the incorrect static precheck and the
native benchmark's corrected integer-literal inference failure.

At this stage, still required before production were general record/variant
mapping APIs, typed principal/function/service references, argument sequences,
recursive mapping and complete compatibility coverage, budget/error contract
qualification, fuzzing, allocation measurements and installed IC instructions.
The prototype remains unpublished and production dependencies are unchanged.

## Named records and argument sequences

The next prototype stage adds declarative mappings for existing Rust records
and single-payload variants, canonical Platform Principal encoding/decoding,
and zero through eight argument tuples. Argument sequences remain distinct from
tuple-record values. Callers select exact arity or compatible missing/extra
arguments; extra values still pass the bounded validating skip path. Record
extensions use that same skip implementation and missing fields use the typed
mapping's existing admission rules. No second Principal identity was introduced.

The matched workload is an account-state record with Principal, arbitrary Nat,
a status variant carrying text or a vector, and optional byte-vector memo.
Both Wasms decode and re-encode through the same IC0 input/reply helpers.
The owned arm uses the canonical Types Principal; the upstream arm uses its
Candid Principal. Differential checks compare their raw Principal bytes.

```sh
WASM_BENCH_PROFILES='codec-generic-record codec-upstream-record' \
WASM_BENCH_RECIPES=3-fat-1 bash scripts/benchmarks/wasm-size.sh
```

Evidence: `target/evidence/platform-wasm-size.5zL8WU`, Rust 1.99.1,
optimization 3/fat LTO/one codegen unit, panic abort, stripped, IC-Wasm 0.11.1
shrink. Both arms use the captured lock selecting Candid 0.10.40.

| Named record decode + encode | Original total bytes | Original code bytes | Optimized total bytes | Optimized code bytes |
| --- | ---: | ---: | ---: | ---: |
| Prototype | 53,615 | 50,618 | 50,830 | 47,833 |
| Upstream | 557,862 | 509,452 | 518,931 | 470,504 |

The optimized difference is **468,101 total bytes (90.2%)** and **422,671 code
bytes (89.8%)** for this fixture. This is not a measured whole-application saving.
The prototype's normal Wasm graph excludes Candid, parser and external Principal;
it includes canonical Types' Serde/text dependencies and num-bigint. The native
test graph explicitly enables upstream Candid as the independent reference.

Thirty correctness tests pass, with two native timing experiments deliberately
ignored in this run. New checks cover zero/one/eight argument sequences, exact
versus compatible arity, malformed extra arguments, named-record extensions,
missing optional fields, variant mismatch recovery, Principal lengths 0/1/28/29,
overlong/opaque Principal refusal, and truncation at every byte. The named-record
workload cross-decodes in both directions and matches upstream encoded bytes
exactly; this does not claim every equivalent schema has identical table layout.
Native all-target Clippy and upstream Wasm Clippy pass with warnings denied.
An earlier fixture retained its unused-import Clippy failure at D8t4PV; the
maintained source was corrected before capturing this passing fixture.

The macros currently require explicit sorted numeric field labels and support
single-payload variants, not a complete Rust derive interface. Typed function
and service references, recursive Rust mappings, broader compatibility and
budget/error qualification, fuzzing, allocation measurements and installed IC
instructions remain. No production feature, dependency or API was replaced.

## First owned adoption

The maintainer authorized adoption and removal of upstream Candid dependencies.
The implementation now lives once at `crates/ic-platform/src/candid_codec`,
selected by `candid-codec`. Size fixtures import that owner rather than copying
an experimental codec. Public mapping macros are `candid_record!` and
`candid_variant!`; their generated names are qualified against consumer aliases.
The reader's compatibility and budget limitations described above remain; this
is not a blanket replacement of all previously qualified decoders.

The first migrated feature is `nns-registry-types`:

- Before: Registry DTOs → `candid` → upstream codec/derive and external Principal.
- After: Registry DTOs → `candid-codec` → canonical Types/num-bigint; explicit
  `principal-external` retains the existing public field identity.
- Selecting `candid` separately still enables the DTOs' original CandidType
  implementation for the existing upstream-based protocol adapter. The Registry
  feature itself no longer activates that dependency.

Serde derives, public field types, optional values and arbitrary error strings
are unchanged. Tests compare exact upstream bytes and cross-decode requests and
every response state, including absent and maximum-length Principals. Macro
consumer checks run without the upstream codec selected. Encoding schema
registration now uses iterative depth-first traversal and the caller's output
bound, replacing the prototype's unrelated 256-entry cap. A 1,024-entry trusted
schema regression and recursive-schema tests cover this change. The decoder's
16-level wire admission is unchanged.

Final named-record evidence: `target/evidence/platform-wasm-size.Nr3RWw`.
Compiler, lock, source, feature selections, raw/optimized artifacts and hashes
are retained by the same matched harness described above.

| Named record decode + encode | Original total bytes | Original code bytes | Optimized total bytes | Optimized code bytes |
| --- | ---: | ---: | ---: | ---: |
| Platform owned codec | 51,034 | 47,791 | 48,209 | 44,966 |
| Upstream | 557,862 | 509,452 | 518,931 | 470,504 |

Savings are 470,722 optimized total bytes (90.7%) and 425,538 code bytes (90.4%)
for this workload, not a whole-canister claim. An earlier run was correctly
refused by source-checksum validation after edits overlapped its capture; only
Nr3RWw is used for this adoption comparison.

Focused evidence is retained at `target/evidence/candid-codec-adoption.WdRMyA`:
54 unit tests passed (three optional experiments ignored), two public consumer
tests passed without Candid, four existing contract-consumer tests and nine
pinned-DID/schema tests passed. Native affected-target and Wasm Clippy passed.
Doctests, inventory, documentation and formatting checks passed. Exhaustive
selection combinations of just `candid-codec`/`nns-registry-types` passed on
native and Wasm; explicit graph checks forbid Candid/parser/Agent/runtime in
both and forbid external Principal in codec-only selection. No blanket fleet
matrix, deployment or fresh macOS/installed-IC acceptance is claimed.

The all-feature graph still contains Candid. On Wasm the remaining incoming
edges are Platform's unmigrated DTO/endpoint adapters and
`ic-canister-sig-creation` (also selected by signature verification). Native adds
`candid_parser`, `ic-agent`, `ic-management-canister-types` and
`ic-transport-types`. Removing these transitive edges requires changes to those
suppliers or retiring their capabilities; the codec migration does not authorize
replacing their cryptographic implementations. Management/ledger mappings,
typed references, endpoint quotas and installed qualification remain separate
unfinished adoption work. No production callers were silently redirected to a
different budget contract.

## CMC, Cycles and native assets adoption

The next migration removes implicit `candid` activation from `cmc-types`,
`cycles-ledger-types` and `certified-assets-types`. Unit and named-field variants
now share the same mapping macro and bounded record/variant traversal. Migrated
Nat fields use the owned unbounded carrier, with explicit registry conversion,
binary-byte and JSON/CBOR Serde parity. This is a Rust identity change, not a
numeric narrowing. Principal and unmigrated Cycles/ICRC archive identities stay
unchanged; see [the migration contract](../../docs/migration.md#owned-binary-codec-unreleased).

Focused proof is retained at `target/evidence/owned-dtos.hq851r`: reference bytes
and cross-decoding for every CMC/Cycles error branch, settings/visibility/subnet
variants, optional fields, asset requests/results and naturals through 8,192 bits;
all-prefix truncation/trailing refusal; pinned DID/schema and retained protocol
adapter checks; Candid-free consumers; native/Wasm Clippy. The 32 combinations
of the five owned-codec/DTO features plus default selection passed on both targets
at `target/feature-graphs/attempt.fCXyVq`. Signature/Agent suppliers remain outside
this removal; no cryptographic implementation or consumer policy was changed.

The new matched workload decodes and re-encodes CMC `NotifyTopUpResult`, including
unbounded success values and the complete unit/payload/named error schema:

```sh
WASM_BENCH_PROFILES='codec-cmc-owned codec-cmc-upstream' \
WASM_BENCH_RECIPES=3-fat-1 bash scripts/benchmarks/wasm-size.sh
```

Evidence: `target/evidence/platform-wasm-size.GFO6VI`, Rust 1.99.1, identical
captured lock/source, optimization 3/fat LTO/one codegen unit, abort/strip and
IC-Wasm 0.11.1 shrink. Both use the migrated DTO; the upstream arm selects its
explicit CandidType/Nat bridge, including a magnitude clone on encoding. This
compares the two codecs on that DTO, not a frozen pre-migration application.
Both enforce exact one-result arity; work/allocation accounting remains different
and no budget equivalence or instruction-count claim is made.

| CMC result decode + encode | Original total bytes | Original code bytes | Optimized total bytes | Optimized code bytes |
| --- | ---: | ---: | ---: | ---: |
| Owned codec | 50,081 | 46,934 | 47,403 | 44,256 |
| Upstream codec with explicit bridge | 512,903 | 481,490 | 474,445 | 443,005 |

The optimized difference is 427,042 total / 398,749 code bytes for this fixture.
The earlier HzuwRo attempt failed to compile its upstream arm because it selected
the wrong helper; that failed evidence is retained and excluded from comparison.

## ICP transfer/archive adoption

`icp-ledger-types` now selects the owned codec and explicit external Principal,
retaining SHA-224/CRC solely for the existing account-identifier implementation.
Its imported constructor body and license fingerprints pass unchanged; the import
now names the same external Principal through Platform. The callback constructor
and fields remain, with an owned `references::Func` payload and an optional
upstream conversion/trait bridge. This carrier identity change is explicit in the
[migration contract](../../docs/migration.md#owned-binary-codec-unreleased).

Typed references retain function mode and check structural signature compatibility
using an iterative pair worklist, including contravariant arguments, covariant
results, optional omission, record/variant/service width and recursive schemas.
Schema traversal, pair processing and lookup work are charged; expected-schema
and worklist storage consume the caller's logical-allocation budget. The incoming
16-level wire admission remains. Callback authority and submission remain entirely
consumer-owned. Standalone typed service values and ICRC callback adoption remain
unfinished; this does not redirect existing endpoints to different quotas.

Evidence: `target/evidence/owned-icp.diDFaV`. Sixty-three unit tests passed, including
256 cross-signature pairs against upstream, five additional service-width/Principal
pairs, malformed tags/lengths/UTF-8, mutation parity, truncation and budgets. Four
owned-contract tests pass with exact ICP request/response/callback bytes and
cross-decoding. Two Candid-free public consumer tests and the unchanged account
vectors pass; existing schema/contract/protocol suites pass (9/4/5 tests).
Native/Wasm Clippy, source/license guards, graph policy fixtures, docs and format
checks pass. All 64 combinations of six migrated features plus default selection
pass on both targets at `target/feature-graphs/attempt.eHYhhS`. The explicit ICP
normal/build graph excludes Candid/parser/Agent/runtime. No new installed IC or
macOS acceptance is claimed.

The matched workload decodes and re-encodes `QueryEncodedBlocksResponse`, including
archive callbacks and their complete signatures:

```sh
WASM_BENCH_PROFILES='codec-icp-owned codec-icp-upstream' \
WASM_BENCH_RECIPES=3-fat-1 bash scripts/benchmarks/wasm-size.sh
```

Evidence: `target/evidence/platform-wasm-size.kHgzsD`. Rust 1.99.1, the same
captured lock/source, optimization 3/fat LTO/one codegen unit, abort/strip and
IC-Wasm 0.11.1 shrink. Both use the migrated DTO; the reference arm selects its
upstream CandidType bridge. Both require exactly one reply value; budget units
are not equivalent and this is not an installed instruction benchmark.

| Archive reply decode + encode | Original total bytes | Original code bytes | Optimized total bytes | Optimized code bytes |
| --- | ---: | ---: | ---: | ---: |
| Owned codec | 60,415 | 56,783 | 57,178 | 53,546 |
| Upstream codec with explicit bridge | 551,634 | 502,513 | 510,977 | 461,831 |

The optimized difference is 453,799 total / 408,285 code bytes for this workload.
Documentation-only edits after the capture do not change the measured code.

## Isolation and coverage design

Wire's `crates/ic-wire-benchmarks` separates size, installed IC instructions,
native allocation and heap probes. Its qualification allocation suite returns
the actual frame outside timing, and distinguishes allocation/drop from reuse.
Storage's `testing/runtime-qualification/crates/ic-storage-benchmarks` uses one
type/workload catalogue against independently compiled implementations, records
seeds and input sizes, and refuses mismatches. Adopt those principles here using
the existing Platform fixture/harness rather than another workspace.

The generic codec investigation should have one shared case catalogue consumed
by correctness and benchmark runners. Required families:

- All primitive wire types, signed/unsigned boundaries, unbounded Nat/Int,
  float bit patterns, Unicode, blobs, empty and large collections.
- Nested options, records, variants, recursive schemas, principals and
  function/service references. Generic Rust mappings share binary machinery.
- Record extension/reordering, missing optional fields, subtyping, skipped
  values, arbitrary valid type-table numbering and argument arity.
- Truncation at each byte, malformed LEB128, invalid indices/opcodes/UTF-8,
  overflowing lengths, trailing data, unused malformed types, cyclic schemas
  and exact byte/work/nesting admission boundaries, including levels 16/17.
- Real management status/settings, ledger results/errors and archive callbacks;
  compare independently generated upstream bytes in both decode directions.

Use the published Candid specification and upstream as independent references.
Round trips alone are insufficient. Distinguish valid encodings from the subset
admitted by Platform's existing resource policy. Preserve arbitrary integer
ranges; a u128-only implementation is not a complete Candid codec.

Next measurements should isolate encode/decode/skip and malformed-input work,
report IC instruction counts with matched counter overhead, native allocations
and allocated bytes, throughput distributions and complete Wasm/code sizes.
Keep correctness checks outside timed regions, retain operation results against
dead-code elimination, and include one/many-type scaling to detect duplicated
monomorphized code. Record implementation LOC separately from tests/fixtures.

Upstream already supports typed decoding; avoiding a generic value tree alone
does not explain the measured cost. Inspect retained functions and investigate
schema representation, subtype matching, diagnostic formatting, traversal and
generated code before attributing savings. Existing cryptographic suppliers can
retain Candid transitively; full application graphs must be measured before
claiming dependency removal. The experimental codec traits are not public and
production Candid dependencies remain unchanged.

## ICRC adoption

ICRC DTOs, recursive values, consent records and both standard/Cycles provider
archive callbacks now use the owned codec. The contract corpus compares exact
upstream bytes, cross-decoding and every truncated prefix; pinned DID comparisons
remain independent. Repeated declarations are merged in the writer's type table
without unfolding recursive graphs. A 1,024-node trusted-schema regression covers
registration and compaction; incoming wire still has the fixed 16-level ceiling.

The unchanged upstream Nat hash-input encoder is retained separately from the
owned writer with license/source guards. Original ICRC value hashes and vectors
remain unchanged. This stage adds no new Wasm-size or installed instruction claim;
earlier measurements describe their recorded source snapshots. Native/Wasm graph
closure evidence is `target/feature-graphs/attempt.kbdKG0`; focused test logs are
under `target/evidence/owned-icrc-20261109`.

## Management status adoption

Evidence: `target/evidence/platform-wasm-size.tFV6c6`. The two probes decode and
re-encode the same maintained `CanisterStatusResult` on Wasm with explicit exact
arity and bounded decoders. Rust 1.99.1, opt-level 3, fat LTO, one codegen unit,
abort/strip and ic-wasm 0.11.1 shrinking are identical. Each uses the same raw
input/reply plumbing and source/lock snapshot; the supplier arm explicitly adds
`candid` and uses its trait bridge on the migrated DTO. That Nat serialization
bridge clones the magnitude; this is not a pre-migration supplier-native DTO
measurement. Work/allocation budgets are codec-specific, not equivalent quotas.

| Codec | Raw total | Raw code | Optimized total | Optimized code |
| --- | ---: | ---: | ---: | ---: |
| Owned | 95,129 | 90,643 | 89,259 | 84,773 |
| Upstream bridge | 639,274 | 586,136 | 591,185 | 538,037 |

The optimized reduction is 501,926 total bytes and 453,264 code-section bytes.
This measures reachable codec code in an isolated management-status probe. It is
not a whole-canister saving or instruction benchmark; other selected suppliers
can still retain upstream Candid. New installed IC qualification remains pending.

## Legacy bridge retirement

The old upstream DTO/decoder benchmark profiles are retired with the production
bridge. Retained measurements above describe their captured source revisions,
not the current source. Current probes exercise the owned codec. The raw-rand
comparison now measures the owned general codec versus the narrow entropy codec;
it must not be compared as a new upstream-Candid baseline. Byte/schema oracle
tests remain native unit tests using development suppliers.

## Published baseline versus final owned codec — 2026-11-09

Run `bash scripts/benchmarks/candid-removal.sh` to compare the published Platform
0.27.1 source (97fe286) with the current implementation. This archives the baseline
inside evidence; it neither edits a sibling nor creates another checkout.
Both probes use `ic0` plus `management-canister-types`, the same ingress/reply
body and the CanisterStatusResult decode/re-encode workload. The baseline uses
its bounded exact-single-value upstream adapter; the current arm uses the owned
codec. The decoder budget models differ: the owned arm additionally enforces a
4 MiB allocation and 1 MiB encoding ceiling. This is a footprint comparison,
not an assertion of identical resource accounting or error diagnostics.

Both builds start from the same current supplier lock (including the retained
Candid 0.10.40 draft), use Rust 1.99.1, opt-level 3, fat LTO, one codegen unit,
panic abort, symbol stripping and ic-wasm 0.11.1 shrink. The baseline therefore
isolates the source migration rather than reproducing its historical release lock.
Evidence: `target/evidence/candid-removal.fREwuJ`, including generated probes,
resolved metadata/locks, source hashes, Clippy logs and raw/optimized artifacts.

| Uncompressed Wasm measurement | Published implementation | Owned codec | Reduction |
|---|---:|---:|---:|
| Raw total | 639,344 | 95,121 | 544,223 (85.1%) |
| Raw code section | 586,331 | 90,635 | 495,696 (84.5%) |
| After shrink, total | 591,239 | 89,251 | 501,988 (84.9%) |
| After shrink, code section | 538,217 | 84,765 | 453,452 (84.3%) |

Both probe selections pass Wasm Clippy with warnings denied. The current Wasm
normal graph contains neither Candid, candid_parser nor Agent. This is not a
whole-application saving, installed instruction measurement, or removal of
Candid from separately selected signature/native supplier graphs. Existing
byte/schema oracle and installed qualification evidence above remains relevant;
this size run does not replace those tests.
