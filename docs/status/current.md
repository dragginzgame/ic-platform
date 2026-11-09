# Current handoff — 2026-11-09

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

## Qualification ownership (#152)

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
