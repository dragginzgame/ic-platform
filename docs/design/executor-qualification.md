# Independent executor qualification — 2026-11-10

Subsequent tooling adoption resolves the earlier IC0 admission and root-file
metadata blockers. This report retains its original qualification scope; see
[the current handoff](../status/current.md) for remaining work.


The [contract](executor-contract.md), [implementation record](executor-implementation.md)
and [public-test record](executor-public-tests-implementation.md) document the
separate source/test contexts and their limits. This record covers parent-run
integration and measured artifacts. It is not a legal-clearance statement.

## Correctness and boundaries

Rust 1.99.1 native all-target Runtime `cdk` Clippy and Wasm library Clippy pass
with warnings denied. Runtime has 19 passing unit tests, including five newly
authored executor-state tests, plus 13 new public executor lifecycle tests,
three CDK consumer tests and two doctests. Tests cover method/query retention,
foreign wake routing, protected priority/FIFO, migratory isolation, cached waker
identity, coalescing, stale generations, cancellation/destructor reentrancy,
recovery policy and Send/Sync handle compatibility. No new unsafe code or
dependency was added.

The pre-replacement [public observation trace](executor-api-observations.txt)
matches the replacement. A separate before/after retention probe identified one
intentional correction: dropping the last retained method handle outside
execution previously left a pending task allocated (drop count 0); the new owner
reclaims it immediately (drop count 1). Destruction remains outside executor
borrows. Explicit handle destruction inside another context observes that
surrounding context; automatic context exit clears the context first.

All four installed owner profiles pass using the prepared PocketIC 16.1 server:

```text
POCKET_IC_BIN="$PWD/.tools/ic-testkit-server/16.1.0-x86_64-linux/pocket-ic" \
  CARGO_NET_OFFLINE=true bash scripts/ci/check-platform-runtime.sh --prepared-call
```

Raw Wire, raw external-Candid, prepared Wire and prepared external-Candid cover
awaits, callback/encoding traps, cancellation, upgrades, cycles, stable rollback,
certificates, management bytes and endpoint admission. Evidence directory
`target/cdk-runtime.w76c08` has status 0 and four PASS profile rows. Captured inputs
were checked again at completion. This is Linux installed evidence, not fresh
macOS acceptance or Scheduling-owned Timers composition qualification.

Native/Wasm IC0/executor feature closures pass, as do actual Cargo singleton
refusals/coexistence (`target/runtime-singleton.fu8auQ`). The exact guard and
coordinated owner versions are unchanged. Retained CDK/management source/license
guards and mutation-refusal tests pass. The executor's retired fingerprint/license
checker was removed with its replaced source; no new source fingerprint substitutes
for behavioral tests.

Parent logs are `/tmp/platform-independent-executor-{clippy,tests,wasm-clippy,installed,feature-closures,singleton}.log`.
The retention probe logs are `/tmp/platform-executor-retention-{before,after}.log`.

## Matched Wasm sizes

These are uncompressed Cargo release artifacts, without a later wasm-opt or
stripping pass. Code sizes are section 10 payload bytes measured by
`scripts/benchmarks/wasm-sections.pl`. Custom sections are included in total size.

| Installed fixture | Before total | After total | Total reduction | Before code | After code | Code reduction |
|---|---:|---:|---:|---:|---:|---:|
| Raw Wire | 273,265 | 226,685 | 46,580 (17.0%) | 178,753 | 150,462 | 28,291 (15.8%) |
| Raw external Candid | 732,376 | 677,515 | 54,861 (7.5%) | 509,018 | 473,762 | 35,256 (6.9%) |

Before artifacts are in `target/cdk-runtime.nhOywW/artifacts/1.99.1`;
after artifacts are in `target/cdk-runtime.w76c08/artifacts/1.99.1`. Both observations
use rustc 1.99.1, commit `58cf8f9c35b355199727627d07ae34a4e2846d67`, on Linux
x86_64 targeting wasm32-unknown-unknown. The generated workspace and canister
manifests, all canister fixture sources, Cargo.lock, compiler records and captured
Cargo config compare byte-identical. Raw profile feature selections match.
Normal dependency graphs differ only in the disposable fixture path.

The Runtime executable-source difference is the executor implementation; other
Runtime differences are docs, test registration/new tests, and removed licenses/
provenance. Types, guard and macro executable sources match the before snapshot.
No runtime-speed, IC instruction-count or arbitrary consumer-size claim follows
from these two fixtures. Prepared profiles were qualified but have no paired
before-size measurement in this report.

Artifact SHA-256:

| Artifact | SHA-256 |
|---|---|
| Before raw Wire | `284e979d064ab53ee120d2c96d3887781ceeaf48e8ba4a364bb611c04f6ffd69` |
| After raw Wire | `bc206a76b7e96501c4a89d3082627ad7d6bfad68c874bac47858d5fa4184d2e8` |
| Before raw external Candid | `1b096c450a12e760062b405bd3f9074fe45573e4e9f6717b0f2ec714658b9156` |
| After raw external Candid | `0e0932430be32fa648993e81993b4f91b1b8c00965f10ee656a8bf65855f7a9c` |

The matched lock SHA-256 is `c60ac3dfd13902044e2242839adb5b4efd99b52175a3ca817fe1188c2050bb26`.
Before/after captured input-receipt hashes are respectively
`ce4dc754db78854897bced5b5e5a1e57e8f2140ab372aaa5e6e9b1040423c998` and
`4166ef56d6a86cc2b6820bf4e83396e78006b903aeae87168eb8b2852347eee0`.
Build artifacts and temporary probes are reclaimable; these observations and
the maintained lifecycle tests remain the durable qualification record.

## Remaining release limitations

First-party license text/metadata is pending the maintainer's replacement.
The installed Toko 0.38.2 checker still rejects Runtime's registry IC0 edge;
its committed supplier correction is not yet adopted here. The separate ICP
derivation draft remains unresolved. CDK adapters, generator and retained ledger/
management imports still carry their applicable source notices and guards.
No full release gate, commit, tag, push or publication was performed.
