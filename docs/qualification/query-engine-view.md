# Query EngineView integration experiment

## Scope and source custody

On 2026-11-15, test the actual Query EngineView collector and projection against
the candidate Protocol package in a disposable snapshot. Query source is pinned
to `263a8e92da873543825c25e7dd6530c4db5ab673`; candidate source is Platform rebuild
`08c72219ad6f261067c00eece9c8c71852e6b561`. The source checkout `../ic-query` has
unrelated uncommitted work, so `git archive` of the pinned commit supplied the
experiment. No sibling files, refs or lockfiles were modified, and no Query
source/tests were imported into this repository.

The retained local snapshot is `/tmp/ic-platform-query-probe.IVRwAg`. It contains
the adapted files, original lock, build output and `integration.patch`. This is
temporary reproducibility material, not a production checkout or durable release
receipt. Query's committed sources remain the authority for its policies/tests.

## Adaptation

| Location in Query snapshot | Change |
| --- | --- |
| Root Cargo catalog | Add local `ic-platform-protocol`, defaults off, `cloud-engine-response` selected |
| `crates/ic-topology/Cargo.toml` | Inherit that dependency |
| `cloud_engine/engine/mod.rs` | Import `candid` and `cloud_engine` from Protocol; call `decode_get_engine_views_response` |
| Decoder limits | Preserve bytes/header/types/work/allocation, move the existing skip limit into Limits, select an explicit vector cap of 1,024 |
| `cloud_engine/engine/types.rs` | Use Protocol's request EncodeError and response Error in existing EngineError variants |
| Existing engine tests | Keep fixture encoding and request-decoding in published Platform; convert test-only response values through Serde to the published encoding DTO |

The vector cap is a new explicit decoder bound, not a claim that every historical
input remains admitted. Query still owns one returned view and 128-node-list
admission. Production projection, identity, membership, deadline, cancellation,
transport and evidence policies were unchanged. The test-only Serde conversion
does not enter production custody or add Encode to candidate response records.
Existing independent literal response fixtures remain unchanged.

Query retains published Platform for Agent, Registry and other Cloud Engine
operations. This experiment neither retires that dependency nor qualifies its
crypto/advisory obligations. Protocol itself remains independent of those edges.

## Checks and results

On Linux x86-64 with Rust 1.100.0, run the seven existing
`cloud_engine::engine` tests first against the untouched snapshot and then the
adapted snapshot. All seven passed in each run, covering:

- Opaque IDs and pre-I/O bounds.
- Independent literal raw values and source evidence.
- Missing versus empty Registry copy and distinct node membership/order.
- Identity/count/field/relationship refusals.
- Deadline errors and cancellation without retries.
- One argument, one query and successful omission.
- Decode budgets and validated skipped fields.

The adapted `ic-topology --all-targets` Clippy check passed with warnings denied.
`cargo check -p ic-query-cli` also passed. All builds/tests used cached sources
offline; after the one local-package lock addition, checks used `--locked`.
No mainnet requests were made and the full Query suite was not executed.

The shared graph checker required one each of Types, Protocol and published
Platform and excluded Runtime. It passed for `ic-topology` normal/build edges
on `x86_64-unknown-linux-gnu`. Both codecs resolve registry Types 0.20.4; no
Principal byte conversion or wrapper was added to production.

Reproduce in a fresh archive of the pinned Query commit using the adaptations
above, with the Protocol catalog path pointing to the candidate checkout:

```sh
cargo test --offline --locked -p ic-topology cloud_engine::engine
cargo clippy --offline --locked -p ic-topology --all-targets -- -D warnings
cargo check --offline --locked -p ic-query-cli
```

Run the first test command before adaptation too. Resolve the added local
dependency once with `cargo test --offline -p ic-topology cloud_engine::engine`,
then inspect the lock diff before using the locked commands again. The observed
lock diff adds only Protocol 0.1.0 and the topology-to-Protocol edge; no registry
supplier version changed.

Observed SHA-256 identities:

- Original Query lock: `286213f023c27a89ead9f39be6f5e98e15bec0b84395e19a0e6087faaa8ec105`.
- Adapted lock: `3037780267bfbccf632d5132b41441835c48da82a2ddb564ae977fc4f9d9240f`.
- Local six-file integration patch: `5dd1cbb8fec6d71191e594b78ec08cc57428753636ef67192472d1d45fa99726`.

This establishes focused compatibility with a real consumer at a pinned revision.
It is **not Query adoption**: no consumer change was committed, pushed or deployed.
The first projection gate remains open pending an authorized consumer migration
and remaining acceptance decisions. macOS results from this repository's CI do
not qualify this separate Query integration experiment.

## Portable Git-pinned patch

The reviewable [integration patch](../../qualification/patches/query-engine-view.patch)
replaces the machine-specific path with the public repository URL, exact
`08c72219ad6f261067c00eece9c8c71852e6b561` Git revision and package version
0.1.0. It includes the matching lock change and sorted manifest entries. No
registry publication is needed. This is a six-file proposal for Query, not an
already-applied migration or an additional product implementation.

The patch contains narrowly scoped edits and context from the pinned Query
revision identified above, under that repository's `LICENSE.md`; it does not
import a crate or test suite here. Query retains ownership of its source and
consumer admission policies. The published codec remains test fixture machinery
and a dependency for unrelated Query capabilities.

After selecting this Git source, the same seven tests, topology Clippy with
warnings denied, CLI compilation and single-Types graph checks passed again.
No registry supplier version changed. `git apply --check` passed against both
the pinned baseline and the existing sibling working tree; the latter was only
read, and its unrelated dirty files remain untouched.

- Portable patch SHA-256: `ca14d6436e0981e5be65c7fec56d65ee741c3cb5cb27373a4646b41abc5bc487`.
- Git-pinned Query lock SHA-256: `c7e1b73cfb55e0c7df908a214932b8ea4e394424d43a73c3d8b4037196f7b5c0`.

From this repository root, reproduce without changing the sibling checkout:

```sh
snapshot="$(mktemp -d)"
git -C ../ic-query archive 263a8e92da873543825c25e7dd6530c4db5ab673 | tar -x -C "$snapshot"
git -C "$snapshot" apply "$PWD/qualification/patches/query-engine-view.patch"
cargo fetch --locked --manifest-path "$snapshot/Cargo.toml"
cargo test --offline --locked --manifest-path "$snapshot/Cargo.toml" -p ic-topology cloud_engine::engine
cargo clippy --offline --locked --manifest-path "$snapshot/Cargo.toml" -p ic-topology --all-targets -- -D warnings
cargo check --offline --locked --manifest-path "$snapshot/Cargo.toml" -p ic-query-cli
```

Actual application to `../ic-query` requires separate authorization under the
read-only sibling rule. The patch deliberately leaves Query versions/release
ledgers untouched: delivery/version selection belongs to an authorized consumer
migration, and neither crate publication nor consumer delivery is implied.

## Authorized local application

The maintainer subsequently authorized applying the prepared patch to `../ic-query`.
The six affected files were clean and the patch passed a fresh application check.
Application touched only those files; the pre-existing modified and untracked
files were outside its scope. All six applied files match the validated
Git-pinned snapshot byte for byte.

The seven EngineView tests, topology Clippy with warnings denied, CLI compilation
and the single-Types/no-Runtime graph check passed again against the actual dirty
Query working tree. Its unrelated edits were included in compilation but were
not changed or separately qualified by this work. Build output went to the
temporary experiment target directory.

This is now a locally applied consumer integration, superseding the earlier
experiment-only status. It remains uncommitted and unpushed in Query. No package
version or release ledger was changed, no registry publication occurred, and no
mainnet request or deployment was made. Consumer delivery and the remaining
performance/acceptance work are still separate.

## Protocol 0.2.0 follow-up

The application evidence above used Protocol 0.1.x and registry Types 0.20.4.
Protocol 0.2.0 now exposes local Types 0.21.0, a deliberate Rust type-identity
break. The [local cohort qualification](types-consumer.md#protocol-cohort-020)
does not requalify Query's existing patch: Query and its identity-bearing callers
must move together before adopting this increment. This task leaves Query unchanged.
