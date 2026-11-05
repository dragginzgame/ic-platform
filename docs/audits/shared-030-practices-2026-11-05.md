# Shared 0.30 maintenance audit

## Current local adoption

Platform adopts published Shared **0.30.5**, supplier
`be3771031ebfb340583b238dee5446be65929c62`, through the canonical exporter.
It includes the explicitly approved inherent-implementation checker correction,
now committed on remote main. The original focused adoption used `a08f03a`;
all 138 payloads are reverified against the published supplier for delivery.
Platform 0.23.2 delivers the management request and qualified native layout;
the interrupted local 0.23.1 attempt remains recorded in the group ledger.
The unrelated incoming Candid/derive 0.10.40 lock draft retains separate custody.

The preceding [0.30 audit](https://github.com/dragginzgame/ic-platform/blob/bd4a6a9/docs/audits/shared-030-practices-2026-11-05.md)
retains earlier source/lint, supplier and qualification detail in Git. This is
one current maintenance summary, not an archive of validation runs.

## Applicable rules and repairs

- The Candid source checker is part of `make check-boundary`. Its initial 152 exact
  allowances name external management, ledger, CMC, registry and asset contracts
  or explicit codec qualification fixtures across 217 Rust files. No whole file
  or module is exempted. The consumer fixture admits actual asset DTOs and refuses
  a new application declaration or stale allowance.
- Shared now admits a complete named generic inherent implementation header,
  required for the existing ICRC archive callback. The next token must be its
  body brace; partial trait-name prefixes and changed bounds refuse. Its 39
  focused producer cases and copied lexer closure pass.
- The source check is lexical, not macro expansion or Rust type resolution.
  Imported source/license guards and external schema comparisons retain their
  separate obligations. No cryptographic, DTO, Serde or protocol body changed.
- One benchmark fixture uses normal trait-method dispatch inside its existing
  external Candid implementation. Its Candid-sequence Wasm Clippy selection
  passes on Rust 1.99.1. No benchmark was rerun and no size/performance gain is claimed.
- New benchmark output defaults to `target/evidence/`. The shared formatter/LOC
  scratch fixes are adopted; the actual root LOC report has zero unclassified
  lines. Report diagnostics still identify unexpanded macros/conditional attributes.
- Twenty-five obsolete experiment tables, source snapshots and prototype probes
  are retired. Compact historical conclusions replace them, references are repaired,
  and missing raw artifacts are not described as independently available.
  Git history remains; no replacement permanent archive was manufactured.

## Focused validation

Current logs are under `target/shared0304-adoption/`, reclaimable by Cargo clean:

- `snapshot-check.log`: 138 payloads match the exact committed supplier.
- `boundary.log`: current Cargo/Candid ownership plus real consumer refusal cases.
- `formatting-integration.log`, `format-check.log`: actual Make/hook policy,
  standalone formatting, failure-stop and staged-index isolation.
- `fixture-clippy.log`: strict Candid-sequence benchmark fixture on Wasm.
- `shellcheck.log`: selected changed shell entrypoints; syntax checks also pass.
- `cloc.json`: current source roles, with no unclassified root-workspace source.
- `doc-links.log`: current local references, including the complete Shared export.

No full workspace/fleet/release matrix or new native macOS run was authorized.
Root checks do not qualify the IC provider. Subsequent
[#140](https://github.com/dragginzgame/ic-platform/issues/140) implementation and
native evidence are recorded in the separate
[paired qualification](native-profiler-140-2026-11-05.md).
The [crypto advisory](https://github.com/dragginzgame/ic-platform/issues/50)
is unchanged and has no waiver.

## Protected evidence custody

The approved historical archive remains at
`recovery-evidence/retired-checkouts/20261104-issue139/`; all original bundles,
checksums, receipts and refs remain. The former Runtime 0.18.1 driver executables
remain under `target/recovery-evidence/runtime-0181-delivery.LgU8r6/`. No active
release/recovery evidence or maintained test vectors were deleted. The Platform
maintainer owns review by **2026-12-04**. The existing bounded retention checks
refuse without automatic deletion, mtime renewal or limit inflation.
