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

The executable [profile catalog](../../ci/types-release-profiles.json) retains
Types' obligations. The complete release collector now runs these with the other
product obligations in one serial Make graph; see the
[current flow](../development.md#qualification-and-publication-flow--platform-26).

The obsolete Types-only publisher assumed every other product was unpublished
and reran its catalog. It has been retired. `types-publication-admission` now
selects Types through the same receipt/registry/archive admission as every other
group, requiring an explicit complete `RELEASE_RECEIPT`. The supported publisher
is `make publish RELEASE_GROUP=types PACKAGE=ic-platform-types RELEASE_RECEIPT=…`.

`check-types-publication` now depends on `check-release-evidence`, so the retained
Types CI entrypoint and affected-tooling checks retain admission refusal coverage
without maintaining a second policy implementation. The regression target is
excluded from the Types release catalog; fresh real audit and publication
admission still execute for every release. The tests preserve dirty source,
repository, annotated/remote tag, package eligibility, registry, audit and failed
qualification refusals. The retired prohibition on other publishable products
is superseded by exact one-package selection and release-group ownership.
Shared still owns dispatch and uncertain-upload custody. Final registry bytes
must match the admitted archive and selected clean VCS source. Simulated effects
qualify wiring only; live host/archive/registry evidence remains separate.
