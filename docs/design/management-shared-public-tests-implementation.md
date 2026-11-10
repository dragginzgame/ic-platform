# Independent public shared management tests

For this component the author read only
`docs/design/management-shared-contract.md` and previously admitted policy.
No old or replacement shared declaration source, codec or transform adapter,
existing tests, supplier implementation, history or diff was read. Earlier work
included independently authored native management consumers from their separate
contract, without exposing shared declaration bodies.

The coordinator supplied one additional public input fact: the owned Func
Deserialize buffer begins with byte 5, then unsigned LEB128 UTF-8 method length,
method bytes and remaining Principal bytes. A fixed short fixture uses a single
length byte and a small byte-buffer Deserializer delivered through Serde's map
adapter. No length parser, supplier body or callback implementation was copied.
Original-owner execution additionally established that the public TransformFunc
tuple wrapper requests Serde newtype visitation. The test Deserializer now
visits that wrapper explicitly before delivering the underlying owned bytes;
ordinary byte-buffer fields retain their byte-buffer visitation. This corrects
the fixture adapter, without changing the passing serialization transcript or
product implementation.

Only `crates/ic-platform/tests/management_shared_consumer.rs` and this record
were authored. Four public tests cover trait/default/order/hash contracts,
bytes-versus-sequence Serde, deserialize-only HTTP/transform input, and grouped
complete serialization observations. All 44 selected root names are exercised
directly or through containing public records. Unexported snapshot visibility,
upgrade flags/persistence, load-snapshot and rename payloads are reached through
inferred types, defaults or containing-record deserialization; no export was
added or assumed.

The cases cover every listed enum variant, option defaults/present values,
u64/u32 maxima, Nat above u128, arbitrary strings and byte fields, nested change
origins/details, and settings/memory/query records. Key/signature DTOs carry only
arbitrary passive bytes; no cryptographic operation is performed. Transform
function and context behavior is checked through existing public construction,
Serde input and owned codec APIs without inspecting their maintained owners.

One `MANAGEMENT_SHARED_OBSERVATIONS=` line records 24 grouped observations of
JSON text, CBOR bytes and owned Candid bytes. Its separate deserialize-only
section records HTTP request input plus Candid bytes for HttpRequestArgs,
TransformContext and TransformFunc; it does not synthesize Serialize
implementations for those types. A single serialization helper checks complete
JSON/CBOR/Candid roundtrips for eligible records. The finite transcript has a
60,000-byte guard; there are no environment-driven expectations or author-owned
snapshots. The coordinator owns original capture and the permanent regression
fixture.

The author ran only rustfmt 1.99.1 with an empty configuration and child traversal
disabled. No Cargo command or manifest edit was performed. The coordinator owns
feature registration, Clippy, before/after execution and existing registry/schema
oracles. These tests do not validate policy, URLs, controllers, monetary limits,
network behavior, storage, timers or cryptography, and make no performance or
legal-clearance claim.
