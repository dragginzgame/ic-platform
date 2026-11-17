# IC Platform

IC Platform supplies explicit identity, execution, external-format and authenticated
evidence boundaries for Internet Computer applications. The rebuild preserves
required external bytes, security checks and resource refusals while deliberately
replacing legacy Rust APIs. It is a hard cut: no legacy decoder or compatibility
facade is selected in product graphs.

Start with [the architecture and acceptance plan](docs/DESIGN.md),
[consumer contracts](docs/requirements.md) and [development commands](docs/development.md).
GitHub owns issue status; qualification documents retain source-bound evidence.

## Published scope

The latest recorded release is Platform 0.34.15 / Protocol 0.3.5, alongside
Types 0.21.2 and Runtime/macros/guard 0.23.17. See the
[release and archive verification record](docs/releases.md#independent-status-admission-and-qualification-flow-release)
and independent [group changelogs](CHANGELOG.md). Private qualification packages
remain outside product release groups. Publication does not establish every
consumer's production migration or completion of the entire design.

| Owner | Implemented capabilities | Evidence |
| --- | --- | --- |
| Types | Canonical Principal, Account/Subaccount, byte/text/Serde admission and opt-in self-authentication | [Types consumer qualification](docs/qualification/types-consumer.md) |
| Runtime | Consuming call preparation, one-way/awaited transport, one method-attached executor, query/update/lifecycle exports, explicit guard/codec/reply composition, system observations, certification, raw operations, memory/cycles and timer roots | [Installed lifecycle](docs/qualification/runtime-installed.md), [Auth execution](docs/qualification/runtime-auth.md), [timer boundary](docs/qualification/runtime-timers.md), [Storage boundary](docs/qualification/runtime-storage.md) |
| Protocol | Bounded selected Cloud Engine/Registry, management/snapshot/chunk, ledger/CMC/Caffeine, SNS/ICRC and HTTP request formats; decode capability does not require Encode | [Contract map](docs/requirements.md), [snapshot profile](docs/qualification/management-snapshots.md), [Canic management](docs/qualification/canic-management.md), [SNS](docs/qualification/sns-neurons.md) |
| Platform | Strict untrusted certificate decoding, explicit authentication, canister signatures, upstream crypto adapters, authenticated native reads, ingress admission, signed/anonymous prepared transport and original-request recovery | [Certificate boundary](docs/qualification/platform-certificate.md), [Auth crypto](docs/qualification/auth-crypto.md), [Query](docs/qualification/query-hard-cut.md), [signed transport](docs/qualification/blob-suppliers.md), [anonymous transport](docs/qualification/anonymous-and-blob-codecs.md) |

Defaults remain empty. Protocol has no Runtime dependency; Platform has no
Protocol or Runtime dependency. Runtime's macros and singleton guard belong to
its coordinated release group. Public crossings require one canonical Types
source identity and one executor; equal versions from different Cargo sources
are not the same type identity. Qualification and native-only oracle tools stay
outside product graphs.

## Security and execution contracts

Certificate decoding returns unverified data. Authentication requires explicit
trusted root, target/subnet binding, signing/delegation time windows and resource
limits. Nested delegation refuses; successful decoding alone establishes no
trust. Fresh PocketIC observations and frozen mainnet direct/delegated evidence
replay through native and Wasm public APIs. Broader interoperability remains a
separate acceptance obligation.

Native transport retains exact original request bytes, IDs and expiries. Signed
admission never falls back to anonymous. Explicit anonymous admission requires
an anonymous sender and absent authentication fields. Bounded dispatch and
certified status reads do not infer retry safety, renew expiry or supply
application approval. Query authentication is distinct from consensus
certification; consumers retain endpoint/root and whole-operation policy.

Runtime's awaited executor remains an experimental, bounded contract. Installed
checks cover callbacks, traps/cleanup, refunds, resource refusals, stale wakes,
uncertain remote outcomes and stopped upgrades. Remaining limits include broader
retention/reclamation accounting, timer-dispatch integration and context/late-call
cases; see [remaining gates](docs/qualification/runtime-installed.md#remaining-gates).
Local handle termination does not establish remote rollback or cancellation.

The supported upgrade workflow is **stop → wait for Stopped → upgrade → start**.
Callbacks drain before installation; suspended futures and callback IDs remain
heap-local. Runtime adds no live-call reconstruction or automatic retry.
Use ICP CLI (`icp`) exclusively; see the [operator workflow](docs/development.md#canister-upgrades).

## Consumer and legacy comparison evidence

The recorded actual Auth fixture passes native and installed qualification without
legacy Platform. Query's isolated registry-backed caller and packaged corpus pass
without Agent or the old Platform dependency. These supplier fixtures do not
establish downstream delivery; their exact source and host limits are linked above.

Backup reports a complete local production port, including 20-MiB capture/restore,
interrupted transfers and offline recovery. Its remaining direct test-only
Candid/SDK dependencies are a separate fixture-ownership obligation in
[Backup #2](https://github.com/dragginzgame/ic-backup/issues/2); Platform #11 is complete.
Blob reports actual adoption of the selected codecs and signed transport.
Canic also records local adoption and installed anonymous deployment/recovery/upgrade
acceptance, closing [#25](https://github.com/dragginzgame/ic-platform/issues/25).
Its retained runs do not establish delivery or qualify subsequent working-tree changes.
The old Agent identity/derivation crypto obligation remains tracked in
[#4](https://github.com/dragginzgame/ic-platform/issues/4); an audited current
Platform graph does not qualify those retained external paths.

The source-review baseline is legacy commit
`c24e3a59583eabcf5b6bb87987269c13a209a51f`. Historical
[EngineView heap comparisons](docs/qualification/engine-view-heap.md) showed one
fewer allocation per selected workload, saving 32–72 requested bytes. The
[paired Wasm probe](docs/qualification/engine-view-wasm.md) showed a 1,693-byte
code-section increase. These are bounded historical measurements, not a current
whole-product speed or deployed-size comparison. Size acceptance is undecided;
contribution analysis remains deferred. Required correctness checks stay intact.

## Development and delivery

Rust 1.100.0 and Toko Tooling 0.45.0 are selected by the repository. Run
`make install-tools tools-check` for admitted tools, including checkout-local
ICP CLI 1.7.0. Regular native CI includes signed/anonymous installed transport
and Blob native/Wasm codecs on Linux and both macOS architectures. Configured
coverage is not a claim that the latest hosted run passed.

The complete local release catalog, clean-source receipt, packaged/archive
checks and registry readback remain required. Fresh isolated RustSec audits cover
both lockfiles and bind their exact evidence into publication admission. See
[development](docs/development.md) and [release evidence](docs/releases.md).
Hosted CI is asynchronous and is not awaited for deployment.

Sibling checkouts remain read-only unless separately authorized. Production
consumer migration, application authorization, scheduling, storage allocation,
trust-root discovery and business retry policy remain with their owners.
