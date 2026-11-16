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
