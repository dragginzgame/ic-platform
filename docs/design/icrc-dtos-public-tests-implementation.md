# Independent public passive ICRC DTO tests

This component's author read only `docs/design/icrc-dtos-contract.md` and
previously admitted policy. No old or replacement DTO source/tests, supplier
implementation, value/hash source, source fingerprint, history or diff was read.
The coordinator supplied two supplementary public facts: consent names are
exported directly from `icrc_ledger_types::icrc21`, and ICRC3Value::Text(String)
is available for a passive BlockWithId fixture. No other value variant or hash
operation is exercised.

Only `crates/ic-platform/tests/icrc_dtos_consumer.rs` and this record were
written. Seven public tests cover signed/unsigned metadata extrema and values
beyond u128, owned String/byte allocation moves, memo's fixed-width big-endian
encoding and transparent byte Serde, all transfer error displays, optional-field
defaults, subaccount/consent-argument sequence Serde, narrowing order and limits,
passive consent values and nested archive callback codec roundtrips.

The finite serialization test prints exactly one JSON line prefixed
`ICRC_DTO_OBSERVATIONS=`. Its 23 labeled observations contain ordinary JSON text,
CBOR bytes and owned Candid bytes. JSON remains text inside the transcript to
retain declaration field order. Grouped enum values cover every selected
MetadataValue, TransferError, DisplayMessageType, consent Value, ConsentMessage
and Icrc21Error variant. The records cover metadata, transfer, allowance, blocks,
certificates, supported block types and consent, including empty/default and
extreme fields. A single serialization helper makes the tuple format uniform;
there is no type registry, alternate codec, policy framework or environment-based
expectation. The transcript is capped at 30,000 bytes for reviewability.

The coordinator owns baseline capture and freezing the original transcript as
a permanent fixture, then unchanged before/after execution. These tests preserve
ordinary callback Serde asymmetry by recording nested archive output without
claiming JSON self-roundtrip for callback-containing records. Owned Candid
roundtrips separately check the nested archive and consent values.

Coordinator integration made the borrowed-buffer copy test compare its original
result with the subsequently changed input, resolving an unused-assignment lint
without an allowance. All seven tests and Clippy then passed against the old
owner. The coordinator froze those 23 observations in
`tests/fixtures/icrc-dto-observations.json` and added an exact transcript assertion;
observation inputs and construction did not change. This fixture contains public
API outputs, not copied implementation or supplier test bodies.

The author ran only rustfmt 1.99.1 with empty configuration and child traversal
disabled. No Cargo command or product manifest edit was performed. The
coordinator owns registration, Clippy, runtime execution, the existing schema
oracle and unchanged hash/source guards. No network, policy, cryptographic,
performance or legal-clearance conclusion follows from these passive tests.
