# Owned account provenance

Account representation, effective-subaccount equality/ordering/hashing and text
format derive from icrc-ledger-types 0.2.0, dfinity/ic revision
`3fe742eebad228950d1029d1fc14079a2ea9f1e5`,
`packages/icrc-ledger-types/src/icrc1/account.rs`, under Apache-2.0.
The original notice is retained as LICENSE, with full text in LICENSE-APACHE.

Platform uses its owned Principal, omits Candid/CBOR/storage traits and principal
mapping helpers, and provides explicit feature-gated external DTO conversions.
Parsing and formatting use bounded stack storage instead of intermediate strings.
The CRC32(owner || subaccount) checksum algorithm is unchanged; no cryptographic
primitive, account-identifier derivation or hash domain is implemented or changed.
Parse errors are owned, bounded diagnostics; oversized inputs are rejected first.
Absent and explicit zero subaccounts remain distinguishable serialized fields,
while equality, ordering, hashing and displayed text treat them identically.

The approved optional-subaccount Serde cut uses the standard serde_bytes adapter
with exactly 32 payload bytes. Present values become byte strings rather than
tuples; the absent/zero distinction and logical/text identity remain unchanged.
Wire owns the matching schema and coordinated consumer adoption.
