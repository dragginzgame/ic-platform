# Supplier cohort publication — 16 November 2026

All six crates are published from clean source
`9dfdc0083fa5d6e56da7b90c04c20de4b2d3cc9f`. Annotated tags
`types-v0.21.2`, `runtime-v0.23.13`, `protocol-v0.2.5` and
`platform-v0.34.6` were pushed atomically with main. Each upload executed once
through the supported Toko publication adapter, after a successful dry run and
source/archive admission. Hosted CI was not awaited.

| Published package | Registry archive SHA-256 |
| --- | --- |
| [ic-platform-types 0.21.2](https://crates.io/crates/ic-platform-types/0.21.2) | `3b08201b2ebbb9a9f0e1c8ee2854f48ff3f68a777c441147273c600bf6723d7d` |
| [ic-platform-runtime-guard 0.23.13](https://crates.io/crates/ic-platform-runtime-guard/0.23.13) | `86a97003ee80dc942a11a313430685929df56c6dc5eeff506239904355ef7aa5` |
| [ic-platform-macros 0.23.13](https://crates.io/crates/ic-platform-macros/0.23.13) | `d5afcfb99f22c7153355b387ff56b4d19e78ae1ea0023cf713e25124b2db613e` |
| [ic-platform-runtime 0.23.13](https://crates.io/crates/ic-platform-runtime/0.23.13) | `9f5f46eebbf414e38b7ad6db694ce05bebeae40ff8df28eba6c1e48a6c308df4` |
| [ic-platform-protocol 0.2.5](https://crates.io/crates/ic-platform-protocol/0.2.5) | `93ed6052f78c99bf45bcf1ffdd11bd6eb4de4463dbe425a7b63e093b8f7a0ed2` |
| [ic-platform 0.34.6](https://crates.io/crates/ic-platform/0.34.6) | `0759aec97b24f85cc16c042c3aa388955e84bd79e11703197fe0c60049db1b25` |

## Tooling and scope

Toko Tooling 0.44.9 is installed from its registry checksum
`bdfd5bc761f55ede02e82f8af36ca82415696209b3ff45e8f080e82cab85c418`.
Both workspaces and materialized candidate Query/Auth/Storage consumer manifests
select it. Original archived source and frozen comparison evidence are retained.
The installer verified all 85 managed files and admitted the selected tools.
Types now permits compatible thiserror updates: the product lock selects 2.0.21,
while the PocketIC qualification graph retains 2.0.18 where required.

The published additions are documented at their owning contracts:
[Runtime memory/cycles](runtime-memory-cycles.md), [raw_rand and HTTP pricing](raw-rand.md),
[chunk-cache messages](management-chunks.md) and [ingress packets](ingress-packets.md).
Registry publication does not claim actual Scheduling, Entropy, Backup or Wasm
Store migration, Query portable-fixture delivery, or resolution of the separate
Agent signing/derivation obligation. Those issue acceptance gates remain open.

## Qualification and registry proof

The complete local Types, repository, Protocol, Runtime and Platform catalogs
passed at the tagged source. Receipt:
`target/release-qualification/1794856281407-3912442/receipt.json`.
Both profile catalog hashes and complete log hashes are bound into that receipt.
The refreshed RustSec database at `02e2517f457a359fc6d3440ea4560c8ea188fee8`
reported zero vulnerabilities in the release lock. Native execution evidence is
Linux; no unobserved macOS or hosted-CI success is claimed.

Every extracted package passed all-feature native/Wasm Clippy and native tests.
Source provenance was independently fetched from the declared GitHub repository.
After each upload, an independent download matched the non-yanked sparse-index
checksum, the locally admitted Cargo archive and its clean tagged VCS identity.
Types was published before the Runtime support packages, then Runtime, Protocol
and Platform. Reproduction and retained attempts:

- ic-platform-types: check `target/publication/check.selection.cljfYY`; upload `target/publication/publish.selection.l237Es`; readback `target/registry-readback/1794856520132-3966229/verified.json`.
- ic-platform-runtime-guard: check `target/publication/check.selection.GTI99n`; upload `target/publication/publish.selection.e3AImG`; readback `target/registry-readback/1794856552650-3969795/verified.json`.
- ic-platform-macros: check `target/publication/check.selection.BRyByv`; upload `target/publication/publish.selection.sxqZh8`; readback `target/registry-readback/1794856586908-3978403/verified.json`.
- ic-platform-runtime: check `target/publication/check.selection.lvqxSU`; upload `target/publication/publish.selection.oTD0SL`; readback `target/registry-readback/1794856617253-3983390/verified.json`.
- ic-platform-protocol: check `target/publication/check.selection.pYUv0E`; upload `target/publication/publish.selection.viXF9q`; readback `target/registry-readback/1794856653074-3995759/verified.json`.
- ic-platform: check `target/publication/check.selection.k3MDkQ`; upload `target/publication/publish.selection.GSy8FD`; readback `target/registry-readback/1794856722420-4024704/verified.json`.

The registry-backed actual Query fixture passes all 184 topology/CLI tests and
strict Clippy; actual Auth passes all five installed tests. Metadata proves one
canonical registry Types 0.21.2 and Platform 0.34.6, Protocol 0.2.5 for Query and
Runtime 0.23.13 for Auth, with no Agent or legacy Platform edge. Toko is 0.44.9 in
both consumer graphs. Evidence and reproducible script:
`target/registry-cohort/1794856730010/verified.json`.
The independently authored Query schema fixture remains private, so this is
registry product proof, not a claim that issue #10's portable fixture delivery
is complete. All sibling working trees remain unchanged.

The separately maintained browser-codec fixture also passes after adopting Toko
0.44.9: eight Rust boundary tests and 37 retained browser frames, including
detached buffers and redundant outer framing. Its explicitly frozen published
Types 0.21.1/Wire 0.24.2 comparison selection is preserved. Evidence:
`target/auth-wire/1794856827468/verified.json`.
