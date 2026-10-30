# Selected ICRC contracts

Platform maintains selected Apache-2.0 declarations and helpers imported from
`icrc-ledger-types` 0.2.0, DFINITY IC revision
`3fe742eebad228950d1029d1fc14079a2ea9f1e5`, directory
`packages/icrc-ledger-types`. The complete license is `LICENSE-APACHE`.
`provenance.json` records the original file and license digests.

The maintainer authorized removing the registry package and narrowing this
surface. Public module paths and external Candid/Serde representations remain;
the selected Rust types, including archive callbacks, now have Platform identity.
The Candid Principal, Nat, Int, function and Serde-byte identities stay upstream.

Packaging changes:

- Retain selected account, transfer/memo, allowance, metadata, value, archive,
  block/certificate and ICRC-21 consent declarations. Retarget private imports.
- Remove account CBOR derives, stable-storage integration and unexposed
  subaccount-allocation helpers. Account text retains CRC32 and unpadded Base32;
  use the already selected `data-encoding` implementation instead of `base32`.
- Delegate external Account display to the owned Account formatter. External
  parsing and its error identity remain separate; checksum and text bytes remain.
- Omit unexposed deprecated archive DTOs, metadata-key helpers and consent
  message construction/number/time formatting. Consumers own consent policy.
- Preserve the complete value implementation, hash bodies, integer encoders
  and upstream fixed test vectors. Disable only its two property-test items,
  retaining their source without acquiring the unused property-test framework.
  The source guard reconstructs the original token fingerprint after reversing
  only the import and those two attributes. It also checks the complete license.
- Keep ICRC-10's independently maintained protocol record unchanged.

No hash algorithm, domain separation, value conversion or cryptographic vector
is replaced. Legacy `Value::Nat64` and the narrower `ICRC3Value` remain distinct.
This surface supplies no CBOR or stable-storage encoding contract.

`tests/fixtures/icrc-reference.did` freezes all 28 selected registry schemas,
generated independently using the original 0.2.0 package and Candid 0.10.39.
Its digest is recorded in `provenance.json`. Schema comparison includes recursive
values and callback arguments/results/mode; existing upstream byte vectors and
hash vectors separately qualify their representations and hashes.
