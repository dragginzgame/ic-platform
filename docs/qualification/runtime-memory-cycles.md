# Synchronous stable memory and cycle observations

The Runtime 0.23.13 candidate extends the existing dependency-free `system-raw`
boundary for Scheduling issues [#14](https://github.com/dragginzgame/ic-platform/issues/14)
and [#15](https://github.com/dragginzgame/ic-platform/issues/15).
No allocator, checkpoint format, timer policy or executor is added.

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
