# Toko Tooling 0.39.6 adoption

The root catalog selects exact `toko-tooling = "=0.39.6"`, inherited as a
development dependency by all five product members. Cargo updated only that
package's version/checksum from the saved pre-adoption lock; existing unrelated
lock selections remain. `cargo fetch --locked` completed. No product version,
release group, sibling checkout or published package changed.

The published package was located through Cargo metadata. Its installer fetched
and authenticated crates.io payloads, previewed the complete retained entrypoint
selection, then applied it. Supplier Git source identity is
`5806e6e4943f6be600cd53385498ce08a5f804c4`; archive SHA-256 is
`00fcdf9fa866007e0d8e802f711257cd23e3081a467daf69225b3844f91dbc1b`.
The snapshot now contains 96 files instead of 160. Its three Markdown payloads
are the root baseline and canonical root files. Shared reference guides retired
through the exporter; consumer links point to immutable upstream paths.

Required consumer-only callers were checked before retirement. In particular,
`validation-evidence.pl` and `fixture-lifecycle.sh` are explicit roots because
Platform calls them directly. The unused shared changelog-parser test is no
longer selected. `TOKO_TOOLING_RELEASE := 1` explicitly retains the existing
three-group release workflow, with tooling admission ahead of effects.

## Product documentation

Seven native guides moved into architecture, protocols, development and history.
The [docs index](../README.md) separates current contracts, qualification and
historical evidence. The dependency inventory now describes the actual owned
codec, registry IC0, independent implementations and native-only clients; its
older narrative remains a dated historical document. All checked manifest facts
were regenerated from Cargo metadata. The handoff remains 100 lines.

Inventory, CI-selection, pin-evidence and Make documentation callers follow the
new paths. The documentation gate checks all nested native docs and the small
installed baseline. Shared document copies are not restored as local policy.
Historical benchmark input manifests retain their original recorded source paths.

Rust members inherit the sole root license-file. Browser builds generate their
archive copy under ignored `dist/`; package checks compare it to the root.
Retained third-party source keeps its notices, source records and guards while
the separately authorized replacement work proceeds.

## Focused validation

- `make toko-tooling-check`: exact member selection, lock and all installed bytes.
- `make tools-check`: existing host/Rust/browser tool selections admitted;
  no compiler or application dependency upgrade was needed.
- `make check-boundary`: Runtime's registry IC0 edge is now admitted by the
  installed supplier. Three new DTO comparison targets have narrow test-only
  CBOR allowances; production CBOR rules remain enforced.
- Inventory and its mutation tests; CI-selection tests; pin checks.
- Documentation links: 386 local references across 91 documents, including
  deliberate missing-link refusal for both a nested native guide and the baseline.
- Actual three-group release/publication routing, retained-state refusals and
  validation-plan/recovery fixture checks. Stage bodies in the latter were
  substitutes; these are not real release qualification.
- All five Rust package listings include `LICENSE.md` using an ordinary
  disposable source-copy fixture. Direct listing in the live dirty checkout is
  obstructed by the already removed native-provider Gitlink; no Git retirement
  was staged to conceal that condition.
- Offline npm packing includes `dist/LICENSE.md` with bytes identical to root.
  Changed Node scripts pass syntax checks; no browser API behavior changed.
- Shared management and ICRC value/hash reconstruction guards still pass.
- Changed shell/Node syntax, ShellCheck and scoped diff-whitespace checks pass.
  The unrelated ICP draft still has its pre-existing final blank-line warning.

Evidence: `target/tooling-0396/`, `/tmp/platform-toko-*.log`. No full Rust suite,
installed IC run, native macOS or browser-runtime execution was required by this
tooling/documentation batch. No delivery was performed. The separate ICP
derivation draft remains unresolved; current scope is in the
[handoff](../status/current.md).

The moved migration guide has an intent-to-add index entry solely because the
pin checker requires tracked evidence. No file contents were staged.

## Follow-up: 0.39.7

The requested patch adoption selects published 0.39.7 at supplier source
`fcbbcd147678fdfe5ab22c36710e017fb2c018e4`, archive SHA-256
`fd795f6ef878496ac27a990a9208b87fa786a9148c18580338ef8ac79df501d7`.
Only Toko changed in the lock. The released installer retained all 96 payload
paths; changes are the baseline and pinned-document diagnostics in the two
release entrypoints. Canonical root file bytes and tool versions are unchanged.
The previous release-routing fixture evidence remains applicable; no release
command was executed for this patch.

All five Rust members still inherit the root license-file, and browser package
metadata selects its generated archive copy. The root files match 0.39.7's
canonical bytes. Seven retained third-party areas and four DID fixtures remain
explicit in the [source inventory](../architecture/dependency-inventory.md#source-maintenance).
The ICP source checker still reports `missing upstream ICP constructor` for the
separate existing draft; this adoption neither changes it nor removes its records.
Tooling, pins, inventory, links and dependency boundaries pass. Evidence is under
`target/tooling-0397/` and `/tmp/platform-toko-0397-*.log`.

## Follow-up: 0.39.8

The requested refresh selects published 0.39.8 at supplier source
`e3053d30870ae9b542519996daa1be9dbdf80eab`, archive SHA-256
`baf4177553a6a818837ae9765afc021ac3e3c0722e4622f773a990dc4b41cbc9`.
Cargo changed only Toko's version/checksum against the pre-adoption lock;
`cargo fetch --locked` completed. The released installer previewed and retained
all 96 payload paths, then installed and admitted the selected snapshot/tools.
Executable payloads, baseline rules and compiler selections are unchanged.
The pin exception and generated dependency inventory match the new selection.

Tooling admission, pins, inventory, documentation links and dependency boundaries
pass. Browser qualification passes 23 tests, both supported TypeScript consumer
compilers, packed exports, the root-file packaging comparison and provider SDK
identity checks. No real Chromium or native macOS execution was performed.
ICRC source admission passes. The pre-existing ICP derivation draft still fails
its source guard (`missing upstream ICP constructor`) and has a trailing blank
line reported by `git diff --check`; neither was altered by this adoption.
These prevent claiming whole-batch push readiness. No delivery was performed.
Evidence: `target/tooling-0398/`, `/tmp/platform-toko-0398-*.log`.
