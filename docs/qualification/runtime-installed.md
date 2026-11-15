# Installed update and callback rollback — Runtime 0.23.4

This probe runs the candidate's generated update exports in PocketIC 16.1.0 on
an application subnet. It covers immediate update rollback and two resumed-handler
trap/cleanup paths, not the complete Runtime acceptance gate.

## Ownership and setup

`qualification/runtime-rollback` builds one private Wasm canister using the
candidate Runtime, guard, macros and Types. Its graph check requires exactly one
of each and excludes Testkit and the published runtime. No historical canister
source or generated fixture was copied.

The native driver is the `qualification/runtime-installed` member of the
standard qualification workspace. It selects published `ic-testkit` 0.106.0
and uses its upstream PocketIC client re-export. Testkit owns checksum-verified
server setup, offline admission and managed process teardown. The driver owns
one instance with two canisters and explicitly closes it after the checks. Failure drops the owned
server, stopping its disposable instance. No public network canisters are used.

The product catalog pins `thiserror = "=2.0.18"`, matching PocketIC 16.1.0, at
the maintainer's direction. Product and qualification locks select that same
version. Native adapter identities never cross into the canister graph; the
driver passes raw bytes. There are only the product and qualification workspaces,
with the same exact Toko version and shared formatting/dependency commands.

Prepare locked sources with
`cargo fetch --locked --manifest-path qualification/Cargo.toml`,
then run `make install-runtime-server` explicitly. `make runtime-server-check`
verifies the installed bundle offline. `make qualify-runtime-installed` performs
strict host/fixture Clippy, builds the Wasm with the existing fixed build settings,
runs the driver and checks its production graph. Tests do not download a server.
CI uses these same targets on each configured native host.

## Observed contract

The canister has one byte of thread-local state and a raw one-byte command.
Its generated export uses explicit bounded decoding and manual replies. The
handler also verifies that a second ordinary reply is refused.

| Update | Expected committed result |
| --- | --- |
| Read initial state | Reply `[0]` |
| Increment and reply | Reply `[1]` |
| Increment, then panic before reply | `CanisterTrapped`, no reply |
| Read after initial trap | Reply `[1]` |
| Increment, reply, then panic | `CanisterTrapped`, no reply |
| Read after post-reply trap | Reply `[1]` |
| Increment after both failures | Reply `[2]` |

Rust's Wasm panic executes `unreachable`; the expected IC error is
`CanisterTrapped`, not `CanisterCalledTrap` (an explicit IC trap). Both failures
must be typed execution rejections. A rejection alone does not establish rollback:
the subsequent reads assert the committed state, and successful updates verify
continued admission and usable reply authority.

## Awaited callback rollback and cleanup

The driver installs the same candidate artifact into a separate peer canister.
The caller's `await_probe` receives an explicit mode and the peer's Principal
bytes, admits them through the generated wrapper, increments a started counter,
and awaits an unbounded raw call to the peer's incrementing `probe` method.
The successful callback returns the peer's exact one-byte reply.

Four awaited calls exercise success, trap immediately after await, trap after
the resumed handler issues its reply, and success after both failures. After
each call, independent updates read caller counters and the peer's committed
state. Expected observations are:

| Mode | Caller started/completed/dropped | Peer state |
| --- | --- | --- |
| Success | `[1, 1, 1]` | `[1]` |
| Trap after await | `[2, 1, 2]` | `[2]` |
| Trap after callback reply | `[3, 1, 3]` | `[3]` |
| Subsequent success | `[4, 2, 4]` | `[4]` |

The completed counter is written before either callback trap and must roll back.
The started counter commits before suspension and remains. A destructor held
across suspension increments the dropped counter and attempts an ordinary reply:
normal completion must refuse with `AlreadyReplied`, while cleanup must refuse
with `NoActiveTask`. The restored future is therefore destroyed exactly once
in these cleanup paths. Both trapping callbacks return `CanisterTrapped`; the
post-reply trap must not deliver its rolled-back reply. The peer's increment
survives each caller trap, demonstrating that caller rollback does not undo
already committed remote effects.

These are real replica-simulator callbacks and cleanup invocations, not direct
host calls into function-table entries. They qualify this single-outstanding-call
path; no lifecycle-handle observation or cleanup-failure recovery is inferred.

Local Linux x86-64 validation passed seven immediate updates, four awaited calls
and eight observation updates with Rust 1.100.0,
Testkit 0.106.0 and its admitted PocketIC 16.1.0 server. Strict host/fixture Clippy
and the Wasm graph check passed. Hosted/macOS execution remains pending.

Installed artifact SHA-256:
`a6a251dd50ca3eb310bbfded1179a5aad7fe7d288ceb5e8c6fe151013fe9912e`.

## Remaining gates

Cleanup failure, refunds, orphan callback custody, bounded-wait uncertainty,
upgrade reconstruction and context isolation still need installed evidence.
This is not an old/new performance comparison or proof of the full design gate.
