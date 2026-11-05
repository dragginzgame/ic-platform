# Shared 0.30 adoption and Platform hygiene audit

The local maintenance batch starts at clean Platform `b22813c`. It initially adopted
Shared 0.30.0, committed supplier
`08cd17240fd60dfd309e35eb131e73323e883e6f`, through the canonical exporter,
including the LOC contract referenced by the Rust hygiene guide. Supplier and
sibling source checkouts remain unchanged. No package version, dependency
selection, release, tag or publication changes. Pending notes belong to Platform
0.22.2 and Runtime 0.19.2; lint-only Types edits need no separate release note.

Method: Shared’s [code hygiene audit](../../audits/code-hygiene.md) and
[baseline](../../DRAGGINZGAME.md), with the local [overlay](../../AGENTS.md).
This is inspection plus authorized local repairs and focused Linux execution,
not release, cryptographic, vulnerability-freshness or native-host acceptance.
Evidence is retained under `target/shared030-audit/`; explicit Cargo cleanup can
reclaim those diagnostic outputs. The historical recovery archive is separate.

## Subsequent exported-document correction

The same-day documentation repair adopts committed Shared
`4d0961821a31766e3044c08fbe5954ed81213ec9` through the exporter, retaining version 0.30.0.
The producer’s LOC references now use immutable upstream links; explicit export
selection supplies `audits/ci-release-integrity.md` and
`scripts/distribution/governance-files.txt`. The now-unneeded local fleet guide
is retired by the exporter. Current integrity covers 136 supplier payloads.

The before scan found three missing targets. All 57 exported Markdown documents
now pass local target checking. Platform’s actual Make gate also includes its
20 local documents, and a regression mutates only a disposable exported guide
to prove missing targets fail. This reuses Shared’s checker and validated snapshot
records; no new Markdown parser or snapshot edits. The regression belongs to the
normal test target. Evidence is in `target/shared-doc-links-20261105/`, including
before/after results, export impact, source integrity and LOC caller smoke proof.
Earlier Rust/native evidence below applies to unchanged product code; no Rust
compilation or broad matrix is required for this documentation repair. Shared’s
concurrent uncommitted work and all sibling checkouts remain untouched.

## Rule coverage and resulting changes

| Rule family | Audit result |
| --- | --- |
| Adoption and provenance | Exported 135 supplier-bound payloads, including the classifier, retention checker and linked LOC guide. Integrity is checked against the independently selected commit. No snapshot payload was edited locally. |
| Workspace, release groups and compiler | Keep the virtual root, five members and three groups. Rust minimum/development/release remains 1.99.1. Correct root dependency-role headings from actual member/fixture use without changing requirements, lockfiles or feature selections. |
| Directory/module/test roles | Main members have no singular `test/` or `test.rs` paths and no path/include module wiring. Add explicit roles to generated/template test and benchmark manifests, including macro consumers. Move 20 native provider/overlay inline test modules into ordinary `tests.rs`/`*_tests.rs` files. Native path-module debt remains tracked in #140 below. |
| Lint expectations | Main crates go from 22 allowance attributes to one. Remove two obsolete allowances; convert the other 19 applicable attributes to expectations, splitting one grouped scope. The generated internal spawn retains a reasoned allowance because downstream disallowed-method lists differ. Keep unfulfilled expectations enabled and qualify native/Wasm configurations. |
| Imported bodies and licenses | Preserve all existing Principal, IC0/executor, callback and ICRC fingerprints. Guards reconstruct only the exact new lint scaffolding; mutation checks still refuse body/ABI/vector/license and altered-normalization inputs. Native assembly pins include moved tests and exact patch inputs. No cryptographic body, check or test suite is adopted or altered. |
| Errors and standard library | No `core::` imports in the std-based main crates. Derive the mechanical stable-memory error using Runtime’s existing thiserror, with unchanged variant, display and source. Preserve passive display types, test error adapters, upstream-owned hash/display bodies and facade errors rather than adding an unrelated dependency. |
| Source capture and fixtures | Replace Rust-text patching in the Canic benchmark with an ordinary fixture calling the public Host build API and explicit admitted tools. Macro linking tests also copy ordinary Rust fixtures. Preserve historical captures. The actual capture now rejects line-break paths and source symlinks, with a focused positive/negative regression in the normal test target. |
| Inputs and native operations | Root crates contain no native process owner or raw libc wrapper; macro compiler output is diagnostic. Native provider OS/signal adapters remain outside the root graph; preserve their clock origin/ABI and paired acceptance rather than substituting a different observation contract. No new process/filesystem engine. |
| Dependencies and product ownership | Pin/inheritance, external IC boundary and inventory checks pass. Keep canonical Types/Runtime/facade identities, empty defaults, explicit external Candid, consumer budgets, Wire tooling-only ownership, upstream cryptography, Tooling gzip and browser/native-client boundaries. No new wrappers, workspace, abstraction or algorithm. |
| Make, hooks and validation | Keep the installed shared formatting/tool mechanics. Add ordinary fixture sources to the actual formatting roster. Focused routing covers three groups, publication subsets and retention refusal; consumer/feature checks retain fail-closed behavior. Do not activate unqualified selected-stage reuse or replay full release/fleet matrices. |
| Scratch, delivery and retention | New local proof scratch uses the owning target. Keep supplier-owned helper contracts intact. Apply the specifically approved historical custody moves, preserving every byte and all Git refs. Shared supplies admission; no cleanup engine or automatic deletion is introduced. |
| Reporting and release notes | Update local handoff, qualification guide and numbered pending notes. Keep the published ledgers and prior release evidence. The native limitation has an owning GitHub issue rather than a second local task queue. |

The new schema-2 root report separates library/application, tests, examples,
benchmarks, build/tooling and unclassified source. The main workspace has zero
unclassified lines; its macro notes explicitly identify unexpanded source bodies.
The actual generated native HTTP proof classifies entirely as tests, with zero
library or unclassified lines. Independent source-provider and diagnostic graphs
are not silently included in the root report. No Wasm-size or runtime-performance
claim follows from LOC or lint changes.

## Approved retention custody

Before relocation, the inspected Git-side roots contained 678,399,027 logical
bytes. Shared refused two old Runtime driver binaries, the retirement archive
and the aggregate budget. With explicit maintainer approval:

- Move the complete 25-file retirement directory to
  `recovery-evidence/retired-checkouts/20261104-issue139/`. Verify its entire
  before/after file inventory and archive/bundle checksums. All original receipts,
  source, bundles and refs remain; the directory now includes its relocation
  receipt. It contains 278,283,846 logical bytes at observation.
- Move only the two reproducible Runtime 0.18.1 driver executables, totaling
  351,664,168 bytes, to `target/recovery-evidence/runtime-0181-delivery.LgU8r6/`.
  Verify each checksum and retain logs, original checksums and relocation receipts
  at the old Git-side paths. They are now reclaimable by ordinary Cargo cleanup.

The inspected Git-side roots now contain 48,460,373 logical bytes and pass
Shared’s default admission. No Git pruning, archive recompression, source deletion
or mtime renewal occurred. The durable archive survives Cargo cleanup. Platform’s
maintainer owns review by 2026-12-04, before its original bulk mtimes expire.
`make check-retained-evidence` applies Shared’s checker with the explicit Makefile
budget (320 MiB aggregate, 256 MiB/file, 30 days) before release preflight and
publication. Expiry/overflow refuses without deleting state. This scoped approved
archive budget does not raise Shared’s Git-side limits or authorize new archives.

## Focused evidence

- `snapshot-check.log` / `adoption-doc-closure.log`: exact supplier integrity and
  exporter custody; `declaration-checks.log`: pins/inheritance, IC ownership and
  inventory. Compiler identity is admitted separately by the existing checker.
- `types-clippy.log`, `runtime-clippy.log`, `runtime-wasm-clippy.log`,
  `facade-clippy.log`, `facade-wasm-clippy.log`, `macros-clippy.log`: affected
  graphs, Rust 1.99.1 and `-D warnings`, including unfulfilled expectations.
- `types-tests.log`: 18 unit tests plus one doctest; `runtime-tests.log`: 20
  unit tests; `macros-tests.log`: 16 tests including actual multi-crate linking;
  `dto-tests.log`: 12 selected ICRC/management/identity consumer tests.
- Original source guards and their negative mutation tests pass. Exact lint
  reconstruction changes cannot conceal altered ABI, body, vector or license.
- `feature-selections.log`, `feature-closures.log`, `feature-graphs.log`:
  actual closure contracts and 138 batch admission cases, including printer,
  management, retired PocketIC and native-only exclusion failures.
- `release-routing.log`, `consumer-admission.log`, `prepared-admission.log`:
  actual consumer wiring with controlled fixtures. Oversized/expired recovery
  evidence refuses release/publication; substitutes do not qualify a release.
- `formatting-integration.log` and final format/doc checks: owning Make/hook
  policy and standalone source coverage, without changing the real index.
- `native-http.log`: strict Clippy before four actual admission tests;
  `native-assembler-tests.log`: 52 existing assembler/custody tests;
  `native-manifest-final.log`: exact current source/overlay/patch inventory.
  `native-test-body-parity.log`: all 20 relocated modules match the formatted
  original test bodies, including their vectors and assertions.
- `benchmark-driver-clippy.log`: ordinary driver compiled through frozen Canic
  Host 0.115’s public API. Its disposable lock was resolved offline before the
  successful locked check. The earlier locked refusal is not a pass.
  `benchmark-capture-tests.log` covers unchanged input bytes and invalid path/link
  refusals. No matched Wasm matrix was rerun for this maintenance.
- `cloc-final.json`, `cloc-http-fixture.json`: actual source-role observations;
  `retention-sizes.json`, `retired-archive-checksums.log`,
  `retained-state-after.log`, `retained-evidence-check.log`: custody admission.

Initial lint probes exposed obsolete suppressions and the missing-doc warnings
that still need scoped expectations; those probes were corrected and rerun.
They are not presented as successes. Full CI, installed Runtime/Timers, Auth,
Chromium, macOS and the full native IC build were not run for this hygiene batch.
Prior release receipts qualify their original sources only.

## Release qualification correction

The first Runtime release gate from source `95baef5` stopped at full Clippy,
before version preparation or publication. The earlier library-only probes missed
the private native management schema-test configuration: acronym warnings apply
there, while generated missing-doc warnings apply to public Wasm exports. Scope
those expectations with `cfg_attr`; preserve the complete DTO declarations,
external bytes and original reconstructed fingerprint. Focused native/Wasm
all-target checks and source-mutation refusals cover the corrected configurations.
Failed gate evidence remains at
`target/release-validation/qualification.B8Ovtd/`; it is not a passing receipt.

The second release gate at `e9a6419` passed Clippy and browser checks but exposed
two consumer fixture copies missing newly declared Shared companions. Use the
canonical companion-aware copier for CI selection and release recovery. The
affected fixtures, three-group preparation/rollback/retry/local delivery/resume,
publication routing, editor bootstrap and changelog checks pass. The failed gate
at `qualification.hLObj9` remains retained; individual correction evidence is
under `target/platform0222-delivery/`. No production stage or refusal was removed.

## Native provider qualification boundary

[Platform #140](https://github.com/dragginzgame/ic-platform/issues/140) owns the
remaining 18 provider path attributes and three patch module mappings, additional
patch-only test layout and paired native acceptance. The main workspace is free
of that wiring; the native source profile is not yet fully compliant. Its consumer
supplies product/rendering/viewer/system sources, and the independent system crate
has Cargo/Bazel paths tied to the existing assembly layout. Moving the entire
provider destination alone breaks those inputs; no partial move is retained.

The 20 moved test modules and five native argument-count expectations retain
behavior but still need the actual primary native-server Clippy and installed
qualification before a new provider pin is adopted. Standalone HTTP/assembler
passes are explicitly narrower. No sibling source, source pin or release changed.
