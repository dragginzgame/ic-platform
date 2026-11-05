# Current handoff — 2026-11-05

## Platform 0.23.1 delivery preparation

Shared **0.30.5** is adopted at published supplier
`be3771031ebfb340583b238dee5446be65929c62`, including the approved checker correction.
The pending Platform batch covers #140, #146 and the reviewed cleanup below;
Runtime, Types and browser versions are unchanged. The incoming Candid/derive
0.10.40 lock draft retains separate custody and is restored after delivery.

`make check-boundary` now enforces the named external Candid source policy.
It admits 153 exact declarations across 226 Rust files, without module exemptions.
External DTO admission and new/stale declaration refusal pass; type identity and
source/license guards retain their separate duties. No cryptographic body changed.

#146 adds the pinned management `CreateCanisterArgs` and typed creation codec;
see the [pending API contract](../migration.md#management-create_canister-request-pending-0231).
Native/Wasm Clippy, schema/byte/range and reply-admission checks pass. Logs:
`target/issue146/`. Canic adoption and Platform publication remain separate.

Twenty-five obsolete experimental files are retired; compact conclusions and
references remain in the [audit](../audits/shared-030-practices-2026-11-05.md).
New output uses `target/evidence/`; recovery evidence and vectors are untouched.

Focused snapshot, Cargo/Candid boundary, consumer refusal, formatting/hook, LOC,
documentation and strict Wasm fixture Clippy checks pass. Logs:
`target/shared0304-adoption/`. No release/fleet matrix, benchmark or macOS run.

## Native Profiler #140, local only

The provider removes its 21 path attributes while preserving public aliases,
private parents, recorder identity and viewer locations. Nine patch test modules
and two standalone tests move with their original bodies. The native gate covers
the moved selections and strict library/binary Clippy. See the
[paired qualification](../audits/native-profiler-140-2026-11-05.md).

The exact IC pin is unchanged. Validation uses committed Profiler `50d9548` as
read-only archived fixture input under `target/issue140/consumer`; the sibling's
concurrent work is untouched. Linux qualification passes on Rust 1.99.1, including
83 recorder, 114 product and six installed scenarios against hash `e2240c66dc9f`.
Evidence: `target/issue140/qualification-summary.json`. No macOS/Bazel execution,
consumer pin adoption, commit, release or publication is claimed.

## Published Rust groups

**ic-platform 0.23.0** is published at release commit
`7d2230c585b1a91c11ff12d0fcfce24bda424550`, tag `platform-v0.23.0`.
Runtime remains **0.19.2**, Types **0.18.1**. #141/#145 are closed. The native
metrics decoder returns `Result<Option<SubnetMetrics>, CborError>`; consumers
must advance their Platform requirement from `0.22` to `0.23`.
Features remain `certification-cbor`, native `ic-protocol`, and development-only
`ic-protocol-fixtures`. See the [migration table](../migration.md#typed-ic-packet-and-fixture-boundary-0223).
Sibling adoption/qualification and releases remain separately owned.

All twenty Linux release stages passed on Rust 1.99.1, source `4170f4d`.
The exact-source resume after a retained Candid draft completed without bypassing
the original proof. Receipt:
`target/release-validation/qualification.AvTtIC/receipt.json`.
Registry-only certificate/protocol consumers, full-u128/null refusal cases,
archive checksum and clean VCS provenance match. Summary:
`target/platform023-delivery/deployment-verification.json`.
This is the recorded release evidence, not qualification of the new local snapshot.
Prior details remain in the [delivery handoff](https://github.com/dragginzgame/ic-platform/blob/07b442e/docs/status/current.md).

#50 remains the crypto advisory without a waiver. The
[Canic report](../audits/platform-022-canic-wasm-2026-11-05.md) remains
the current matched size evidence; cleanup introduces no new size/performance claim.

## Published browser SDK 0.2.0

@tokostudios/ic-platform 0.2.0 is published on npm. Source commit d30e01c and
annotated tag browser-v0.2.0 are pushed to origin/main. The release replaces
generic Cbor exports with typed ingress inspection and explicit development
response/corruption fixtures. SDK Core remains exactly 6.1.1; decoding, hashing
and cryptography stay upstream. Consumer limits, request binding, verification
and effect policy retain their owners.

All 21 browser tests, packed consumers on TypeScript 6.0.3/7.0.2, export/bundle
isolation and Caffeine/Auth singleton checks pass. The final archive also passes
actual Chromium page and module-worker qualification. Release Clippy and fresh
prepublishOnly checks pass. Registry integrity/checksum and the sole SDK Core
dependency match the qualified archive. Exact custody and logs are under
target/browser020-delivery/, including publication-complete.json. Earlier
authentication failures are reconciled by that matching registry observation.

Auth is adopting the published package in its own checkout; supplier proofs do
not establish consumer adoption. Unrelated Rust Cargo.lock edits remain preserved
and uncommitted. Other Rust release groups were not advanced by this npm release.

## Evidence custody

The approved archive/bundles remain in
`recovery-evidence/retired-checkouts/20261104-issue139/`, with checksum and
relocation receipts. Runtime 0.18.1 reproducible binaries remain under
`target/recovery-evidence/runtime-0181-delivery.LgU8r6/`. Review remains due
**2026-12-04**, owned by the maintainer. Retention checks refuse excess/expiry
without deletion or extension. Cargo clean can reclaim target evidence; the
historical archive and `.git/issue-139-suppliers/` remain separately protected.
Recovery refs `refs/retained/platform-023-unrelated` and
`refs/retained/platform-023-late-candid` are retained.
