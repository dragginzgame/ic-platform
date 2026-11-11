# IC Platform contributor instructions

Apply the installed [Toko Tooling baseline](TOKO-TOOLING.md) and its
revision-bound references. [.toko-tooling.snapshot](.toko-tooling.snapshot)
records the adopted payload; do not edit it or follow a moving sibling baseline.
This overlay owns product safeguards, local commands and approved exceptions.
The baseline owns common action authority, validation, compiler and reporting rules,
including scoped `git add -N` when validation requires new task-owned paths.

Read [the current handoff](docs/status/current.md) before implementation.
Custom deployment adapters must run `toko-tooling-check` before effects.

The root `LICENSE.md` and `CONTRIBUTING.md` are canonical Toko Tooling payloads.
Rust members inherit the root license-file; browser packaging generates its
distribution copy. The component-by-component replacement is recorded under
`docs/design`; ledger protocol composition now has independent implementations
and frozen original/replacement comparisons. External algorithm packages retain
their own ownership and distribution requirements.

The five product packages remain in the root workspace under `crates/`.
The [package placement audit](docs/development/qualification.md#package-placement)
records generated fixture exclusions and retirement of the native provider. Separate maintained
qualification packages, if introduced, use an independent `qualification/`
workspace and do not join product release groups.

- This repository owns the controlled Candid/CDK dependency boundary. Read
  [the migration contract](docs/architecture/migration.md) before changing its public surface.
  Keep [the IC inventory](docs/architecture/dependency-inventory.md) aligned with dependency, feature,
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
- No default features. Executor-only builds must remain Candid-free. Upstream
  binary Candid is a development oracle or native-client transitive, not an owned
  codec/runtime feature selections.
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
  their own scope and qualification under [the migration contract](docs/architecture/migration.md).
- Rust 2024 and minimum/development/release compiler 1.99.1 (#133). Read the
  workspace floor and pinned toolchain; qualify each distinct graph once on
  this compiler, preserving native hosts and installed-runtime obligations.
  Inherit workspace metadata and dependencies; ordinary
  directory modules only. Do not add Python tooling.
- GitHub issues are the follow-up tracker once a remote exists. Migration and
  provenance documents specify contracts and evidence, not a second issue queue.

## Owned binary Candid codec

The maintainer authorized removing the legacy `candid`, `candid-parser` and
`candid-service` public features/modules and production CandidType/carrier bridges.
Use the existing facade-owned `candid_codec` with explicit budgets and argument
arity. Upstream binary Candid is a dev-only independent byte/schema oracle;
Primitive and callback oracle bridges compile only under cfg(test). Do not restore legacy
forwarders or a compatibility feature. Management DTOs use the owned contract
below; native clients and development signature oracles still pull Candid transitively.
The maintainer retired all textual DID export, inspection, parsing and fixtures
on 2026-11-11. Do not add declaration-Wasm export or native DID inspection;
#154/#155 are declined. Retain independent binary schema/byte fixtures instead.
No candid_parser dependency or DID-specific build/validation target remains.
Cryptographic implementations remain upstream-owned; their restrictions are
independent of the codec hard cut.

## Test source layout

Follow Shared's `tests.rs`/`tests/` layout for owned unit tests. Integration tests
already live under crate `tests/` directories, and doctests stay beside their APIs.
The retired imported ICRC hash tests no longer require an inline-test exception.
Ledger composition is qualified by independent public integration tests.

The generated internal spawn retains its scoped `disallowed_methods` allowance:
consumer Clippy configurations may forbid the method or omit that lint entirely,
so one expectation would fail for supported consumers. Keep its explicit reason.

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
its optional `cdk` surface. Ordinary entrypoint macros and the `cdk` feature
belong only to Runtime; the facade retains its explicit external-Candid bridge. The maintainer authorized maintaining
the generator independently of upstream acceptance. Its implementation and shared
syntax helpers now come from a specification-only context, with separate public
consumer tests. Preserve coordinated package versions, exact IC export names,
guard/context ordering, explicit codecs and typed adapter paths. Its imported
source/license were retired together.
The maintainer authorized adopting the reviewed non-cryptographic CDK 0.20.3
adapters inside Platform and removing the `ic-cdk` package. Call transport,
System API, stable and task-policy adapters were replaced through specification-only
contexts; their public contracts and installed checks remain. The call transport
confines unsafe code to callback registration and
owned completion-pointer custody, with local expectations and safety arguments.
No borrowed request data crosses that callback boundary. The maintainer also
authorized spare-capacity copies for `msg_arg_data`, `msg_reject_msg`,
`msg_method_name`, `root_key` and `data_certificate`: their private `system_bytes`
helper sets vector length only after the selected maintained IC0 all-or-trap
uninitialized-buffer copy returns. Keep the unsafe expectation local to that helper.
The maintainer now requires upstream implementations to come from packages
rather than copied source. IC0 delegates to registry `ic0` 1.2.0 through the
existing explicit exports; no embedded IC0 bindings remain. The executor is now
independently implemented. Ledger value hashing and account framing now have
separate specification-only implementations and public byte comparisons.
Preserve the System API ABI and workspace unsafe deny lint. Endpoint macros require
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
Certification primitives and BLS cryptography belong to registry suppliers;
Platform owns the authorized canister-signature protocol composition below.
Platform exposes explicit certificate/tree names with upstream identity;
Agent uses that same certificate identity without copying conversions. Keep their
external bytes and consumer-owned decoding/trust/freshness policies unchanged.

## Cryptographic boundary

On 2026-11-11 the maintainer authorized adopting the qualified clean-room BLS
wrapper into the existing facade `bls` module. Its source-unexposed implementer
received protocol/API facts; independent comparisons remain separate. Arithmetic,
pairings, hash-to-curve, point/scalar codecs and SHA-256 call external libraries
unchanged. The Digest 0.10 trait adapter forwards to SHA-2 0.11; it implements no
hashing or message expansion. Preserve checked subgroup admission, identity-key
refusal and caller-owned prepared-key custody. This supersedes the earlier
registry-verifier/type-identity requirement only for this reviewed composition.
Do not keep a second implementation in qualification or introduce another product
crate. See [BLS qualification](docs/design/bls-wrapper/qualification.md).

The maintainer authorized independent canister-signature protocol composition
on 2026-11-10. Platform owns key-envelope framing, certificate/delegation and
witness checks, and authenticated signing-time extraction. Use canonical Types
Principal throughout. BLS/curve operations, SHA-256 and certified-tree hashing
remain external package implementations; do not implement their algorithms.
Preserve #72 DER framing admission and #127 identity-key refusal at both root
and delegated keys. Extract time only from the successfully authenticated signing
certificate; consumers retain trust roots, freshness, authority and budgets.
The creation and verification production graphs must exclude upstream Candid,
registry Principal, registry signature composition, raw IC0 and an executor.
DFINITY signature packages are development-only comparison/benchmark oracles.
No signature store, timer or application lifecycle is added.

The maintainer also authorized finishing ICRC value hashing and ICP account
framing through specification-only contexts on 2026-11-10. Preserve protocol
bytes, domain input, map digest ordering and the legacy signed-integer admission
and framing documented in [ledger qualification](docs/design/ledger-composition-qualification.md).
SHA-224/SHA-256 and CRC32 stay external. This replaces protocol composition,
not cryptographic algorithms, and grants no wider primitive-rewrite authority.

For this adoption, apply the maintainer's independent-ownership rule: derive a
reviewed behavior specification from public interfaces, protocol facts and
black-box observations; use a separate unexposed implementation agent with no
upstream bodies or expressive tests in its context. Public DER/OID/key constants
and API signatures are permitted factual inputs. Keep specification/context
separation and independent qualification evidence. Do not copy or rebrand a fork.
This scoped adoption does not edit the installed Toko snapshot or grant a general
cryptography rewrite exception.

Shared's [hashing policy](https://github.com/dragginzgame/toko-tooling/blob/5806e6e4943f6be600cd53385498ce08a5f804c4/rules/hashing.md) permits approved pure hashing directly
at its consumers. The BLAKE3 forwarding surface is retired (#150); do not restore
a facade dependency for pure hashing. Algorithm, input/domain and persisted-format
changes remain separate explicit decisions.

BLS curve primitives belong to registry ic_bls12_381 0.10.x, selecting
alloc/groups/pairings/experimental with defaults and RNG disabled. Keep no
embedded curve/hash implementation or imported algorithm-test/vector suite.
Platform owns the qualified BLS wrapper and all its key/signature/error types;
registry ic-verify-bls-signature remains a development oracle and native Agent
transitive. Preserve bytes, typed error variants and #127 identity-key refusal,
including keys derived from a zero private key. Certificate verification uses
the same checked BLS boundary for root and delegated keys without a second parse.

Our team does not include cryptographers. Do not implement, replace or alter
cryptographic primitives: curve arithmetic, pairings, public-key derivation,
hash-to-curve, point encoding or hash algorithms. The authorized BLS and
canister-signature compositions use those external primitives with unchanged
protocol domains and bytes. Retain applicable upstream licences for dependencies;
do not substitute earlier primitive-replacement prototypes.

Dependency scaffolding may be simplified: manifests, feature gates, unused
modules, duplicate generators and integration plumbing are in scope when they
preserve cryptographic behavior. Source variants must retain provenance/licenses
and unchanged cryptographic function bodies. Qualify their actual consumer graphs,
type/source identities and wire bytes; a dependency-local Cargo patch is not a
supported distribution mechanism for downstream consumers.

The owned runtime RejectCode similarly replaces only its external catalogue
dependency; Agent codes retain their external identity. PocketIC catalog
declarations retain the upstream identities exposed through Testkit's native
re-export; do not recreate them in Platform.

## Independent Runtime owner (#131)

Runtime owns the sole IC0/executor/CDK/prepared-call implementation and
releases with macros and the exact guard. Facade paths delegate to it; crypto,
certification and native clients stay outside its release group. Types remains
independent. Keep empty defaults, raw executor/IC0 isolation, explicit-codec
macros, independent lifecycle checks and singleton refusal. Timers and Endpoint must
select Runtime directly and own their downstream composition qualification.
Platform qualifies its standalone source/archive Runtime before publication.
Use the current contract in docs/architecture/migration.md; imported-source history never
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

Owned DTOs and callbacks use canonical Types Principal and the owned Candid
codec. `principal-external` and external aliases/conversions remain retired.
This product restriction does not prohibit native simulator handles or explicit
validated-byte conversions at the qualification boundary described below.
The maintainer authorized owning self-authentication framing: SHA-224 of the
supplied bytes followed by `0x02`, with no new key parsing or validation.
Only `principal-self-authenticating` selects upstream `sha2` 0.11 without defaults.
Registry `ic_principal` remains a development oracle, never a production Types
dependency. Base Types remains registry-Principal/crypto-free.

Keep empty defaults, raw bytes, length-first ordering, bounded 63-byte text
parsing, marker-free binary Serde and consumer-owned range policy. The optional
`principal-self-authenticating` capability composes the specified identity bytes
over RustCrypto's SHA-224; do not embed a hash algorithm. Preserve supplier-byte
equivalence and feature isolation. This grants no change to hash inputs, output
format, cryptographic primitives or consumer authentication policy. Types and
facade hashing share the workspace's SHA-2 0.11 selection.
Raw executor/IC0-only profiles exclude application types. CanisterSigPublicKey and
certified range decoding use owned Principal. Preserve DER/hash/verifier behavior
under the scoped protocol-composition authority above, and qualify CBOR bytes
and range ordering. Do not add Candid to the owned type.

## Executor identity

Runtime owns the embedded executor and explicit registry IC0 boundary. Keep
exactly one executor across Platform, Timers and application callbacks. Preserve
the inherited Cargo links guard in the optional, production-dependency-free internal
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
Preserve IC0 1.2.0 raw safe/unsafe signatures and registry source admission.
The executor was replaced through specification-only implementation and test
contexts; their input records live under `docs/design`. Retain generation-safe
handles and independent lifecycle tests, not a hash guard of the retired source.
The call/System API adapters have also been replaced; keep their independent
tests and installed proofs. The generator now has its own independent implementation,
public consumers and native/Wasm export checks.
No timer entrypoint, storage allocation or application scheduling policy is added.
Changes require standalone await, callback-trap, cancellation and upgrade
qualification plus relevant native executor regressions. Scheduling owns its
Timers composition tests; Platform must not depend on a scheduler release. See the
[embedded runtime contract](docs/architecture/migration.md#embedded-ic0-and-executor).

## Management contract

Use owned management DTOs and codec on native and Wasm targets; the registry
management package is a dev-only schema oracle. Preserve the selected 62 native
and 44 Wasm exports, including the 14 native snapshot declarations and their
alias dependency, and the four native install/create/log additions. Those native
extensions now use independent implementations; retain their schema/value/Serde
oracle tests and frozen public observations. See
[native qualification](docs/design/management-native-qualification.md).
The 52 shared schemas now also have specification-only declarations, including
nested types and the Canic runtime-probe HTTP contract; do not widen that surface.
Preserve independent schema/value/Serde checks, frozen public observations and
installed byte proofs. Fields, variants, attributes, trait sets and function
references remain unchanged; use owned Nat/Reserved and serde_bytes without
upstream Candid on either target. The owned codec and TransformFunc adapter
remain unchanged. See [shared qualification](docs/design/management-shared-qualification.md)
for declaration selection, source separation and compatibility evidence.

## PocketIC maintenance boundary

The maintainer retired the instrumented Profiler source provider from Platform.
Keep its native IC checkout, patches, assembler, provider-only fixtures and CI
removed. Profiler's existing immutable Platform pins retain their historical
source; future provider ownership and adoption belong outside this repository.
This retirement grants no permission to remove crypto or runtime qualifications
for the maintained Rust packages. No Python tooling remains owned here.

Native simulator fixtures consume the upstream PocketIC client through Testkit's
`ic_testkit::pocket_ic` re-export. Its native client handles and DTOs retain
upstream identity; application/Wire DTOs retain canonical Types Principal.
Convert validated Principal bytes explicitly only where those boundaries meet;
do not restore product aliases or a second client implementation. Testkit owns
managed server setup and custody. Keep normal/build dependencies directed from
Testkit to Platform; Platform's Testkit use remains qualification-only. Preserve
the historical transfer ledger (#139 / Tooling #324) and the separate retired
native Profiler provider history.

## Owned account

Owned Account/Subaccount accompany Principal under the `principal` feature.
Preserve effective-zero equality, ordering and hashing, and preserve
absent versus explicit zero fields during serialization and external conversion.
Principal and Account were replaced through separate specification-only
implementation contexts, with input/compatibility records under `docs/design`.
Types inherits the workspace license; its SHA-224 primitive remains upstream.
Ledger value/hash and ICP framing have separate replacement evidence below.
The ICRC Account/archive adapters now have specification-only implementations.
Preserve the external Account's detailed text errors and sequence-shaped Serde;
canonical equality/order/hash/display delegate to Types. Archive references carry
the canonical Principal directly and retain explicit query signature checks.
Their account/archive compatibility checks remain independent of ledger hashing.
The remaining passive ICRC metadata/transfer/allowance/block/consent modules also
have specification-only replacements. Preserve their exact fields/variants,
numeric ranges, conversion behavior and Serde/Candid bytes. Value/ICRC3Value
non-cryptographic declarations, conversions and formatting also have independent
implementations; preserve their public observations and owned-buffer transfer.
The final `value/hash` and ICP account-framing replacements use specification-only
authors and independently written public tests. Preserve the frozen 782 original
observations and exact integer compatibility, without restoring source fingerprints
or upstream test bodies. See [ledger qualification](docs/design/ledger-composition-qualification.md)
and the earlier [value qualification](docs/design/icrc-value-qualification.md).
The four CMC/ICP/Cycles/assets declaration families also have specification-only
replacements. Their 33 declarations, six aliases and 14 supported method schemas
retain public traits, feature gates, Serde admission and Candid bytes. Preserve
the independently captured binary schema fixtures and frozen public observations;
do not restore full supplier service fixtures. Existing owned codec mappings and
callback adapters remain unchanged. See
[external DTO qualification](docs/design/external-dtos-qualification.md).
The maintainer authorized replacing the broad registry ledger package with
selected maintained DTOs under the existing feature/module.
Ledger types/callbacks have Platform identity and canonical owned Principal/Nat;
Candid/Serde bytes and ledger hash outputs remain unchanged.
Keep byte/schema comparisons; omit unused storage/CBOR and consent-formatting machinery.
No Candid,
storage implementation, cryptographic account-identifier derivation or application
subaccount allocation policy belongs in the owned account surface.

## External Candid nesting admission

The maintainer superseded #73's draft opt-in policy with one fixed admission rule:
Platform's owned Candid decoder and external endpoint adapters reject more than
16 constructed wire levels. Preserve explicit byte/header/type/work/allocation
limits, per-vector preallocation caps, independent validated skipping and closed
error variants. Inspect unused type definitions and skipped values and fail
closed on uninspectable forms. This wire admission is not a universal stack-safety
claim for arbitrary Rust schemas/custom visitors. Keep the comparison and installed
16/17 evidence; do not restore the retired upstream-decoder forwarding API.

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
Darling solely for compatibility. Submission, callback and cleanup behavior
is now protected by independent tests and installed qualification of the
specification-only replacement, rather than a retired-source fingerprint.
