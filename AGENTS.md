# IC Platform contributor instructions

Read [the current handoff](docs/status/current.md), then
[DRAGGINZGAME.md](DRAGGINZGAME.md). Shared revision
`dc1ced7e240e90f0c6daa89fb6248fcb074164eb` (Shared Tooling 0.26.4) is recorded in
[the snapshot](.shared-tooling.snapshot). Do not edit snapshot payloads.

- This repository owns the controlled Candid/CDK dependency boundary. Read
  [the migration contract](docs/migration.md) before changing its public surface.
- Keep one consumer library under `crates/ic-platform`. Add no blanket upstream
  re-exports, automatic resource limits or alternative executor implementation.
  Explicit upstream-coupled exports belong to the documented contract.
- No default features. Executor-only builds must remain Candid-free; host parser
  features must never be enabled by canister/runtime features.
- Storage implementation and allocation are out of scope. The reviewed CDK
  stable passthroughs preserve upstream bytes and semantics; they do
  not transfer storage ownership. Wire, Testkit, Host, Canic and Query
  retain their product responsibilities; existing siblings remain read-only.
  The authorized #45 manager exports retain upstream 0.7.2 identities solely for
  Memory infrastructure; allocation, checked reopening and leases remain there.
- Preserve dirty work and check for Cargo/rustc processes sharing target/ before
  editing or compiling. Use focused checks from README; full CI is release-owned.
- Run full `make clippy` (all targets/features, warnings denied) before longer
  tests, as requested by the maintainer. Fix failures before proceeding.
- Keep 0.6.2 backward-compatible. The maintainer has deferred further hard cuts
  until Canic is live; do not remove APIs, features or supported consumer paths.
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
The imported callback module permits reviewed unsafe code. The maintainer also
authorized the function-local `msg_arg_data` exception: set the vector length
only after pinned ic0's all-or-trap uninitialized-buffer copy returns.
Other functions retain the workspace deny lint. No executor is forked.
Preserve registry executor =2.0.0 and ic0 =1.2.0 identities. Endpoint macros require
explicit consumer-owned codecs; all owned application endpoints use Wire, including
public endpoints called by third parties. Candid is an explicit adapter for required
management/system, NNS/SNS, ledger and other pre-existing Candid interfaces. Caller
location or visibility never selects Candid. Existing Candid checkpoint
bytes are a legacy compatibility exception, never a new storage format or owner.
Runtime-only raw/executor consumers retain their independent features.

## Cryptographic boundary

For the authorized 0.6 packaging cut, unused creation delegation/CBOR helpers
compile only in tests. Preserve their complete bodies and vectors. The source
guard checks both exact annotated source and reconstruction of the 0.5.1 source
after removing only eight admitted attributes. No algorithm changes are allowed.

The maintainer authorized the 0.5.1 deletion of the unused, unexported
`IC_ROOT_PUBLIC_KEY` lazy cache and its import/dependency. This narrow scaffolding
exception does not authorize changes to root-key bytes, DER extraction,
cryptographic function bodies or test vectors. The creation-source fingerprint
records only that deletion; the remaining source/license guards stay exact.

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

The maintainer authorized the 0.4 signature packaging migration: retain the
upstream creation/verifier bodies and vectors inside `signature_upstream`, changing
only the documented imports and omitting the unexposed store module. Enforce exact
source/license fingerprints; do not refresh hashes to permit algorithm changes.
The owned runtime RejectCode similarly replaces only its external catalogue
dependency; Agent/PocketIC codes keep their external identities.

## Owned Principal

The maintainer authorized the 0.6 feature split: `principal` selects the owned
Principal/Account family and external conversions. No default features. Runtime,
Candid, signature creation and external DTO/client profiles select it explicitly;
raw executor/IC0-only profiles exclude application types and their dependencies.
This supersedes the earlier unconditional availability decision.

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

Owned Account/Subaccount accompany Principal under the `principal` feature.
Preserve effective-zero equality, ordering and hashing, and preserve
absent versus explicit zero fields during serialization and external conversion.
Retain Apache provenance and the existing external ledger DTO identity. No Candid,
storage implementation, cryptographic account-identifier derivation or application
subaccount allocation policy belongs in the owned account surface.

## Transport and codecs

The 0.4 runtime accepts/returns raw transport bytes only. Do not restore implicit
Candid request/response helpers or a combined transport/codec error union. Use the
separate Candid boundary for required external IC interfaces; consumers retain
encoding-failure, decoding, arity, resource, diagnostics and retry policy.
The maintainer authorized retiring unused `cdk::storage` helpers in the 0.5 hard
cut. Do not restore a compatibility owner. Raw stable passthroughs remain;
existing checkpoint formats and migrations belong to their consumers. The
installed fixture alone retains explicit legacy-byte qualification.
The 0.5 cut also retires the no-op `hidden` entrypoint option and replaces
`Call::change_timeout` with explicit `WaitPolicy` selection. Keep the Syn-based
option parser's ordered guards and strict duplicate/type errors; do not restore
Darling solely for compatibility. Submission, callback and cleanup source
fingerprints remain unchanged.
