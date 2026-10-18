# Candid and runtime ownership

IC Platform owns the controlled Candid/CDK dependency boundary. Consumers declare
`ic-platform`, not `candid`, `candid_parser` or `ic-cdk*` directly. IC Wire owns
protocols whose producers and consumers we control. External management-canister,
NNS/SNS and ledger interfaces retain their required Candid formats. Storage and
ic-memory changes are out of scope; no persisted representation is changed here.

## Qualified Candid surface

The `candid` feature supplies `decode_one`, `decode_exactly_one`, `decode_args`,
`DecodeLimits` and `DecodeError`. The `candid::contract` module explicitly exports
`CandidType`, `Principal`, `Nat`, `Int`, `Reserved`, `Error`, `ArgumentEncoder`,
`ArgumentDecoder`, `encode_one`, `encode_args`, Serde, and the machinery needed
by Candid derives. These are upstream identities, not independent codec types.
Arbitrary-precision integers enable Candid's `bignum` feature with `candid`;
the separate native parser also enables it through upstream Candid's `all` feature.
No default features, automatic limits or second executor are introduced.

`ic_platform::Principal` is available unconditionally from `ic_principal` 0.1.5,
with its standard conversion, self-authenticating and Serde support. Consumers
can remove direct `ic_principal` declarations and use this root export without
selecting Candid or executor features. The existing contract export uses the
same type. Default builds are Candid/CDK-free, not dependency-free.

The decoder implementation and six tests match Testkit's current owner byte for
byte after excluding the added contract module. Reply length is admitted before
constructing the decoder. All five caller limits are forwarded: reply bytes,
decoding quota, skipping quota, type-table entries and header bytes. Zero is a
real limit. These limits do not promise heap, stack or elapsed-time bounds.

- `decode_one` requests one value using upstream compatibility rules: extra values
  are skipped within quota; an absent optional/null result may be synthesized.
- `decode_args` requests a top-level argument tuple, with the same compatibility
  and bounded skipping rules. `(T, U)` is two arguments; `((T, U),)` is one record.
- `decode_exactly_one` requires one actual wire argument, including for optional
  and null types, and rejects additional arguments. All paths reject trailing
  undecodable bytes and preserve malformed-input and quota errors.
- `ReplyTooLarge` retains actual/maximum lengths; `Candid` retains the upstream
  error as its typed source. Compact diagnostics are selected, but may disclose
  small inputs. Raw bytes, authentication, disclosure and aggregate budgets stay
  with callers.

## Consumer audit evidence

Read-only working-tree review on 2026-10-18 used the shared code-hygiene method at
`4eabe7f771907ccebfebceafffe10657b911c0eb` and this repository's AGENTS.md overlay.
Platform started at `9ada261`. Sibling HEADs below identify bases, not clean-source
claims: Testkit, Canic, Timers and Host have pending changes which were preserved;
Query was clean. No sibling builds or edits were performed.

| Consumer base | Actual use and adoption boundary |
| --- | --- |
| Testkit `55ac397` | `crates/ic-testkit/src/pic/calls.rs` requires generic argument encoder/decoder bounds; transport preserves decoder sources. `ic-testkit-dev` also consumes the decoder. The performance probe still uses CDK entrypoint macros. |
| Query `e90cb2ef` | `src/candid_call.rs` uses generic argument encoding; `src/candid_decode.rs` owns quota policy. Ledger DTOs use `Nat`/`Int`; governance projections use `Reserved`. The CLI also declares Candid and the old decoder. `nns/governance/source/canister.rs` uses CDK calls. Paths are under `crates/ic-query` unless noted. |
| Canic `5583496bf` | `canic-macros/src/adapter/mod.rs` generates absolute Candid paths; `canic/src/macros/start.rs` exports a service. `canic-core/src/infra/ic/call.rs` wraps raw CDK calls. Host paths parse/type-check DID, generate bindings and compare services. Paths are under `crates`. |
| Timers `194204d` | `crates/ic-timers/src/platform/{ffi,backend}.rs` use executor method handles, callbacks and trap recovery. Production uses raw IC system calls; unpublished testing packages also use Candid/CDK. |
| Host `c8cd434` | `crates/ic-host-tools/src/candid/service/mod.rs` parses service facts; pending `candid/decode/mod.rs` adds aggregate byte accounting. That accounting differs from Testkit and must survive consolidation. |

## Certification extension

The optional `certification` feature extends this boundary explicitly beyond
Candid/CDK. It selects `ic-certification` 3.2.0 with upstream Serde support and
exports only the observed consumer surface: `Certificate`, `Delegation`,
`HashTree`, `Label`, `LookupResult`, `empty`, `fork`, `labeled`, `leaf`, `pruned`
and `fork_hash`. These are direct upstream exports, with no replacement structs
or conversion wrappers. Inherent lookup/digest operations remain upstream.
Certification selects neither Candid nor the executor and is absent by default.

The follow-up read-only review used pending working trees at Host `c8cd434`,
Query `8982c2a4` and Canic `5583496bf`; all three were dirty. Host's
`crates/ic-host-tools/src/certificate/mod.rs` constructs owned upstream values;
Query's `crates/ic-query/src/certificate/mod.rs` passes those certificates to
Agent verification and checks witness digests. Canic's `certified_custody` and
`operator_mint/receipts` modules consume labels, lookups and certificates, while
`canic-core/src/ops/auth/signature_store/mod.rs` uses `fork_hash` for signature trees.

For adoption, add/inherit Platform and replace `ic_certification::` paths with
`ic_platform::certification::`, including tests. Host's `certificate` feature
must select Platform's `certification` while retaining `ciborium`. Query's
`certificate-host` and Canic's signature capabilities must enable the same
Platform feature in place of `dep:ic-certification`. If Platform is already
non-optional for Candid or Principal, use `ic-platform/certification` rather than
introducing a second dependency alias. Remove the direct upstream declaration
from each member and the unused workspace catalog; update locks and dependency
ownership rules. Do not remove transitive dependencies of Agent or other upstream
consumers, or claim full workspace isolation before runtime/parser migration.

Host retains its strict bounded CBOR decoder, errors, complete-input byte limit,
nesting limit, duplicate/unknown-field refusal, byte-string admission, exact tree
tag/arity checks and trailing-data refusal. Do not replace it with generic upstream
Serde decoding. Consumers retain combined budgets, trust roots, signature and
delegation verification, time policy, witness admission, retry semantics and
application leaf interpretation. No certificate CBOR, hash commitments or persisted
data change. Constructed or decoded values remain untrusted until authenticated.

The standalone certification consumer has no direct upstream dependency and tests
lookup outcomes, pruned witnesses, fork hashing, a fixed leaf digest and certificate
fields on native/MSRV with Wasm compilation. A disposable copy of Host's actual
decoder and tests changes only import paths; its retained lockfile and source hashes
bind that focused proof. It is not a migrated or built Host/Query/Canic workspace.
Source hashes are recorded under `target/certification-host/source.sha256` and
fixture evidence under `target/certification-host/`. Consumer qualification must
still run actual authentication/receipt/signature tests against the selected Agent
and protocol packages to confirm one upstream certification identity. Native macOS
and installed canisters are separate qualification. Dependency control alone
does not remove `ic-certification` or establish a smaller Wasm artifact.

## Testkit and Query migration

The Candid surface is usable for local integration qualification now. Registry
adoption requires an explicitly published, qualified Platform version; repository
creation or a local commit does not establish publication. The public GitHub remote
exists, but consumers have not been migrated by this work.

1. Declare the selected `ic-platform` version in each workspace dependency catalog
   with `default-features = false, features = ["candid"]`; child manifests inherit
   it with `workspace = true`. For local qualification only, select an explicit
   Platform path. Replace direct Candid and `ic-testkit-candid` dependencies in
   normal/dev/build/target tables, CLI and development packages too. Remove unused
   root catalog entries and regenerate each affected lockfile deliberately.
2. Replace `ic_testkit_candid` paths with `ic_platform::candid`. Import DTO types,
   encoders and generic argument bounds from `ic_platform::candid::contract`.
   Replace `candid::Result<T>` with `Result<T, contract::Error>`. Use
   `contract::serde::Deserialize` (and its `de` traits) where appropriate;
   independent Serde use need not move. Principal-only code uses `ic_platform::Principal`.
3. Add `#[candid_path("ic_platform::candid::contract")]` to every Candid-derived
   type. When deriving Serde through Platform, also add
   `#[serde(crate = "ic_platform::candid::contract::serde")]`. Preserve field and
   variant labels, rename attributes, optionality and numeric types. Update
   generators/templates and regenerate their DTOs, rather than patching output.
4. Replace `Encode!()` with `contract::encode_args(())`, `Encode!(&request)` with
   `contract::encode_args((&request,))`, and multi-argument macros with tuples.
   Replace test `Decode!` and unbounded decoders with the intended bounded API
   and explicit test-owned limits. Do not replace argument tuples with records.
   No compatibility macro or blanket Candid crate alias is required.
5. Preserve Query's `candid_decode.rs` policy: transport-owned byte admission,
   `min(bytes.len() * 32 + 1_000_000, 256_000_000)` using saturating arithmetic,
   skipping quota 100_000, 4_096 type entries and 64 KiB header limit. Preserve its
   existing error projection. Testkit keeps PocketIC calls/lifecycle, transport
   errors and caller-selected limits. Neither policy belongs in Platform defaults.
6. Qualify real consumer DTOs and generic call helpers with only Platform declared
   for Candid. Retain Testkit malformed/quota/arity/source cases and Query's large
   byte vector, recursive DTO, bounded skipping, ledger and governance tests.
   Compare encoded request bytes and external DID schemas; preserve retry/call
   identities, source authentication and family-owned error handling.
7. In the coordinated adoption, remove `crates/ic-testkit-candid`, its workspace
   membership, dependency catalog entries, package README/publication entries and
   release metadata. Update Testkit's Makefile, `scripts/release/metadata.sh`,
   `publish-workspace.sh`, package checks and release/publish guard fixtures.
   Migrate Query/CLI and other actual users (including Canic and Timers tests)
   before claiming retirement complete. Keep one decoder/test owner here, without
   a compatibility re-export package or permanent duplicate implementation.

Host-side Candid adoption is distinct from whole-workspace dependency isolation.
Testkit's `ic_testkit_perf_probe` still needs CDK macros; Query's `canister` feature
still needs `canister_self`, `Call` and `CallFailed`. They cannot drop all direct
CDK dependencies until the runtime boundary is qualified. Do not declare a
workspace migrated while such manifests or independent fixture workspaces remain.

## Runtime trace and required qualification

The existing `executor` feature explicitly exports upstream task/context APIs.
Platform, Canic and Timers currently lock `ic-cdk-executor` 2.0.0. Matching version
text is source evidence only: the combined final dependency graph must resolve
one executor package identity (version and source), including its method handles.

CDK 0.20.3's `futures/internals.rs` delegates entrypoint contexts to
`in_tracking_executor_context` and `in_tracking_query_executor_context`.
Canic's adapter emits those CDK context wrappers and async spawning. Its raw call
builder preserves bounded/unbounded waits, cycles and exact `with_raw_args` bytes.
Management adapters in `infra/ic/mgmt` and external ledger calls remain Candid.
No change here selects deadlines, retries, limits or protocol resets for them.

CDK `call.rs` retains a method handle when submitting a call, restores it in
`in_callback_executor_context_for` before waking the future, and uses
`in_trap_recovery_context_for` to cancel attached tasks plus the saved task after
callback rollback. Timers' `platform/ffi.rs` follows the same identity through
self-dispatch, response and cleanup; its own flight table, retry and rearm policy
stay in Timers. Replacing this with another executor would break that contract.

CDK macros 0.20.3 accept a runtime `crate` path override but still generate
`::candid::DecoderConfig`, decode/encode helpers and `candid_method` attributes.
Canic separately generates these absolute Candid references and service exports.
Changing imports or re-exporting an attribute macro alone cannot eliminate the
consumer dependency. Full runtime APIs and generators are deliberately not exposed
as supported here until a concrete generated-path solution is qualified.

The next runtime proof must use a standalone canister with only Platform at this
boundary: query/composite query/update/init/post-upgrade exports, DTO/service
extraction, raw calls and management calls, reply/reject/submission failure,
awaited/protected/migratory tasks, callback traps and cancellation. Combine Timers
in the installed fixture to verify cleanup, single ownership of timer exports and
one executor graph. Preserve CDK semantics; no alternative executor implementation
or blanket upstream re-export is an acceptable shortcut.

## Host parsing and aggregate admission

The optional `candid-parser` feature now exposes `ic_platform::candid_parser`
on native hosts, selecting upstream `candid_parser` 0.4.1. The dependency and
module are target-gated out of Wasm, including all-feature builds. No Candid,
executor or certification feature enables host parsing. Parser-only native
selection does pull transitive Candid with the parser's upstream `all` feature;
it does not select Platform's `candid` feature. This is dependency isolation,
not transitive removal or a Wasm-size measurement.

The explicit surface covers the reviewed Host service inspector and Canic's
frontend bindings, protocol argument helpers and service/type comparisons:
`IDLProg`, `check_prog`, `TypeEnv`, `Type`, `TypeInner`, `FuncMode`, `Dec`,
`IDLMergedProg`, `Error`, `Result`, `IDLArgs`, `parse_idl_args`, `CandidSource`,
`instantiate_candid`, `service_equal`, `TypeContainer`, `type_equal`,
`compile_javascript` and `compile_typescript`. These directly preserve upstream
identities; there is no blanket crate/module export or new parser implementation.

To adopt, enable `features = ["candid-parser"]` on the existing inherited
Platform dependency in host members, or `ic-platform/candid-parser` from an
optional product capability. Remove direct `candid_parser` declarations from
members and the unused root catalog; update locks and dependency-boundary rules.
Canister libraries that use parser helpers only in integration tests must select
the host feature through their dev-dependency rather than runtime features.
Replace `candid_parser::` imports with `ic_platform::candid_parser::` and flatten
`syntax::{Dec, IDLMergedProg}` and `utils::{CandidSource, instantiate_candid,
service_equal}` imports. Use root `FuncMode`, `Type`, `TypeInner`, `TypeEnv`,
`TypeContainer` and `type_equal` for those host type operations. Replace
`bindings::javascript::compile` with `compile_javascript` and
`bindings::typescript::compile` with `compile_typescript`, preserving the merged
program supplied to TypeScript generation. Move ordinary DTO/encode/decode uses
of the parser's blanket Candid export to Platform's `candid` contract/decoder
instead; this host surface deliberately provides no `candid` crate alias.

Retain consumer source/output byte admission, method bounds, import rejection,
artifact hashes, expected service/type rules, aggregate budgets and error
projection. Parsing/type checking has no automatic stack, heap or work ceiling.
`check_prog` ignores imports rather than rejecting or resolving them; consumers
requiring self-contained text must inspect `Dec` first. `CandidSource::File`
reads files/imports and the upstream pretty loaders can print diagnostics;
`CandidSource::Text` and `service_equal` can print parse diagnostics too. Canic's
silent `parse_text` path must continue parsing `IDLProg` and using `check_prog`
directly. Do not replace it with `CandidSource::load`.

The standalone parser-only consumer requires no direct Candid/parser dependency.
It exercises constructor typing, generated browser bindings, malformed grammar,
undefined types, import detection, service drift, Rust type equality and a fixed
nat64 argument wire vector. Host's service inspector and existing tests are also
qualified from a disposable copy with only imports changed; source hashes and
fixture evidence are under `target/parser-before/host-source.sha256` and
`target/parser-host/`. These are focused proofs, not migrated sibling workspaces.
Actual Host/Canic adoption must still compare service inventories, generated
binding bytes, argument bytes and existing protocol/effect/retry tests. Neither
consumer product code nor external Candid formats or persisted data change here.

Host's pending decoder first rejects oversized replies, then atomically subtracts
admitted bytes from the caller's remaining budget before decoding. Malformed
admitted replies still consume budget; failed admission does not. Consolidation
must preserve those semantics outside the per-reply Platform decoder. Host's
pending helper does not select compact diagnostics; adopting Platform changes
that diagnostic policy and needs explicit caller qualification.

Host's service inspector uses `IDLProg`, type checking and `TypeEnv` to project
bounded method facts, rejects imports, and leaves Wasm export identities to Host.
Canic additionally consumes parser bindings, textual arguments and service equality;
method inventory alone cannot replace those uses. The new explicit host surface
covers those operations; Host and Canic retain the surrounding implementation.

## Evidence and isolation

`make test` and `make msrv` exercise the retained six decoder cases and expanded
DTO fixture. `make check-consumer` builds separate workspaces with only Platform
as a direct dependency: native tests on Rust 1.99 and 1.88, plus Wasm compilation
for Candid and certification consumers. The parser consumer is native-only.
It exercises recursive/renamed DTOs, generic argument helpers, `Reserved`, typed
error downcasting, and independent Candid wire vectors for nat 2^128 and int -65.
This is representative boundary evidence, not a build of migrated siblings.

`make check-features` checks all sixteen feature combinations plus actual defaults on native/Wasm:
Candid/CDK-free defaults and certification-only selection, Candid-free executor
selection, optional certification isolation, explicit native-only parsing and no
Testkit transport. Empty default features are checked from Cargo metadata too.
`make wasm-check` compiles all features together while excluding the host parser. Consumer adoption
must also run the shared dependency-boundary checker on every workspace, including
independent fixtures and unused root catalog declarations; aliases are not escapes.

Direct dependency isolation does not remove transitive Candid from Agent, PocketIC,
protocol DTO packages or full CDK. No Wasm-size reduction has been measured.
Native Linux, native macOS Intel/Apple Silicon, Wasm compilation and installed IC
behavior are separate evidence categories. This batch does not qualify native
macOS, installed callbacks/trap recovery, publication or consumer adoption.

## Source provenance

The decoder and tests match the Testkit working tree at HEAD `55ac397`, reviewed
2026-10-18. Original MIT notices are retained. SHA-256 before adaptation:

- `crates/ic-testkit-candid/src/lib.rs`: `93d20eb5c28391f12df274993adbdc2de2ec2480175ff39cafa4b27bbe86996f`
- `crates/ic-testkit-candid/src/tests.rs`: `d69ba1128ee5ccc1d09c9ebbe6697089ff990f1cee18f65100c24f9a30d34b6c`

Working-tree provenance does not claim publication. Shared Tooling owns release
execution; the local adapter remains a thin consumer of its reviewed snapshot.
