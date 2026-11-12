# Current handoff — 2026-11-12

## Reviewed deployment batch

The maintainer authorized delivery of the reviewed local batch. Select patch
releases in supplier order: Types 0.19.5, Runtime/macros/guard 0.21.3 and Platform
0.31.3. All five public package manifests inherit the changed exact Toko
dev-dependency; their normalized published requirements change. Platform also
adds the native certificate/witness authentication API. No browser/npm or sibling
release is selected. Pending release notes cover the complete batch.

The native API requires `agent,certification-cbor`, delegates certificate,
delegation and freshness authentication to the caller-configured Agent, and
checks the exact canister's certified-data witness commitment. It adds no dependency
or Wasm feature edge. Focused Clippy and tests pass with and without `bls`, including
signed evidence, unsigned/tampered/stale refusal, byte budgets and trailing CBOR.
The advisory adapter's refusal tests and ShellCheck pass. Registry review confirms
the adopted Toko/Testkit/Host versions are current; optional Wire 0.21.3 and editor
0.83 migrations remain explicitly deferred in the Platform ledger.

Initial batch `batch-VCzP0I1E` stopped before preparation, delivery or upload:
the release-routing fixture omitted the newly included dependency-maintenance
Make entrypoint. The corrected fixture selects that entrypoint and its declared
companions. Preserve its `qualification.YY6dQL` / `validation.MNl4ew` evidence;
do not resume its obsolete source. A fresh batch will run the complete local gate
for each group; focused tests do not replace that qualification or macOS evidence.

## Published state

Frozen batch `batch-CqgSHOoJ` completed and was reconciled into ordinary main.
All five packages are independently verified present and not yanked on crates.io;
annotated GitHub tags dereference to the matching release commits.

| Group | Version | Commit | Tag | Local receipt |
| --- | --- | --- | --- | --- |
| Types | 0.19.4 | `3c0633613e5df26f9b7e646814eb5a3d809a419c` | `types-v0.19.4` | `qualification.OLqmeh` |
| Runtime, macros, guard | 0.21.2 | `dc7ee066fd12551a95c4f7e8e6339f73affde983` | `runtime-v0.21.2` | `qualification.pvNjpf` |
| Platform | 0.31.2 | `8b4ab73fa8fc7bdd44b32594d33b572d93b7469a` | `platform-v0.31.2` | `qualification.BvBpZQ` |

Each group passed all 19 mandatory local stages, including Clippy, product tests,
consumers, feature graphs, Wasm builds and installed Runtime qualification.
Publication verified each archive. Platform requires Types `^0.19.4` and Runtime
`^0.21.2`; Runtime's guard matches exactly. Browser remains 0.2.0.
No sibling or npm release was included. Toko 0.39.15 is installed; compiler is 1.99.1.

Source batch: `e63679a`, corrected by `0d396cf`. Normal log:
`target/deploy-testkit-custody-corrected.log`; runners and receipts remain under
`target/release-frozen/batch-CqgSHOoJ/target/release-validation/`.
The independent qualification lock now follows local Platform 0.31.2 and Runtime
0.21.2 automatically; that release's registry baseline was exactly 0.30.0. Locked offline
metadata passes after reconciliation without another lock edit.

## Delivered cleanup

Pending local adoption selects Toko 0.39.15 in both workspaces. The authenticated
exporter preserves prior entrypoints and adds canonical `make/cargo-dependencies.mk`
and its Perl implementation (98 managed files). `make deps-update` is wired into
Make/help/README; package/precise selection scopes normal sequential Cargo updates.
The Toko-only invocation covered both workspaces without further lock changes.
Snapshot, pin, inventory, documentation and formatting checks pass. This adoption
and the #50 CI fix remain uncommitted/unpublished; no sibling changes were made.

The published batch selected Testkit 0.100.2 in both catalogs and the maintained
Runtime lock. Drivers use
its upstream async PocketIC re-export; old ClientError conversions and the old
Platform 0.24 dependency path are gone. Native dependencies increase; this is
maintenance cleanup, not a production Wasm saving. Upstream transport panics and
best-effort deletion apply; startup custody stays with Testkit. Application
Principal and protocol bytes remain unchanged.

Installed raw/prepared × Wire/external-Candid, raw_rand cleanup/cancellation,
Agent identity/wrong-root proofs and all four benchmark driver builds passed
before the complete release gates. Native identity admission now traces Testkit →
PocketIC → Candid and refuses a different Principal supplier.

Release transactions include the independent benchmark lock, preserving the
registry baseline/foreign pins while advancing local owners. All three real
metadata/rollback fixtures pass. Hosted Runtime now prepares ic-wasm.
Toko's authenticated update preserves all 96 managed files and entrypoints and adds
explicit supplier-freshness reviews. No snapshot was hand-edited.

That release's freshness review recorded Testkit 0.100.2, Toko 0.39.12 and
Host helpers 0.76.4. Optional Wire 0.10/0.12.1 → 0.21.2 fixture migration is deferred
to the next qualification-dependency maintenance batch, and editor 0.73 → 0.83 to
the next release-adapter maintenance batch. Revisit both before the next freeze;
the group ledgers record these dispositions, not permanent exemptions.

## Pending qualification dependency adoption

Both catalogs and the maintained Runtime lock now select published Testkit
0.101.0. Its native snapshot helpers add registry Platform 0.31.2 with only
`candid-codec`, `management-canister-types` and `principal`; no Runtime or crypto
features are selected there. Identity admission records that infrastructure
separately from the local product under qualification. The owned canister graphs
remain unchanged. At the maintainer's request, qualification now selects the latest
published Platform 0.31.2 as its exact BLS baseline; both arms use prepared keys.
Historical 0.30.0 measurements remain historical evidence, not the current selection.

Preserve the maintainer's compatible independent-qualification lock refresh.
`crypto-common` 0.1.7 requires exactly `generic-array` 0.14.7, explaining Cargo's
downgrade from 0.14.9. PocketIC 16.1.0 requires exactly `thiserror` 2.0.18 and
prevents 2.0.21 in that native graph; the main product lock already uses 2.0.21.
No cryptographic implementation or direct production dependency changed.

Focused Linux checks pass: measurement-caller Clippy, four benchmark caller
Clippy/builds, four installed raw/prepared × Wire/external-Candid Runtime
profiles, and Agent queries/update/read-state/wrong-root checks across Ed25519,
delegated, P-256 and secp256k1 identities. Snapshot, pins, inventory, docs,
formatting and identity-admission tests pass. Runtime evidence:
`target/cdk-runtime.vVZ9QC/`; Agent: `target/agent-installed.4o8LFB/`.
Logs: `/tmp/platform-testkit101-{clippy,checks,identities,runtime,agent,drivers}.log`.
This adoption remains local and uncommitted; no sibling changes, publication,
full CI matrix or fresh macOS qualification were performed.

The latest-published baseline refresh also passes the focused BLS runner, including
both Wasm Clippy selections, native driver Clippy, installed verification/refusal
cases and source-custody checks. Evidence: `target/evidence/bls-wrapper.KkxrFD/`.
Published/local arms both measure 246,052 raw total / 238,906 raw code bytes and
237,357 optimized total / 230,205 optimized code bytes. The registry selection
is exactly 0.31.2; no Platform 0.30.0 package remains in qualification's lock.
The pinning exception now documents this reproducible latest-adopted comparison.

## Hosted evidence and limits

At handoff, the final release's [native CI](https://github.com/dragginzgame/ic-platform/actions/runs/43085107761),
[installed Runtime](https://github.com/dragginzgame/ic-platform/actions/runs/43085107808),
[Wasm comparison](https://github.com/dragginzgame/ic-platform/actions/runs/43085107849),
[browser](https://github.com/dragginzgame/ic-platform/actions/runs/43085107637) and
[advisories](https://github.com/dragginzgame/ic-platform/actions/runs/43085107561)
were running. Linux passes do not establish fresh native macOS or real browser
acceptance. This handoff stays local to avoid cancelling those workflow/ref runs
with a documentation-only push.

#50 remains unresolved. On 2026-11-11 the maintainer approved its known workspace
finding as expected/non-blocking in CI. The local adapter keeps the raw finding
and status, then excludes only that advisory from the workspace gate. Runtime,
other advisories and incomplete/tool failures remain blocking; no global ignore
or claim of safety is added. Revisit at the next Agent dependency review.
Focused tests/ShellCheck pass. Real online cargo-audit 0.22.2 evidence at
`target/expected-advisory-live.vi6VFr/report/` has workspace raw/gate 1/0,
runtime 0/0 and complete index coverage. This CI fix is local and uncommitted.
The prior hosted group-preparation failure and missing ic-wasm
have local corrections; fresh hosted results remain independent proof.
The earlier verification-size finding remains unresolved: optimized total +135 /
code +133 bytes against `fc55ae1`, exceeding the unchanged 128-byte allowance.
Follow-up experiments were rejected; verifier source is unchanged. See the
[audit](../benchmarks/platform-031-audit.md#post-release-cleanup) and existing
[BLS qualification](../design/bls-wrapper/qualification.md). No certificate,
delegation, identity-key, witness or freshness check was removed.

## Recovery custody

Retain completed `batch-CqgSHOoJ`, its journals, candidate Git, archives and proofs.
Preserve failed `batch-A6SG2UhN` / `qualification.J7lzye`: a stale projection fixture
stopped Types qualification before preparation/upload. `0d396cf` fixed its required
locks and rejection cases; never resume the failed source.
Preserve failed `batch-uCPG6Jvb` / `qualification.5kqUXx` and all earlier batches,
lock refs, bundles and receipts recorded in the
[previous handoff](https://github.com/dragginzgame/ic-platform/blob/0d396cf01d5e33d83f10c34be9c37bf4539fe5b4/docs/status/current.md).
Durable batch records: `.git/release-state/frozen/`; candidates/evidence:
`target/release-frozen/`. Do not resume old batches over published versions.
Deployment authority includes no artifact retirement or deletion.

Keep `recovery-evidence/retired-checkouts/20261104-issue139/`; maintainer review is
due 2026-12-04. No deletion, mtime renewal or budget increase is authorized.
Keep `recovery-evidence/deferred-signature-comparison-20261110.rs`, SHA-256
`9fdee2e8b41c9b13e0d2a3127ddf95559a1094fa48d14dd17b7577d4ec86ed4a`.
Previously removed artifacts must not be claimed present.
