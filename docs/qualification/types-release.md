# Independent Types release qualification

The maintainer selected independent Types release after retained-contract,
packaged-crate and real-consumer qualification. Production consumer migration
remains a separate gate. The existing [Principal](types-principal.md),
[Account](types-account.md) and [consumer](types-consumer.md) obligations remain.

## Real consumer boundary

`make qualify-types-auth-archive` archives the passive `ic-auth-protocol-types`
source from public [Auth commit b78991e](https://github.com/dragginzgame/ic-auth/tree/b78991e9833320692921bf766e51ad361c9c8c4d/crates/ic-auth-protocol-types),
package tree `1c11a2231ae712cec6670c947977db04cb302f67`. Its license is retained
in the disposable workspace. No upstream tests or implementation are copied into
this repository. The source checkout is read-only; CI checks out that exact commit
under ignored `target/auth-source`.

The runner prepares the actual Cargo Types archive, extracts it, and points the
isolated consumer manifest at that extracted package. The only extracted-manifest
addition is a `[workspace]` marker to stop enclosing-workspace discovery. Product
source, features and dependency declarations stay unchanged. The archive itself
is not modified. This is prepublication archive qualification, not registry proof.

Independent assertions construct Auth's real PortableClaims using Types Principal
and Account.owner, cross public boundaries without byte conversion, and check
literal identity bytes and opt-in SHA-224 self-authentication. Both default and
self-authentication configurations run native and import-free standalone Wasm.
Metadata admits exactly one Types identity. Archive all-feature Clippy runs native
and Wasm, and archive tests run native. Each attempt records source/tree identity,
archive and lock SHA-256, compiler, Node and Wasm hashes under `target/types-auth/`.

The published-oracle consumer additionally checks every Principal length against
every one-bit Subaccount (7,680 cases), rendering, parsing, case handling and typed
Principal refusals. Earlier literal and Serde token fixtures remain independent
expected-value evidence. Upstream cryptography is reused, not reimplemented.

This qualifies the passive model's canonical identity crossing. It does not
qualify Auth's whole application, credentials policy, ECDSA integration or deployed
migration. Runtime crossing tests similarly establish only the local exact-source
identity boundary, not release readiness of Runtime.

## Release evidence

The executable [profile catalog](../../ci/types-release-profiles.json) is the
Types-specific roster. `types-publication-admission` requires successful hosted CI
for the exact source, reruns the catalog and retains its logs, records audit and
registry observations, and checks Cargo archive VCS provenance against a fresh
fetch from the declared repository. The shared publication runner separately
retains each check/upload attempt. Final registry bytes must match the admitted
archive and carry the selected clean source revision before delivery is complete.

`make check-types-publication` exercises dirty source, wrong repository, lightweight
/local-remote tag mismatch, expanded publishable roster, failed hosted validation,
unavailable registry, existing version, failed audit and failed profile observations.
A controlled success verifies the full admission path without dispatching an upload.
These checks qualify policy wiring; actual host, archive and registry evidence remain
separate. Live checks also refused the dirty preparation and a Platform selector.
