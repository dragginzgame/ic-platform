# ic-platform-types

The canonical Principal, Account, Subaccount and public identity errors used by
IC Platform. This package inherits the Rust workspace version and releases with
Platform, its macros and its runtime guard. Platform re-exports these exact types;
Wire owns implementations of its schema trait. Incompatible workspace minors can
require matching Wire adoption even when identity representations are unchanged.

The default feature set is empty. Bytes, bounded text and Serde are always
available; `principal-external` adds registry Principal identities/conversions,
and `principal-self-authenticating` selects unchanged upstream public-key hashing.
There is no runtime, executor, Wire or Candid dependency.

Existing Platform consumers keep their import paths. External bytes and account
semantics remain unchanged. Coordinated release notes live in the workspace's
root changelog; this package's changelog retains its initial publication history.
