# Platform documentation

## Current contracts

- [Dependency and ownership inventory](architecture/dependency-inventory.md):
  package owners, feature boundaries, remaining imports and checked manifest facts.
- [Migration contracts](architecture/migration.md): public compatibility rules,
  completed hard cuts and pending source replacements.
- [External DTO contracts](protocols/external-contracts.md): selected binary interfaces.
- [Current handoff](status/current.md): local work, blockers and recovery custody.

## Development and delivery

- [Development and qualification](development/qualification.md): focused commands,
  generated fixtures and tool preparation.
- [Release validation](development/release-validation.md): the three Rust groups,
  required local checks and hosted evidence boundaries.
- Group ledgers: [Platform](../releases/platform/CHANGELOG.md),
  [Runtime](../releases/runtime/CHANGELOG.md), [Types](../releases/types/CHANGELOG.md).
- [Browser SDK](../packages/browser/README.md): separate npm packaging and checks.

## Design and qualification

Implementation contracts and source-separation records live under `design/`.
The corresponding qualification reports are the evidence entrypoints:

- [Identities](design/identity-qualification.md).
- [Executor](design/executor-qualification.md),
  [System API/task adapters](design/system-adapters-qualification.md),
  [calls](design/call-transport-qualification.md),
  [entrypoint generator](design/entrypoint-generator-qualification.md).
- [ICRC adapters](design/icrc-adapters-qualification.md),
  [ICRC DTOs](design/icrc-dtos-qualification.md),
  [native management DTOs](design/management-native-qualification.md),
  [shared management DTOs](design/management-shared-qualification.md).
- [External CMC/ICP/Cycles/assets DTOs and schemas](design/external-dtos-qualification.md).
- [Value helpers](design/icrc-value-qualification.md) and
  [ledger hash/account composition](design/ledger-composition-qualification.md).
- [Candid codec comparison matrix](benchmarks/candid-codec-2026-11-10/matrix.md)
  and [canister-signature measurements](benchmarks/canister-signatures-2026-11-10.md).
- [Toko 0.39.6 adoption](development/toko-0396-adoption.md).

`audits/` contains dated technical/source inventories. `benchmarks/` retains
methods and measurements. `history/` preserves superseded contracts and handoffs;
historical selections do not describe the current graph.

## Shared engineering references

[TOKO-TOOLING.md](../TOKO-TOOLING.md) is the installed baseline;
[AGENTS.md](../AGENTS.md) contains Platform's local safeguards. Supporting shared
guides stay upstream at immutable revisions, including the adopted
[installation guide](https://github.com/dragginzgame/toko-tooling/blob/5806e6e4943f6be600cd53385498ce08a5f804c4/docs/adoption/consuming-snapshots.md).
The receipt owns installed executable bytes; do not copy reference guides back
into this documentation tree.
