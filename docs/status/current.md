# Current handoff

## Source and release boundaries

Default main and published Platform 0.6.20 are `401a8327b669c82fe34de82963a48a952061d1d4`.
The issue batch began clean. New compatible Rust notes target 0.6.21; the Rust
package version is unchanged. The user separately authorized npm 0.1.1 publication,
completed below; its manifest/lock version edits remain uncommitted.
Shared Tooling remains 0.26.21 at
`66c91c4412856baa147482e648fc80428db8b179` (106 verified payloads).
No staging, commit, push, Rust release, deployment or paid operation was performed.
Sibling checkouts remain read-only, including Canic's active dirty batch.

## Remaining Canic export patch (pending 0.6.21)

Adopted only the source exports from Canic's prepared external-contract patch,
reviewed at `b9482ec52`; its hash/provenance and exact mappings are recorded in
[the migration contract](../migration.md#canic-external-contract-closeout-exports).
The existing exact management 0.11.0 feature now exposes lifecycle/settings,
ECDSA request/reply and complete origin/change-record identities. Existing ICRC
0.2.0 exports now include `TransferArg`, `Memo` and a narrow `icrc21` namespace.
Consent `Value` stays distinct from ledger-generic `Value`. All are registry Rust
identities; there is no wrapper, copied crypto, new service engine, dependency,
feature or changed default. Consumers retain effects, limits and consent policy.

Full `make clippy` passed before longer tests. Exact upstream identity tests pass.
Extended standalone `management` and `icrc` consumers pass native and Rust 1.91
execution and both Wasm compilation routes, without direct consumer Candid/DTO
dependencies. New tests cover sender-version/settings/install requests, history
origins/records, ECDSA DTO bytes, transfer/memo and consent request/display/error
families. They perform no signing or paid calls. Evidence:
`target/boundary-consumer.wbVf8F`, `target/boundary-consumer.c8bYEW`;
logs `/tmp/platform-backlog-20261027/{clippy,identity,consumer}.log`.
Actual default/executor/runtime and management/ICRC/combined native/Wasm graphs
pass (`target/backlog-20261027-graphs`). Snapshot, inventory, docs and formatting
pass; root dependency lock is unchanged. Native macOS, installed consumer/payment
acceptance and full release CI were not run for this pending export extension.

These extra exports are absent from immutable 0.6.20. A future supplying release
is needed before Canic can remove those remaining mirrors. Quote new operations
from the canonical request and retain original bytes for uncertain retries;
complete upstream settings/install types need not match old subset type tables.
No new cryptographic implementation or authority is provided.

## Published DTO supply (#61–64)

The five optional external DTO families are verified in non-yanked registry
Platform 0.6.20. Archive SHA-256:
`7345878bbf473daa1989d28b1f9828e22e63c2f21e776dd5a3e9eace8609016e`.
Archive VCS equals released main; selected DTO module source is byte-identical.
The [external contract guide](../external-contracts.md) now names the actual
supplier, features, source/license provenance and adoption mappings.

Fresh registry-only standalone consumers pass for portable CMC/ICP/Cycles/Registry
(4 tests), native assets (1) and pinned-DID schema equality (5), including Rust
1.91 execution and portable Wasm compilation. Evidence:
`target/boundary-consumer.62bKiL`, `.T81QEY`, `.1hZtHp`;
`/tmp/platform-backlog-20261027/published.log`. These use published 0.6.20,
not the pending management/ICRC exports. Earlier host-asset Wasm import refusal
and source-license/package checks remain under `/tmp/platform-61-64`.

Canic has selected 0.6.20 but has not enabled/migrated/qualified these external
DTO features. Its current handoff explicitly retains that work and its active
supplier/installed acceptance. #61–64 stay open for actual Canic adoption, not
version selection alone. Original retry bytes, target/subnet/controller binding,
fees/refunds, receipt authentication and frontend bounds remain consumer-owned.

## Published browser supply (#59)

The four upstream signing-request exports are in source and passed the prior
packed-consumer, exact-identity and Chromium page/module-worker proofs. Evidence:
`/tmp/platform-0620-{clippy,browser,chromium}.log`,
`target/browser-consumer.891Oy8`, `target/browser-consumer.JWXVZk`.
The user authorized publication of compatible `@tokostudios/ic-platform` 0.1.1.
Public npm accepted it at 2026-10-27T09:06:09.829Z with latest=0.1.1; SDK Core is
still exactly 6.1.1 and no external lock selection changed. Initial saved login
was expired; browser login and separate publication verification completed. The
first noninteractive attempt was refused with EOTP before upload; interactive
web verification completed the same qualified version. No password/code was handled.

Full Clippy passed before release checks. Dry-run and actual prepublish hooks
pass all 16 adapter/graph tests and strict packed/provider consumers. Prepared
Chromium page/module-worker qualification also passes. The public archive's
integrity equals the qualified tarball; SHA-1 is
`33bf72eaf3039111c462072d5d2bc0aa6dd80f45`. Fresh registry-only install,
strict TypeScript/runtime signing-request smoke and singleton SDK graph pass
without a direct consumer SDK declaration. Evidence: `/tmp/platform-npm-011`,
`target/browser-consumer.jzEnrN`, `.0yfvt0`, `.pwu6B8`, and
`target/npm-registry-consumer.1ClKKR`. Published package metadata is retained there;
its Git-head field is baseline 401a832, not a new version/README commit.
#59 is closed as supplied by npm 0.1.1. Auth's application policy/adoption and installed
acceptance remain consumer-owned; no new native macOS/provider qualification is
claimed. Local browser README and generated Agent files match the public archive.

## Closed advisory preparation (#56); crypto blocker (#50)

#56 is closed after verifying the released fix and live 0.6.20 advisory workflow
[40472663771](https://github.com/dragginzgame/ic-platform/actions/runs/40472663771).
Both retained index statuses are complete. Workspace audit still fails for
crypto-bigint/#50; runtime audit passes. No advisory is suppressed. Full workflow
log is `/tmp/platform-backlog-20261027/advisory-ci-full.log`; earlier real
prepared/yanked/missing-index regressions remain under `target/issues-38-50-56`.

Fresh registry/advisory records on 2026-10-27 still show crypto-bigint 0.5.5 as
the newest non-yanked 0.5 release, with patched >=0.7.6. Agent 0.49.3 and exact
ic-secp256k1 0.3.0 still require K256/P256/elliptic-curve 0.13 and affected ^0.5
arithmetic. Newer incompatible crypto versions do not resolve the coupled paths.
No compatible fix or justified exemption was found. Resident private-key signing
precludes a blanket verification-only claim. No crypto/Agent fork, override,
exemption or in-house algorithm change is supplied. #50 stays open; raw records
are retained under `/tmp/platform-backlog-20261027`. This is not a new local audit
scan or exploitability proof; the live workflow independently confirms the match.

## Compiler retirement (#38)

The old live-compiler-call blocker is stale: Canic default `aed32be2` frontend ops
now consumes retained Wire schemas and compiles Wire bindings. The inspected
local Canic/Blob/Host/Tooling/Query/Auth/Backup/IcyDB crate sources have no references
to Platform's `compile_javascript`/`compile_typescript`. Default source is retained
at `/tmp/platform-backlog-20261027/canic-default-frontend.rs`.
This is source adoption, not full #613 closeout or installed fleet qualification.
Retirement removes public Platform exports and still requires the agreed explicit
breaking boundary; the maintainer's compatible 0.6 restriction remains in force.
Required external parsing/service behavior stays supported. No compiler hard cut,
parser fork or dependency/size reduction is claimed. #38 stays open with corrected
evidence rather than retaining the obsolete caller assertion.
