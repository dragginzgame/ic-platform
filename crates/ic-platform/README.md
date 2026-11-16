# ic-platform

Bounded certificate decoding and direct-root/single-delegation authentication, including canister signatures. Decoding produces unverified data. Authentication requires explicit trust root, target, time policies and resource bounds. Broader mainnet interoperability and production adoption remain separate gates.

Defaults are empty. Use one exact package/source cohort at public Types and
Runtime boundaries. This release publishes the implemented scope; it does not
claim completion of the full Platform rebuild or migrate applications.

See the [design and qualification records](https://github.com/dragginzgame/ic-platform/tree/main/docs).

Opt-in `ed25519` and `ecdsa` expose original upstream Dalek and K256 types on
native and Wasm. Consumers retain strict verification, canonical-scalar/high-S
admission, exact message framing and key policy. These features do not select
Agent, Runtime or RNG integration. See the
[Auth primitive contract](https://github.com/dragginzgame/ic-platform/blob/main/docs/qualification/auth-crypto.md).

`certificate-construction` supplies bounded certificate/signature encoding and
canister-key framing without authenticated evidence. Native fixture signing is
a separate opt-in capability. `native-read` supplies anonymous HTTP queries with
verified node signatures and authenticated selected-subnet observations. Callers
provide endpoint, root, root-subnet binding, time windows, resource limits and
whole-operation deadlines. There is no implicit root discovery or retry policy.

`native-signed` adds explicit Ed25519 PEM signing, exact offline update custody,
single-attempt dispatch and authenticated original-request observation, including
expired-original recovery. Signed queries reuse the native read verifier. There
is no implicit polling, retry, renewal or root discovery. See the
[Blob supplier contract](../../docs/qualification/blob-suppliers.md).
