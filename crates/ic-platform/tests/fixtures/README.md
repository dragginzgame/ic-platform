# Observed certificate fixtures

Captured locally on 2026-11-16 by the independently authored
`qualification/crates/platform-certificates` runner and its WAT canister.
These are raw protocol observations from PocketIC server 16.1.0 on Linux x86-64,
using ic-testkit 0.106.0 and pocket-ic client 16.1.0. They are not copied upstream
test assets, locally signed helper fixtures, or mainnet observations.

The server issues both certificate signatures. The candidate reads/verifies their
original bytes and never supplies a signing key. `root.der` is the explicitly
selected simulator root, not a product default. Both canisters certify 32 zero
bytes. The fixture tests pin canister/subnet identities and observed nanoseconds.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| root.der | 133 | 968c6c2122d3577878be624c1dc37609286f2f9ad7ee2cdae5d6bd45f0a05e8c |
| direct.cbor | 433 | 7931867746688fbd8e0445cf75720eed5794b109f12e0be176cbea94a4d21c79 |
| delegated.cbor | 1144 | 27fb75515279973d39b6a9d48db6471cd59e1d225013f780b992c4afdab09a1f |

Run `make qualify-platform-observed` to capture fresh evidence under
`target/platform-certificates/`. Fresh runs do not overwrite these frozen files;
review bytes, identities, timestamps and hashes before replacing the corpus.
PocketIC evidence does not establish mainnet interoperability, canister-signature
witness verification or production consumer migration.

Capture identities (SHA-256):

- Server executable: `b44e1eccd66e02328146b3e209e3403e3a3428a25d25555aca2f67500e5ae9db`.
- WAT source: `2d2a8d241794217810616d46a83a4deda4d9417646e44940587de4e389a428a9`.
