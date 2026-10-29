# IC Platform contributor instructions

Read [the current handoff](docs/status/current.md), then
[DRAGGINZGAME.md](DRAGGINZGAME.md). Shared revision
`5683060a5d8ce79af2fcfedb53b3eb24037fd064` (Shared Tooling 0.26.26) is recorded in
[the snapshot](.shared-tooling.snapshot). Do not edit snapshot payloads.

- This repository owns the controlled Candid/CDK dependency boundary. Read
  [the migration contract](docs/migration.md) before changing its public surface.
  Keep [the IC inventory](docs/ic-inventory.md) aligned with dependency, feature,
  export and ownership changes; `make check-inventory` checks manifest facts.
- The maintainer authorized `crates/ic-call` as the additional prepared one-shot
  ICC library for #75. It depends downward on `ic-platform` and reuses its sole
  call implementation and executor. It owns method/argument custody, not codecs,
  retries, Relay delivery, timers, domain endpoints or application policy.
- Keep the low-level consumer boundary under `crates/ic-platform`. Add no blanket upstream
  re-exports, automatic resource limits beyond the authorized Candid ceiling below,
  or alternative executor implementation.
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
- The maintainer has deferred further hard cuts
  until Canic is live, except the explicitly authorized coordinated owned-endpoint
  Wire migration in docs/migration.md. Retire its consumed compiler/service
  exports only after consumer adoption; preserve required external Candid paths.
- The authorized #38 compiler-only retirement shipped in 0.7.0 after the Canic
  Wire caller/qualification audit. Do not restore the JS/TS compiler facade
  exports or exhausted merged-program helper; retain external parsing/service
  operations. That narrow cut does not authorize other hard cuts or a parser fork.
- Rust 2024 and MSRV 1.91.0. Inherit workspace metadata and dependencies; ordinary
  directory modules only. Do not add Python tooling.
- GitHub issues are the follow-up tracker once a remote exists. Migration and
  provenance documents specify contracts and evidence, not a second issue queue.

## Browser SDK boundary

The maintainer authorized the separate ESM npm package under `packages/browser`
alongside the sole Rust consumer library. It owns SDK Core dependency selection
and shared non-cryptographic integration for Blob and Canic; it is not a second
Wire generator, storage owner, authentication flow or application policy owner.
Target Blob's approved local 0.48 graph with SDK Core 6.1.1. Preserve upstream classes/type identity,
cryptographic implementations and external bytes; expose explicit subpaths only.
Principal text admission, checked identity restoration and Agent construction
plumbing may be owned here. Consumers retain trust roots, budgets, retries,
intent journals, authorization and release/schema binding. Never infer update,
polling or resubmission permission from a transport response.

Use `make check-browser` for strict TypeScript, adapter and packed-consumer proofs;
`make check-browser-runtime` additionally requires explicitly prepared Chromium.
Keep real browser/worker evidence separate from installed IC/provider acceptance.
SDK Core is an exact ordinary npm dependency; consumer adoption waits for a
published package. Existing SDK peers stay until then. Rust `make publish` remains
crates.io-only; npm release/publication is a separate maintainer action.
Caffeine 1.1.2 still declares SDK 5; its SDK-6 override belongs to the consuming
application. Auth 10.0.2 is a compatible SDK-6 type fixture, not Canic adoption.

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

The maintainer authorized #72's narrow DER admission exception: one owned envelope
check precedes the unchanged imported canister-key parser. The source guard admits
only that exact call at parser entry, then reconstructs the original fingerprints.
Encoding, cryptographic bodies, constants and upstream vectors remain unchanged;
this authorizes no other key-format changes or fingerprint refreshes.

The maintainer authorized the optional BLAKE3 maintenance surface with explicit
upstream `hash`, `Hasher` and `Hash` identities. Keep the implementation upstream,
select only `std`, and qualify singleton/source and effective-feature graphs.
Consumers retain hash-format/domain policy; no existing SHA-256 contract changes.

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

The maintainer explicitly authorized transferring IC Profiler's native provider,
including its existing assembler/fixtures, into `native/ic-profiler`. That opt-in
source profile owns replica adapters and the patched client below this facade;
its IC-free recorder/viewer remain consumer-owned. Preserve the legacy tooling
during this transfer; this is no general permission to introduce Python tooling.
Keep Motoko/lifeline outside the admitted native server graph. Follow the source
profile contract in `native/ic-profiler/README.md` and preserve IC copyrights.

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

## External Candid nesting admission

The maintainer superseded #73's draft opt-in policy with one fixed admission rule:
Platform's bounded Candid decoders and external endpoint adapters reject more than
16 constructed wire levels before upstream decoding. This is the explicitly
authorized resource-policy exception; keep existing byte/work/header/type budgets,
function signatures and closed error variants. Cap admission work using the
saturating sum of existing decoding/skipping quotas, with no new policy object or
macro option. Inspect unused type definitions and skipped values, retain bytes,
and fail closed on uninspectable forms. Low-level upstream APIs retain their
behavior; callers may run the same check explicitly. This is a bounded framing
walk, not a Candid fork or universal stack-safety claim for arbitrary trusted Rust
schemas/custom visitors. Keep installed Wasm evidence at 16/17 and the original trap.

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
