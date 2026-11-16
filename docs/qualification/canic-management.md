# Canic certified observations and retained management requests

This record covers the supplier additions for
[Platform #22](https://github.com/dragginzgame/ic-platform/issues/22) and
[Platform #23](https://github.com/dragginzgame/ic-platform/issues/23).
The selected releases are Platform 0.34.10 and Protocol 0.3.0. Publication
status and verified registry evidence are recorded in [releases](../releases.md). Canonical Types remains 0.21.2. Canic migration, root-subnet admission,
approval digests, installed recovery and delivery remain consumer obligations.
No sibling source was changed.

## Certified native observations

Under `native-read`, `ReadClient::read_canister(canister, expiry, time,
maximum_controllers)` returns immutable `CanisterObservation` with the selected
canister, controller slice, hosting subnet, signing time and original certificate.
It requests the exact canister controller path and time, authenticates before
extracting fields, and uses the existing range/delegation verifier. A delegated
observation binds the canister to the root-authorized signing subnet. A direct
root certificate must contain the selected canister's controller leaf and uses
only the caller's independently admitted `ReadConfig::root_subnet` binding.
Neither a routing canister nor unverified metadata supplies that trust input.

Controller leaves follow the [IC certified state contract](https://docs.internetcomputer.org/references/ic-interface-spec/):
a bounded CBOR array of principal byte strings. Text, numeric arrays, invalid
principals, duplicates and trailing bytes refuse. Missing paths remain
`Read(MissingPath)`; pruned witnesses return `PrunedPath`; explicit count refusal
returns `ControllerLimit`. Other transport, malformed and authentication errors
remain typed under `CanisterReadError::Read`. The existing `ReadError` API is
unchanged. An authenticated empty controller set is distinct from missing proof.

The message cap bounds input; the controller count is admitted before result
allocation, with fallible incremental reservation. Existing combined tree/depth,
delegation-byte and range limits apply. These are not total process-memory or
whole-parser-work guarantees. There is no discovery, cache, signer, retry or
lifecycle policy in this operation. The observation describes its signing time,
not later placement or permission to upgrade.

Three focused independent signed-certificate tests cover direct/delegated success,
wrong roots, wrong canisters/ranges, separate stale signing/delegation windows,
missing/pruned paths, malformed controller encodings, exact/over count and input
limits, tree-budget refusal and truncated/trailing certificates. Existing HTTP
fault and signed recovery tests remain in `make qualify-native-read
qualify-native-signed`. The installed fixture independently compares controllers
and subnet against PocketIC for both root-subnet and application-subnet canisters.

## Complete management schema and migration

Protocol's `management` feature now provides `CreateCanisterArgs` and
`InstallChunkedCodeArgs`. `management_chunks::ChunkHash` remains the shared hash
record. Its writer is enabled only for `management`, preserving the standalone
chunk decoder feature boundary. The selected `management_snapshots::CanisterStatus`
now requires the observed `version: u64`; missing or mistyped versions refuse.
Status/controller/module-hash decoding remains one shared projection.

`management::CanisterSettings` is the thirteen-field management schema, including
minimum incoming call cycles, log memory, snapshot visibility and status visibility.
`ProvisionalCreateArgs` and `UpdateSettingsArgs` select it. The original
`CreationSettings` remains the nine-field CMC schema and is still used only by
`ledger::CmcCreationArgs`. No automatic conversion conflates these contracts.
This follows the current [management interface](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/).

The settings type replacement and required status version break the previous
Rust/decode contract, so Protocol advances from 0.2.8 to **0.3.0**. Management
callers change `CreationSettings` to `CanisterSettings`; CMC callers retain
`CreationSettings`. This restores legacy management request bytes rather than
promising equality with the incomplete Protocol 0.2.x management schema. Never
rewrite, normalize or re-sign retained attempts to migrate them. Consumers must
bind each retained request to its actual originating schema and approval.

All writers retain complete-size admission before output allocation. Shared
response byte/header/type/work/allocation/vector/skip limits remain required.
No generic codec, workflow, cycle accounting, authorization or effect custody is
added. Independently authored status tests require the maximum u64 version and
refuse missing/wrong versions, prefixes, trailing bytes and insufficient input.

## Independent retained-byte evidence

The eight `root-*.bin` request fixtures are produced by registry Platform 0.33.7,
source `663cf913997490f448553119a6caf66b3cdecc81`, archive SHA-256
`3e07e0565c70df9222016e93d6f488312176ef5e7036ded60db8b4bc3f61c05f`.
Its source is under the Toko Studios Network Licence 1.0, retained in the registry
archive. Only fixture output is admitted; no codec, generated mapping or algorithm
implementation is copied into product code. The original schemas/encoder execute
only in an isolated, explicitly invoked oracle. Its lock and small authored driver
are retained under `qualification/management-retained/`.

`node qualification/management-retained/regenerate.mjs` deliberately regenerates
those requests; ordinary tests never regenerate them. The distinct status frame
comes from the independent Candid schema in `tests/management_snapshots.rs`; use
`REGENERATE_ROOT_STATUS=1` with that test to regenerate it explicitly.

`make qualify-management-ledger qualify-management-snapshots
qualify-management-chunks` checks native and standalone Wasm evidence, strict
Clippy/docs and dependency boundaries. Frozen requests cover optional settings,
ordinary/provisional creation, settings updates, CMC distinction and chunked
installation with absent/present store/sender fields. Installed qualification
sends a retained signed chunked upgrade while stopped, then starts the canister
and compares required before/after versions to independent PocketIC observations.

Native execution evidence is Linux. These tests remain portable to the supported
macOS runners; this work does not claim new macOS execution, mainnet effects or
Canic's actual installed recovery acceptance. Publication is separate.

Focused local results on 2026-11-16: six ledger/management cases, ten snapshot
cases, standalone ledger/snapshot/chunk Wasm replays and all 42 Protocol unit
regressions passed. Three new certified-canister unit cases and the existing
native HTTP/signing checks passed. The final installed fixture passed for both
subnet kinds, including retained signed chunked upgrade and independent version
comparison. Standalone native-read Clippy/Rustdoc and empty-default Protocol
Clippy passed. Logs: `target/issue23-final.log`, `target/issue23-chunks.log`,
`target/issue23-regression.log`, `target/issues22-23-native.log` (native-read pass;
its subsequent fixture compile error was repaired),
`target/issue22-23-installed-final.log` and `target/issue22-feature.log`.
