# Current handoff

## Source and release boundaries

Baseline is released Platform 0.6.19 (`0502bce`), plus merged changelog-only
[PR #60](https://github.com/dragginzgame/ic-platform/pull/60) at `c75fdff`.
Compatible pending notes target 0.6.20; own Rust/npm package versions are unchanged.
Shared Tooling 0.26.21 at `66c91c4412856baa147482e648fc80428db8b179` is adopted
locally through the canonical exporter (106 payloads). Existing snapshot/browser
work and unrelated root lock edits are preserved. No staging, new commit, push,
release, publication, deployment or paid operation was performed for this batch.
Sibling checkouts remain read-only.

## External DTO supply (#61–64)

Five optional modules now supply selected external Candid contracts without any
new package dependency or service engine: CMC top-up/mint/rates, legacy ICP
transfer/encoded blocks, Cycles Ledger creation/withdrawal, Registry subnet lookup
and native-only certified-asset reads. Defaults, runtime and executor do not imply
these features. Asset exports are absent on Wasm; its explicit Candid prerequisite
remains if that feature is selected there. No feature implies a host parser.

The [external contract guide](../external-contracts.md) owns exact feature/import
mappings, source/license provenance, pinned DID hashes, projection differences and
consumer qualification. Required external Candid is preserved. Rate/tree/certificate
fields, asset `content_type`, exact archive query signatures, unbounded nats,
creation duplicate/refund evidence and all withdrawal rejection variants are retained.
Cycles Ledger creation settings are distinct from management/current CMC settings.
Registry uses `Some(target)` for both quotation/submission; its canonical optional
field changes request bytes relative to Canic's old required-principal projection.
Upstream subtyping admits the old request, but that is not byte identity.

CMC/ICP/Cycles/asset declarations are narrowly schema-derived with retained Apache
licenses. Registry's DID is under an IC-1.0 ancestor; it is not imported or
mislabelled Apache. Its tiny public protocol facts are independently expressed in
Platform Rust/tests. No service implementation or cryptographic body is copied.
These are Platform-maintained DTOs, not upstream Rust struct identities.

Full `make clippy` passed before longer tests. Standalone consumers declare only
Platform: portable CMC/ICP/Cycles/Registry tests (4) and native asset tests (1)
pass on native and Rust 1.91, with corresponding Wasm compilation. Native schema
proofs (5) pass on native/MSRV, comparing complete selected types against pinned
DIDs and Registry protocol facts. First schema attempt exposed an unquoted
reserved field in the test input; corrected proof passes, earlier failure retained.
Asset tests exercise first/chunk responses, bounds, missing/wrong digest and wrong
encoding through an explicitly consumer-owned fixture policy, not Platform defaults.

Evidence: `target/boundary-consumer.yPe7An` (portable),
`target/boundary-consumer.1wsIeE` (assets),
`target/boundary-consumer.O62pVF` (schema), `target/external-contract-graphs`;
logs under `/tmp/platform-61-64`. Actual native/Wasm default/executor/runtime and
all five DTO feature graphs pass. Graph refusal regressions and the updated
feature-selection model pass. Source fixture hashes, root lock preservation,
snapshot/inventory/documentation and pending changelog admission were checked.
`target/assets-wasm-refusal.dbJJNN` proves the host asset API cannot be imported
on Wasm; package listing includes every new DTO module and retained license.
Native macOS, installed services, full release CI and Canic recovery/payment
qualification were not run. No size/performance improvement is claimed.

#61–64 remain open: no supplying publication or Canic adoption is claimed.
The maintainer must release the supplying version before Canic removes mirrors;
Canic retains target authority, limits, original uncertain retry bytes, journals,
fees/refunds, receipt authentication and frontend acceptance.

## Pending browser extension (#59)

Explicit `/agent` exports include upstream `Endpoint`, `SubmitRequestType`,
`ReadRequestType` and `Expiry`, matching Auth's actual signing-request fixtures.
SDK Core remains exactly 6.1.1; cryptographic implementation and policy stay upstream.
Full Clippy, all 16 browser adapter/graph tests, strict packed consumers and
Chromium page/module-worker checks passed. Signing requests preserve exact SDK
identities, expiry serialization and signed envelopes for call/query/read-state.
Logs: `/tmp/platform-0620-{clippy,browser,chromium}.log`; evidence:
`target/browser-consumer.891Oy8` and `target/browser-consumer.JWXVZk`.
Published npm 0.1.0 lacks these exports; a compatible npm release and Auth's own
consumer qualification remain required. No npm bump/publication was performed.
Earlier browser supply is recorded in closed #57; Blob #103 and Canic #637 retain
actual adoption. Local producer proofs do not establish consumer acceptance.

## Released advisory repair (#56) and crypto blocker (#50)

The #56 fix is present on released main in 0.6.19. Both maintained audit inputs
are prepared using isolated locked Cargo fetches without compilation. Raw audit
statuses and complete/incomplete index observations remain separate; missing
records reject coverage even if cargo-audit exits zero. Offline mode claims no
freshness. Evidence: `target/issues-38-50-56/online-audit-final`, `yanked.log`,
`missing-index-probe`; `/tmp/platform-issue56-online-final.log`.
The real workspace failure for crypto-bigint remains visible; runtime audit passed.

#50 remains unresolved. Agent 0.49.3 and exact ic-secp256k1 0.3.0 retain the
K256/P256/elliptic-curve 0.13 family and affected crypto-bigint 0.5.5; the previously
observed patch is >=0.7.6. Resident private-key signing prevents a blanket
verification-only exemption. No compatible correction, exemption, Agent fork or
in-house cryptographic replacement is supplied. Prior graphs/registry records
remain under `target/issues-38-50-56`; they are not a fresh feed check this batch.

## Compiler retirement (#38)

The earlier Canic audit found live JS/TS compiler calls on its default branch;
its working Wire migration was not qualified adoption. Retirement still requires
actual consumer adoption and an authorized breaking boundary. Host's bounded
external service inspection and textual external/init arguments retain parsing.
No parser fork, compiler hard cut or dependency/size reduction is claimed here.
Previous Wire 0.7.16 qualification is retained in `/tmp/platform-adoption-*.log`
and its migration contract; this batch does not claim a fleet Wire upgrade.
