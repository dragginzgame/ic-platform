# Upstream dependency qualification, 2026-10-31

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
