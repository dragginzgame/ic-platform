# ic-platform-types

The canonical Principal, Account, Subaccount and public identity errors used by
IC Platform. This group owns a literal package version and changelog, independently
of facade/runtime releases, while sharing the same workspace, dependency catalog
and lockfile. Platform re-exports these exact definitions; Wire owns its schema
implementations. The independent series continues from published Types 0.15.0.

The default feature set is empty. Bytes, bounded text and Serde are always
available; `principal-external` adds registry Principal identities/conversions,
and `principal-self-authenticating` selects unchanged upstream public-key hashing.
There is no runtime, executor, Wire or Candid dependency.

Existing Platform consumers keep their import paths. External bytes and account
semantics remain unchanged. Its [group ledger](https://github.com/dragginzgame/ic-platform/blob/main/releases/types/CHANGELOG.md)
owns release notes and links the earlier coordinated history. Initial 0.1 notes
remain historical. Release and publication require explicit RELEASE_GROUP=types.
