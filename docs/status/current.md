# Current handoff

## PR #71 gzip feature-validation correction (2026-10-27)

The gzip branch adds a native leaf without Principal. The previous graph-cache
admission assumed every native leaf selected Principal and therefore refused
gzip before querying the Wasm graphs. Each native leaf now declares its exact
local prerequisites alongside its dependencies; gzip has none. Existing optional,
target, dependency-count and alias checks remain strict. Regression fixtures cover
native/Wasm and mixed selections, changed contracts, malformed policy, and actual
Cargo metadata. No codec, public API, dependency or cryptographic code changed.

Qualification: full `make clippy` passed before longer tests; `make fmt-check`,
ShellCheck, gzip's native consumer test, inventory and documentation-link checks
passed. Rust 1.91 gzip-only Wasm compilation and complete routine
`make check-features` passed: 415 selections per target, 384 native and 234 Wasm
graph queries, plus native-empty and Wasm executor compilation. Hosted macOS and
installed-provider acceptance are separate; this correction adds no such evidence.
The existing advisory blocker remains tracked by #50. No release or publication
is included. The earlier handoff below records prior work, not current PR status.

## Source and authorized boundary

Released main remains Platform 0.6.21 at
`43928b07c9d04a63b6a77f90b185a5c4afda89fa`; npm browser 0.1.1 is separately
published. Shared Tooling remains verified 0.26.21 at
`66c91c4412856baa147482e648fc80428db8b179` (106 payloads).
The maintainer explicitly requested #38 after the prior blocker assessment.
That narrow compiler retirement is now prepared for **0.7.0**, the next breaking
minor; own package versions remain 0.6.21 until maintainer release preparation.
No staging, commit, push, release, publication, deployment or paid operation was
performed in this task. Incoming post-release documentation edits are preserved;
Cargo.lock and the original index are byte-identical. Siblings remain read-only.

## Compiler retirement (#38): implemented locally

Removed native `candid_parser::{compile_javascript, compile_typescript}` and the
unused compiler-only `IDLMergedProg` export. Removed only the exhausted browser
compiler assertions from the standalone parser consumer. No compatibility alias,
replacement generator, parser fork or fallback codec was added. Remaining APIs
retain external parsing/typechecking, import detection, textual argument encoding,
constructor typing, service comparison, dynamic refusal/value serializers and
Rust type equality. The separate external `candid-service` feature stays intact.
The migration contract includes the retained API/consumer inventory and exact
[0.7.0 removal instructions](../migration.md#pending-070-compiler-only-retirement-38).

The audit found no references to any removed name in Canic default `aed32be2`
crates or current Canic/Query/Host/Tooling/Blob/Auth/Backup/IcyDB/Jobs/Metrics/
Persistence/Relay crate sources. Host retains its external service inspector;
Canic retains `fleet_ensure::ops::init_arguments`. Owned frontend artifacts now
use Wire metadata/bindings. Canic's owner records passing installed frontend
handoff (275s) and later dedicated/embedded generated-client proof (637s) at
local `b9482ec526c5dfb9f5dd349d22b1f60190b2dfdf`. The inspected qualification
record is captured under `/tmp/platform-issue38-20261027/canic-qualification.json`;
its source/hash are recorded in the migration contract. These are retained owner
observations, not new sibling execution, full #613 closeout or live deployment.

Full `make clippy` passed before longer tests. Standalone parser (2), dynamic
refusal (2), external service (1) and pinned external schema (5) tests pass on
native and Rust 1.91 with no direct consumer Candid/parser dependencies.
Fixed nat64 argument bytes, malformed grammar, undefined types, import detection,
service drift and constructor/type equality remain covered. The service consumer
also compiles for Wasm. Rust 1.91 parser-feature Wasm library compilation passes;
a real Wasm consumer import of the host parser is correctly refused.
Actual native/Wasm default, executor, runtime, entrypoint, parser and external
service normal/build graphs pass. Snapshot, inventory, formatting, doc links,
diff checks and 0.7.0 pending changelog admission pass.

Evidence: `/tmp/platform-issue38-20261027/{clippy,consumers,graphs,wasm,policy}.log`,
`target/boundary-consumer.jtMvNt` (parser), `.Kkxhr9` (dynamic), `.PjyEZx` (service),
`.36PvZw` (external schemas), `target/issue38-graphs` and
`target/parser-wasm-exclusion.rKTWnF`. No new native macOS, installed-canister or
full release CI run is claimed; those remain separate release/consumer duties.

The dependency remains candid_parser 0.4.1; actual native graphs still include
Handlebars 6.4.4, derive_builder 0.20.2 and Darling 0.20.11. Only supported facade
surface is retired. No transitive dependency removal, cryptographic/runtime/data
change, Wasm-size improvement or performance gain is claimed. Existing external
Candid bytes, checkpoints, aggregate limits and retry/effect policy stay unchanged.
Keep #38 open until the committed cut reaches default main and its minor-release
requirements are satisfied. Use a minor release, not release-patch. Other hard
cuts retain their existing restriction; this task authorizes no broad retirement.

## Existing published suppliers and consumer work

Platform 0.6.20 supplies optional CMC, ICP Ledger, Cycles Ledger, Registry and
native asset DTOs. Published 0.6.21 adds the exact pinned management lifecycle/
settings/ECDSA/history and ICRC transfer/memo/consent exports. Its verified archive
SHA-256 is `f82b7556c1a6acf55a82b0bdc789fc6d4cb4f9c3e0113e911354817387b866e0`.
Fresh registry-only management/ICRC native/MSRV/Wasm proofs passed before this
cut (`/tmp/platform-0621-live`, `target/boundary-consumer.Zt7IyX`, `.j3pDdx`).
The [external contract guide](../external-contracts.md) and migration contract own
exact imports/provenance and canonical shape differences. Those external contract
surfaces remain unchanged by #38. #61–64 stay open for Canic's actual adoption;
Canic's dirty 0.112 migration remains owner-controlled, not a new published release.
No old 'management export supplier is unpublished' blocker should remain.

#59 is closed: public `@tokostudios/ic-platform` 0.1.1 supplies the four SDK
signing-request/expiry exports with Core exactly 6.1.1. Archive SHA-1 is
`33bf72eaf3039111c462072d5d2bc0aa6dd80f45`; registry metadata, exact integrity,
packed/Chromium proofs and fresh registry-only strict TypeScript/runtime evidence
are retained in `/tmp/platform-npm-011`, `target/npm-registry-consumer.1ClKKR`.
Auth/application adoption and provider acceptance remain separate.

## Advisory status

#56 is closed: released CI has complete index observations for both maintained
locks; the remaining workspace audit failure is #50. Live workflow evidence is
[40472663771](https://github.com/dragginzgame/ic-platform/actions/runs/40472663771),
retained in `/tmp/platform-backlog-20261027/advisory-ci-full.log`. No advisory was
suppressed; earlier prepared/yanked/missing-index regressions remain retained.

#50 remains unresolved. Latest observed compatible crypto-bigint is 0.5.5;
patched >=0.7.6 is outside the selected K256/P256/elliptic-curve 0.13 family.
Latest Agent 0.49.3 and exact ic-secp256k1 0.3.0 retain those coupled paths.
Resident private-key signing precludes a blanket verification-only exemption.
No compatible correction, exemption, override, Agent fork or in-house crypto
change is supplied. Raw records are in `/tmp/platform-0621-live` and the earlier
backlog evidence. This compiler retirement does not qualify crypto safety.
