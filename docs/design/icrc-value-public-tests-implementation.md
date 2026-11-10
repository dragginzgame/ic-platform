# Independent non-cryptographic ICRC value tests

For this component the author read only `docs/design/icrc-value-contract.md`
and previously admitted policy. No original or replacement Value implementation,
old tests, hash module, source guard, supplier source, history or diff was read.
Earlier tasks used public value variants as passive DTO fields but did not expose
value-helper or hashing implementation bodies.

Only `crates/ic-platform/tests/icrc_value_consumer.rs` and this record were
written. Six fresh public tests cover both enum trait/order sets; constructors
and duplicate converted map keys; all consuming accessor/TryFrom mismatch
families and exact errors; signed/oversized integer admission; recursive
Nat64-to-Nat conversion; Principal/Account bytes and refusal order; None versus
explicit-zero subaccounts; and literal Display shapes/options/error propagation.
The empty-account panic is observed in a fresh native thread without asserting
its implementation-specific message.

Negative Int conversion errors are compared to the public BigInt-to-BigUint
conversion's Debug error, inferred through Nat's public constructor. Principal
admission and slice-to-array failures similarly use their public error displays.
No numeric, cryptographic or identity algorithm is reproduced. The overflow
message explicitly checks the existing literal `{nat}` braces.

One finite JSON line begins `ICRC_VALUE_OBSERVATIONS=` and contains 15 labeled
Value/ICRC3Value rows with JSON text, CBOR bytes, owned Candid bytes, Display and
Debug, plus eight selected detailed error results. Candid roundtrips are required;
JSON/CBOR deserialization records successful equality or its exact existing error
without adapting serializer output. The transcript is guarded below
30,000 bytes and has no environment-selected expectations. The coordinator owns
original capture, the frozen fixture and unchanged replacement comparison.

The tests never call hash methods, derivation or cryptographic primitives and
contain no algorithm/vector suite. They check blob conversion contents without
requiring the original implementation to transfer its allocation. Intended
allocation improvements and any measurements remain separate coordinator-owned
post-replacement work.

Only rustfmt 1.99.1 with an empty configuration and child traversal disabled was
run here. No Cargo command, manifest edit or source inspection beyond the
contract was performed. The coordinator owns feature registration, Clippy,
execution, retained-crypto safeguards and qualification. These observations do
not establish performance or legal clearance.

Original-owner execution supplied two compatibility facts that corrected test
assumptions without product changes. Nat64 uses its raw u64 Display; conversion
to ICRC3Value widens it to Nat, whose Display may group decimal digits. The
standard-value expectation now delegates to public Nat Display. Also, owned Int
serializes through a BigInt limb sequence that its Deserialize rejects. The
tests explicitly preserve that known JSON sequence rejection and capture exact
JSON/CBOR outcomes rather than assuming Serde self-roundtrip. The coordinator's
separately appended allocation-observation test remains unchanged.
