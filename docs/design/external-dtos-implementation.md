# External DTO declaration implementation record

## Scope and inputs

The sole component specification read by the isolated implementer was
`docs/design/external-dtos-contract.md`. The coordinator supplied public
API declaration facts, trait sets, Serde names, feature gates and protocol facts,
without algorithm bodies or expressive documentation. Previously admitted
policy inputs are `AGENTS.md`, `TOKO-TOOLING.md`, and
`docs/principles/rust-code-hygiene.md`.

The implementer did not read the original four modules, their tests, codec
mappings, DID fixtures, supplier source, crypto or callback child modules,
Git history or diffs. Review was confined to the contract and newly authored
source. Earlier specification-only tasks remain in the implementer's context;
this record does not claim erased context or model training exposure, legal
clearance, or mathematically proven independence.

## Authored drafts

Four complete, unreferenced drafts were written at:

- `crates/ic-platform/src/cmc_types/replacement/mod.rs`
- `crates/ic-platform/src/icp_ledger_types/replacement/mod.rs`
- `crates/ic-platform/src/cycles_ledger_types/replacement/mod.rs`
- `crates/ic-platform/src/certified_assets_types/replacement/mod.rs`

The drafts contain 33 passive struct/enum declarations and six aliases. They
preserve declaration field and variant order, exact trait sets, canonical
Principal and unbounded Nat fields, and the three ICRC-feature-gated cycles
records. Every byte vector retains ordinary sequence Serde. No Serialize,
Default, Copy, ordering or Hash implementation was added. The drafts add no
validators, normalization, limits, network operations, cryptographic algorithms,
storage behavior, timer behavior or application policy. All public declaration,
field and variant documentation was written afresh.

The ICP draft retains private `account_identifier` and `callback` declarations
and explicit exports of `account_identifier` and `QueryEncodedBlocksFn`. Neither
child implementation was read, copied or modified. Codec mapping macros are
intentionally absent: the coordinator owns appending the existing owned tails
unchanged when installing the new declarations.

## Validation and handoff

A process-name check found no active Cargo or rustc process before drafting.
Scoped source counts confirmed 33 struct/enum declarations, six aliases and
three ICRC-gated declarations. The four authored drafts were formatted
successfully with:

```text
rustfmt +1.99.1 --edition 2024 --config-path /dev/null --config skip_children=true crates/ic-platform/src/cmc_types/replacement/mod.rs crates/ic-platform/src/icp_ledger_types/replacement/mod.rs crates/ic-platform/src/cycles_ledger_types/replacement/mod.rs crates/ic-platform/src/certified_assets_types/replacement/mod.rs
```

No Cargo command, compilation, Clippy, tests or runtime measurement was performed
by this implementer. No duplicate private tests were added. The drafts were
frozen and handed to the coordinator. The compiled module files were not read
or edited by this implementer.

The coordinator subsequently reported a successful original-implementation
baseline of 17 existing tests and four independent public tests. It installed
each draft by concatenating it with that module's unchanged original owned codec
tail, then removed the empty `replacement` directories. The coordinator also
reported exact old/new DID equality across 14 methods. These are attributed
coordinator results, not checks performed by this implementer.

At this record update, the coordinator is running replacement qualification;
no further after-installation test result has been supplied. Independent
observations, feature/target qualification and source retirement remain
coordinator responsibilities. This update changed only this record, without
source reads or Cargo commands.

The coordinator subsequently completed replacement qualification and source
retirement; results are recorded in
[the qualification report](external-dtos-qualification.md). This parent-added
link does not expand the implementation author's input set.
