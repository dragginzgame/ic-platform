# Upstream dependency qualification, 2026-10-31

## Executor cleanup experiment, 2026-11-02

The production executor removes each completed protected task by scanning its
method's task vector. A batch of N ready tasks therefore performs quadratic
bookkeeping. An isolated candidate counts removed keys and compacts in spawn
order once at least half the keys are dead. It retains standard Vec, SlotMap
generation checks and destructor execution outside RefCell borrows; bookkeeping
keeps at most twice as many keys as live tasks after deletion. No production source
or dependency was changed by this experiment.

Installed PocketIC 16.1.0 probes use Rust 1.99.1, wasm32-unknown-unknown,
opt-level z, fat LTO, one codegen unit, panic abort, stripped symbols and
ic-wasm 0.11.1 `shrink`. Both variants use the same probe and recipe. This is a
library capability probe, not a complete application deployment recipe or
whole-canister speedup forecast. Code bytes are the Wasm section-10 payload
measured by the existing sections.pl helper.

| Final Wasm | Baseline bytes | Candidate bytes | Delta bytes |
| --- | ---: | ---: | ---: |
| Complete module | 27,095 | 27,243 | +148 |
| Code section | 23,320 | 23,468 | +148 |

Median instruction counts from three installed update samples per case:

| Ready tasks in one method | Baseline cleanup | Candidate cleanup | Change |
| --- | ---: | ---: | ---: |
| 1 | 3,207 | 3,267 | +1.9% |
| 8 | 13,374 | 12,844 | -4.0% |
| 64 | 168,238 | 91,168 | -45.8% |
| 256 | 1,650,498 | 358,188 | -78.3% |
| 1,024 | 22,324,250 | 1,425,020 | -93.6% |
| 4,096 | 340,947,838 | 5,690,968 | -98.3% |

For ready tasks, the cleanup interval includes draining polls, future drops,
benchmark-handle disposal and context teardown; total counts additionally include
spawning. At 1,024 tasks total instructions fall from 23,263,263 to 2,364,036.
Forward/reverse individual cancellation also improves at high counts. Bulk
cancellation remains effectively unchanged (three extra cleanup instructions).
Every case asserts the exact completion/drop counts. One extra usize per method
and delayed stale-key removal are additional memory costs; no future or waker
is retained by a stale key. Reserved Vec capacity remains a separate concern.

Runtime and probe Clippy pass. All twelve native executor tests pass, including
new cancellation-order/slot-reuse and long-lived-method bounded-retention checks.
Combined CDK/Timers callback, trap and upgrade qualification was not run: this is
an experiment, not an adopted or release-ready runtime change. Current Timer
dispatch and generated async entrypoints each initially spawn one task; multiple
awaits within a task do not create the measured fan-out. No general production
speedup is established. Retain the candidate for a workload-specific decision;
the existing size-first production choice remains unchanged.

Evidence is in `target/executor-cleanup-20261102.M99fEL/`: baseline/candidate
source snapshots, candidate patch, probe/driver source and locks, build/run
scripts, compiler/optimizer identities, raw/final Wasms, source/artifact hashes,
Clippy/test logs, all instruction samples and machine-readable summary.json.
The initial driver compile exposed the old Testkit client identity; the final
driver uses the existing Runtime suite's explicit server-URL boundary and passes.

## Owned Principal SHA-2 provider qualification, 2026-11-01

Compared published source `3cae1e5` with SHA-2 0.11.1 (defaults disabled) for
owned Principal self-authentication and, experimentally, certification/creation.
The existing `wasm-size.sh` probes used Rust 1.91.0, Wasm32, fat LTO, one codegen
unit, panic abort, stripped symbols, `cpubits=32` and ic-wasm 0.11.1 `shrink`.
Each pair uses identical settings. These are reachable-capability probes,
not whole-canister forecasts or execution-performance measurements.

| Probe / recipe | Old module bytes | New module bytes | Old code bytes | New code bytes |
| --- | ---: | ---: | ---: | ---: |
| Owned Principal / 3-fat-1 | 18,408 | 19,192 | 17,782 | 18,310 |
| Owned Principal / z-fat-1 | 17,518 | 17,132 | 16,134 | 16,139 |
| Combined verifier / 3-fat-1 | 499,553 | 508,429 | 464,953 | 473,572 |
| Combined verifier / z-fat-1 | 301,229 | 310,727 | 265,449 | 274,466 |

Retain only the owned Principal provider update. Its module shrinks by 386 bytes
with size optimization and grows by 784 bytes at opt-level 3; its code section
does not shrink in either recipe. Defer certification/creation: adding SHA-2 0.11
alongside BLS's required 0.10 increases the combined verifier by 8,876–9,498 bytes.
Candid's Principal, certification, BLS and ledger hashing keep their dependencies.
This changes dependency scaffolding, not imported derivation bodies or vectors;
source fingerprints remain unchanged. SHA-2 implementation stays upstream.

Full Clippy, 12 Types tests and its doctest, 35 certification tests, 37 signature
tests, the certification consumer test, source mutation refusals and the selected
native/Wasm feature graphs passed for the experiment. Final Principal retains
the tested provider/body; certification/creation edits were discarded.
Local source identities, candidate patch, compiler/optimizer receipts, locks,
raw/optimized Wasm, checksums and logs are under `target/sha2-011/`.
`before`/`after` cover opt-level 3; `before-z`/`after-z` cover opt-level z.

## BLS dependency trial, 2026-11-01

The subsequent manifest-only trial selected digest 0.11.3, ff/group 0.14.0,
pairing 0.24.0, rand_core 0.10.1 and SHA-2 0.11.1 in an isolated source copy.
Resolution succeeded. Full `make clippy` failed with 23 library errors, including
removed Digest `generic_array`/`core_api` exports, refactored curve/affine traits
and incompatible RNG method signatures. No tests or Wasm size comparison followed
the failed compilation. No BLS source bodies, fingerprints or production dependency
selections changed. The accepted Principal update remains intact.

Evidence is under `target/bls-dependency-trial/`: complete isolated source and lock,
accepted Principal patch, resolved metadata and `cohort-clippy-offline.log`.
The initial Clippy attempt failed to download pairing; explicit dependency fetch
succeeded before the offline compile exposed the source incompatibilities.

Upstream inspection found no documented DFINITY rationale for the retained
dependency versions. Its [release history](https://github.com/dfinity/bls12_381/blob/main/RELEASES.md)
records previous SHA/Digest upgrades. The newer
[zkcrypto release](https://github.com/zkcrypto/bls12_381/blob/main/RELEASES.md)
updates ff/group/pairing/RNG but retains Digest/SHA-2 0.10 and also changes the
hash-to-curve draft version. It is not a reviewed drop-in replacement for the IC
fork. Updating the field/group family alone does not unlock SHA-2 0.11; digest
compatibility is a separate source-port obligation. No runtime saving or evidence
of abandonment follows from this failed manifest-only trial.

## Earlier qualification

These are reachable-capability probes, not whole-application size forecasts.
Rust 1.91.0 targets wasm32-unknown-unknown, with panic abort, stripped symbols,
one codegen unit and fat LTO. `3-fat-1` uses opt-level 3; `z-fat-1` uses opt-level
z. All reported sizes follow ic-wasm 0.11.1 `shrink`. Recipes, source copies,
dependency locks, compiler identities, raw/optimized Wasm and SHA-256 records
are retained under `/tmp/platform-upstream-work-20261031` on the qualification
host. The source base is Platform `f6b3293e2d6adabf65bd5afdf0194463b84330ea`
with the local qualification changes recorded in those snapshots.

## ECDSA security upgrade and backend selection

The same compressed-public-key/prehash/compact-signature verification endpoint
compares K256 0.13.4 against upstream 0.14.0, elliptic-curve 0.14.3 and
crypto-bigint 0.7.10. No cryptographic implementation is copied or edited.

| Recipe | K256 0.13.4 bytes | K256 0.14 default bytes | K256 0.14, 32-bit bytes |
| --- | ---: | ---: | ---: |
| 3-fat-1 | 77,069 | 124,278 | 84,531 |
| z-fat-1 | 48,583 | 107,421 | 57,575 |

This addresses the direct ECDSA path's
[bigint timing advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html).
The default backend incurs a substantial size cost. RustCrypto's supported
32-bit backend recovers most of that growth while retaining the correction;
the size-optimized probe remains 8,992 bytes larger than the old implementation.
The direct ECDSA graph no longer selects the affected bigint
generation. Agent/native public derivation still do; there is no repository-wide
advisory clearance. Platform's public upstream types change, requiring a minor
release and caller adaptation. Fixed old-version SEC1/signature bytes and Auth's
strict policy tests qualify compatibility of the exercised byte/policy paths.

Reproduce the size comparison with the captured old/new source workspaces and:

```bash
WASM_BENCH_PROFILES=ecdsa WASM_BENCH_RECIPES='3-fat-1 z-fat-1' \
  bash scripts/benchmarks/wasm-size.sh SOURCE_ROOT NEW_OUTPUT_DIRECTORY
```

Add `WASM_BENCH_CPUBITS=32` to select the qualified backend. The benchmark supplies
and records explicit compiler flags, overriding ambient Cargo flags so `auto`
still measures upstream's heuristic even with Platform's repository config.

### Attribution and installed backend qualification

Named Wasm profiling located most growth in K256's field/point operations.
Upstream cpubits 0.1.1 explicitly promotes Wasm32 to 64-bit arithmetic, selecting
K256's 5x52 field implementation. Its
[documented cfg override](https://docs.rs/cpubits/0.1.1/cpubits/#overriding-the-selection-result-via-cfg)
selects the unchanged 32-bit/10x26 backend instead, consistently across K256 and
its bigint dependencies. The version graph still contains exactly one K256 0.14
and one crypto-bigint 0.7.10 in the Wasm probe. No crypto body is patched.

| Recipe | Default valid verification instructions | 32-bit instructions | Reduction |
| --- | ---: | ---: | ---: |
| 3-fat-1 | 7,586,809 | 2,925,732 | 61.44% |
| z-fat-1 | 8,532,810 | 4,381,468 | 48.65% |

Instruction probes add only counter/reply plumbing to the same endpoint, and
their sizes are recorded separately from the uninstrumented table above.
Three samples for each backend/case were identical. Both backends accept the
known valid compact signature and refuse changed prehash/signature bytes,
invalid keys/scalars, high-S and short/long signatures. The prepared PocketIC
16.0.0 server and Testkit 0.72.1 match the BLS setup below. Auth's nine isolated
chain-key policy tests and Platform's fixed 0.13.4 byte consumer also pass with
32-bit arithmetic at Rust 1.91. This qualifies the exercised consumer paths;
it is not an independent cryptographic security review or live-IC acceptance.

Evidence is retained under `/tmp/platform-ecdsa-footprint-20261031/qualified`:
source snapshot, locks, flags, artifact checksums, raw samples and Wasm. Reproduce:

```bash
POCKET_IC_BIN=/path/to/prepared/pocket-ic \
  bash scripts/benchmarks/ecdsa.sh /tmp/new-ecdsa-evidence
```

Use the target-specific cfg in each canister workspace's build configuration,
merging existing flags as shown in the root README. The published Platform
library cannot propagate its Cargo config; explicit RUSTFLAGS or encoded flags
can override it. No DFINITY update, fork or new Platform feature is required for
this backend choice. Agent/native IC secp256k1 dependency upgrades remain separate.

## Upstream BLS allocation experiment (before wrapper adoption)

Compare upstream ic-verify-bls-signature 0.6.0 with and without its existing
`alloc` feature. Both variants retain identical upstream cryptographic source.
The no-allocation variant removes lazy_static and BLS's hex dependency in
isolation; hex remains reachable through the combined canister-signature graph.

| Recipe / capability | Current alloc bytes | No-alloc bytes | Saving |
| --- | ---: | ---: | ---: |
| 3-fat-1 / BLS | 234,526 | 226,140 | 8,386 |
| 3-fat-1 / canister-signature verification | 499,863 | 493,226 | 6,637 |
| z-fat-1 / BLS | 72,349 | 68,659 | 3,690 |
| z-fat-1 / canister-signature verification | 302,977 | 300,988 | 1,989 |

The installed BLS probe uses the existing fixed public-key/signature vectors
over `platform-signature`. It measures the IC instruction counter around parsing
and verification, excluding request/reply framing. One initial update measures
cold state; five further updates retain the prepared-generator cache. Each warm
sample was identical. A query verifies the same bytes; changed messages and
invalid signature points are refused by both variants.

| Recipe | Alloc cold | Alloc warm | No-alloc cold/warm | Warm increase |
| --- | ---: | ---: | ---: | ---: |
| 3-fat-1 | 226,069,907 | 212,452,673 | 320,257,482 | 50.74% |
| z-fat-1 | 245,753,136 | 231,043,094 | 348,543,350 | 50.86% |

Testkit 0.72.1 admitted the prepared PocketIC 16.0.0 server. Executable SHA-256:
`69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`.
This is installed simulator evidence, not live-IC acceptance or elapsed-time
benchmarking. No full canister-signature instruction benchmark was performed.

Keep the optimized allocated path: saving about 2 KB in the size-optimized
combined verifier does not justify roughly 51% more BLS verification instructions.
The subsequent source adoption below preserves that path. Reproduce both size and instruction
measurements with an explicitly prepared Testkit-supported server:

```bash
POCKET_IC_BIN=/path/to/prepared/pocket-ic \
  bash scripts/benchmarks/bls.sh /tmp/new-bls-evidence
```

That script recreates the original registry wrapper only inside its disposable
source variants. It does not reintroduce the wrapper into the production graph.

## Maintained BLS wrapper adoption

Platform imports the 0.6.0 wrapper and all eight original upstream tests. The
cryptographic bodies, domain, encodings and vectors reconstruct to exact original
hashes. The optimized generator cache uses std LazyLock with the same initializer;
the original random helper and Rand are test-only. Public paths stay the same,
but key/signature/error types now have Platform identity. Curve arithmetic remains
the qualified registry ic_bls12_381 implementation.

| Recipe / capability | Registry wrapper bytes | Maintained wrapper bytes | Change |
| --- | ---: | ---: | ---: |
| 3-fat-1 / BLS | 234,526 | 234,541 | +15 |
| 3-fat-1 / canister-signature verification | 499,863 | 499,880 | +17 |
| z-fat-1 / BLS | 72,349 | 72,452 | +103 |
| z-fat-1 / canister-signature verification | 302,977 | 301,952 | −1,025 |

The installed z/fat-LTO probe retains success/failure results and the optimized
path: warm valid verification changes from 231,043,094 to 231,043,115 instructions,
and cold verification from 245,753,136 to 245,753,183. Wrong messages and invalid
signature points remain rejected. These are capability measurements with no
whole-application claim. Evidence, sources, artifacts and hashes are retained
under `/tmp/platform-bls-adoption-20261031`.

The existing full canister-signature consumer roundtrip also executes successfully
inside installed Rust-1.91 Wasm, covering the original accepted certificate,
authenticated signing time and its existing malformed/changed-input refusals.
The proof copies the maintained consumer and fixtures unchanged, without adding
a second verifier. Its Wasm/driver checksums and execution log are retained there.

Standalone BLS/signature native and Wasm graphs exclude ic-verify-bls-signature,
lazy_static and production Rand. Native Agent still retains its registry verifier
transitively; importing this wrapper does not fork Agent or eliminate that edge.

## Runtime fixture

Published Timers 0.34.0 accepts Platform 0.15, resolving the previous 0.8 runtime
conflict without bypassing singleton guards or removing timer coverage. The
refreshed lock admits one embedded runtime on native/Wasm. Installed Wire and
external-Candid runs on Rust 1.99.1 and 1.91.0 cover awaits, callback traps,
cancellation, timer callbacks, upgrade, rollback and certificate behavior.
This proof uses the current local 0.15.3 package identity; releasing a new
Platform minor still requires Timers to adopt that new compatible requirement.
