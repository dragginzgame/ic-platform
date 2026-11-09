# ICP account identifier derivation

The `AccountIdentifier::new` body is retained unchanged from
`ic-ledger-types` 0.16.0, DFINITY cdk-rs revision
`80d82348b7de1f10117aead50ad26669da7e2b79`,
`library/ic-ledger-types/src/lib.rs` (Apache-2.0).
Copyright Internet Computer contributors. The complete supplier license is
`LICENSE-APACHE`; original source, function and license fingerprints are recorded
in `provenance.json` and checked by `check-icp-account-source.sh`.

Only the selected constructor and its private fixed-size carrier declarations
are retained. Canonical Principal supplies the original bytes. RustCrypto SHA-224 and
crc32fast retain the original algorithm, domain bytes and checksum. The SHA-224
provider now selects SHA-2 0.11.1 through an import alias, sharing Types
self-authentication; the constructor body fingerprint is unchanged.
The public adapter returns the exact array. No new key, account identity, generic hash API,
ledger client, CDK dependency or allocation policy is introduced.
