# Signed transport supplier publication — 16 November 2026

The five changed crates are published from clean source
`a813d4ea5378515984a547a9f4e9d37f2124cf14`. Main and annotated tags
`runtime-v0.23.14`, `protocol-v0.2.6`, and `platform-v0.34.7` were
pushed atomically. Types 0.21.2 remains unchanged and was not republished.

| Published package | Independently downloaded archive SHA-256 |
| --- | --- |
| [ic-platform-runtime-guard 0.23.14](https://crates.io/crates/ic-platform-runtime-guard/0.23.14) | `239aeacc00fa6015ea4656defdfe643cdd6116f180887ea27b8164105c58a2ce` |
| [ic-platform-macros 0.23.14](https://crates.io/crates/ic-platform-macros/0.23.14) | `0376721a68d30392d7005bfaeca422bb0674cfa3c1778ca69b6661183327792c` |
| [ic-platform-runtime 0.23.14](https://crates.io/crates/ic-platform-runtime/0.23.14) | `6f91abeacf7b1602e67b49391ea1e66d1c8f7e45779de79a5835b9c8dac3afb8` |
| [ic-platform-protocol 0.2.6](https://crates.io/crates/ic-platform-protocol/0.2.6) | `758e0b8455b2ab340a4503f211357b7725f52a55a2c8c209ee2f01b122763b7e` |
| [ic-platform 0.34.7](https://crates.io/crates/ic-platform/0.34.7) | `479e4899fe08d50c582e6b5b57668ed5cdcc3e0f75b81fde40ffb3ed2b14c093` |

Every package passed the supported publication dry run and extracted-archive
all-feature native/Wasm Clippy and native tests. Source provenance independently
fetched the exact public GitHub source. Each upload executed once, in dependency
order: guard, macros, Runtime, Protocol, Platform. Independent readback matched
the non-yanked sparse-index checksum, locally admitted archive, clean tagged VCS
commit and package path before the next package was published.

## Tooling and qualification

Toko Tooling 0.44.11 is installed with checksum
`7d16b815ae322ae3f88580ee6af1611bdafde08b77592cd886bfa0eece0f1be8`.
Both workspaces and materialized candidate consumers select it; original archived
source and frozen comparison cohorts remain unchanged. The supported installer
verified all 85 managed files. CI setup steps now retain bounded, sanitized
failure diagnostics with seven-day artifact retention. Local failure-path tests
cover original exit status, hidden attempt output, redaction, bounds, missing
diagnostics and collector refusal. Hosted artifact upload/retrieval has not been
claimed from these local tests.

The complete Types, repository, Protocol, Runtime and Platform release catalogs
passed at the tagged source. Receipt:
`target/release-qualification/1794861081344-569509/receipt.json`.
It binds compiler, both catalogs and every complete profile log. The retained
release lock audit found no known vulnerabilities; existing unmaintained supplier
notices remain. Native execution evidence is Linux, not macOS qualification.

The earlier Toko Tooling 0.44.10 attempt stopped in the offline Auth fixture because its
Cargo cache lacked the selected tooling archive. Toko Tooling 0.44.11 became available
during that attempt; both workspaces adopted it, dependencies were fetched, and
the entire catalog was rerun successfully at the final clean source.

## Scope and remaining consumer gates

Platform adds explicit Ed25519 signing, exact-envelope dispatch, signed queries,
authenticated original-request observations and certified module hashes. Protocol
adds selected management snapshot formats and bounded Caffeine codecs. Runtime
adds synchronous replicated-execution observation. Query qualification now uses
its portable schema corpus. Actual Blob supplier journeys, Query, installed Auth
and Storage fixtures passed within the retained local catalog.

Supplier publication does not establish complete production migration. Full Blob
CLI custody/management journeys, broader Backup acceptance and the separate
legacy Agent crypto obligation remain open. See [Blob supplier limits](blob-suppliers.md),
[management snapshots](management-snapshots.md), and [Runtime boundary](runtime-memory-cycles.md).
No sibling checkout was modified or delivered.

## Retained publication evidence

- ic-platform-runtime-guard: dry run `target/publication/check.selection.LMV1Gr`; upload `target/publication/publish.selection.TqT7Hg`; readback `target/registry-readback/ic-platform-runtime-guard-0.23.14/verified.json`.
- ic-platform-macros: dry run `target/publication/check.selection.bfqSBL`; upload `target/publication/publish.selection.BSi6W2`; readback `target/registry-readback/ic-platform-macros-0.23.14/verified.json`.
- ic-platform-runtime: dry run `target/publication/check.selection.pKikHR`; upload `target/publication/publish.selection.QViHe0`; readback `target/registry-readback/ic-platform-runtime-0.23.14/verified.json`.
- ic-platform-protocol: dry run `target/publication/check.selection.fvIYpc`; upload `target/publication/publish.selection.A7lAGD`; readback `target/registry-readback/ic-platform-protocol-0.2.6/verified.json`.
- ic-platform: dry run `target/publication/check.selection.RPVuGQ`; upload `target/publication/publish.selection.27MgmD`; readback `target/registry-readback/ic-platform-0.34.7/verified.json`.

The reproducible dispatch script is `target/release-selection/publish-cohort.mjs`;
combined readbacks are `target/release-selection/published.json`.
Hosted CI was observed once after publication and was not awaited:
[CI](https://github.com/dragginzgame/ic-platform/actions/runs/43951423559) and
[Types qualification](https://github.com/dragginzgame/ic-platform/actions/runs/43951423447)
were both in progress. This is not a hosted-CI success claim.
