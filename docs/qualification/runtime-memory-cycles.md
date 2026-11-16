# Synchronous stable memory and cycle observations

The Runtime 0.23.13 candidate extends the existing dependency-free `system-raw`
boundary for Scheduling issues [#14](https://github.com/dragginzgame/ic-platform/issues/14)
and [#15](https://github.com/dragginzgame/ic-platform/issues/15).
No allocator, checkpoint format, timer policy or executor is added.

## Published replicated-execution observation

[Issue #19](https://github.com/dragginzgame/ic-platform/issues/19), discovered during
Scheduling's Entropy integration, is supplied locally by
`raw::in_replicated_execution() -> bool` under `system-raw`. It forwards the IC
scalar directly, without allocation, root admission, source mutation or a call.
Entropy's `require_replicated` maps its old scalar `!= 0` test to this boolean and
retains `Error::NonReplicated` before source claim or transport creation.
The native Entropy branch remains its existing typed refusal; raw is Wasm-only.

Ordinary non-replicated queries observe false. Updates and replicated queries
observe true, following the IC rather than inferring an export kind. This is not
update authentication, reply authority, freshness, authorization or permission
to perform a context-restricted operation. IC context restrictions remain intact.

The focused raw qualification passes on Linux: 17 exact IC imports, 14 independent
ABI cases, strict Clippy/docs, zero-dependency product/fixture graphs and installed
ordinary-query/update/replicated-query checks. Repeated ordinary queries refuse
before the fixture's custody counter changes; an update changes it exactly once.
Existing stable-memory, cycle-burn, rollback and stopped-upgrade cases still pass.
Wasm SHA-256: `ec24f75d27b76ab8b89ea137518c70798dc4c86c01ad1116b246ac296ade1576`.
Evidence: `target/runtime-context-qualification.log`.
Actual Entropy source and installed lifecycle migration remain with its read-only
consumer; Runtime 0.23.14 is published; current actual-consumer evidence follows below.

## Published memory and cycle boundary

The ABI contract is the original
[IC canister interface](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/).
The implementation consists of safe slice wrappers around the existing approved,
item-scoped IC import boundary; no upstream implementation was copied.

`raw::stable64_size`, `stable64_grow`, `stable64_read` and `stable64_write`
preserve 64-bit offsets and page counts. Growth returns `Option<u64>`: `None`
represents the IC's failure sentinel, and `Some` contains the previous page count.
Reads require initialized destination slices. Empty operations are forwarded;
they do not establish that a nonempty range at that offset is valid. IC traps
retain whole-message rollback. There is no recovery or persistence policy here.

`raw::liquid_cycle_balance` observes the liquid u128 balance.
`raw::cycles_burn` accepts u128 and returns the actual IC-reported burn, which
can be smaller than requested. Freezing reserves and execution-context restrictions
remain the IC's responsibility. Callers own whether and how much to burn.
`Call::required_cycles` remains the submission-cost quote for Scheduling's
self-call; it must be constructed with the actual method and 16-byte argument.

`raw::cost_http_request_v1` supplies the original version-one IC HTTP quote for
Entropy's held-response probe. Its request-byte input is the IC chargeable
request size, not the encoded Candid argument length. Pair this quote with
Protocol's explicit version-one encoder; do not silently use current/default
pricing with a version-one quote.

## Focused proof

`make qualify-runtime-raw` checks the exact import set, safe wrapper lints and
public documentation, plus the actual installed PocketIC fixture. The installed
checks exercise new-page zeroing, a cross-page write, impossible growth, empty
operations, nonempty high-offset refusals, u128 burns above u64, clipped burning,
and monotonic version-one HTTP quotes. A deliberately trapping update grows
memory, overwrites bytes and burns more than u64: all three effects roll back.
The fixture stores and restores its value through pre/post-upgrade stable memory
under stop/observe-Stopped/upgrade/start. Raw-only production graphs have no
dependencies and no executor.

This is supplier evidence. The read-only Scheduling baseline
`3d2b3fd3775252e779181c4175a8ecf661afb60b` still needs migration of its actual
checkpoint and refusal callers, including its timer transport. The new argument
order is `stable64_read(offset, destination)` / `stable64_write(offset, source)`;
its old grow assertion must expect `Some(0)`. The once/watchdog repeated-refusal,
funding and exact-once completion scenarios remain separate consumer gates.
No registry delivery, Scheduling production migration or macOS proof is claimed.

## Actual consumer acceptance — 16 November 2026

Read-only inspection of clean Scheduling commit
`fc5a4f16c3be0b48cec0532b0d091192ace8c039` establishes the retained
consumer acceptance for [Platform #14](https://github.com/dragginzgame/ic-platform/issues/14),
[Platform #15](https://github.com/dragginzgame/ic-platform/issues/15) and
[Platform #19](https://github.com/dragginzgame/ic-platform/issues/19).
The consumer commit is local and unpushed at this observation; its delivery
and broader release gates are separate. Earlier pending statements above
describe the prior supplier-only evidence.

Scheduling's maintained Runtime migration passes the actual Once/watchdog/Jobs
checkpoint, upgrade, unresolved custody, retry, cancellation and catch-up suite.
Both real cycle-refusal cases preserve repeated backoff without acceptance, then
complete exactly once after funding (actual quote 42,102,448,000 cycles).
Its pinned Entropy 6c400b2 adapter uses the public boolean and retains typed
query refusal before claim/submission. All 24 native tests and the installed
background_refill_and_lifecycle pass, including suspension, upgrade and reinstall.
No guard bypass, sibling edit or new executor was introduced here.

Reviewed Scheduling evidence under target/evidence/platform-02313:

| Log | SHA-256 |
| --- | --- |
| installed-final.log | 8bba30333620002a913fd68df099d250550ff0839cdac8035f4e05f28db2875d |
| refusal-fixed.log | c0247feb577a6e3aa85d51587b13a7af4a9d1fd458a12d0e0ab37ef2d2e1f72a |
| entropy-02314-installed-fixed.log | 9f4946393579b86fdae8ead20cdcfa0864e1eb27ea73c28d228c3abc2230b0ff |

The maintained Entropy patch SHA-256 is
`2b663d050c2271370b78a0b3458e68bc972a43f0688016584262939ee91c0d64`.
Registry Runtime 0.23.13 provided the memory/cycle proof; 0.23.14 adds only
the context observation and supplies the actual Entropy proof.
