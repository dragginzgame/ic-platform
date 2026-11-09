# Current handoff — 2026-11-09

## Toko 0.37.4 and owned bigint 0.5.1 — current

Adopted registry Toko Tooling 0.37.4 through its authenticated installer, retaining
recorded entrypoints and local policy choices. The owned codec/ledger now selects
num-bigint 0.5.1 independently of optional suppliers. Agent is a native-only
optional normal dependency, not a dev-dependency; canister-signature suppliers
also retain Candid's num-bigint 0.4.8. Codec-only Wasm selects only 0.5.1.
Public Nat.0/Int.0 bigint identities change with the pending hard cut. Test-only
Candid oracles bridge via decimal values rather than sharing Rust bigint types;
wide integer, Candid, JSON, CBOR and ledger encoding comparisons still pass.
All-feature facade Clippy, tests and doctests pass. Runtime qualification lock
was refreshed while preserving unrelated package selections. No publication.

Crypto-common audit: direct SHA-2/ECDSA/Ed25519 suppliers already resolve 0.2.2.
Older BLS/certification/signature/native suppliers retain 0.1.6 through digest
0.10.7. Isolated 0.1.7 resolution forces generic-array 0.14.9 → 0.14.7; its
changelog lists only that exact pin. No crypto lock change was adopted.
Evidence: target/crypto-common-audit.IpBwGt. The 0.37.4 baseline now explicitly
prefers current direct dependencies independently of older DFINITY transitives.

## Qualification identity admission (#152) — implemented locally

Runtime, Agent, raw-rand and benchmark fixtures now retain separate selected-owner
and Testkit-infrastructure identity reports. Source admission checks exact owner
paths; published admission checks the selected Runtime/macros/guard cohort.
Agent additionally admits the real Testkit CanisterId/Agent Principal identity
crossing. Older infrastructure facades remain permitted. BLS's unused native
facade dependency and ECDSA driver's unused principal feature are removed.
See [the coverage table](../platform-release-validation.md#qualification-identities-152).

Focused Linux evidence: all four standalone Runtime profiles installed and passed
at `target/cdk-runtime.1XCP3r`; Agent installed checks and Clippy passed at
`target/agent-installed.pPW0jC`. Singleton coexistence/refusals passed at
`target/runtime-singleton.znGiyn`; raw-rand graph admission passed at
`target/evidence/raw-rand.3oAlCO`. Both benchmark drivers passed Clippy/build in
`target/qualification-152-bls` and `target/qualification-152-ecdsa`; the BLS Wasm
probe retained per-profile identities and measured artifacts at
`target/evidence/platform-wasm-size.QKJnQS`. Synthetic identity tests cover wrong
source, missing owners, duplicate selected types, Principal mismatch and wrong
published cohort members. No macOS acceptance, commit or publication is claimed.

## Pending hard-cut release

Pending cohort: Types 0.19.0, Runtime/macros/guard 0.21.0, Platform 0.28.0.
Manifest versions remain 0.18.3/0.20.3/0.27.1 until authorized delivery. Work is
uncommitted/unpublished. Browser and siblings are excluded. The final committed
batch still needs the complete frozen release gate; focused evidence is not a
release receipt. No new macOS, worker or native Profiler acceptance is claimed.

The migration contract, inventory and pending changelogs own the detailed scope:
- Owned typed binary codec replaces the legacy Candid/parser/printer surfaces;
  explicit budgets and 16-level admission remain.
- Principal/Account use canonical Types. External Principal aliases/conversions
  and Types' production ic_principal are retired; its dev oracle remains.
- Management has owned DTOs on both targets (58 native/44 Wasm exports).
  Registry management 0.11.0 is now an oracle or supplier transitive dependency.
- Ledger hashing shares SHA-2 0.11.1 with Principal self-authentication, preserving
  imported hash bodies, domains, fixed vectors and licenses.
- Cryptography remains upstream. Native/signature capabilities still select
  upstream Candid; do not claim a universally Candid-free package graph.

## Qualification and downstream ownership

Platform no longer selects Timers in its catalog, fixtures, CI or release roster.
Scheduling #91 owns downstream Timers composition; Tooling owns its adoption.
Neither is a supplier-release gate. Standalone raw/prepared × Wire/external
profiles retain trap, cancellation, upgrade, cycles/stable and certificate tests.
The local release roster has 19 stages and one Runtime fixture lock.

Latest installed Linux proof after bigint 0.5.1: target/cdk-runtime.Mc2iLW,
all four profiles passed, including driver Clippy/tests and external endpoint,
auth-before-decode, management await and nesting refusal. Facade all-feature
Clippy/tests/doctest pass; two explicit timing experiments remain ignored.
Prior complete release-command checks passed with real group preparation and
recovery: target/platform-group-preparation.4aUAPR. This is not complete release
qualification of today's final batch.

Earlier management/Principal graph and consumer evidence remains at
 target/feature-graphs/attempt.hgzmPr, target/feature-graphs/attempt.JdFvoI and
 target/boundary-consumer.FFXkA0. Canonical contracts are in docs/migration.md.
Measured pre-bigint codec savings and reproduction are in
scripts/benchmarks/candid-codec.md: 501,988 optimized bytes saved in the management
probe against published 0.27.1 (target/evidence/candid-removal.fREwuJ). Ledger SHA
consolidation saved 6,784 optimized bytes in the combined hashing probe, comparing
platform-wasm-size.olhycp and platform-wasm-size.aLmoWE under target/evidence.
Neither result measures every canister or the new bigint version's size.

## Last delivery and recovery custody

Last recorded published versions and group tags: Types 0.18.3 (17d5906),
Runtime/macros/guard 0.20.3 (38d6e48), Platform 0.27.1 (97fe286).
Batch batch-mG_yG4uo delivered all five packages; archive/registry/tag proofs are
under target/deploy-toko036-20261108. Browser remains 0.2.0.
Preserve failed/completed batches and their original recovery code, including
batch-_leIreCI, batch-C5BoFKA5, batch-AaHi2pgO, batch-O3V4GL4T,
batch-PIIqwepD and batch-mG_yG4uo. Do not resume older failed batches over
already-published versions.

The pre-existing supplier lock draft was excluded from that deployment, then
restored locally: arc-swap 1.9.4, Candid/derive 0.10.40, cc 1.11.1 and smallvec
1.16.5. Preserve those selections; never replace today's lock with the old full
file. Original durable custody is refs/retained/platform036-lock-draft-20261108,
commit 0263ab6e679ae3b8e9f89bddd796ca951b3f1df4. Earlier custody is
refs/retained/platform0270-lock-draft-20261108 at
588b11a38c60b15ab5af19f30aef1c310a8b0975. #50 remains deliberately deferred.

Historical bundles/receipts remain in
recovery-evidence/retired-checkouts/20261104-issue139/; maintainer review is due
2026-12-04. No cleanup, retirement, mtime renewal or custody-limit change is
approved. Previously removed target evidence and Runtime 0.18.1 binaries must
not be claimed present. Canic #749/#750 own adoption; Platform #151 is closed.
