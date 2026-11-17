# Anonymous prepared transport and Blob qualification codecs

Local candidates: Platform 0.34.13 and Protocol 0.3.5, based on delivered
`e9cf54a`. The crates.io sparse index on 2026-11-17 confirmed baselines 0.34.12
and 0.3.4 before version selection. This evidence is focused supplier qualification,
not registry publication or downstream production migration.

## Platform #25: explicit anonymous admission

`PreparedUpdate::from_bytes` and `PreparedRequestStatus::from_bytes` remain
signature-only. Their `from_anonymous_bytes` counterparts require sender
`2vxsx-fae` and absent public key, signature and delegation fields. Empty and null
authentication fields are refused. Sender info remains unsupported. The immutable
update exposes `AuthenticationProfile`; admission infers no controller authority.

Status admission binds the same profile, sender and exactly one original
`request_status/<request ID>` path. Both original expiries and encoded envelopes
remain intact. Status expiry is independent of update expiry. Dispatch and
observation reuse the existing bounded client, explicit root/target/time policies,
and certificate verifier. There is no signer fallback, retry or expiry renewal.

`make qualify-native-signed` covers:

- Existing direct Ed25519/secp256k1 and delegation refusal regressions, private
  certificate/status tests, strict Clippy and Rustdoc.
- Native anonymous exact-byte dispatch and status reads, original-ID and path
  binding, profile separation, absent versus empty/null fields, malformed,
  trailing and oversized envelopes. Transport failure establishes no nonexecution.
- Existing installed signed direct/delegated certificate paths and upgrade proof.
- New `native-anonymous-installed`: actual ICP CLI 1.7.0 `--identity anonymous`
  preparation; isolated PocketIC create, install, call, stop, certified Stopped,
  upgrade, start and subsequent call. Each dispatch response is discarded and
  recovery uses the retained original update and CLI status envelopes. Request
  IDs, exact bytes, effective target, offline certificate replay and version
  advancement are asserted. Host Process bounds CLI execution and owns cleanup.

Set `ICP_CLI` to an explicitly selected ICP CLI 1.7.0 executable (default `icp`).
This Linux run used `/home/adam/projects/canic/.tools/ic/bin/icp` read-only;
the PATH binary was 1.6.0. The server was PocketIC 16.1.0, Testkit 0.107.4 and
Host Process 0.80.4. The pre-existing Host Process lockfile update is preserved.
No sibling checkout was edited. Actual Canic adoption, application approvals,
root-subnet policy and production journey remain Canic's obligations.

## Platform #24: bounded Blob formats

| API | Selected contract | Qualification ceilings |
| --- | --- | --- |
| `management::decode_canister_metrics` | One record, `cycles_consumed.instructions: nat`; arbitrary execution-cycle charges | bytes 4096, header 1024, types 32, work/allocation/skipping 16384 |
| `caffeine::decode_top_up` | One `opt record { target_balance: opt nat; account: opt principal }` | bytes 1024, header 512, types 16, work/allocation/skipping 4096 |
| `caffeine::decode_top_up_result` | One result; validate/skip arbitrary `Ok`, or typed `NotAuthorized(principal)`, `AccountBalanceOverflow`, `InternalError(text)`, `TopUpWithoutCycles` | bytes 8192, header 2048, types 32, work/allocation/skipping 65536 |

Vector ceilings are explicit too (4096, 1024 and 8192 respectively). These are
caller-selected limits, not hidden global policy. Unknown selected errors,
incompatible fields, malformed selected/discarded values, wrong arity, trailing
bytes and budget exhaustion are refused. No generic public Candid codec was added.
Execution charges are not instruction counts or balance differences. Top-up
classification does not establish enrollment, accepted cycles, credit or settlement.

`make qualify-caffeine qualify-management-ledger` covers independent upstream
Candid schemas, outer/field option distinctions, dynamic principal bytes,
naturals exceeding u128, all four errors, arbitrary success records/vectors,
malformed discarded payloads, truncation/arity/trailing refusals and exact versus
one-below resource thresholds. Frozen independent vectors replay on wasm32;
strict Clippy, Rustdoc and product graph checks retain the no-Candid/no-Runtime
production dependency boundary.

The literal `tests/fixtures/top-up-without-cycles.hex` is the actual Blob response
from `dragginzgame/ic-blob-storage` commit
`3993d90ead8df49a79834f384eff6f0556a65be7`, original path
`qualification/ic-blob-storage-enrollment-probe/examples/enrollment/fixtures/top-up-without-cycles.hex`.
It is a wire-data fixture, not copied implementation or a generic legacy decoder.
Independent schema declarations and the optional regeneration switches in the
consumer tests establish provenance of the new Wasm vectors.

These focused checks passed on Linux with Rust 1.100.0. They do not claim native
macOS execution. Blob's production/qualification migration, actual capacity and
enrollment suites, focused callback proof and removal of its two legacy graphs
remain downstream gates; no sibling files or legacy dependencies were changed.
