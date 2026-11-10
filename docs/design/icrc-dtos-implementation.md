# Passive ICRC DTO implementation record

On 2026-11-10 this specification-only context authored the remaining passive
DTO declarations and helpers from [icrc-dtos-contract.md](icrc-dtos-contract.md).
Previous target modules, tests, supplier code/documentation and Git history were
excluded; ledger value/hash implementations were not inspected.

## Inputs and separation

The only repository file newly read for this batch was
`docs/design/icrc-dtos-contract.md`, relative to
`/home/adam/projects/ic-platform`. Previously admitted `AGENTS.md`,
`TOKO-TOOLING.md` and `docs/principles/rust-code-hygiene.md` supplied policy.
The coordinator confirmed the public `Nat::from(u64/u128)` and
`Int::from(i64/i128)` constructors. No codec implementation was read to discover
those APIs.

This context had authored other replacement components earlier; that history is
not claimed to be erased. No Account/archive/Types source was newly consulted or
copied for this batch. The separate public consumer tests and before/after
observation artifacts were not read or used as implementation text. No work was
delegated by this context.

Drafts were written only beneath `/tmp/icrc-dtos-draft/` during the baseline
hold. The coordinator reviewed them and explicitly released product writes after
its baseline capture. An exact process-name check found no Cargo/rustc process
immediately before installation. The seven modules were then overwritten from
the drafts without reading their previous contents.

## Authored files and behavior

Only these product modules and this record were written:

- `crates/ic-platform/src/icrc_ledger_types/metadata/mod.rs`
- `crates/ic-platform/src/icrc_ledger_types/transfer/mod.rs`
- `crates/ic-platform/src/icrc_ledger_types/allowance/mod.rs`
- `crates/ic-platform/src/icrc_ledger_types/blocks/mod.rs`
- `crates/ic-platform/src/icrc_ledger_types/icrc21/requests/mod.rs`
- `crates/ic-platform/src/icrc_ledger_types/icrc21/responses/mod.rs`
- `crates/ic-platform/src/icrc_ledger_types/icrc21/errors/mod.rs`
- `docs/design/icrc-dtos-implementation.md`

Declarations preserve the specified names, field/variant order, trait sets and
ordinary Serde behavior. Transfer/allowance defaults and transparent Memo remain
explicit. Subaccount and consent argument vectors keep sequence serialization;
ByteBuf fields keep their supplied byte representation. Field and variant docs
were independently written as short descriptions, without blanket lint allowances.

Metadata conversions preserve full integer ranges and move owned Nat/String/Vec
inputs. Memo's u64 conversion retains all eight big-endian bytes; its buffer
conversions move ownership. TransferError retains its exact public Display
strings without adding std::error::Error. Block-range narrowing borrows each
magnitude, checks start before length and never adds the resulting values.
Consent records add no validation, presentation, consent or payment policy.

No new abstraction or conversion family, dependency, feature, wrapper, crypto,
codec mapping or value/hash change was introduced. Existing NumTokens and
BlockIndex aliases remain. No private tests were added merely to repeat the
separately authored public compatibility matrix.

## Validation and limitations

Rustfmt 1.99.1 succeeded on all seven drafts and again on all seven installed
modules with `--edition 2024 --config-path /dev/null --config skip_children=true`.
This implementer ran no Cargo command, Clippy, compilation, tests or measurements.
The coordinator owns frozen observations, schema/byte checks, native/Wasm
qualification and provenance/source-guard updates; their results are separate
from implementer-run validation.

This record describes admitted inputs and actions. It does not provide legal
clearance, claim absence of model training exposure, or mathematically prove
independent authorship. Compatibility results alone cannot establish those
conclusions.
