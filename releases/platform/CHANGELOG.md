# Platform changelog

This group covers ic-platform, ic-platform-macros and ic-platform-runtime-guard.
Earlier coordinated history is retained once in [history](../history.md).
See also the [Types ledger](../types/CHANGELOG.md).

## [0.15.2] - 2026-10-31

### 🩹 Fixed

- Adopt symmetric crate groups with explicit package versions and group selection
  for release, resume and publication. Preserve numeric versions, dependency
  selections, transactional runtime metadata and recorded release evidence.

### 🥾 Governance

- Adopt Shared Tooling 0.27.19, including symmetric group ownership and delegated
  release/publication workflows, and move canonical group ledgers under releases/.
  Root Cargo has no shared version; root changelog is an index.

## [0.15.1] - 2026-10-31

### 🩹 Fixed

- Restore independent Platform/Types release groups in one workspace, starting
  from existing 0.15.0 identities. Preserve Types during runtime-only releases,
  qualify transactional runtime metadata custody and select publication by group.
  [#118 — Independent identity releases](https://github.com/dragginzgame/ic-platform/issues/118).

### ⬆️ Dependencies

- Adopt Shared Tooling release commit 0.27.17 (`39ba983`), including
  common metadata custody, Cargo group comments and canonical-ledger policy,
  through the snapshot exporter.
