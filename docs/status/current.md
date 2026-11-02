# Current handoff

## Verified Rust publication — 2026-11-02

All six workspace crates are published and independently verified on crates.io.
The four groups keep their independent versions; no previously delivered package
was uploaded again.

| Group | Packages | Version | Release commit / tag |
| --- | --- | --- | --- |
| Types | ic-platform-types | 0.17.0 | 42beb561942f33e5859afef95b8b35a2131cb530 / types-v0.17.0 |
| PocketIC | ic-platform-pocket-ic | 0.18.0 | 42a6b5c7d987ee4c7e9c38a437df0950f9f210c2 / pocket-ic-v0.18.0 |
| Runtime | ic-platform-runtime, ic-platform-macros, ic-platform-runtime-guard | 0.18.0 | d3343171c942a20590513fb1752d5fecf7b5d670 / runtime-v0.18.0 |
| Platform | ic-platform | 0.18.0 | 81490a94fbe35c28f05c6f74d2dc4e50a643822a / platform-v0.18.0 |

Shared 0.27.27 at 651f5f7acb2e3055abd51ba5f835a2112801666a remains the
installed executable snapshot. All six packages require Rust 1.99.1. Each group
passed the complete existing CI and installed owner/Timers suites before its
standard release, tag, atomic push and Cargo publication. No optional stage reuse
or group-validation mapping was enabled. Official registry metadata, archive
checksums and clean VCS identities match every exact release commit above.
A fresh registry-only consumer additionally passed native/Wasm Clippy and proved
canonical Principal/Account, prepared-call and native PocketIC identities without
local patches. Native execution here was Linux; no fresh macOS or live IC
acceptance is implied. Browser/npm and sibling publications were unchanged.

Platform 0.18 removes the stable-structures feature/module, direct dependency,
identity fixtures and obsolete pin exception. The workspace lock and native/Wasm
normal/build graphs exclude that package. Runtime's raw stable-memory System API
passthroughs remain unchanged. This completes source commit ee58738; the pending
compiler/ownership batch and Principal provider update are also delivered.

Durable release/publication/readback logs are under
`.git/release-state/platform-018-final-delivery.9UcrQT/` and
`.git/release-state/platform-018-deployment/`, with the four normal group plans.
The earlier failed Runtime intent is preserved as `prior-runtime-0.18.0.plan`
in the final-delivery directory. Archive readback receipts are in unique registry
subdirectories under the earlier deployment directory. Focused removal evidence
is in `target/stable-structures-removal/`; the published registry consumer is in
`target/shared-02727-release/published-018-consumer/`.

## Sibling adoption and qualification source

The earlier Runtime preparation refusal is resolved. The renewed deployment
request authorized the already-tested Scheduling topic branch push. Exact commit
`1c57a6d9abfd432fc683c5016d4625763682483a` is available on
`adopt/runtime-018-platform-qualification`. Only its Runtime requirement and
pending notes change; timer bodies and Scheduling's lock remain unchanged.
Platform source commit 6c50bf3 selects that immutable Timers 0.37.0 source and
refreshes the actual composition lock. Runtime 0.18 preparation and both installed
suites passed with the exact singleton guard retained. No Scheduling merge,
release or crate publication was performed. Return the test-only Git selection
to a qualified registry release after the Scheduling owner publishes it.

Tooling's native Testkit memory instrumentation and Canic's native delegation
fixture still select stable-structures through Platform 0.17. The maintainer
explicitly owns those feature/import removals before facade 0.18 adoption; both
sibling trees remain untouched by this cut. Blob Client/Testkit direct Types and
Runtime source migrations were integrated earlier; their version adoption and
delivery remain owner-scoped. The mistaken Tooling deployment prompt remains
withdrawn. Do not restart its release workflow.

Preserve the existing [Runtime ownership contract](../migration.md#independent-runtime-owner-131),
canonical Types, guarded imported bodies and consumer-owned policy. The BLS
modernization trial remains unadopted; the maintainer chose the qualified DFINITY
provider versions. Historical qualification and earlier delivery details remain
in the [pre-finalization handoff](https://github.com/dragginzgame/ic-platform/blob/81490a94fbe35c28f05c6f74d2dc4e50a643822a/docs/status/current.md).
Shared snapshot payloads stay immutable; refresh only through the exporter.
