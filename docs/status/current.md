# Current handoff

## Source and released boundaries

Baseline is released Platform 0.6.18 (`98a2a8a`). Compatible working-tree notes
use 0.6.19; own package versions and Git refs are unchanged. The incoming staged
advisory repair and root Wire 0.7.16 lock update are preserved without staging edits.
No commit, push, release or publication was performed in this batch. Sibling
checkouts remain read-only. Shared Tooling is now 0.26.20 at
`a7f85d45d4212d4c39225f58597580560a333550`, exported canonically with 106 payloads.

## Current adoption

Both workspace Wire requirements now select at least published 0.7.16. The
incoming root lock was already at that version and is byte-identical to the
starting input; the separate runtime lock moves only Wire/derive from 0.7.2 to
0.7.16. No sibling checkout or cryptographic dependency selection was changed.

Shared's new SDK checker is used on the actual packed consumer manifests and
TS proof sources in both standalone and provider phases. It is not applied to
Platform's browser producer, which deliberately owns the direct SDK dependency.
Provider overrides/transitives retain their separate identity qualification.
The compact baseline makes detailed accounting optional; local full-Clippy-first
ordering remains the maintainer's explicit requirement. Existing full CI/release
gates remain intact; no docs-only hosted routing or sibling SDK enforcement is
claimed by this snapshot adoption. Prior #56 evidence below predates Wire adoption.

Qualification passed: full `make clippy` before longer tests; snapshot integrity,
inventory and formatting; actual grouped release metadata and pending changelog
admission; packed browser consumers and a direct-SDK-import rejection probe;
Rust 1.91 Principal/Account Rust–TypeScript frames and malformed-input refusals;
native/Wasm runtime dependency graphs and runtime canister compilation. The
`wire-schema` Wasm check passes with host metadata excluded; default, executor
and runtime graphs remain free of Wire metadata. Evidence is retained under
`target/principal-wire-schema.65A45J`, `target/browser-consumer.AcM6I9`,
`target/adoption-sdk-refusal.3Uq71f`, `target/adoption-graphs` and
`target/cdk-runtime.q2i5k1`; logs are `/tmp/platform-adoption-*.log`.
The packed browser archive still matches published npm 0.1.0. Native macOS,
fresh Chromium, installed-canister acceptance and full release CI were not run
for this adoption. No performance or Wasm-size improvement is claimed.

Published Rust 0.6.18 supplies the optional upstream BLAKE3 boundary. Existing
hash formats, domain policies, cryptographic bodies and executor/IC0 identities
are unchanged. [IcyDB #467](https://github.com/dragginzgame/icydb/issues/467) owns
routing its existing uses; [Tooling #252](https://github.com/dragginzgame/ic-tooling/issues/252)
owns the bounded private-cache evaluation. Previous measurements and constraints
remain in the [migration contract](../migration.md#optional-blake3-maintenance-boundary).

The browser supplier is published `@tokostudios/ic-platform` 0.1.0 / Core 6.1.1;
[Platform #57](https://github.com/dragginzgame/ic-platform/issues/57) is closed.
Public archive integrity and a fresh registry-only consumer were verified.
[Blob #103](https://github.com/dragginzgame/ic-blob-storage/issues/103) and
[Canic #637](https://github.com/dragginzgame/canic/issues/637) retain actual adoption.
Do not repeat npm publication for this Rust patch. #58's acknowledgement/routing
queue was withdrawn; compatibility decisions stay with existing owners/docs.

## Advisory index repair (#56)

The local fix prepares both captured maintained locks before auditing. Online
mode creates an isolated Cargo home, fetches the locked workspace and reuses the
existing runtime `--prepare` operation for its template-generated workspace.
It compiles nothing and avoids the user's shared package-cache lock. This is
Cargo's supported dependency fetch, including crate sources, not an index-only
fetch or a new registry implementation. CI explicitly prepares common host tools
needed by the existing runtime renderer.

Audits retain `--no-fetch` against the isolated committed RustSec database.
Raw audit exit codes remain in `*.status`; independent `*.index-status` records
complete/incomplete yanked observations. Missing records/index failures reject
coverage even if cargo-audit exits zero. Existing vulnerability/warning policy
is unchanged; no finding is ignored. Preparation errors and concurrent maintained
lock changes stop before audit. Offline mode fetches nothing and claims no freshness.

Full all-target/all-feature Clippy passed before longer tests. Adapter fixtures,
Bash syntax, ShellCheck and workflow lint pass. Real cold-cache preparation/audit
uses RustSec `21d909e4c8bd4e14c7aea58bf980c881a0053ab1`: both index observations
are complete with no lookup errors; workspace audit exits 1 for #50, runtime 0.
A real yanked crypto-bigint 0.7.8 fixture reports the expected warning. A real
empty-cache probe gives raw audit status 0 for both locks but adapter status 1
with incomplete coverage. Locks were unchanged during that qualification; the
subsequent Wire-only updates are described above.
The integrated `make test` target, including adapter regressions, Rust unit tests
and doctests, passes; log `/tmp/platform-issues-tests.log`.

Evidence: `target/issues-38-50-56/online-audit-final`, `yanked.log` and
`missing-index-probe`; `/tmp/platform-issue56-online-final.log` and
`/tmp/platform-issues-clippy.log`. The first preparation attempt exposed the
runtime's template-only layout; it is retained separately as `online-audit`.
The corrected path reuses the existing preparer without duplicating its renderer.
Native all-target/all-feature and Wasm all-feature checks pass at Rust 1.91;
logs are `/tmp/platform-issues-{msrv,wasm}.log`. Native macOS, hosted execution of
the changed advisory workflow and full release/installed CI are not claimed.
#56 stays open until the fix lands on the default branch.

## Crypto advisory (#50)

Fresh registry records still show crypto-bigint 0.5.5 as the latest non-yanked
0.5 release. Agent 0.49.3 and exact ic-secp256k1 0.3.0 retain K256/P256 0.13;
elliptic-curve 0.13.8 requires crypto-bigint ^0.5. The advisory still lists
>=0.7.6 as patched. Newer K256/P256/elliptic-curve families have different exposed
identities and do not provide a compatible fix to all three paths.

Locked native normal/build graphs select affected arithmetic under `ecdsa`,
`agent` and `secp256k1`, excluding it from default/executor/runtime profiles.
Agent's K256/P256 identities sign with resident private keys; there is no basis
for a blanket verification-only exemption. The completed audit keeps the real
failure visible. No cryptographic body, cryptographic dependency constraint,
type identity or exemption changed. Registry records, graphs and lock hashes
are retained under `target/issues-38-50-56`. #50 remains unresolved under the
existing compatibility, no-Agent-fork and no-in-house-cryptography constraints.

## Compiler retirement (#38)

Canic's default-branch frontend still imports/calls Platform's JS/TS compiler
exports. Published Canic Host 0.111.0 still generates Candid bindings through its
older direct parser dependency. The dirty working tree at `cf2d3d1fb` has removed
those calls for Wire, but its handoff says the batch is not push-ready and defers
builds/tests; [Canic #613](https://github.com/dragginzgame/canic/issues/613) is open.
That source preparation is not live consumer adoption or installed acceptance.

The compiler exports remain intact. Retirement requires the agreed consumer
adoption and an explicit breaking release, rather than this compatible patch.
Host's bounded external service inspection and Canic's textual external/init
arguments still need parsing/typechecking. No parser fork, dependency-removal or
size improvement is claimed. No sibling source or build was changed.
