# Runtime and Protocol publication — 16 November 2026

The maintainer requested deployment of all current crates. Runtime and Protocol
had new versions; Types 0.21.1 and Platform 0.34.3 were already published and
unchanged. They were verified against the registry and were not uploaded again.

Source: `cbbf26ade6644ed6fbcadfe3ccbafe809c362f25`. Annotated tags
`runtime-v0.23.11` and `protocol-v0.2.3` were pushed before publication.

| Package | Published version | Registry archive SHA-256 |
| --- | --- | --- |
| ic-platform-runtime-guard | [0.23.11](https://crates.io/crates/ic-platform-runtime-guard/0.23.11) | `8fe0fe86fcd6d68f2ae8858f2cd810d8901ff6fa3c419e5bdfcc76756a16ea43` |
| ic-platform-macros | [0.23.11](https://crates.io/crates/ic-platform-macros/0.23.11) | `ff10e5cea3359d8474178634fd4d7ec28c770288adc2dd0e28f8f1bef8a0ee79` |
| ic-platform-runtime | [0.23.11](https://crates.io/crates/ic-platform-runtime/0.23.11) | `4dc0059577167175de70e44128d415b772d9d20f3276cb3d8aeb09b01448ec09` |
| ic-platform-protocol | [0.2.3](https://crates.io/crates/ic-platform-protocol/0.2.3) | `9fe6bc76a461f60b585bee7d6e5117bbe6e63de87ef3cfe4982f0895c00fad91` |

All four downloaded archives match their sparse-index checksums, admitted local
archives and exact clean-source VCS records. Each upload ran once; none was
retried. All six current product package selections are now available on crates.io.

## Qualification and evidence

The full local release catalog passed repository, Protocol, Runtime and Platform
profiles. Receipt: `target/release-qualification/1794846567499-2419318/receipt.json`.
The admission checks bound every publication to its source, compiler, catalog and
complete log hashes. Product and qualification lock audits reported no known
vulnerabilities; previously documented unmaintained dependency notices remain.
No dependency maintenance or compiler/tooling upgrade was included. CI completion
was not awaited, as requested; local results do not assert hosted/macOS results.

Each package passed the supported publication dry run, extracted-archive native/
Wasm all-feature Clippy, native tests and source provenance verification.

| Package | Dry run | Upload |
| --- | --- | --- |
| Runtime guard | `target/publication/check.selection.6SW2cw` | `target/publication/publish.selection.RCqrTy` |
| Macros | `target/publication/check.selection.je4N87` | `target/publication/publish.selection.Ree9NK` |
| Runtime | `target/publication/check.selection.dhxrcn` | `target/publication/publish.selection.W0q0X7` |
| Protocol | `target/publication/check.selection.AKnM7R` | `target/publication/publish.selection.zsG7Z5` |

Registry-only consumer evidence:
`target/registry-cohort/1794846927632/verified.json`. It records exact registry
identity for all six products, Protocol HTTP schema/limit tests, Types/Runtime
consumer tests, execution of both managed and raw Wasm exports, and the raw
feature's dependency-free graph. Its reproduction script and metadata are retained
beside the result.

Auth's actual five installed fixture tests also pass with Runtime 0.23.11 and
Types 0.21.1 resolved only from crates.io:
`target/auth-runtime/1794846870152/registry-verified.json`. The pinned consumer is
`b78991e9833320692921bf766e51ad361c9c8c4d`; legacy Platform crypto remains an explicit
supplier. This proves the packaged Runtime cohort, not replacement of that crypto
supplier or a production Auth migration.

The release includes Auth's restored system/entrypoint capabilities, Storage's
synchronous `system-raw` boundary (#6), and Protocol's bounded management HTTP GET
encoder (#5). Storage's installed migration/benchmark reruns and Testkit's
held-reply adoption remain downstream work. Existing Runtime experimental
transport and failed-cleanup limits remain unchanged.
