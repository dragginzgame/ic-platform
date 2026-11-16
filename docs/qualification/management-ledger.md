# Selected management and ledger formats

Protocol 0.2.7 supplies
[Platform #20](https://github.com/dragginzgame/ic-platform/issues/20) and the
installation portion of [Platform #11](https://github.com/dragginzgame/ic-platform/issues/11).
Protocol 0.2.7 is published; see [delivery evidence](../releases.md#external-signing-and-managementledger-release). Consumer migration/delivery remains separate.

## Public mapping

| Capability | Supported surface |
| --- | --- |
| `management` | `ProvisionalCreateArgs::encode`, `UpdateSettingsArgs::encode`, `InstallCodeArgs::encode`, `decode_created` |
| Existing status/start/stop | Reuses `management_snapshots::CanisterArgs`, `decode_canister_status`, `decode_empty_response`; these are re-exported by `management` |
| Creation settings | Controllers; compute, memory, freezing, reserved-cycle and Wasm-memory values; log visibility and environment variables. Fields preserve optional presence. This is the selected CMC-compatible settings record, not every management setting. |
| Installation | Explicit install/reinstall/upgrade; `Upgrade(None)` remains absent flags. Optional skip-pre-upgrade and keep/replace heap policy, exact Wasm/argument bytes, target and sender version remain explicit. |
| `ledger` | `CreateCanisterArgs::encode`, explicit subnet/filter settings, `decode_created` with every selected success/error payload, and `decode_amount` for arbitrary Nat balances/fees |
| Accounts | `encode_balance_account(&Types::Account, maximum)`, exact empty `encode_fee_args`, and `icp_account_identifier(&Types::Account)` |

Each encoder accepts an inclusive complete-output byte ceiling and counts the
entire output before reserving storage. Request data is caller-owned; no protocol
function authorizes the chosen mode, target, controllers, cycle amount or retry.
Source subaccounts are statically 32 bytes. ICRC account encoding preserves None
versus Some(zero); ICP account identity intentionally maps both to effective zero.
The latter frames the standard account domain and uses upstream SHA-224/CRC32,
not an authored cryptographic implementation.

Decoders select exactly one result and share explicit complete-byte, header/type,
traversal/work, logical-allocation, vector and skipped-work limits. Malformed,
truncated, trailing, wrong-arity and unknown selected variants refuse. Skipped
fields are still validated under the same budgets. Returned Principal values use
the canonical Types dependency. Numeric amounts/error codes/block IDs are arbitrary
Nat; no u64/u128 narrowing occurs. Duplicate retains both its block and optional
canister ID. FailedToCreate retains optional fee/refund blocks and diagnostic.
None of these observations establishes settlement or authorizes another create.

## Provenance and independent qualification

Declarations were independently expressed from the selected external schemas;
no legacy decoder, generated code, Agent or implementation is copied. Field and
variant IDs follow Candid's label rule. Native-only upstream Candid 0.10.40 and
management types 0.8.0 independently check the encoded requests. Cycles-ledger
schema provenance is the original
[DFINITY interface at 11c6debe](https://github.com/dfinity/cycles-ledger/blob/11c6debe4249a51c2a04f1d4e1a828ca20779c06/cycles-ledger/cycles-ledger.did).
Only the selected create and balance/fee operations are implemented; no textual
DID parser/tooling or generic public codec is introduced.

ICP identifier framing follows the established ledger account contract. Its
frozen management-principal/zero-subaccount answer was independently calculated
with Node/OpenSSL SHA-224 and zlib CRC32. Product suppliers remain original
sha2 0.11.1 (MIT/Apache-2.0) and crc32fast 1.5.5 (MIT/Apache-2.0), locked in Cargo.

`make qualify-management-ledger` passes five independent native schema/refusal
cases, strict Clippy, denied-warning Rustdoc, a standalone Wasm replay and the
normal/build graph check. Cases include all creation errors, Duplicate with/without
canister ID, exact/over encoding ceilings, every reply truncation, trailing bytes,
unknown selected errors, discarded-field budgets, optional controllers/subaccounts,
explicit upgrade flags, exact one-MiB Wasm and arguments, u64-max timestamps/versions
and 2^128 amounts/block/error values. The Wasm fixture independently replays a
native-Candid-generated successful creation reply and all short prefixes/budgets.
Its SHA-256 is `ffebf105080dd7a0fa582716d51f403d18c66e81b89937e6c541a06ce8dfc899`.
Log: `target/management-ledger-qualification.log`.

Regenerate `qualification/engine-view-wasm/fixtures/ledger-created.bin` only with
`REGENERATE_LEDGER_FIXTURE=1` on the native `engine-view-consumer --features ledger
--test ledger` run, then review bytes and rerun without regeneration. It is test
data, not a published fixture package. Existing status/snapshot qualification
retains its own independent schemas and finite capture/restore limits.

Product defaults remain empty, with no Runtime, Agent, Candid or management-SDK
production dependency. These supplier checks do not qualify Canic's retained
journals or original-request recovery, Backup capture/restore, or Blob stopped-host
CLI upgrade/activation. Their complete consumer journeys remain with those owners.
