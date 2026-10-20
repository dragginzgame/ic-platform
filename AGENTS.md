# IC Platform contributor instructions

Read [the current handoff](docs/status/current.md), then
[DRAGGINZGAME.md](DRAGGINZGAME.md). Shared revision
`d57c631206e3618755e45bebf2c88c673564ddc1` (Shared Tooling 0.25.1) is recorded in
[the snapshot](.shared-tooling.snapshot). Do not edit snapshot payloads.

- This repository owns the controlled Candid/CDK dependency boundary. Read
  [the migration contract](docs/migration.md) before changing its public surface.
- Keep one consumer library under `crates/ic-platform`. Add no blanket upstream
  re-exports, automatic resource limits or alternative executor implementation.
  Explicit upstream-coupled exports belong to the documented contract.
- No default features. Executor-only builds must remain Candid-free; host parser
  features must never be enabled by canister/runtime features.
- Storage implementation and allocation are out of scope. The reviewed CDK
  stable/checkpoint passthroughs preserve upstream bytes and semantics; they do
  not transfer storage ownership. Wire, Testkit, Host, Canic and Query
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

## Owned entrypoint generator

`ic-platform-macros` is the implementation proc-macro crate behind the sole
consumer library's optional `cdk` surface. The maintainer authorized maintaining
the generator independently of upstream acceptance. Preserve its imported
Apache-2.0 license (the necessary exception to the workspace's MIT metadata),
coordinated package version, original generator semantics and explicit paths.
The maintainer authorized adopting the reviewed non-cryptographic CDK 0.20.3
adapters inside Platform and removing the `ic-cdk` package. Keep their Apache-2.0
license/provenance. The facade's mixed-source license is MIT AND Apache-2.0.
Only the imported callback and legacy byte-copy modules permit reviewed unsafe
code; other modules retain the workspace deny lint. No executor is forked.
Preserve registry executor =2.0.0 and ic0 =1.2.0 identities. Endpoint macros require
explicit consumer-owned codecs; all owned application endpoints use Wire, including
public endpoints called by third parties. Candid is an explicit adapter for required
management/system, NNS/SNS, ledger and other pre-existing Candid interfaces. Caller
location or visibility never selects Candid. Existing Candid checkpoint
bytes are a legacy compatibility exception, never a new storage format or owner.
Runtime-only raw/executor consumers retain their independent features.

## Cryptographic boundary

Our team does not include cryptographers. Do not implement, replace or alter
cryptographic primitives or algorithms, including public-key derivation,
signature verification, cryptographic key encoding or hash/domain-separation
behavior. Retain upstream-authored cryptographic implementations and test vectors;
do not substitute the earlier public-derivation or BLS replacement prototypes.

Dependency scaffolding may be simplified: manifests, feature gates, unused
modules, duplicate generators and integration plumbing are in scope when they
preserve cryptographic behavior. Source variants must retain provenance/licenses
and unchanged cryptographic function bodies. Qualify their actual consumer graphs,
type/source identities and wire bytes; a dependency-local Cargo patch is not a
supported distribution mechanism for downstream consumers.

## Owned Principal

The maintainer authorized the Platform-owned Principal for the breaking 0.3 line.
Retain its Apache-2.0 provenance, raw bytes and length-first ordering. Keep parsing
bounded by the 63-byte text grammar, binary Serde marker-free and range policy
consumer-owned. Delegate self-authentication to the unchanged upstream crypto
implementation. External DTO/client principal and error identities remain registry
types, named explicitly through principal::ExternalPrincipal/ExternalPrincipalError;
Candid contract types stay external. Do not add Candid to the owned type.

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

## PocketIC maintenance boundary

The optional native-only `pocket-ic` feature owns explicit upstream client and DTO
identities. Testkit retains server provisioning, compatibility admission,
lifecycle, transport and retry policy. Keep PocketIC out of defaults, executor-only
and canister deployment graphs; do not replace its management DTOs with the
independently selected external management DTO surface.

## Owned account

The maintainer authorized unconditional owned Account/Subaccount alongside owned
Principal. Preserve effective-zero equality, ordering and hashing, and preserve
absent versus explicit zero fields during serialization and external conversion.
Retain Apache provenance and the existing external ledger DTO identity. No Candid,
storage implementation, cryptographic account-identifier derivation or application
subaccount allocation policy belongs in the owned account surface.
