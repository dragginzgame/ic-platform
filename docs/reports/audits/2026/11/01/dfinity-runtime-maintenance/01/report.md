# Production Rust dependency maintenance audit

The selected Platform 0.16 Wasm graphs retain **three DFINITY runtime packages
from two repositories**: `candid`, `ic_principal` and `ic-stable-structures`.
`candid_derive` is a fourth build dependency executed on the compiler host.
The raw runtime, owned Principal, certification and canister-signature profiles
select none of these packages. Native Agent and PocketIC are separate concerns.

**Verdict: PASS WITH FINDINGS for source and dependency-boundary inspection.**
This audits the contingency that DFINITY stops maintaining Rust SDK libraries;
it does not establish that abandonment has occurred. Selected Cargo graphs are
not proof of linked functions, deployed canister contents or measured Wasm size.

## Scope and identities

- Trigger: the maintainer requested an audit of remaining production dependence
  on DFINITY maintenance, after publishing Platform 0.16.0.
- Platform source: `1be6a1728ae8a488db91e337fd701aa8c3a22c18`, initially clean.
  Package release: `b1303ffa38daf2ec1551cbce6e39197ad76681ff`.
- Method: `audits/complexity-and-technical-debt.md`, with the dependency and
  artifact-evidence portions of `audits/ic-canister-applications.md`.
- Shared snapshot: `143a9abb9ae81feae9400ff3844b03b1ddfdd75a`; local overlay:
  `AGENTS.md`. Their fingerprints are in [input identities](artifacts/input-sha256.txt).
  This is a provisional maintenance-scope overlay, not a full application audit.
- Fresh execution: frozen/offline normal/build dependency queries for eleven
  individual Platform profiles, all Platform features on Wasm, one native Agent
  contrast and nine selections across six sibling workspaces. No builds, tests,
  dependency updates, releases or sibling edits were performed by this audit.
- Siblings are observed working checkouts during active 0.16 adoption, not a
  published fleet cohort. Commit IDs, dirty-path lists, root manifests and input
  hashes are retained alongside their graphs. Root manifest/lock hashes stayed
  unchanged across each initial consumer query. Later source captures are
  separately fingerprinted in [source traces](artifacts/source-traces.json).
- Exclusions: browser SDK, native Profiler/replica implementation, live network
  deployments, complete endpoint inventories and new size/performance measurements.
  Existing crypto evidence is referenced as historical evidence, not rerun here.

## Capability and owner map

| Selection or capability | External DFINITY runtime dependency | Maintenance responsibility if upstream stops |
| --- | --- | --- |
| Platform `runtime`, `cdk`, owned `principal` | None | Platform owns IC0 ABI wrappers, the single executor, callbacks and generator; consumers own lifecycle/policy. |
| Platform `certification`, `signature-verification` | None | Platform packages retained upstream algorithms, provenance, vectors and admitted patches. This does not establish independent cryptographic review capacity. |
| `principal-self-authenticating` | `ic_principal 0.1.5` | Types currently delegates public-key-to-Principal derivation to the registry implementation. |
| Management, ledger and other external Candid protocols | `candid 0.10.39`, `ic_principal 0.1.5` | Platform owns selected DTOs/admission and external compatibility tests; the Candid Rust decoder remains externally supplied. |
| `stable-structures` / Memory | `ic-stable-structures 0.7.2`, `ic_principal 0.1.5` | Persistence owns storage, allocation and reopening contracts; follow its existing migration plan. |
| Agent / PocketIC | Native dependency families | Separate host/client maintenance; taking these over is not a production-Wasm optimization. |

The all-feature Wasm graph contains no registry CDK, executor, IC0, management DTO,
certification, BLS wrapper/curve, Agent, PocketIC or Candid parser package.
Imported DFINITY-authored source still exists inside Platform: absence of an
external package is a maintenance transfer, not an independent reimplementation.

The [graph summary](artifacts/graph-summary.json) records packages and dependency
paths; the original `.tree` files retain the query results. Repository detection
uses package metadata plus an explicit provenance correction for
`ic-verify-bls-signature`, whose registry metadata lacks a repository field.

## Consumer observations

| Observed selection | DFINITY runtime packages |
| --- | --- |
| Timers, no default features | None |
| Auth, base and certified-proof/session-restore | `ic_principal` |
| Memory | `ic-stable-structures`, `ic_principal` |
| Relay, base and runtime | `ic-stable-structures`, `ic_principal` |
| Blob service library | `candid`, `ic_principal`, `ic-stable-structures` |
| Canic, base and control-plane/ICRC | `candid`, `ic_principal`, `ic-stable-structures` |

Candid selections additionally use compiler-host `candid_derive`. Canic's observed
graph contains both Platform 0.15.3 and 0.16.0 facades but only the 0.16 runtime
guard. This is not evidence of two executors. No completed fleet adoption or
installed composition is inferred from a successful graph query.

## Findings and decisions

### LOW: Canic still selects the service printer in ordinary runtime graphs

`canic/crates/canic/Cargo.toml` enables `candid-service` unconditionally. Both
observed Canic graphs retain `pretty` and its helper packages. The existing
[Canic #717](https://github.com/dragginzgame/canic/pull/717) owns the correction;
it was open and reported unmergeable at audit readback, head `f982f454`.
Disposition: finish that existing contribution when the consumer cohort is ready.
Prior controlled measurements found about 35 KB raw and 14–16 KB gzip savings;
those are historical results, not fresh size measurements of this adoption.

### LOW: Auth's otherwise owned identity path still requires external Principal

Auth enables `principal-self-authenticating` in its base dependency. Its public
`portable::browser_principal` builds the fixed Ed25519 DER envelope and calls
owned `Principal::self_authenticating`, which delegates to registry Principal.
The current dependency is intentional under the existing crypto contract.
The next bounded ownership candidate is retaining the unchanged upstream
derivation body and vectors in Types while leaving SHA-224 upstream. This needs
an explicit feature-contract decision and qualification, not a new algorithm.
It could remove Auth's standalone DFINITY package dependency; it will not remove
Principal from applications that also use Candid or stable-structures, nor does
graph removal establish a byte saving. Owner: Platform Types; disposition at
audit baseline: accept until the maintainer selects that packaging change.

### Retain Candid as the deliberate external-interface exception

Canic's management/ledger/NNS helpers and Blob's external funding interfaces have
real Candid callers. The [Candid specification](https://github.com/dfinity/candid/blob/master/spec/Candid.md)
defines an external data contract; using it does not require adopting the entire
SDK. The Rust implementation nevertheless remains a maintenance dependency.
Keep owned endpoints on Wire, preserve the bounded external decoder and byte
fixtures, and reconsider ownership only for a concrete unsupported protocol or
decoder defect. The previously rejected codec fork is not reopened by this audit.

### Retain the existing storage ownership plan

Memory selects stable-structures and its external Principal `Storable` support.
A storage cut must qualify existing bytes, reopening and migrations at Persistence;
removing a dependency declaration alone cannot transfer those responsibilities.
No second storage migration or blanket re-export is proposed here.

## Maintenance rehearsals

1. **A new IC management field or syscall:** Platform updates its owned DTO/ABI
   adapter from the network contract; consumers qualify actual calls and rollback.
   The installed simulator/network proof remains necessary even with local source.
2. **A Candid decoder defect:** the external codec remains the correction boundary.
   Existing byte/work/type budgets and 16-level framing admission constrain use,
   but do not prove every upstream defect harmless. A concrete defect triggers an
   owner decision; no speculative fork is authorized by this report.
3. **A BLS verification defect:** Platform can deliver a reviewed upstream-authored
   correction, as with #127, retaining fingerprints and vectors. Novel arithmetic,
   signature or domain-separation changes require qualified cryptographic review;
   source possession is not that qualification. Native Agent retains a separate
   registry verifier and is not fixed by changing the owned Wasm implementation.

## Verification and limits

All 22 dependency queries succeeded with `--frozen`, target-specific
`--edges normal,build --prefix depth`. Consumer commands select their named package
and `--no-default-features`; additional feature selections are recorded in
[consumer status](artifacts/additional-consumer-status.tsv). Platform metadata
was captured with `cargo metadata --all-features --frozen --format-version 1`.
Tool versions, input identities and original graphs are retained in `artifacts/`.
No current runtime defect or high-severity finding was demonstrated. This report
does not claim a complete security review, current deployed Wasm attribution,
macOS qualification, browser independence or independence from the core IC.
