# Native PocketIC client

Maintained PocketIC 16.1.0 Rust client with management-canister types 0.11.
Native consumers may depend on `ic-platform-pocket-ic` directly and import
`ic_platform_pocket_ic::{PocketIc, PocketIcBuilder}`. The optional
`ic_platform::pocket_ic` facade re-exports the same client and Rust identities.
This package releases independently with `RELEASE_GROUP=pocket-ic`.
Dependencies remain native-only; keep it outside canister deployment graphs.
The simulator server and IC execution engine remain external.
Testkit retains server setup, compatibility admission and lifecycle policy.
See [provenance](upstream/NOTICE.md).

The pending breaking cut returns `Result<_, ClientError>` from network operations
and builder execution. Both clients preserve transport/body/decode sources,
server diagnostics and accepted operation IDs. Match `ClientError::Canister`
for canister rejections. Do not infer retry permission from an uncertain result.
Request budgets cover dispatch, body reads and polling using a monotonic clock.
Explicit `drop_and_take_state(&mut self)` retains state on deletion failure;
destructor cleanup does not repeat that failed attempt.
