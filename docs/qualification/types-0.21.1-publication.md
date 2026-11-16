# Types 0.21.1 publication

Published `ic-platform-types` 0.21.1 to crates.io on 2026-11-16 from
`2797582e24f782e442df841d24db1ea0fac7bb92`, annotated tag `types-v0.21.1`, public
repository `https://github.com/dragginzgame/ic-platform`.

The maintainer selected local qualification as the publication gate, without
waiting for hosted CI. Contract/Serde/oracle checks, default and self-authentication
native/Wasm consumers, the actual archive through pinned Auth passive source,
Runtime identity crossing, dependency checks, publication refusal wiring, security
audit and independently fetched source provenance all passed. The supported
publication dry run and single upload attempt completed successfully.

Retained local evidence:

- Dry run: `target/publication/check.selection.08XvIT`.
- Upload: `target/publication/publish.selection.pHlqRM`.
- Local upload admission: `target/types-publication/1794842640047-1817893`.
- Independent registry readback: `target/registry-readback/types-0.21.1`.

The sparse index reports an unyanked 0.21.1. The independently downloaded archive
matches both its index checksum and the locally admitted/uploaded archive:
`339477b696137ec7fd6f96725c5733b80d67c5fd75b27c8dfaa83846629b9958`.
Its `.cargo_vcs_info.json` identifies the clean source commit above and
`crates/ic-platform-types`. No retry or uncertain upload occurred.

[Registry package](https://crates.io/crates/ic-platform-types/0.21.1).
Production consumer migrations and other product releases remain separate.
