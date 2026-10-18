# IC Platform contributor instructions

Read [the current handoff](docs/status/current.md), then
[DRAGGINZGAME.md](DRAGGINZGAME.md). Shared revision
`cab12cd3dfa4fdb34846b25397f0d72881bb9ae0` (Shared Tooling 0.24.0) is recorded in
[the snapshot](.shared-tooling.snapshot). Do not edit snapshot payloads.

- This repository owns the controlled Candid/CDK dependency boundary. Read
  [the migration contract](docs/migration.md) before changing its public surface.
- Keep one consumer library under `crates/ic-platform`. Add no blanket upstream
  re-exports, automatic resource limits or alternative executor implementation.
  Explicit upstream-coupled exports belong to the documented contract.
- No default features. Executor-only builds must remain Candid-free; host parser
  features must never be enabled by canister/runtime features.
- Storage and allocation are out of scope. Wire, Testkit, Host, Canic and Query
  retain their product responsibilities; existing siblings remain read-only.
- Preserve dirty work and check for Cargo/rustc processes sharing target/ before
  editing or compiling. Use focused checks from README; full CI is release-owned.
- Rust 2024 and MSRV 1.88.0. Inherit workspace metadata and dependencies; ordinary
  directory modules only. Do not add Python tooling.
- Ordinary implementation leaves staging, commits, release versions, publication
  and remote creation to the maintainer. Initial local repository creation is
  authorized; no remote, commit or publication is implied.
- GitHub issues are the follow-up tracker once a remote exists. Migration and
  provenance documents specify contracts and evidence, not a second issue queue.

## Executor identity

The published boundary constrains `ic-cdk-executor` to `=2.0.0` to preserve
Timers' existing qualified callback/trap identity in downstream resolution.
Any change requires renewed combined CDK/Timers await, callback-trap,
cancellation and upgrade qualification; a lockfile-only check is insufficient.
See [the executor constraint issue](https://github.com/dragginzgame/ic-platform/issues/7).
The optional `ic0` boundary similarly preserves `=1.2.0` and upstream safe/unsafe
signatures. It installs no timer entrypoint and transfers no storage ownership.
Changes require renewed Timers submission/callback/trap/upgrade qualification.

## Management contract

The optional management-canister DTO boundary uses `=0.11.0`, as explicitly
requested by the maintainer. Preserve upstream type identities and external
Candid bytes, including HTTP pricing/replication fields. Changing this constraint
requires renewed consumer DTO/wire qualification; do not replace it with a
lockfile-only constraint or a blanket upstream export.
