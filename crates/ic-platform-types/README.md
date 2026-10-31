# ic-platform-types

The canonical Principal, Account, Subaccount and public identity errors used by
IC Platform. This package has an independent release identity: unrelated Platform
runtime, host, DTO or endpoint changes must not change its version requirements.
Platform re-exports these exact types; Wire owns implementations of its schema trait.

The default feature set is empty. Bytes, bounded text and Serde are always
available; `principal-external` adds registry Principal identities/conversions,
and `principal-self-authenticating` selects unchanged upstream public-key hashing.
There is no runtime, executor, Wire or Candid dependency.

Existing Platform consumers keep their import paths. Moving to this canonical
owner is a one-time Rust type identity change; external bytes stay unchanged.
Release/publication awaits the admitted Shared Tooling release-group mechanism.
