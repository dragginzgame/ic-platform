# IC Platform contributor instructions

Read [the current handoff](docs/status/current.md) and the installed
[baseline](DRAGGINZGAME.md). [.shared-tooling.snapshot](.shared-tooling.snapshot)
records the installed revision; do not edit its payloads. Shared 0.31.0 at
`74313941af2fce4399bec0e6909ca6229673c3f8` is the adopted baseline.

- This repository owns the controlled Candid/CDK dependency boundary. Read
  [the migration contract](docs/migration.md) before changing its public surface.
  Keep [the IC inventory](docs/ic-inventory.md) aligned with dependency, feature,
  export and ownership changes; `make check-inventory` checks manifest facts.
- Runtime owns prepared one-shot calls and method/argument custody; the facade
  delegates to that implementation. There is no separate `ic-call` package.
  Codecs, retries, Relay delivery, timers and application policy stay consumer-owned.
- Keep Types, Runtime and facade ownership explicit. Add no blanket upstream
  re-exports, automatic resource limits beyond the authorized Candid ceiling below,
  or alternative executor implementation.
  Explicit upstream-coupled exports belong to the documented contract.
- Generic gzip codecs belong to IC Tooling. Consumers use its Host Artifacts
  API directly; keep Platform free of the forwarding surface and dependency.
- No default features. Executor-only builds must remain Candid-free; host parser
  features must never be enabled by canister/runtime features.
- The maintainer moved Principal/Account Wire schema integration to Wire. The
  published Platform facade must not depend on Wire or implement its schema
  trait. Keep Principal/Account bytes and Serde here; Wire remains tooling-only.
- Storage implementation and allocation are out of scope. The reviewed CDK
  stable passthroughs preserve upstream bytes and semantics; they do
  not transfer storage ownership. Wire, Testkit, Host, Canic and Query
  retain their product responsibilities; existing siblings remain read-only.
  The maintainer retired the unused stable-structures facade and dependency;
  allocation, checked reopening and leases remain consumer-owned.
- Preserve dirty work and check for Cargo/rustc processes sharing target/ before
  editing or compiling. Use focused checks from README; full CI is release-owned.
- Run affected-package Clippy before longer relevant tests; fix warnings in that
  selection. Full-workspace Clippy follows the shared broad-validation boundary.
- The accepted Rust SDK ownership and Runtime/Types splits are current. Earlier
  Canic-dependent hard-cut deferrals are superseded for those approved changes.
  Keep retired compiler/JS/TS facade exports removed; new product changes retain
  their own scope and qualification under [the migration contract](docs/migration.md).
- Rust 2024 and minimum/development/release compiler 1.99.1 (#133). Read the
  workspace floor and pinned toolchain; qualify each distinct graph once on
  this compiler, preserving native hosts and installed-runtime obligations.
  Inherit workspace metadata and dependencies; ordinary
  directory modules only. Do not add Python tooling.
- GitHub issues are the follow-up tracker once a remote exists. Migration and
  provenance documents specify contracts and evidence, not a second issue queue.

## Test source layout

Follow Shared's `tests.rs`/`tests/` layout for owned unit tests. Integration tests
already live under crate `tests/` directories, and doctests stay beside their APIs.
Imported ICRC test bodies retain their
upstream layout where provenance/reconstruction guards bind their exact source.
Those inline tests are deliberate exceptions; changing
their protected layout requires a separately reviewed reconstruction adaptation.
Owned tests beside imported adapters follow the ordinary layout when outside
protected regions, including CDK call policy and System API text tests.

The generated internal spawn retains its scoped `disallowed_methods` allowance:
consumer Clippy configurations may forbid the method or omit that lint entirely,
so one expectation would fail for supported consumers. Keep its explicit reason.

The opt-in native Profiler provider uses manifest source-prefix mappings for
ordinary Rust module discovery while preserving recorder/viewer locations.
See [the paired native qualification](docs/audits/native-profiler-140-2026-11-05.md).
Its test/layout/lint edits require native acceptance before adopting a new
provider pin; root crate checks do not qualify the IC workspace.

## Recovery evidence custody

The maintainer approved moving the historical contribution archive and bundles
to ignored `recovery-evidence/retired-checkouts/20261104-issue139/`. Preserve them;
checksums and relocation receipts retain their original custody. The Platform
maintainer owns review by 2026-12-04, before original bulk mtimes reach 30 days.
`make check-retained-evidence` uses Shared’s checker and the budget in Makefile,
also required by release preflight and publication. It refuses expired/oversized
state without deleting anything. Review or further relocation requires the
maintainer; do not renew mtimes, split files or raise limits to evade refusal.
Only the separately approved reproducible Runtime 0.18.1 driver binaries moved
to `target/recovery-evidence/`, where ordinary Cargo cleanup can reclaim them.

## Browser SDK boundary

The maintainer authorized the separate ESM npm package under `packages/browser`
alongside the Rust release groups. It owns SDK Core dependency selection
and shared non-cryptographic integration for Blob and Canic; it is not a second
Wire generator, storage owner, authentication flow or application policy owner.
Use the selected browser manifest and qualification inputs. Preserve upstream classes/type identity,
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

## Owned entrypoint generator

`ic-platform-macros` belongs to Runtime's coordinated release group and supplies
its optional `cdk` surface, also forwarded by the facade. The maintainer authorized maintaining
the generator independently of upstream acceptance. Preserve its imported
Apache-2.0 license (the necessary exception to the workspace's MIT metadata),
coordinated package version, original generator semantics and explicit paths.
The maintainer authorized adopting the reviewed non-cryptographic CDK 0.20.3
adapters inside Platform and removing the `ic-cdk` package. Keep their Apache-2.0
license/provenance. The facade's mixed-source license is MIT AND Apache-2.0.
The imported callback module permits reviewed unsafe code. The maintainer also
authorized the function-local `msg_arg_data` exception: set the vector length
only after the maintained IC0 all-or-trap uninitialized-buffer copy returns.
The maintainer authorized embedding IC0 1.2.0 and executor 2.0.0 inside Platform,
including the reviewed IC0 unsafe bindings. Preserve Apache-2.0 provenance and
the unchanged System API ABI. Other functions retain the workspace deny lint. Endpoint macros require
explicit consumer-owned codecs; all owned application endpoints use Wire, including
public endpoints called by third parties. Candid is an explicit adapter for required
management/system, NNS/SNS, ledger and other pre-existing Candid interfaces. Caller
location or visibility never selects Candid. Existing Candid checkpoint
bytes are a legacy compatibility exception, never a new storage format or owner.
Runtime-only raw/executor consumers retain their independent features.

## Rust SDK maintenance ownership

The maintainer authorized taking responsibility for the Rust IC stack; the IC
network, replica and simulator remain external infrastructure. This supersedes
the earlier general hard-cut deferral for this adoption work. Keep product owners
and cryptographic restrictions intact; imports retain licenses and attribution.
Certification and canister-signature cryptography now belong to registry
suppliers. Platform exposes explicit certificate/tree names with upstream identity;
Agent uses that same certificate identity without copying conversions. Keep their
external bytes and consumer-owned decoding/trust/freshness policies unchanged.

## Cryptographic boundary

The maintainer reversed embedded BLS, certification and canister-signature
ownership. Consume upstream cryptographic implementations; keep no embedded
algorithm bodies or upstream algorithm-test suites for these capabilities.
Platform retains delegating key adapters, owned Principal fields and integration
checks. Preserve #72 DER framing admission before the upstream parser and #127
identity-key refusal at both root and delegated keys. Signing-time extraction
must follow successful upstream verification and read only the signing certificate;
consumers retain freshness and authority policy. Registry signature packages
currently bring Candid/printer, CBOR, representation-independent hashing, a lazy
root-key cache and raw IC0 transitively. Admit those only with the selected
signature capability; raw Runtime/executor/IC0 and standalone BLS stay isolated.
Registry raw IC0 is not an additional executor. Do not export the upstream store.

The maintainer authorized the optional BLAKE3 maintenance surface with explicit
upstream `hash`, `Hasher` and `Hash` identities. Keep the implementation upstream,
select only `std`, and qualify singleton/source and effective-feature graphs.
Consumers retain hash-format/domain policy; no existing SHA-256 contract changes.

BLS cryptography belongs to registry ic-verify-bls-signature 0.6.x and
ic_bls12_381 0.10.x. Select only alloc/groups, with defaults and RNG disabled.
Keep no embedded curve/verifier source or algorithm-test/vector suite. Platform
retains delegating key adapters and #127 identity-key admission: point parsing,
identity testing, derivation, signing, serialization and verification all call
upstream APIs. Signature/error types have upstream identity; key adapters retain
Platform identity. Preserve existing bytes, typed errors and identity-key refusal
through integration tests, including keys derived from a zero private key.

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

The owned runtime RejectCode similarly replaces only its external catalogue
dependency; Agent codes retain their external identity. PocketIC catalog
declarations now belong to the approved native implementation, preserving their
numeric and protocol names.

## Independent Runtime owner (#131)

Runtime 0.17 owns the sole IC0/executor/CDK/prepared-call implementation and
releases with macros and the exact guard. Facade paths delegate to it; crypto,
certification and native clients stay outside its release group. Types remains
independent. Keep empty defaults, raw executor/IC0 isolation, explicit-codec
macros, protected source bodies and singleton refusal. Timers and Endpoint must
select Runtime directly; qualify source/archive consumers before publication.
Use the current contract in docs/migration.md; imported-source history never
creates a second implementation or changes the current release-group owner.

## Release groups and canonical identities

`ci/cargo-release-groups.json` owns Platform, Runtime and Types membership,
version authorities and ledgers. Root `Cargo.toml` is the dependency catalog;
root `CHANGELOG.md` is an index and `releases/history.md` retains coordinated history.
Release, resume and publication require explicit `RELEASE_GROUP=platform`,
`runtime` or `types`. Keep the shared qualification-lock and recovery custody;
publication subsets stay within the selected group. A facade-only change does
not advance Runtime or Types. Platform re-exports the sole canonical Types;
conversions, bytes, Serde and provenance remain with their current owners.

## Owned Principal

The maintainer authorized narrowing `principal` to owned Principal/Account bytes,
text and Serde without external Principal or hashing. `principal-external` adds
registry identity and byte conversions; `principal-self-authenticating` separately
selects the exact imported ic_principal 0.1.5 derivation with upstream SHA-224,
without implying external conversions. Preserve its source/license/vector guard;
this packaging permission authorizes no cryptographic algorithm changes. No default features. Runtime
and canister-signature creation/verification select only the owned capability;
Candid and external DTO/client profiles select external conversions explicitly;
raw executor/IC0-only profiles exclude application types and their dependencies.
CanisterSigPublicKey and certified range decoding use owned Principal.
Preserve original DER/hash/verifier bodies via import-only
reconstruction and qualify CBOR bytes and length-first range ordering.

The maintainer authorized the Platform-owned Principal for the breaking 0.3 line.
Retain its Apache-2.0 provenance, raw bytes and length-first ordering. Keep parsing
bounded by the 63-byte text grammar, binary Serde marker-free and range policy
consumer-owned. Retain the exact upstream self-authentication body and upstream
SHA-224 implementation. External DTO/client principal and error identities remain registry
types, named explicitly through principal::ExternalPrincipal/ExternalPrincipalError;
Candid contract types stay external. Do not add Candid to the owned type.

## Executor identity

Runtime owns the embedded executor and IC0 implementations. Keep
exactly one executor across Platform, Timers and application callbacks. Preserve
the inherited Cargo links guard in the optional, dependency-free internal
`ic-platform-runtime-guard` package, selected only by `executor`. Its requirement
must exactly match the Runtime version so different runtime owners cannot share a
guard. Non-runtime consumers may coexist with an unrelated external executor;
never admit an external ic-cdk-executor beside the enabled embedded owner. Existing public module paths remain; task/method handles
have Runtime identity and require compatible consumer adoption.

Retain SlotMap generation checks. The maintainer selected standard Vec task lists
after the 2026-10-30 benchmark, prioritizing reduced production Wasm size over
SmallVec's small-method allocation advantage. Do not restore that runtime dependency.
Keep protected wake routing while polling, coalesced wakeups, cached wakers,
completed-task reclamation and destructor execution outside RefCell borrows.
Preserve IC0 1.2.0 raw safe/unsafe signatures and imported source/license guards.
No timer entrypoint, storage allocation or application scheduling policy is added.
Changes require combined CDK/Timers await, callback-trap, cancellation and upgrade
qualification plus relevant native executor regressions. See the
[embedded runtime contract](docs/migration.md#embedded-ic0-and-executor).

## Management contract

The maintainer authorized adopting only the management DTOs used by sibling
canisters. Wasm uses 44 explicit exports backed by 52 complete declarations from
0.11.0, including nested types and the Canic runtime-probe HTTP contract. Native
builds retain the existing 58-name registry =0.11.0 surface and identities for
Backup/Testkit/host tooling; do not import the host-only transfer DTOs speculatively.
Keep Candid bytes, fields, variants, Serde attributes and function-reference
signatures unchanged. Select bignum/serde_bytes without Candid's printer on Wasm.
Keep source/license guards, upstream schema comparisons and installed byte proofs;
record the type selection and consumer evidence in the maintained provenance.
This supersedes the earlier cross-target registry-identity constraint for Wasm only.

## PocketIC maintenance boundary

The maintainer explicitly authorized transferring IC Profiler's native provider,
including its existing assembler/fixtures, into `native/ic-profiler`. Profiler
#4 / Platform #90 return the product CLI/query renderer and shared PNG/range
mechanics to the consumer; keep only their native integration/state adapters here.
The paired native profile selects Host Artifacts gzip directly and excludes
registry Platform; its exact cohort belongs in the provider manifest and handoff.
Consumers retain the matching product imports and
immutable provider pin; older consumers keep their qualified source. The current
consumer library has no gzip surface. That opt-in
source profile owns replica adapters and the patched client below this facade;
its IC-free recorder/viewer remain consumer-owned. Preserve the legacy tooling
during this transfer; this is no general permission to introduce Python tooling.
Keep Motoko/lifeline outside the admitted native server graph. Follow the source
profile contract in `native/ic-profiler/README.md` and preserve IC copyrights.

The maintainer authorized transferring the maintained PocketIC 16.1 client to
Tooling’s existing `ic-testkit::pocket_ic` module (#139 / Tooling #324). Preserve
the historical ledger; no client package, release group, facade feature or
forwarder remains here. Platform’s development fixtures select the exact Testkit
supplier. Keep normal/build dependencies directed from Testkit to Platform.
The separately pinned native Profiler provider remains outside this transfer.

## Owned account

Owned Account/Subaccount accompany Principal under the `principal` feature.
Preserve effective-zero equality, ordering and hashing, and preserve
absent versus explicit zero fields during serialization and external conversion.
Retain Apache provenance. The maintainer authorized replacing the broad registry
ledger package with selected maintained DTOs under the existing feature/module.
Ledger types/callbacks now have Platform identity; external Candid Principal/Nat,
Candid/Serde bytes and unchanged upstream value-hash bodies/vectors remain.
Keep source/license guards; omit unused storage/CBOR and consent-formatting machinery.
No Candid,
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
