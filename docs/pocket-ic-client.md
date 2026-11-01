# Maintained PocketIC Rust client

IC Platform maintains the released PocketIC 16.1.0 Rust client in the native-only
implementation package `ic-platform-pocket-ic`. Consumers select the existing
`pocket-ic` feature and import `ic_platform::pocket_ic`. This is one client owner
and one facade contract; the registry `pocket-ic` package leaves that graph.

The maintainer selected a signature-preserving first transfer. HTTP/JSON fields,
Candid management payloads, polling, rejections and existing operation behavior
are retained. The client still has its inherited panic and convenience behavior;
this does not claim typed transport failures or new retry/provisioning semantics.
Testkit continues using explicit server setup and compatibility/lifecycle policy.
No simulator server or IC execution code is imported. The native Profiler
provider retains its independently qualified patched client/server protocol.

Client, builder, Time, rejection and locally defined REST DTO Rust identities
move from the registry client to this implementation package. Their names and
method signatures through the facade stay the same. Consumers exchanging registry
client types must align on the new facade release. Required external management
DTOs retain their 0.8 identities, independently of Platform's 0.11 management
surface. Types, macros and runtime ownership otherwise stay unchanged.

A separate implementation package is needed because directly enabling the
facade's shared Candid/crypto dependencies would also select them on Wasm.
Every implementation dependency is native-target scoped; defaults and
executor-only graphs remain free of the client. Its version follows the Platform
group; publish it before the facade through the existing group publisher.
Existing versions stay at 0.16.0 (Types 0.15.1); the new implementation matches
its Platform group. The breaking client identity cut is pending 0.17.0.

[Source provenance, retained license and upstream review](../crates/ic-platform-pocket-ic/upstream/NOTICE.md)
record the source transfer. The source guard checks the reviewed implementation
and license hashes; intentional changes retain original identity, document the
deviation and add behavioral evidence before updating those fingerprints.

Qualification selects the owned client/facade tests, full required Clippy gate,
native/Wasm dependency policies, Rust 1.91 consumers and Testkit against the
existing prepared 16.1.0 server. A local source override is qualification only,
never the published distribution mechanism. Native macOS and hosted results
remain separate from local Linux evidence. Source transfer alone does not prove
independent simulator recovery or offline rebuilding of the complete stack.
