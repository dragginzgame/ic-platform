# Independent external DTO consumers and minimal schemas

For this component the author read only `docs/design/external-dtos-contract.md`
and previously admitted policy. No existing or replacement DTO, codec, callback,
account-identifier or Value implementation, existing tests/DIDs, supplier source,
history or diff was read. The coordinator confirmed public candid_parser API
names for parsing and inspecting service method arities/modes. The admitted Func
byte-buffer/newtype facts were reused from the contract and earlier independently
authored fixture helpers; no callback body was inspected.

The author wrote `crates/ic-platform/tests/external_dtos_consumer.rs`, the four
fresh `tests/fixtures/owned-{cmc,icp,cycles,assets}.did` files and this record.
The Rust target has a native-only file gate. Four tests cover passive optional
fields and malformed Serde input; callback signatures and provider Nat64;
deterministic representations; and exact minimal-service rosters/arities/modes.
No Serialize implementation is invented for the Deserialize-only DTOs.

The transcript prints one `EXTERNAL_DTOS_OBSERVATIONS=` JSON line with 27 grouped
explicit Serde input observations and two constructed callback/provider records.
Each Serde observation retains JSON input text, CBOR input bytes, decoded Debug
and owned Candid bytes, and checks equivalent decoded values and roundtrips.
CBOR input construction replaces only occurrences of the fixture Principal's
text with its binary bytes; no ordinary text sample equals that sentinel. All
ordinary byte vectors remain sequences. This is test-fixture construction, not
normalization of product input. The constructed section covers the archive
function fields whose owned-byte Serde input intentionally differs from a map.

Cases cover all selected declarations and enum variants, empty/present optional
fields, arbitrary bytes/Unicode/NUL strings, u64 extrema and Nat above u128.
The provider fixture retains Value::Nat64 separately from Nat and exercises the
public Value variants only as passive codec data. Its records intentionally lack
Eq; field checks and re-encoding establish the tested observations. Callback
checks retain query mode, one argument/result, owned Principal/method identity,
and refusal of the standard ICRC3 result signature where the provider's Nat64
result is required. No account-identifier or Value hashing function is called.
The finite transcript has a 60,000-byte guard and no environment-selected
expectations; the coordinator owns freezing original output.

The four new DID files were written directly from the supplied declaration and
selected-method facts. They contain exactly 14 service methods: three CMC, two
ICP, seven Cycles and two asset methods. They are supported-subset schema
oracles, not complete remote service descriptions. Permanent tests parse only
these new fixtures and verify exact names, arities and modes. The coordinator
reports a separate original/new fixture comparison already passed exact selected
method type equality, including recursive callback types; no old DID was exposed
to this author. Historical interface revisions remain documented in the reviewed
contract, without claiming independent origin of the protocol facts.

Only rustfmt 1.99.1 with an empty configuration and child traversal disabled was
run here. No Cargo command or manifest edit was performed. The coordinator owns
registration, Clippy, original/replacement execution, frozen evidence and source
retirement. These tests do not implement crypto, network or application policy,
and establish neither performance nor legal clearance.

Coordinator baseline Clippy requested naming the method/fixture tuple shapes.
Two local type aliases replace the complex annotation without changing cases,
observations or adding a lint suppression.
