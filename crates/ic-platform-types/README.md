# ic-platform-types

The canonical Principal, Account, Subaccount and public identity errors used by
IC Platform. This group owns a literal package version and changelog, independently
of facade/runtime releases, while sharing the same workspace, dependency catalog
and lockfile. Platform re-exports these exact definitions; Wire owns its schema
implementations. The independent series continues from published Types 0.15.0.

The default feature set is empty. Bytes, bounded text and Serde are always
available. `principal-self-authenticating` selects the unchanged imported derivation
with upstream SHA-224. The external-Principal feature, aliases and conversions
are removed. Registry Principal remains only a development oracle in this package.
There is no production runtime, executor, Wire, Candid or registry Principal dependency.

Existing Platform consumers keep their Rust import paths and account identity
semantics. Present subaccounts serialize through `serde_bytes` with exactly 32
bytes. This changes binary Serde contracts: Postcard adds the `0x20` length prefix
after Some's `0x01` marker; None stays `0x00`. Coordinate Wire schemas/clients,
qualify budgets and preserve original retry/hash bytes. Platform-owned external DTOs now use this canonical Principal. Its [group ledger](https://github.com/dragginzgame/ic-platform/blob/main/releases/types/CHANGELOG.md)
owns release notes and links the earlier coordinated history. Initial 0.1 notes
remain historical. Release and publication require explicit RELEASE_GROUP=types.
