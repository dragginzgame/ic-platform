# Current handoff — 2026-11-09

## Native EffectiveId correction — Platform 0.29.2 published

Another session reported that published 0.29.1 omitted
`ic_platform::agent::EffectiveId`. The selected upstream Agent already supplies
the enum (including the admitted 0.49.2 lower bound). Platform now explicitly
re-exports it, preserving upstream identity and canister/subnet routing. No
dependency, cryptographic, transport or Wasm implementation changes.

Native Agent consumer Clippy and all six tests pass, including both enum variants
passed directly to `Agent::read_state_raw` without polling/network effects.
Inventory, docs, boundaries and formatting pass. Evidence:
/tmp/platform-effective-id-{clippy,tests,checks}.log. A compatible Platform-only
0.29.2 patch completes this missing deployment surface. Sibling checkouts remain untouched.

Frozen batch `batch-pPdjnmcJ` published only Platform to crates.io and GitHub:
commit `f687d96c1d423e87e3083c840aa6d6472f41c165`, tag `platform-v0.29.2`.
The normal checkout is reconciled. All required local release stages passed,
including installed Runtime and raw_rand. Receipt:
target/release-frozen/batch-pPdjnmcJ/target/release-validation/qualification.GyCi52/receipt.json.
The standalone registry consumer at target/platform-0292-registry selects exact
0.29.2 with defaults disabled and only `agent`; Clippy passes both routing
variants and their direct admission to `Agent::read_state_raw`.
An additional executable run failed at linking (bus error) with the filesystem
at 100% capacity; it is not claimed as passing. Log: /tmp/platform-0292-registry.log.
The downloaded registry archive matches the publication archive, SHA-256
`a391440c441bebbabcce38251ff61473de37cdca09050fe6c3f71cd10ca7be42`,
and records the release commit. Types, Runtime and browser were not released.
No new macOS evidence or sibling CLI acceptance is claimed; hosted CI remains
asynchronous. Callers can now adopt 0.29.2 for the missing export.

## Native service export — Platform 0.29.1 published

Frozen batch `batch-v1S7Tarz` delivered only Platform to GitHub and crates.io.
Release commit `38c813012cb1ee0931869bfd16968d492cb21511` and annotated tag
`platform-v0.29.1` are verified on origin; the normal checkout was reconciled.
The draft 0.30.0 candidate was corrected to 0.29.1 before release because this
is a compatible addition under the adopted pre-1.0 version policy.

All required local release stages passed, including installed Runtime and
raw_rand. Receipt: target/release-frozen/batch-v1S7Tarz/target/release-validation/qualification.ZbLcVw/receipt.json.
The registry-only consumer at target/platform-0291-registry passes both public
service tests and pinned didc strict Caffeine equality. It selects exact
`ic-platform =0.29.1`, defaults disabled, feature `candid-codec`; candid_parser
is solely its development oracle. Log: /tmp/platform-0291-registry.log.
Downloaded archive SHA-256
`161d78dfbe8ae905828da482c5045e25c31bdb1a4b6575be4573ef8ad6e12e6a`
matches the retained publication archive and records the release commit.
Types 0.19.0, Runtime 0.21.0 and browser 0.2.0 were not released. No new macOS
or native Profiler acceptance is claimed; hosted CI remains asynchronous.

[#153 — Document the native external-service schema/export owner](https://github.com/dragginzgame/ic-platform/issues/153)
is implemented and published in 0.29.1. Native
`candid_codec::service::{Method, Mode, export}` generates bounded DID text from
owned mappings. Pinned didc owns trusted fixture parsing and strict structural
equality; no parser/Runtime production edge or Wasm export is added. The
[consumer migration](../native-candid-service.md) identifies the retained Blob
export API, retired reflection calls and independent textual-input boundary.
No sibling edits or releases were performed. The Platform 0.29.1 ledger links
the issue. The issue was closed after focused implementation checks and now has
the supplying publication evidence.

Focused Clippy, three schema unit tests, two public consumer tests, pinned didc
positive/negative conformance, decoder regressions, inventory, documentation,
boundary and formatting checks pass. Evidence: /tmp/platform-153-clippy.log,
/tmp/platform-153-tests.log, /tmp/platform-153-didc.log,
/tmp/platform-153-decoder-regression.log and /tmp/platform-153-checks.log.
The didc artifacts are at target/candid-service.m8PVfg. A standalone native
consumer builds without Candid/parser/Runtime normal dependencies; the Wasm codec
build passes and a Wasm service-export import correctly refuses. Evidence:
target/platform-153-boundary and /tmp/platform-153-{native-boundary,native-graph,wasm,wasm-refusal}.log.
This is native schema/CLI evidence, not installed callback or Blob adoption
acceptance. Full release/native-host gates remain release-owned.

## Platform 0.29.0 decoder admission — published

Frozen batch `batch-rDRVOX7Y` completed GitHub and crates.io delivery. Release
commit `5f4b66f4a5fd97c6e66722aca254f2d09fabe05e` and annotated tag
`platform-v0.29.0` are verified on origin. The normal checkout was reconciled
through the shared workflow. Registry archive SHA-256
`96a33522b86e1c775fbe3ce124657a4ff8a63c9a4280f3bb851e49e1b4fc210b`
matches the retained publication archive and records that release commit.

All required local release stages passed, including installed Runtime profiles
and raw_rand. Receipt: target/release-frozen/batch-rDRVOX7Y/target/release-validation/qualification.MjrLUJ/receipt.json.
The registry-only consumer in target/platform-029-registry passes both public
codec tests against exact `ic-platform =0.29.0`, with defaults disabled and
`candid-codec` selected (/tmp/platform-029-registry.log). Select
`management-canister-types` additionally when its DTOs are needed. No new macOS
or native Profiler acceptance is claimed; hosted CI remains asynchronous.

The maintainer authorized implementing and publishing the missing Backup codec
controls in Platform only. `Reader::bounded_vec` checks a field's count before
storage reservation; `strict_option` rejects promotion/fallback. Explicit
`decode_one_with_skip_limit` / `decode_args_with_skip_limit` entrypoints provide
independent cumulative skipped-work admission, including zero-skip refusal.
Existing entrypoints retain their behavior. Migration details and consumer
responsibilities are in `docs/migration.md`.

Focused Clippy and 57 library tests pass with management types selected; the
public consumer proves nested 1024/256 bounds and identical bytes. Management
consumer tests and final 32/256/1024/4096 cap/skip regressions pass. Logs:
/tmp/platform-decoder-clippy.log, /tmp/platform-decoder-tests.log,
/tmp/platform-decoder-consumer.log, /tmp/platform-decoder-limits-final.log.
Snapshot/inventory/docs/boundary/formatting and feature-closure checks pass
(/tmp/platform-decoder-checks.log, /tmp/platform-decoder-features.log).
The minimal codec-only unit-test Clippy selection exposes pre-existing unused
shared oracle helpers; the relevant management profile uses those helpers and
passes without suppression. Full release Clippy also passed.

Only Platform was published. Types 0.19.0, Runtime 0.21.0 and browser 0.2.0
remain unchanged; the accepted Toko adoption is included as source metadata.
Backup source, product recovery and deployment remain its owner's work. No
Backup acceptance is claimed from Platform's codec tests.

## Adopted Toko Tooling 0.38.2

Updated the exact workspace dev pin, Cargo.lock and authenticated snapshot to
0.38.2, incorporating the pending 0.37.5 and 0.38.0 adoptions. Only Toko changed in the lock.
All recorded entrypoints and local policy remain selected. Committed with the
decoder work and delivered in Platform 0.29.0. Types and Runtime tooling notes
remain pending for their next releases; no extra group release was performed.

The [placement audit](../development-and-qualification.md#package-placement)
confirms all five maintained packages already belong under `crates/` in the
product workspace. No package paths changed. Disposable fixture manifests and
vendored IC source are explicit policy exclusions, not separate maintained
qualification packages. No empty qualification workspace or extra lock was added.
Existing release qualification and recovery custody remain required.

0.38.2 brings optional qualification-workspace release preparation and scoped
intent-to-add authority for validation. Platform has no qualification workspace
to select; its generated Runtime fixture lock keeps its existing custody.
No intent-to-add entries were needed. The installer added two authenticated
qualification-release helpers; retain both with the snapshot.

Snapshot, pin, inventory, documentation and Candid/CBOR/Cargo boundary checks
pass (/tmp/platform-toko-0382-checks.log), as do group projection tests
(/tmp/platform-toko-0382-projection.log). Locked offline Cargo metadata still
confirms exactly five product members under `crates/`. Prior 0.38.0 feature
isolation and release-routing evidence remains in /tmp/platform-toko-038-closures.log
and /tmp/platform-toko-038-groups.log. No Rust implementation or package membership
changed; full release and native-host gates were not rerun for this tooling-only
adoption.

All three groups pass preparation, rollback and custody checks with the new
release helper; the disposable Runtime fixture also passes delivery/resume.
Evidence: /tmp/platform-toko-0382-preparation.log and
target/platform-group-preparation.E3cOnN. These fixtures do not publish packages
or qualify an actual release candidate.

## Completed hard-cut deployment

Batch `batch-bHpKtblk` captured caa2761ca6da9865554d75cd81138fefe3dc093c and
completed GitHub and crates.io delivery for all five packages:

| Group | Version | Tag | Release commit |
| --- | --- | --- | --- |
| Types | 0.19.0 | types-v0.19.0 | 49ad1302847eda8745eb18072bee71edf415df12 |
| Runtime/macros/guard | 0.21.0 | runtime-v0.21.0 | 6c3a6e33bff66938e011cc1181748a063e3d4125 |
| Platform | 0.28.0 | platform-v0.28.0 | 3362ab53a4b8cb85a2812eafe4bdbe2a70ef4de3 |

The normal checkout was fast-forwarded through release-batch-reconcile. Registry
archives match their index checksums and retained publication archives; archive
VCS identities match the release commits. Remote main and annotated tags were
verified. Evidence: target/deploy-hard-cut-20261109/registry-verification.json.
Browser remains 0.2.0; no browser/npm or sibling release was performed.

All 19 required local Linux stages passed separately for every group. Under
`target/release-frozen/batch-bHpKtblk/target/release-validation`, receipts are
qualification.n5BAgX (Types), qualification.apQsVh (Runtime), and
qualification.0ZXt5w (Platform). All four installed Runtime profiles and raw_rand
checks passed. Only the admitted identical-input policy-fixture leaf reused
results; real package, consumer and installed checks ran for each group.
Hosted native/integration CI remains asynchronous; no new macOS, worker or
native Profiler acceptance is claimed.

## Delivered contracts and adoption

The migration contract, inventory and group changelogs own the detailed API cuts:
- Owned typed binary Candid codec replaces legacy parser/printer/decoder surfaces.
  Explicit budgets, numeric ranges, bytes and 16-level admission remain.
- Principal/Account use canonical Types; external Principal aliases/conversions
  and Types' production ic_principal are retired. Its development oracle remains.
- Owned management DTOs cover both targets: 62 native and 44 Wasm exports. The
  four native install/create/log DTOs retain independent source guards and
  upstream schema, byte and Serde comparisons. Registry management is dev-only.
- Nat.0/Int.0 now use num-bigint 0.5.1. Candid, JSON, CBOR and ledger byte
  comparisons pass; downstream direct bigint users must select matching types.
- Ledger hashing and Principal self-authentication share SHA-2 0.11.1, preserving
  imported hash bodies, domains, vectors and licenses. Cryptography stays upstream.
- Registry Toko Tooling 0.37.4 is installed, with explicit offline governance
  entrypoints. Its current-direct-dependency policy is adopted independently of
  older DFINITY transitives. Every member inherits the exact source-bundle pin.

Crypto-common is transitive: current direct SHA-2/ECDSA/Ed25519 suppliers use
0.2.2; older BLS/certification/signature/native suppliers retain 0.1.6. Isolated
0.1.7 resolution forced generic-array 0.14.9 → 0.14.7 and offered only that pin
change, so it was not adopted. Evidence: target/crypto-common-audit.IpBwGt.

Wire/Tooling adoption is NOT registry-qualified by this release. A registry-only
proof with published Wire 0.18.0 passes its Wire trait for Types 0.18.3 Principal
but rejects Types 0.19.0 Principal. Wire's ^0.18.2 selection cannot unify with
0.19.0; its owner must publish a compatible adoption before downstream users
select that schema integration. Evidence: target/wire-types-registry.NwkBsL,
including metadata and positive/negative compile logs. No sibling was edited.

## Qualification ownership

[#152 — Separate supplier qualification from downstream adoption](https://github.com/dragginzgame/ic-platform/issues/152)
was reconciled against delivered commit `1b8304b` (ancestor of both Platform
0.28.0 and 0.29.0). No implementation change was required. Fresh identity,
selection and graph refusal tests pass: /tmp/platform-152-identities.log,
/tmp/platform-152-selection.log and /tmp/platform-152-graphs.log.
The 0.29 release's `cdk-runtime.T3feSH/driver-identities.json` under
target/release-frozen/batch-rDRVOX7Y/target reports its tested local facade/Types
separately from Testkit 0.92's registry Platform 0.24 / Types 0.18.2. The admission
scripts are unchanged by the pending native service-export work. Historical
Agent evidence below remains explicitly for its recorded source, not a fresh
0.29 Agent acceptance run. The issue can close on its delivered producer fix;
no downstream adoption or new fleet gate is required.

Runtime, Agent, raw-rand and benchmark evidence separately identifies selected
owners and Testkit infrastructure. Exact source paths/published Runtime cohorts
and Agent/Testkit Principal crossings are admitted; unrelated older infrastructure
facades remain allowed. BLS's unused native facade dependency was removed.
See the coverage table in docs/platform-release-validation.md.

Platform no longer selects Timers in qualification or release metadata.
Scheduling #91 owns downstream Timers composition; Tooling owns its adoption.
Neither is a supplier-release gate. Standalone trap/cancellation/upgrade checks
and executor singleton refusals remain. Agent installed evidence is retained at
target/agent-installed.pPW0jC; no local check establishes all sibling adoption.

## Retained failure and recovery custody

Preserve failed batches batch-G3rEbkX7 and batch-QJcFkCdU. Neither published:
the first exposed a stale upstream-Candid admission fixture (fixed in 337b4b6),
the second exposed an undersized general-decoder raw_rand comparison budget
(fixed in caa2761). The fixture-only correction passed installed comparison at
target/evidence/raw-rand.Qetjbz, then every successful release gate.

Preserve all earlier failed/completed batches and their original recovery code,
including batch-_leIreCI, batch-C5BoFKA5, batch-AaHi2pgO, batch-O3V4GL4T,
batch-PIIqwepD and batch-mG_yG4uo. Do not resume old batches over published versions.
The formerly deferred supplier lock selections are included in this qualified
batch; original custody remains refs/retained/platform036-lock-draft-20261108
at 0263ab6e679ae3b8e9f89bddd796ca951b3f1df4 and
refs/retained/platform0270-lock-draft-20261108 at
588b11a38c60b15ab5af19f30aef1c310a8b0975. Never restore an old complete lockfile.

Historical bundles/receipts remain in
recovery-evidence/retired-checkouts/20261104-issue139/; maintainer review is due
2026-12-04. No cleanup, retirement, mtime renewal or custody-limit change is
approved. Previously removed target evidence and Runtime 0.18.1 binaries must
not be claimed present. #50 remains deliberately deferred; Canic #749/#750 own
separate adoption, and Platform #151 is closed.
