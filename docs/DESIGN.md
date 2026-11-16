# Architecture

Status: proposed replacement design, 15 November 2026. Supersedes the supplied Tentative architecture. The companion migration plan below replaces the supplied Migration and qualification plan. This document authorizes no publication or repository deletion.

## 1 Objective and scope

Rebuild the owned Platform integration around explicit identity, execution, encoding and authentication contracts. Existing package layouts, public forwarders, generators and release scripts are evidence to evaluate, not a scaffold to import.

The objective is zero known unresolved correctness obligations at release, fewer independent representations of the same fact, and one owner per state transition. A package split or API rename alone does not satisfy the rewrite.

Keep required external bytes, identity semantics, consumer safety properties and documented resource refusals. Permit deliberate Rust API breaks. Do not rewrite cryptographic arithmetic to demonstrate originality. Do not preserve an executor implementation merely because the first draft declared it untouchable: establish its contract and evaluate its implementation against that contract.

The scope includes Types, Runtime and its entrypoint generation, external protocol codecs, verification, native integration, browser integration and qualification. Scheduling, storage allocation, application authorization, trust-root discovery, business retries and stable-data migration remain outside Platform.

## 2 Verified baseline and architectural findings

The public repository was inspected at commit `c24e3a59583eabcf5b6bb87987269c13a209a51f`. The inspection covered manifests, selected production implementations, public tests, browser boundaries, release orchestration and recent commit history. It was a source review, not an executed qualification or exhaustive security audit.

| Verified source fact | Design implication |
| - | - |
| Five product packages exist; the root declares Rust 1.100.0, Runtime 0.22.5 and Types 0.20.4 | Hold compiler and supplier versions constant during architectural comparisons; these are observed pins, not recommendations to upgrade |
| Platform 0.33.7 exposes identities, runtime forwarders, codecs, protocols, verification and native integration through features | Consumers need clearer owners; moving every existing module into another crate would preserve the underlying design |
| Prepared `call::Call` delegates to a lower public `cdk::call::Call`; the lower API supports reusable oneway calls | Consolidate to one consuming ordinary call API and one private completion backend |
| Completion state is Unarmed, Pending, Completed, Cancelled or Consumed; callback environments cross the ABI as owned raw Arc references | Preserve and independently prove custody and rollback obligations before replacing this implementation |
| The executor distinguishes method-owned protected tasks and migratory tasks | A generic spawn API would erase real context semantics |
| The codec's `Decode` trait extends `Encode`; Cloud Engine views separately declare Rust fields, schema, write order and read logic | Separate decode capability from encode capability; generate selected mappings from one field declaration |
| Historical authenticated certificate results expose public certificate/time fields | Prevent construction or mutation of evidence while retaining authenticated status |
| Release planning compares JSON stage names with regex-parsed Make recipes | Use one authoritative obligation catalog, preserving source/receipt admission |
| Browser tests cover exact SDK identity, selected Buffer bytes and no implicit discovery/retry | Preserve these contracts even if browser implementation needs little change |

Source anchors: [workspace manifest](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/Cargo.toml), [Platform manifest](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/crates/ic-platform/Cargo.toml), [prepared calls](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/crates/ic-platform-runtime/src/call/mod.rs), [completion backend](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/crates/ic-platform-runtime/src/cdk/call/future/mod.rs), [executor](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/crates/ic-platform-runtime/src/executor/runtime/mod.rs), [decoder](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/crates/ic-platform/src/candid_codec/decoder/mod.rs), [Cloud Engine mappings](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/crates/ic-platform/src/cloud_engine_types/views/mod.rs), [historical evidence](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/crates/ic-platform/src/certification/historical/mod.rs), [release plan](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/scripts/release/validation-plan.pl), [browser tests](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/packages/browser/tests/boundary.test.mjs).

Recent commits include selected engine-view/strict read-state changes, development-oracle admission, the Platform 0.33.7 release and its deployment record. Their messages establish history, not evidence that the proposed replacement passes. [Pinned history](https://github.com/dragginzgame/ic-platform/commits/c24e3a59583eabcf5b6bb87987269c13a209a51f).

## 3 Architecture decisions

Use four semantic owners. Six Rust packages are justified only by those owners plus Rust's macro compilation boundary and the executor singleton guard.

| Owner | Package and public surface | Does not own |
| - | - | - |
| Identity | `ic-platform-types`: Principal, Account, Subaccount and representations | Protocol schemas, executor, authorization |
| Execution | `ic-platform-runtime`: `system`, `call`, `task`, `entrypoints`, explicit `ic0` | Codecs, timers, retries, certificates |
| External formats | Proposed `ic-platform-protocol`: `candid`, `arguments`, `raw_rand`, named protocol modules | Transport, trust, endpoint lifecycle |
| Evidence and integration | `ic-platform`: `certification`, `canister_signature`, `crypto`, explicit `native` | Runtime forwarding, external Candid DTOs, application policy |

Runtime's macro package and guard are implementation support packages, released with Runtime. They are not separate product concepts. Protocol is a real implementation owner, not a forwarding crate. Confirm package availability and publishing authority before selecting its final registry name.

Dependencies point toward Types. Protocol has **no Runtime dependency**, including optional endpoint adapters. An endpoint composes an explicit Protocol decoder with Runtime's codec-agnostic entrypoint. This is a stronger boundary than the original proposal.

Platform has no Protocol or Runtime dependency. Native packet code may use its certificate parser and upstream native transport identities; it must not pull in external Candid DTOs. If a shared native format genuinely requires both owners, compose them in the consuming application first.

Browser remains a separate ESM package. Qualification remains a private workspace with no reverse production dependency. Do not create packages merely to hold tooling or accommodate a preferred directory diagram.

## 4 Canonical identity and type custody

Types owns a single canonical Rust Principal and Account identity across production consumers. Preserve principal bytes/text validation, management and anonymous values, account/subaccount interpretation, Serde shapes and self-authentication framing. Identity validity does not establish authority.

Byte equality does not imply Cargo type identity. Record package name, version, source and resolved feature closure at every actual crossing. Do not combine a local Types package with a published Runtime that selects a different source identity and describe that as unified.

Initial experiments may depend on published Types. This is a dependency choice, not rewrite progress. Transfer source ownership only after choosing one publisher and qualifying the local cohort. Do not republish an unchanged Types package simply because it now resides in a new checkout.

External SDK and certification types remain their admitted upstream types. Where native simulator and production identities necessarily differ, use one explicit, checked byte conversion at the boundary. No blanket conversions, certificate copying or new identity wrapper to hide incompatible supplier versions.

## 5 One call API with explicit outcomes

Proposed signature sketch; implementation must compile representative consumers before names are frozen:

```rust
pub enum Wait {
    Bounded { seconds: u32 },
    Unbounded,
}

pub struct Call<'m, 'a> { /* private prepared request */ }

impl<'m, 'a> Call<'m, 'a> {
    pub fn new(
        target: Principal,
        method: impl Into<Cow<'m, str>>,
        wait: Wait,
    ) -> Self;
    pub fn args<'b>(self, bytes: &'b [u8]) -> Call<'m, 'b>;
    pub fn owned_args(self, bytes: Vec<u8>) -> Call<'m, 'static>;
    pub fn cycles(self, amount: u128) -> Self;
    pub fn execute(self) -> CallFuture<'m, 'a>;
    pub fn notify(self) -> Result<Submitted, SubmitError>;
}

pub struct Completion {
    /* immutable response or rejection, plus callback-time observations */
}
```

`CallFuture` implements `Future<Output = Result<Completion, SubmitError>>`. Completion exposes immutable `Reply(bytes)`, `Rejected(code, text)` or `OutcomeUnknown(code, text)` observations plus available callback-time refund information. A task that is terminated may never produce a future result; its local lifecycle is observed separately. Unknown numeric rejection codes retain their raw values and do not acquire an invented settlement guarantee.

No Clone or reusable submission method on Call. Constructing a request or its future is inert. First poll attempts submission once. Dropping an unpolled future sends nothing. `notify` submits synchronously and consumes the request; its receipt establishes local submission only. Wait selection has no response-wait meaning for notify and must not create callbacks.

Arguments and method names may independently borrow or own storage. Borrowed input remains valid for the future's lifetime; no borrowed pointer crosses into a callback environment. The response owns its bytes.

Separate failure categories:

- **Not submitted:** local validation, insufficient balance or synchronous submission refusal.

- **Remote response:** exact rejection code and text, including unknown codes, or exact reply bytes.

- **Uncertain outcome:** best-effort response failure where remote execution cannot be established.

- **Local lifecycle termination:** the runtime stopped the task; this is not a synthetic remote rejection.

A bounded wait may return SYS_UNKNOWN even when the callee executed. Cleanup runs after a reply/reject callback trap, and cleanup can itself trap. These platform facts forbid translating timeout or cancellation into permission to retry. [IC system API](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/).

Capture refund observations in the actual response callback before unrelated polling. Distinguish unavailable observations from zero. A completion record is not a complete accounting ledger and cannot promise the final remote effect or settlement of a bounded-wait call.

Preserve exact raw rejection information while offering typed classification. Do not impose an arbitrary small reply limit in Runtime; decoding budgets belong to consumers, while platform transport limits remain explicit.

Remove the public lower Call builder after consumers migrate. Keep the IC0 capability explicitly advanced: selecting raw bindings permits bypassing ordinary call admission, and the documentation must not claim otherwise.

## 6 Executor and callback ownership

One executor owns task identity, method retention, ready queues and polling context. One transport completion owner owns each callback environment. They communicate through a private, narrow interface; neither owns business retries.

Model task ownership explicitly as method-attached or migratory. Preserve restrictions on query execution and admission of migratory work. Entrypoints establish contexts; arbitrary callers cannot fabricate a method handle.

The reviewed contract must distinguish:

| Event | Runtime obligation |
| - | - |
| Unpolled future dropped | Release local request; no submission |
| Submission refused | Reclaim callback environment exactly once; no live callback authority |
| Successful submission committed | Keep callback environment alive independently of the caller's local request |
| Future dropped after submission | Retire local interest according to the documented task contract; still settle callback custody |
| Reply or rejection | Publish completion once; wake outside internal borrows |
| Response callback traps | Apply qualified rollback/cleanup handling; never rely on native unwinding |
| Task terminated by cleanup | Remove queued authority and expose committed local termination when safely observable |
| Cleanup traps | Do not manufacture a successful termination observation or promise automatic recovery |
| Stale wake or callback | Cannot poll a recycled task or regain authority |

Dropping a handle is not cancellation. Any explicit cancellation API means stopping local polling; it does not cancel remote execution. A live physical call remains a transport obligation after its task loses interest.

No user future poll, arbitrary waker clone/drop, destructor or user hook may run while mutable executor/completion state is borrowed. Test reentrancy from destruction, not only ordinary polling. Keep unsafe callback-ABI transfer in one reviewed boundary with written ownership and rollback preconditions.

For Scheduling, supply a bounded read-only task observation through an opaque handle: running, completed or locally terminated with a reason, where that state has actually committed. Do not promise a user callback on every trap. The timer's committed dispatch record must cover initial execution traps before a task handle can be returned. If cleanup cannot establish termination, Scheduling must retain uncertainty rather than start another attempt.

This observation is an additive requirement, not a claim that the current Runtime already supplies it. Prove it in an installed canister before accepting the new Scheduling recovery contract.

Evaluate the existing executor against this model. Retain individual sound mechanisms where they fit; replace redundant state, overly broad public context manipulation or invalid ownership assumptions. Do not introduce a second executor for comparison inside one canister. Use separate artifacts.

## 7 Entrypoints and system access

Runtime owns one explicit-codec entrypoint generator. It handles argument capture, declared guards, execution context and reply ownership. It neither selects Candid implicitly nor imports Protocol.

An endpoint names its decoder and encoder, or explicitly selects raw/manual reply behavior. Application Wire codecs remain application dependencies. Protocol schema macros generate mappings only; they do not generate independent Wasm entrypoints.

Document and qualify guard/decode ordering for every entrypoint kind. Authorization that needs only caller identity must run before expensive decoding. A guard that depends on decoded fields is an explicit later application check. Never silently reorder old guard behavior during migration.

Only one owner may issue the reply. Compile-fail tests reject conflicting manual/automatic replies, unsupported async/query modes, invalid codec signatures and duplicate export declarations. Generated code respects renamed dependencies and conditional attributes.

The normal system surface is a small set of reviewed operations. Raw IC0 remains an opt-in supplier boundary with its execution-context restrictions. Runtime supplies stable-memory system calls where required, but no stable allocator or persistence policy.

## 8 Protocol declarations and bounded decoding

The new Protocol owner is justified by a different implementation model, not by extracting approximately 5.3k lines from Platform.

Each supported external shape has one declaration containing numeric field/variant labels, Rust fields, required/optional behavior and explicit decode policy. Generate schema and the selected encode/decode implementations from it. Serde remains a separate representation with its own compatibility tests.

Split `Encode` and `Decode`. A response projection should not require a writer for data it was never intended to send. Internally they may share a schema descriptor where useful; the public Decode trait must not inherit Encode. Do not add a second reflection system.

A selected-record declaration needs three distinct field policies:

| Policy | Missing field | Incompatible present field |
| - | - | - |
| Required | Refuse | Refuse |
| Strict optional | None | Refuse |
| Candid-compatible optional | Apply specified Candid compatibility rules | Apply those same documented rules |

Strict optional does not require a new public Option wrapper. Validate the complete skipped value for unselected fields, charge both cumulative skip work and total work, and reject malformed trailing data. Do not decode unselected large vectors into temporary DTOs.

Use an illustrative declaration such as:

```text
projection EngineView
    field id                required
    field subnet_id         strict_optional
    field engine_operator_id strict_optional
    field subnet            strict_optional
    field deleted_at        strict_optional
    field node_ids          required
    unknown_fields          validate_and_skip
```

The actual syntax must also include the canonical numeric labels. Choose the smallest declarative macro that expresses this once. Introduce a derive only if the macro experiment fails clearly stated usability or correctness requirements. No generated code checked in as a second editable schema.

Preserve explicit Candid byte, header, type-table, work, logical-allocation, vector and skipped-work controls and the existing 16-level constructed-wire ceiling until a separately qualified change is approved. Logical allocation charges are not a whole-process heap bound. Public custom mappings are trusted code and cannot inherit an unconditional resource guarantee.

Keep narrow argument and raw-rand codecs independent of general Candid, bigint, Runtime and crypto. Maintain Nat/Int ranges, arity, field labels, function/service references, ledger hash domains and exact account framing. Ledger hashing uses its admitted primitives directly.

Maintain only selected protocol families with named consumers: management, ledger variants, Registry, Cloud Engine, assets, CMC and cycles where the requirement ledger confirms use. Lack of a fixture is not evidence of no consumer. Retire a family only after its consumer inventory is resolved.

Do not add DID parsing, a service printer, endpoint generation or a new persisted format. Existing legacy Candid storage remains a consumer migration obligation; changing crate boundaries does not authorize changing stored bytes.

## 9 Authentication as immutable evidence

Separate these operations:

1. Decode bounded, unauthenticated bytes.

2. Authenticate them under explicit trust and verification inputs.

3. Let the consumer accept them for a specific application action.

Proposed public contract:

```rust
pub fn decode_certificate(
    bytes: &[u8],
    limits: CertificateDecodeLimits,
) -> Result<DecodedCertificate, DecodeError>;

pub fn authenticate_certificate(
    decoded: DecodedCertificate,
    root: &PreparedRootKey,
    effective_canister: Principal,
    time: TimePolicy,
    limits: VerificationLimits,
) -> Result<AuthenticatedCertificate, VerifyError>;
```

DecodedCertificate and AuthenticatedCertificate have private fields. Authenticated evidence has no public constructor, mutable certificate accessor or unvalidated Deserialize implementation. Expose immutable tree/time/binding views. An `into_unverified` conversion may surrender the authentication wrapper; it cannot retain that status after edits.

Retain exact original bytes when required for native/historical evidence. Verification must bind the precise certificate, trust root, effective canister and explicit time policy. Returned context records what was checked; it must not be reusable as proof under another root or target.

Use one private implementation of shared certificate/delegation mechanics. Keep policy differences explicit: historical windows, signing-certificate time, delegation time and live acceptance must not collapse into a Boolean or implicit current clock.

Preserve every applicable DER, subgroup, identity-key, root, signature, delegation-depth, range, witness and tree-ordering check. Validate complete selected range data, including malformed material after an apparent match. A witness match alone does not authenticate a certificate. Unknown/pruned lookup is not absence.

Prepared keys remain caller-owned. There is no global verification cache, default root, silent root fetching or fallback from strict verification to a permissive path. Certificate authentication does not prove provider identity, current authorization, application freshness or retry safety.

Use separate CertificateDecodeLimits and VerificationLimits. A CBOR input-byte allowance must not be labeled a heap/work quota. Add stronger structural bounds only with defined accounting and adversarial proof, rather than relabeling an existing byte limit.

Cryptographic primitives remain admitted external implementations. Rebuild only Platform's checked input adapters and composition where architecture requires it. Preserve exact hash domains and framing; do not introduce new pairing, curve, point-codec, hash-to-curve or hash implementations.

## 10 Native and browser integration

Native integration exposes the actual upstream Agent identity and typed native packet inspection. It must retain original packet bytes and selected-buffer boundaries; decoding or re-encoding a packet does not recreate authenticated original evidence.

Agent network authentication and the portable evidence verifier have explicit roles. Where an adapter combines them, pass explicit trust inputs and document which verifier establishes which claim. Do not imply that two different policies are equivalent because both return success.

Keep browser exports explicit by subpath and preserve upstream SDK class identity. Do not build another Principal implementation, generic CBOR export or Rust-crypto equivalent in TypeScript. Keep roots, byte custody, transport selection and retries explicit. The packed npm consumer is the distribution proof, not a source-directory test.

Select native-on-Wasm rejection for the replacement: choosing a native-only capability on a Wasm target must emit a clear compile error. This is a deliberate break from the currently target-gated silent exclusion. Test both the negative case and all supported portable profiles. Do not run Wasm all-features and treat the intentional rejection as a product failure.

Browser does not release merely because Rust changes. A reviewed unchanged browser component may stay unchanged; that is not counted as rewritten code.

## 11 Feature and release contracts

All defaults remain empty. Feature names express capabilities with measurable dependency closures.

| Selection | Required exclusions from normal and build closure |
| - | - |
| Types base | Runtime, general Candid, crypto algorithms, Agent |
| Runtime raw IC0 | Executor, application identity unless required by its specific adapter, codecs, Agent |
| Runtime executor | Codecs, Scheduling, stable allocator, native transport |
| Runtime calls | Implicit codec, retries, freshness policy |
| Narrow Protocol arguments/raw-rand | General codec, bigint, Runtime, crypto |
| General Protocol | Registry Candid implementation, BLS, Agent, executor |
| Certificate parse/tree | BLS verification, Runtime, Agent |
| Portable verification | Runtime, Agent, Candid and transport |
| Native integration | All production Wasm artifacts |

Do not reduce feature count by merging incompatible dependency closures. Preserve exact upstream certification identity; an Agent pin can constrain the admitted certification major. Resolve that constraint explicitly rather than copying certificates between incompatible versions.

Types, Runtime, Protocol and Platform have separate release groups; Runtime includes its exact guard and macros. Browser is separate. A group boundary is not automatic permission to skip qualification.

Use one reviewed profile catalog for obligations, feature/target closure, commands, input identity, required host and artifact. Generate execution entrypoints from it. Remove parallel handwritten rosters and Make-regex extraction only once equivalent admission tests pass.

Shared tooling owns execution custody, receipts and publication recovery. Platform owns its obligations and supplier adapters. Do not rebuild Shared locally to make the new catalog work. Until supported integration passes, retain the full release obligations using the existing qualified route.

## 12 Deletions and reuse admission

Remove at the hard cut:

- Platform's Runtime, executor, IC0 and identity-only facade paths.

- Platform's external Candid/DTO ownership after Protocol consumers migrate.

- The lower ordinary public Call builder and repeated-call semantics.

- Protocol-to-Runtime endpoint adapters and duplicate entrypoint generators.

- Hand-maintained field/schema/reader/writer duplication for migrated declarations.

- Mutable or forgeable authenticated-evidence wrappers.

- Duplicate qualification rosters, recipe parsers and redundant release wrappers once replacements prove equivalent.

- Dead features, compatibility aliases, fixtures for deleted contracts and obsolete active design documents.

Keep old source and historical release evidence at immutable references. Do not copy the entire design history or script tree into the new active repository.

Reuse a function or tightly bounded module only after its destination contract exists. Record source commit, license/attribution, ownership assumptions, dependencies, safety obligations and independent evidence. No whole-crate imports followed by a promise to simplify later.

Every implementation review must show required behavior, complexity removed and verification. “Moved 5,300 lines into Protocol” is not a completion milestone.

The companion plan defines ordering and release gates. Unresolved launch-critical lifecycle, identity, byte-compatibility or verification obligations block release.

# Migration and qualification

Status: proposed implementation sequence, 15 November 2026. Replaces the supplied Migration and qualification plan and implements the architecture above.

This is a hard cut of internal Rust APIs with preservation of required external behavior. It is not a bulk source migration. Delete the two superseded active drafts only after these replacement files occupy their intended repository paths; preserve historical evidence in version control.

## 1 Start with contracts and evidence

Pin the old source at `c24e3a59583eabcf5b6bb87987269c13a209a51f`. Record the compiler, dependency sources, locks, target settings and fixture hashes. The reviewed root declares Rust 1.100.0; keep it constant for comparisons rather than combining the redesign with compiler changes.

Create a requirement ledger with one row per actual consumer obligation:

- Consumer and named call site or published contract.

- Exact public type/behavior/byte format it depends on.

- Required feature/target/dependency closure.

- New owner and intended replacement API.

- Positive and negative acceptance cases.

- Existing proof, replacement proof and any intentionally retired behavior.

Resolve current consumers of Types, calls, task contexts, entrypoints, protocol families, certificate/signature APIs, native packets and browser subpaths. Use repository search and final dependency graphs, not assumptions that unused-looking APIs are dead.

Classify compatibility explicitly:

| Contract | Treatment |
| - | - |
| External protocol bytes, identity framing, hash domains and Serde formats | Preserve unless separately approved as an external contract change |
| Existing persisted Candid values | Preserve; any storage migration belongs to the consumer |
| Security refusals and documented resource accounting | Preserve or deliberately strengthen with consumer tests |
| Internal Rust import paths, builder cloning and facade exports | Break and migrate together |
| Task ownership and callback/refund timing | Preserve semantics; qualify any intentional change separately |
| Native SDK/browser class identity | Preserve admitted upstream identity |
| Package versions and repository publisher | Select during release planning; no automatic 1.0 or registry cutover |

Tests that only assert old Clone traits or removed import paths are candidates for retirement. Tests that exercise cancellation, malformed data or security refusals remain obligations even if their implementation changes.

Exit gate: every removal has a consumer disposition and every retained obligation has a proof owner. Unresolved inventory blocks deleting that surface.

## 2 Establish the smallest workspace

Create manifests only when a real vertical slice requires the package. Use four product owners with Runtime's macro and guard support packages; do not prepopulate source trees from the old checkout.

Reuse published Types for early pure-code experiments where practical. Do not link a local Types source alongside a published Runtime cohort and silently convert between them. Bring local Runtime and Types together in one deliberately qualified dependency cohort when necessary.

Select installed Shared/Toko entrypoints actually required by the repository. Preserve managed company files and the existing license source. Do not copy old release scripts merely to obtain a passing baseline.

Before publishing any existing package from a new repository, designate one authoritative publisher. Keep publication disabled until that decision is made.

Exit gate: minimal consumers resolve the intended package identities; production graphs contain no qualification/oracle packages.

## 3 Prove the new Protocol model with one real response

Implement Cloud Engine EngineView/EngineSubnet as the first projection. It exercises required fields, strict optional values, nested records, unknown-field skipping and canonical Principal identity.

Build its declaration mechanism before adding other DTO families. Decode does not require Encode. Generate encoding only for declarations that need it. Keep Serde independently tested.

Use frozen literal frames and independent oracle declarations. Do not generate the expected result with the same schema generator under test.

Required cases:

- Required field absent; strict optional absent, None, valid Some and incompatible present value.

- Unknown field valid, malformed, deeply nested and oversized.

- Duplicate/out-of-order labels, malformed type references, arity and trailing bytes.

- Empty vector versus no argument; signed and unbounded integer boundary cases.

- Exactly-at and just-over each byte/header/type/work/allocation/vector/skip limit.

- A failed optional compatibility attempt cannot refund consumed budget.

- Projected decoding does not allocate complete unselected DTOs.

- External text/bytes/Serde forms remain compatible.

Only after this gate, add required management and ledger families incrementally. Share declaration machinery; do not introduce a new generator per family.

Exit gate: at least one real consumer uses the new projection; its declaration has one source of field/schema truth, with measured allocation/work behavior. A renamed copy of the old mappings fails this gate.

## 4 Prove one Runtime call and its ownership transitions

Implement one inert prepared call, one first-poll submission path and one response. Then add consuming notify, borrowed/owned method and argument combinations, cycles and explicit wait selection.

Review the executor and completion backend against the new ownership model before admitting source reuse. Retaining a sound mechanism is allowed; retaining an entire implementation without this review is not.

Use separate old/new canisters with an external driver. Do not install two executors in the same Wasm. Qualify guard rejection for accidental mixed cohorts.

Required installed cases:

- Construct/drop unpolled request; successful first poll; submission refusal.

- Owned arguments survive source destruction; borrowed input never becomes callback-owned.

- Reply, rejection, unknown rejection code and bounded-wait uncertainty.

- Local future drop after submission and eventual callback custody settlement.

- Callback trap, trap after await, initial task trap and cleanup failure.

- Destructor/waker reentrancy, stale wake, subsequent successful call after recoverable failure.

- Query/method-attached/migratory context isolation.

- Callback-time refund capture versus later task polling.

- One-way local receipt with no implied remote delivery or callback.

- Stop/drain/upgrade/start: require IC status Stopped before replacing code,
  prove outstanding callbacks settle first and fresh calls work after restart.
  Suspended Rust futures/callback registrations are not persisted across upgrade.
  Upgrading with live callbacks is outside the supported contract; do not add a
  recovery subsystem or pre-upgrade guard to make that workflow work.

Implement Scheduling's task observation only with a precise committed-state contract. A cleanup failure cannot be converted into “terminated successfully.” Prove the interaction with a committed timer dispatch record. Platform qualification uses a minimal probe for this contract and must not depend on the Scheduling product release.

Exit gate: callback environment custody, method/task lifetime and uncertainty have installed evidence. No assumed destructor-based trap recovery remains.

## 5 Rebuild verification around immutable evidence

Start with one direct certificate and one delegated certificate using explicit root, target, time policy and resource bounds. Introduce private decoded/authenticated wrappers and immutable accessors.

Consolidate common mechanics only after listing differences between historical certificate authentication and canister-signature verification. Do not make weaker optional checks the common denominator.

Required refusal cases include wrong root/target/witness, malformed DER, invalid or identity keys, subgroup rejection, bad signature, nested delegation, ambiguous time, out-of-window signing/delegation time, malformed ranges after an apparent match, bad shard selection, duplicate labels and pruned/unknown lookups.

Preserve the existing historical tests' obligations, including exact boundaries and independent observed frames. [Historical public tests](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/crates/ic-platform/tests/historical_certificate_consumer.rs).

Add compile-fail evidence that consumers cannot construct authenticated values, mutate their certificate/time/binding or deserialize arbitrary bytes directly into authenticated status. Test any consuming conversion back to unverified data.

Exit gate: all required verification profiles pass positive, negative and differential checks; original bytes and checked context stay bound. No algorithm rewrite or implicit root selection is introduced.

## 6 Migrate entrypoints and real consumers

Use one Runtime generator with explicit decoder/encoder composition. Migrate a Wire application endpoint and a required external-Candid endpoint. Verify guards, manual replies, async context and renamed dependency paths.

Then migrate consumers by owner:

1. Passive DTO users select Types or Protocol directly.

2. Canister execution users select Runtime directly.

3. Scheduling adopts the qualified lifecycle observation and exact Runtime cohort.

4. Entropy uses narrow raw-rand encoding plus its existing call/admission policy.

5. Authentication consumers accept immutable evidence and retain their own freshness/authorization checks.

6. Native users select explicit Agent/packet capabilities.

7. Browser users retain explicit SDK subpaths and packed-package identity.

The exact consumer repository revisions must be recorded in the requirement ledger before this step is considered complete. The list above is a migration classification, not proof those consumers have already been audited.

Keep application bodies equivalent in comparison fixtures. Disclose API adapters and count their code. No permanent old-API compatibility facade.

Exit gate: real consumers compile and execute against one local candidate cohort without old Platform runtime/DTO forwarders.

## 7 Replace qualification orchestration without dropping proof

Create one catalog of obligations. Each record includes stable ID, owner/profile, exact command, target/features, normal/build/dev closure, input identity, required host and output artifact.

Derive Make entrypoints and admission rosters from this catalog. Do not maintain another handwritten JSON list and another regex recognizer.

Map every old stage to its replacement or an explicit retirement reason. The old full gate remains the obligation floor until the replacement passes:

- Missing, duplicated and changed profile detection.

- Changed source, tool, lock, features, targets and host invalidation.

- Missing, stale or tampered receipts/artifacts.

- Interruption and resume with identical versus changed inputs.

- Local-path success versus packaged/archive consumer success.

- Registry publication uncertainty and reconciliation.

- Required installed-server and hosted Linux/macOS evidence.

Current receipt admission explicitly checks retained evidence and source identity. Preserve those obligations while replacing its structure. [Existing admission gate](https://github.com/dragginzgame/ic-platform/blob/c24e3a59583eabcf5b6bb87987269c13a209a51f/scripts/release/validation-gate.sh).

Do not activate cross-release proof reuse or group-scoped omissions until Shared supplies a supported contract and the new integration passes these tests. The supplied draft's Toko oracle-admission issue is a supplier dependency to recheck at implementation time; its current resolution was not established by this review. Do not widen local exceptions to conceal it.

Exit gate: one authority defines required proof, and tampering or incomplete execution cannot admit a release.

## 8 Measure the actual redesign

Compare equivalent source behavior under identical compiler, locks, optimization, features, LTO and codegen settings. Record old/new Wasm hashes. Keep native and Wasm results separate, and distinguish production closures from development/oracle closures.

Measure:

| Profile | Evidence |
| - | - |
| Types and raw system access | Dependency closure, raw/optimized Wasm and compile cost |
| Calls and executor | Instructions, callback/task retention, cleanup and cancellation behavior |
| Narrow argument/raw-rand | Wasm size and exclusion of general codec/runtime |
| Management and EngineView projection | Encode/decode instructions, allocation charges, skip work and malformed-input cost |
| Ledger framing | Exact bytes/hashes and representative cost |
| Certificate/signature | Parse, key preparation and verification costs separately |
| Native/browser | Original-byte custody, SDK identity and packed-consumer behavior |
| Qualification | Number of independent declarations/rosters, wall time and retained artifact volume |

Correctness tolerance is zero for broken required contracts. Before implementation measurements, record numerical performance thresholds for each workload from the baseline and launch needs. Any regression outside a threshold requires an explicit tradeoff decision and updated acceptance record; it cannot be hidden in an aggregate improvement.

A package split earns no size claim. Smaller source or test counts earn no correctness claim. Include new consumer glue and supplier/tooling changes when reporting maintenance cost.

## 9 Cutover and final release gate

Remove the old builders, facade paths, duplicate mappings, mutable authenticated wrappers, dead features and replaced release machinery only after their replacement gates pass.

Before the release cut:

- All required consumers use the selected package/source cohort.

- Every security and byte-compatibility obligation has passing evidence.

- Native-on-Wasm rejection and supported portable profiles both pass.

- Runtime singleton/exact guard behavior is verified in final artifacts.

- Cargo archives and packed browser consumers pass without hidden path patches.

- No correctness-critical TODO, unqualified cleanup assumption or temporary compatibility layer remains.

- A single publisher and next version are selected for every changed package.

- Baseline references and old release evidence remain accessible.

Use deliberately breaking versions for removed APIs under the repository's chosen versioning policy. Do not infer a 1.0 release or unnecessary Types/browser release.

Release rollback restores a known dependency/artifact cohort; it does not undo management calls, external effects or stored-data changes. Reconcile uncertain publication before retrying it. Keep application recovery in its existing owner.

The rewrite is complete when consumers use the new contracts and redundant responsibilities have been removed. Ported line counts and green inherited tests alone are insufficient.
