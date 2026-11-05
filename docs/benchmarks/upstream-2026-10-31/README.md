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

The original evidence directory was `target/executor-cleanup-20261102.M99fEL/`: baseline/candidate
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

The original evidence was under `target/bls-dependency-trial/`: isolated source and lock,
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

## Later ownership decisions

The older ECDSA upgrade/backend and BLS allocation/import experiments are
superseded by the current [cryptographic ownership contract](../../migration.md#upstream-cryptographic-dependency-cut-022).
Platform consumes upstream cryptographic implementations. The dependency trial
above did not establish a safe manifest-only upgrade or repository-wide advisory
clearance; [#50](https://github.com/dragginzgame/ic-platform/issues/50) remains.
The old instrumented-runtime observations do not qualify today's provider pin.

This summary retains the useful executor/SHA-2 decisions and selected absolute
results. Historical raw workspaces and artifacts were not reverified during
cleanup and are not claimed as independently available. Git history retains the
superseded detail; active release/recovery records remain under their own custody.
