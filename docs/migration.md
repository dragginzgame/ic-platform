# Candid and runtime ownership

IC Platform is the selected owner of Candid and CDK integration. Consumers depend
on this package for the capabilities they need. IC Wire retains controlled
application protocols; Storage and Memory work is explicitly deferred.

## Initial implementation

The `candid` feature adopts `ic-testkit-candid`'s three decoders and their tests.
Arity, byte/work/header bounds, compact diagnostics and typed sources are retained.
The `contract` module explicitly exports the traits and derive machinery required
by external DTOs, Serde, Principal and the two encoders. This surface is knowingly
upstream-coupled; no claim of codec implementation independence is made.
The `executor` feature selects upstream task/context primitives directly, preserving
one executor identity without importing Candid. Runtime consumers must continue
to agree on that executor identity across entrypoints, calls and timers.

## Testkit adoption contract

After Platform is published and qualified, Testkit and Query select its `candid`
feature and replace `ic_testkit_candid` imports with `ic_platform::candid`.
Move direct Candid encoding/derive uses to `candid::contract`; use `ic_principal`
for identity-only code. Keep PocketIC calls, lifecycle and transport errors in
Testkit, and NNS/SNS/ledger schemas and operations in Query. Delete the old
`ic-testkit-candid` package and publication entries in that coordinated hard cut;
do not keep a compatibility re-export package. Existing consumers have not been
edited or migrated by creation of this repository.

The native integration fixture exercises DTO derives and argument decoding through
the boundary. A disposable standalone consumer with only an `ic-platform` dependency
must also compile: an integration test alone still has access to package dependencies.
Retain malformed input, quota, trailing-byte, tuple-vs-argument and typed-source cases.

## Other boundaries

Host's pending PR #189 decoder is held for consolidation. Its caller-owned aggregate
byte debit is distinct from Testkit's per-reply decoding and must not be lost or
silently introduced into existing call contracts. Host's released service parser
can move here behind a host-only feature; Host keeps the necessary projection
into its Wasm export identities. Platform must not depend back on Host or Testkit.

Canic's generators emit direct Candid paths. Upstream CDK macros also emit Candid
paths even when their runtime crate path is changed. The full runtime migration
requires real entrypoint fixtures, raw calls, management calls, callback contexts
and trap cleanup qualification; it is not solved by blanket crate re-exports.
Timers keeps scheduling and lifecycle policy and can adopt the executor surface
only after combined runtime qualification. No second executor or scheduler is added.

Shared's canonical dependency-boundary checker enforces package names despite
aliases across normal/build/dev/target dependencies. Each migrated workspace must
ban direct Candid/parser/CDK dependencies except this boundary package. Enumerate
independent fixture workspaces and check unused root catalog declarations too.
Agent, PocketIC and protocol type dependencies may retain Candid transitively;
inspect selected graphs separately from direct-manifest checks. Do not claim a
Candid-free artifact without inspecting and qualifying that artifact.

External protocol bytes, schemas, persisted records and retry identities remain
unchanged by dependency relocation. Protocol changes for owned endpoints are
separate Wire cutovers. Native Linux and both macOS hosts, Wasm compilation,
installed IC behavior and measured size are separate evidence categories.

## Source provenance

The initial decoder and tests were copied from the available Testkit working tree
at HEAD `55ac397` on 2026-10-18. Source SHA-256 before adaptation:

- `crates/ic-testkit-candid/src/lib.rs`: `93d20eb5c28391f12df274993adbdc2de2ec2480175ff39cafa4b27bbe86996f`
- `crates/ic-testkit-candid/src/tests.rs`: `d69ba1128ee5ccc1d09c9ebbe6697089ff990f1cee18f65100c24f9a30d34b6c`

Working-tree provenance does not claim publication. Original MIT notices are
retained. The local release adapter follows Storage's thin consumer adapter;
Shared owns version rewriting, source checks and release execution.
