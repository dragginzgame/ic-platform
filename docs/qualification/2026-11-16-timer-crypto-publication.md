# Runtime timer and Platform crypto publication — 16 November 2026

The maintainer requested latest Platform/Testkit adoption in qualification and
deployment. The changed groups are published from clean source
`2881ded6c0206f191df386901101fc7f4d5fa949`, with annotated tags
`runtime-v0.23.12` and `platform-v0.34.4`. Main and both tags were pushed atomically.
Types 0.21.1 and Protocol 0.2.3 retain their unchanged published product versions.

| Package | Published version | Registry archive SHA-256 |
| --- | --- | --- |
| ic-platform-runtime-guard | [0.23.12](https://crates.io/crates/ic-platform-runtime-guard/0.23.12) | `767b271f8154333275a72f4ba369d2b33f33398eade5210cbcdf8af96e9b912d` |
| ic-platform-macros | [0.23.12](https://crates.io/crates/ic-platform-macros/0.23.12) | `44ed0fb9bbfd90e98f1ed5cd46b9bbeeff08c35ca36a7cd34e0202c2b4f464ec` |
| ic-platform-runtime | [0.23.12](https://crates.io/crates/ic-platform-runtime/0.23.12) | `6fa7c8ed9f3a84ba059ea06d2a9b21f6d77061ca33056c50bff2e58468337184` |
| ic-platform | [0.34.4](https://crates.io/crates/ic-platform/0.34.4) | `1f26d836ddaf94dfb76ca3a9e3eb43ea6dee3fbb90266b3532cb218af5abaf26` |

All four independently downloaded archives match their sparse-index checksums,
local admitted archives and exact clean-source VCS identities. Readback evidence:
`target/registry-readback/1794849863104/verified.json`, with retained reproduction.
Each upload ran once. No hosted CI completion was awaited.

## Local admission

The complete repository, Protocol, Runtime and Platform catalog passed at the
release source. Receipt:
`target/release-qualification/1794849531504-2945508/receipt.json`.
It binds compiler, source, catalog and complete profile log hashes. Root and
qualification lock audits report zero known vulnerabilities against RustSec
`02e2517f457a359fc6d3440ea4560c8ea188fee8`; existing unmaintained notices remain.
The release catalog no longer repeats Gitleaks: the adopted baseline owns that
scan in CI once. Product/native correctness checks were retained.

Every package passed native/Wasm all-feature Clippy and native tests from the
extracted archive, source provenance admission and the supported publication dry
run before upload. Retained publication records:

| Package | Dry run | Upload |
| --- | --- | --- |
| Guard | `target/publication/check.selection.eN7JOB` | `target/publication/publish.selection.i8QK9m` |
| Macros | `target/publication/check.selection.59pWEm` | `target/publication/publish.selection.SV90PP` |
| Runtime | `target/publication/check.selection.HzI5PL` | `target/publication/publish.selection.G5HvCh` |
| Platform | `target/publication/check.selection.9f9cda` | `target/publication/publish.selection.WTRMLN` |

## Supplier adoption and retained limits

Testkit 0.107.1 and Host Process 0.80.2 were freshly checked against the registry
and adopted. Toko 0.44.6 remains current. The [server contract](testkit-server.md)
records migration from removed Testkit lifecycle helpers, authenticated prepared
assets, owned cleanup, deadlines and the private unpublished helper. All five
actual Auth installed tests pass using the new Testkit selection. Frozen Auth
Build/Host suppliers remain consumer qualification inputs; broader refresh is
separate from this Platform/Testkit adoption and is revisited with Auth #7.

The prepublication independent Platform oracle was the then-current 0.34.3.
After publication, qualification source `7f9adf8` selects registry Platform
0.34.4 and preserves separate local/registry verifier identities through checked
byte conversions. The installed driver passes with that current oracle.
Legacy Platform 0.33.7 remains explicitly named only where retained codec or
Auth crypto oracle coverage requires it; it is not a candidate product dependency.

[Timer evidence](runtime-timers.md) qualifies the one-executor supplier boundary,
not Scheduling's migration. [Primitive evidence](auth-crypto.md) qualifies Auth's
Ed25519/root-delegation paths, not the complete certified-issuer replacement.
Issues #4 and #7 remain open for their unresolved Agent/derivation, native
integration and certified-issuer obligations. Sibling checkouts remain unchanged.
Linux and standalone Wasm results do not establish macOS or production adoption.

## Published consumer verification

`target/registry-cohort/1794849994561/verified.json` retains the reproduction script,
source archive, metadata and installed output. Runtime, guard, macros and Types
resolve from crates.io for native consumer tests and the installed canister.
The unchanged installed fixture exercises timers, task/callback custody, refunds,
resource refusal, raw certificate publication and drained upgrades. Its certificate
checks deliberately retain separate local and registry Platform verifiers.
A second isolated graph uses registry Platform 0.34.4 for the independent crypto
vectors. Separating these graphs preserves Cargo source identity without aliases
that pretend two names for the same registry package are independent oracles.
