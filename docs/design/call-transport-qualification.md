# Raw call transport qualification — 2026-11-10

Subsequent tooling adoption resolves the earlier IC0 admission and root-file
metadata blockers. This report retains its original qualification scope; see
[the current handoff](../status/current.md) for remaining work.


The [reviewed contract](call-transport-contract.md) defines preserved public
behavior, submission ordering and callback/cleanup ownership. Independently
authored [public tests](call-public-tests-implementation.md) are exercised before
and after integration. This is compatibility evidence, not legal clearance.

## Baseline

Seven public tests pass against the retained CDK call implementation, after
native all-target Runtime Clippy with warnings denied. They check lazy unpolled
construction/destruction, borrowed/owned payload admission, builder extrema,
error conversions/messages/source behavior, rejection identities and
Call/associated-future Send + Sync + Unpin traits. The tests do not invoke IC0.
Logs: `/tmp/platform-call-before-{clippy,public}.log`.

The installed corpus now submits two calls from one method and traps when the
first reply resumes execution. Cleanup must run exactly once, cancel the other
pending future and prevent its later callback from reviving the method. A fresh
call afterward must succeed without a second recovery destruction. The IC may
report the original callback trap or the later `Call already trapped` diagnostic;
both require CanisterError and the same destruction assertions. An initial
diagnostic-only assertion was too narrow and failed on the existing owner;
its evidence is retained at `target/cdk-runtime.urDDcK`.

The corrected baseline passes all four raw/prepared × Wire/external-Candid
profiles at `target/cdk-runtime.RAIlQV`, status 0, with unchanged captured source
hashes at completion. The original corpus also covers callback/encoding traps,
600-call submission pressure, insufficient cycles, reply/reject/refund handling,
composite queries, cancellation, upgrades and stable rollback. This uses Rust
1.99.1 and the prepared PocketIC 16.1 Linux server. No fresh macOS or
Scheduling-owned composition evidence is claimed.

```sh
POCKET_IC_BIN="$PWD/.tools/ic-testkit-server/16.1.0-x86_64-linux/pocket-ic" \
  CARGO_NET_OFFLINE=true bash scripts/ci/check-platform-runtime.sh --prepared-call
```

Baseline uncompressed Cargo release sizes, measured by
`scripts/benchmarks/wasm-sections.pl` (code = section 10 payload; totals include
custom sections, no extra wasm-opt/stripping pass):

| Profile | Total bytes | Code bytes |
| --- | ---: | ---: |
| Raw Wire | 228,455 | 152,268 |
| Raw external Candid | 678,172 | 475,268 |
| Prepared Wire | 231,875 | 154,485 |
| Prepared external Candid | 681,350 | 477,502 |

These fixtures include the new multiplexed cleanup endpoints, so compare the
replacement against these artifacts rather than the earlier adapter-only size
table. Source manifests, compiler, lock, fixture bytes and profile selections
must match for the final size comparison.

## Replacement

The [implementation record](call-transport-implementation.md) describes the
separate source context. Request lifetimes no longer cross the callback ABI;
only owned completion state is registered. Raw environment ownership is reclaimed
once on refusal or committed delivery/cleanup. Displaced state and all arbitrary
Waker clone/drop/wake operations remain outside the completion mutex. The prior
implementation cloned/replaced Wakers while holding its state lock. This is a
deliberate reentrancy improvement, not a claim of exact internal equivalence.

Native all-target and Wasm library Runtime `cdk` Clippy pass with warnings denied.
Tests pass: 52 unit tests (including 16 new call tests), seven public call tests,
three CDK consumer tests, 13 executor consumer tests and two doctests. Public
cases originally passed both implementations; a supplemental Debug trait bound
was added after observing the earlier public future's trait and passes here.
The tests cover full-width costs/errors, failed-submission reclamation,
ordinary pending destruction, latest-Waker and reentrant wake/drop, multi-call
recovery, preservation of later tasks, saved migratory-task cancellation,
external synthetic cleanup and raw rejection bytes. Native tests consume each
mock callback environment once and never simulate rollback by reusing a pointer.
Arbitrary custom RawWaker clone behavior is inspected, not injected in tests.

All four installed profiles pass at `target/cdk-runtime.G67DHY`, status 0,
including the stronger two-call recovery test. Source hashes match after the
run. Final and baseline fixture manifests/sources, lock and compiler observations
match; only product implementation inputs differ. Feature closures pass for
17 selected combinations on each native/Wasm target at
`target/feature-graphs/attempt.FUAOgh`. Inventory, documentation links and retained
management/ICRC source/license guards and mutation-refusal tests pass.
Logs: `/tmp/platform-call-{all-clippy,all-tests,new-wasm-clippy,after-installed,feature-closures}.log`.

The old `cdk::upstream` directory and its call fingerprint scripts are removed.
The release guard invokes the still-required management checks directly; the
separately imported generator and ledger/management source obligations remain.
No dependency, feature, crypto implementation or release group changed.

| Profile | Before total | After total | Before code | After code |
| --- | ---: | ---: | ---: | ---: |
| Raw Wire | 228,455 | 229,405 | 152,268 | 152,874 |
| Raw external Candid | 678,172 | 678,638 | 475,268 | 475,778 |
| Prepared Wire | 231,875 | 231,382 | 154,485 | 154,412 |
| Prepared external Candid | 681,350 | 680,349 | 477,502 | 477,214 |

Results are mixed: raw totals grow 466–950 bytes; prepared totals shrink
493–1,001 bytes. The largest relative total increase is 0.42%. This replacement
is primarily source ownership and safer state custody, not a general size win.
Native x86_64 `size_of` observations show Call unchanged at 96 bytes and its
future increasing from 8 to 112 inline bytes. The request now lives in that
future, while callback state is allocated only on successful preflight during
first poll. Unpolled futures allocate no completion state. Inline size alone is
not total live memory: the prior owner stored its request inside an allocated
shared state. No heap-allocation totals or instruction-speed claim is made.

Final SHA-256 values:

```text
raw-wire ed017e12054a7d92bf8ce79bc3c478c49afb2738916101bc6d36faefaaa6585d
raw-external-candid 52583d88b71db4b79115577ca5cc99e4789c3db5473db4e9634abf34b32c227d
owned-wire ae0ae9f06e7dc38ca7e9a27b6f40e509aec2e9194e9b811c2907f51aa7d681fc
owned-external-candid 626e7d066f9b8b5e260e9a5693fe067dbd22ce90cae9b285eecf7b2c3fafa6c1
```

No fresh macOS or Scheduling-owned composition proof is claimed. First-party
license replacement, installed Toko's IC0 admission and the separately retained
ICP derivation draft remain release blockers. No commit, push or publication.
