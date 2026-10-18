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
Arbitrary-precision integers enable Candid's `bignum` feature only with `candid`.
No default features, parser, automatic limits or second executor are introduced.

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
   independent Serde use need not move. Principal-only code may use `ic_principal`.
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

Host's pending decoder first rejects oversized replies, then atomically subtracts
admitted bytes from the caller's remaining budget before decoding. Malformed
admitted replies still consume budget; failed admission does not. Consolidation
must preserve those semantics outside the per-reply Platform decoder. Host's
pending helper does not select compact diagnostics; adopting Platform changes
that diagnostic policy and needs explicit caller qualification.

Host's service inspector uses `IDLProg`, type checking and `TypeEnv` to project
bounded method facts, rejects imports, and leaves Wasm export identities to Host.
Canic additionally consumes parser bindings, textual arguments and service equality;
method inventory alone cannot replace those uses. A future explicit host-only
feature must cover the justified parser operations and their limits. It must not
be enabled by canister/runtime features or pull Host/Testkit back into Platform.
No host parser has been relocated in this batch.

## Evidence and isolation

`make test` and `make msrv` exercise the retained six decoder cases and expanded
DTO fixture. `make check-consumer` builds a separate workspace with only Platform
as a direct dependency: native tests on Rust 1.99 and 1.88 plus Wasm compilation.
It exercises recursive/renamed DTOs, generic argument helpers, `Reserved`, typed
error downcasting, and independent Candid wire vectors for nat 2^128 and int -65.
This is representative boundary evidence, not a build of migrated siblings.

`make check-features` checks normal/build graphs for empty defaults, Candid-free
native/Wasm executor selection and no parser in current all-feature selection.
`make wasm-check` compiles both supported features together. Consumer adoption
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
