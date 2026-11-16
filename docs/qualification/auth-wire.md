# Auth browser generation with the published Types cohort

[Issue #8](https://github.com/dragginzgame/ic-platform/issues/8) is resolved at the
supplier/cohort boundary by published Wire 0.24.2, Types 0.21.1 and Runtime
0.23.11. Runtime and Types are the previously qualified published pair; Wire
0.24.2 supplies the missing native `platform-types` trait integration. No trait
implementation or shadow Principal belongs in Platform or Auth.

Wire's archive checksum is
`852a98f3b818bf1e6841c63845d88ded595b659181377e749cf8cf50b12b48c6`;
its VCS source is `1da48059e36132b09775f6130638b8b9c3169538`. Auth's generator
selects `ic-auth-wire/typescript`, which enables `ic-wire/derive` and
`ic-wire/platform-types`. The selected integration now reaches Types 0.21.1.

`make qualify-auth-wire` archives actual Auth
`b78991e9833320692921bf766e51ad361c9c8c4d`, selects exact registry Types 0.21.1 and
Wire 0.24.2, and executes the unchanged consumer generator and tests. No source
file is patched and no local Platform/Types dependency is substituted.

The 16 November 2026 Linux run passed:

- All 19 Wire consumer tests, native Clippy and Wasm compilation.
- The actual TypeScript generator and Auth's historical interoperability runner:
  37 retained frames, detached buffer behavior and redundant outer-varint cases
  against pinned oracle `e52a40be75dd9809d0528189d84b43fe96952ad9`.
- Compilation with the consumer's TypeScript 7.0.2 tool and byte-for-byte equality
  of all eight generated client JavaScript/declaration files. All four endpoint
  identities and allowances remain covered by the consumer tests.

Evidence: `target/auth-wire/1794848164260/verified.json`, source archive SHA-256
`6a043179451c4b467dc45ae2d629135e1f643d73d57ff5a7fe40b575a30c0d3b`, selected lock
SHA-256 `b46078c022eb3969afb5f31e6b300349681c49af522ba31f25377cacb45a322b`.
The runner retains generated-file hashes and uses a private bare object store
solely for the consumer's historical read-only oracle. It requires the Auth
checkout and its installed TypeScript tool; it writes neither.

The earlier [registry Runtime/Types installed evidence](2026-11-16-runtime-protocol-publication.md)
continues to cover five installed Auth cases with the identical published pair.
This pass does not rerun the installed browser application or certify its legacy
crypto path. That independent migration remains #7. Production dependency edits
and delivery belong to Auth; no sibling checkout was changed. Local qualification
scripts/documentation are uncommitted and unpublished at this record.
