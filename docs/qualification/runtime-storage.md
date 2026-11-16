# Storage synchronous system boundary — issue #6

Runtime 0.23.11 adds `system-raw`, exposing `ic_platform_runtime::raw` on
`wasm32`. This is a dependency-free synchronous surface for caller-owned IC
exports. It selects neither Types, the Runtime singleton guard, macros, nor the
executor. Default features stay empty. Combining it with other Runtime features
adds their independently declared dependencies; raw operations never initialize
an executor or callback registry.

## Migration

```toml
ic-platform-runtime = { version = "=0.23.11", default-features = false, features = ["system-raw"] }
```

Build canisters for `wasm32-unknown-unknown`. Replace `use ic_platform_runtime::ic0`
with `use ic_platform_runtime::raw as ic0`. Existing synchronous argument/caller
size and copy, controller checks, append/reply and byte-trap calls keep their
signatures. Replace `ic0::performance_counter(0)` with
`ic0::instruction_counter()`. The API intentionally selects only counter zero.

For example, in a caller-owned update export:

```rust,ignore
use ic_platform_runtime::raw as ic0;

fn respond() {
    let size = ic0::msg_caller_size();
    assert!(size <= 29);
    let mut caller = [0; 29];
    ic0::msg_caller_copy(&mut caller[..size], 0);
    if ic0::is_controller(&caller[..size]) != 1 {
        ic0::trap(b"controller required");
    }
    let mut input = [0; 8];
    assert_eq!(ic0::msg_arg_data_size(), input.len());
    ic0::msg_arg_data_copy(&mut input, 0);
    ic0::msg_reply_data_append(&input);
    ic0::msg_reply();
}
```

Raw reply calls do not participate in executor reply ownership. Choose one reply
owner per call; do not mix them with automatic endpoints or managed replies.
The consumer owns authorization, buffers, offsets, exports and upgrade hooks.
No codecs, futures, persistent state, allocation or callbacks are introduced.
The version above is a source candidate until registry publication completes.

## Context and memory contract

The [IC system API specification](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/)
is the authority for valid execution contexts. Copies write only the supplied
initialized mutable slice; pointers are not retained. Wasm32 offsets and lengths
are forwarded without narrowing. The IC checks source bounds, including
zero-length copies, and traps rather than returning truncated or partial data.
Caller bytes must form a valid Principal for `is_controller`; membership is
queried anew on each invocation. No Rust Principal type crosses this API.

Arguments are available only in argument-bearing contexts, excluding pre-upgrade;
caller observations are available in caller-bearing contexts, including the
qualified init/pre/post-upgrade hooks. Replies require a reply-capable context
and no previous reply/rejection. Empty replies need no append. A trap after
appending or completing a reply still rolls back the current message's reply
and state. Earlier committed messages and external effects are not rolled back.
Diagnostic bytes are forwarded unchanged. Performance counter zero reports IC
instructions for the current execution, not time or a stable cross-version cost.

Imports and scoped unsafe expectations remain in the approved `call/ic0`
boundary. The public functions themselves expose safe slices/scalars. Existing
executor system APIs keep their task-authority semantics unchanged.

## Focused evidence — 2026-11-16

`make qualify-runtime-raw` runs Wasm Clippy/docs, the caller-owned export fixture,
a 12-case independent ABI harness and installed PocketIC checks. The ABI harness
admits exactly the nine required imports and verifies nonzero offsets, empty
copies/appends/replies, unsigned maximum offsets, caller lengths 0/1/29,
unchanged diagnostic bytes and fixed counter selector zero. Installed cases prove:

- Exact concatenated reply bytes and an empty reply without a task.
- Argument/caller out-of-range refusal, including empty copies with invalid offsets.
- Duplicate replies and appends after completion refuse.
- A deliberate byte trap after mutation and completed reply leaves the prior value.
- Query/update caller bytes, controller membership and advancing instruction counts.
- Live controller replacement revokes the previous caller and admits the new one.
- Init/pre/post-upgrade observations under stop → Stopped → upgrade → start.

Production graph checks admit only Runtime; the raw Wasm fixture adds only itself.
All-features Runtime Wasm Clippy separately checks feature composition.

`make qualify-storage-runtime` archives read-only Storage source
`3ad8beda21eeb9451c7755791648b2a5cbffb0c1`, applies the reviewable
`qualification/storage-runtime/migration.patch` in ignored storage, and compiles/
lints its actual startup (`managed-bench`), checkpoint (`native-job`) and published-
Storage benchmark Wasm callers. The patch changes dependency selection, imports
and counter spelling; explicit workspace paths keep the archived packages bound
to their own workspace. It does not modify the sibling checkout or its protocols.
Storage's published `ic-storage` 0.74.1 oracle still brings legacy Runtime 0.22.5;
this test does not claim that legacy edge has been removed.

Registry publication, Storage's own installed startup/retention/checkpoint/job and
benchmark reruns, and downstream adoption remain separate. These local checks
prove the supplier surface and consumer compilation, not those consumer results.
