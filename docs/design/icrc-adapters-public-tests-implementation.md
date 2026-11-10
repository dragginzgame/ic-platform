# Independent public ICRC adapter tests

The author read only `docs/design/icrc-adapters-contract.md` and previously
admitted policy for this component. No ICRC or Types implementation, old or new
adapter tests, supplier bodies, source fingerprints, history or diffs were read.
Earlier tasks exposed other newly written Runtime components and independently
authored tests, but no Account/archive implementation.

The coordinator supplied supplementary public facts: archive names are exported
at `ic_platform::icrc_ledger_types`; owned codec function signatures and budget
fields; available Serde test dependencies; and Func's Deserialize-only byte-buffer
framing. The framing is marker 5, unsigned LEB128 method byte length, UTF-8 method
bytes and remaining Principal bytes. The tests use short method lengths with a
single length byte and a small test Deserializer that delivers owned bytes.
No supplier serializer or parser body was supplied or inspected.

Only `crates/ic-platform/tests/icrc_adapters_consumer.rs` and this record were
authored. Public tests cover effective-zero equality/order/hash and canonical
conversion, field-preserving JSON/CBOR distinctions and Serde refusal, Account
text roundtrips, callback Clone/new/conversion without generic bounds, Eq-only
ordering, unchanged String allocation through owned Func conversions, and owned
codec roundtrip/signature refusal with Encode-only parameters lacking Decode.
Callback Serde tests retain ordinary three-field output and distinct function
buffer input. They also exercise malformed input and GetArchivesArgs Serde.

One test prints exactly one JSON observation line beginning
`ICRC_ADAPTER_OBSERVATIONS=`. Each deterministic entry contains a case label,
input byte length, and success or error Display/Debug result, never the input
suffix itself. Cases include valid defaults, management accounts and nonzero
accounts; checksum byte lengths/case; owner/leading-zero/hex precedence; first
period handling; empty suffix; ASCII hex errors; malformed multibyte scalar
positions and padded byte parity; 64/65/66-byte boundaries; and finite oversized
suffixes up to 65,537 ASCII bytes. The coordinator captures and compares this
same transcript before and after replacement; no snapshot or environment-driven
expectation was written. Roundtrip assertions cover additional valid owners and
subaccounts independently of the observation line.

After the six tests passed against the old owner, the coordinator retained its
48-entry public transcript in `tests/fixtures/icrc-adapter-observations.json`
and added an equality assertion to the observation test. Inputs and observation
construction were unchanged. This makes the before/after comparison an ongoing
regression check; the fixture is public API output, not copied implementation
or an imported test suite.

Only rustfmt 1.99.1 with an empty configuration and child traversal disabled was
run by the author. The coordinator owns feature/manifest registration, Clippy,
execution and freezing the before/after public observations. These tests do not
measure parser allocation or speed, exhaust Unicode or u32 domains, qualify
network calls, or establish legal clearance. No cryptographic algorithm was
implemented or altered.
