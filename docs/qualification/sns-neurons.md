# SNS neuron and ICRC display supplier

Protocol 0.2.8 implements the bounded supplier contract in
[Platform #21](https://github.com/dragginzgame/ic-platform/issues/21).
This is codec qualification, not Query adoption, real-canister observation,
publication, authentication or entitlement evidence.

## Pinned wire contracts

- [SNS Governance Candid](https://github.com/dfinity/ic/blob/fe5a15bebf3490ef8f0cec5b5704422ee683c660/rs/sns/governance/canister/governance.did):
  `get_neuron`, `list_neurons`, their response records, neuron IDs, permissions,
  dissolve states and governance errors.
- [ICRC-1](https://github.com/dfinity/ICRC-1/blob/5d670e54d9a58fbf472bf0a25f33743d60cfd0e6/standards/ICRC-1/README.md):
  `icrc1_decimals : () -> (nat8)` and `icrc1_symbol : () -> (text)`.

The `sns-neurons` feature exports `sns::{GetNeuronArgs, ListNeuronsArgs}` and
`decode_get_neuron`/`decode_list_neurons`. It independently selects canonical
Types and Serde, without ledger, Runtime, transport or arbitrary-integer support.
The existing `ledger` feature adds `encode_metadata_args`, `decode_decimals` and
`decode_symbol`. Ledger use requires no SNS feature or governance precondition.
Existing canonical Account arguments and arbitrary-Nat balance decoding remain
unchanged. No legacy decoder or implementation is copied.

## Projection and consumer ownership

The neuron projection retains optional ID bytes, ordered permissions with
optional principals and raw signed codes, cached stake, fees, maturity, optional
staked maturity, creation and aging timestamps, optional dissolve state,
auto-stake flag, vesting period and source NNS neuron ID, and raw voting-power
percentage multiplier. Delay and absolute dissolve time remain distinct.
Governance errors retain signed code and text. Missing result, governance error
and malformed response remain distinct outcomes.

The projection intentionally omits `followees`, `topic_followees`,
`disburse_maturity_in_progress` and `latest_reward_event_participation`. Unknown
record fields are structurally validated and skipped under the same parser
budgets. This is not a full neuron export. Unknown selected variant cases refuse;
unknown integer permission/error codes survive unchanged.

Any explicitly selected deployment implementing the supported SNS/ICRC contracts
is eligible; no deployment allowlist is embedded. Query owns the selected governance/ledger, principal filtering, 32-byte neuron-ID
validation, requested-ID equality, duplicate handling, pagination bounds and
completeness, eligibility and entitlement. No discovery, time or ownership
inference occurs here. Raw amounts do not establish spendable balance. An absent
account subaccount and explicit zero subaccount remain distinct encoded values.
Metadata errors are independent of balance results.

## Bounds and proof

Encoders calculate complete size before allocation and admit an explicit byte
maximum. Decoders require exact single-result arity and complete consumption.
`candid::Limits` bounds complete input, header, type count, work, logical
allocation, vector elements and skipped bytes. Text is UTF-8 checked and bounded
by input and logical allocation budgets before allocation; there is no separate
per-field symbol-length policy. Optional malformed payloads are not converted to
absence. These are logical parser limits, not a process RSS guarantee.

Run `make qualify-sns-neurons`. Independent native Candid schemas test request
shape, optional IDs/principals, signed and unsigned extremes, both dissolve
states, absent values, governance errors, missing results, discarded fields,
malformed UTF-8, wrong types/arity, selected unknown variants, truncation and
trailing bytes. Resource thresholds are tested at the smallest accepted value
and one below. Metadata tests retain a balance above u128 and account subaccount
presence. Six frozen frames are independently produced by the native oracle and
replayed by a standalone wasm32 executable with no imports; Wasm also checks
resource refusals and request byte limits.

The native oracle is qualification-only Candid, pinned in qualification/Cargo.lock.
Product code uses the existing bounded parser and canonical Types. Linux native
checks and Node/V8 Wasm execution do not establish macOS execution, IC transport,
mainnet interoperability or the consumer's authorization and collection gates.
