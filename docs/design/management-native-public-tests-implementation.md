# Independent public native management DTO tests

The author read only `docs/design/management-native-contract.md` and previously
admitted policy for this component. No old or new management implementation,
shared maintained declarations, registry oracle tests, supplier source, imported
prose, history or diff was read. All constructors and public traits used here
come from the reviewed contract. Earlier author tasks did not expose these
management declarations.

Only `crates/ic-platform/tests/management_native_consumer.rs` and this record
were authored. The file is gated with `cfg(not(target_family = "wasm"))`.
Five public tests cover declared ordered/hashable traits, cloning and default
options; read/upload bytes-versus-sequence Serde distinctions; floating Candid
bit preservation; optional metadata fields and unbounded Nat; and finite
complete serialization observations for all 18 selected public types.

The grouped observations cover all snapshot global, timer, hook status, data
kind and data offset variants, both shared SnapshotSource variants, and the
three shared install modes. Cases include empty data, u64/i32/i64 extrema,
Some/None values and Nat above u128. Non-finite floats never enter JSON:
separate Candid assertions preserve positive/negative zero, positive/negative
infinity and positive/negative NaN payloads for f32 and f64 by comparing bits.
The tests neither assume Eq on float-bearing DTOs nor inspect unrelated shared
UpgradeFlags or CanisterSettings fields.

Exactly one JSON transcript line begins `MANAGEMENT_NATIVE_OBSERVATIONS=`.
Its 18 labeled observations contain JSON text, CBOR bytes and owned Candid
bytes, preserving declaration/variant representation. The serialization helper
also checks JSON, CBOR and Candid roundtrips using serialized values rather than
adding Eq bounds. The transcript is limited to less than 30,000 bytes. There
are no snapshots or environment-selected expectations in author output; the
coordinator owns baseline capture and freezing an ordinary regression fixture.

Only rustfmt 1.99.1 with an empty configuration and child traversal disabled was
run here. The coordinator owns native feature registration, Clippy, execution,
the existing independent registry/schema oracle, and before/after comparison.
No Cargo command or product manifest edit was performed. These passive tests do
not implement or qualify snapshot services, orchestration, trust, timers, network
policy or cryptography, and make no performance or legal-clearance claim.
