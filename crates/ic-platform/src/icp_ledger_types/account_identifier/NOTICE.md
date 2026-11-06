# ICP account identifier derivation

The `AccountIdentifier::new` body is retained unchanged from
`ic-ledger-types` 0.16.0, DFINITY cdk-rs revision
`80d82348b7de1f10117aead50ad26669da7e2b79`,
`library/ic-ledger-types/src/lib.rs` (Apache-2.0).
Copyright Internet Computer contributors. The complete supplier license is
`LICENSE-APACHE`; original source, function and license fingerprints are recorded
in `provenance.json` and checked by `check-icp-account-source.sh`.

Only the selected constructor and its private fixed-size carrier declarations
are retained. The external Principal and existing RustCrypto SHA-224 and
crc32fast providers retain the original algorithm, domain bytes and checksum.
The public adapter converts Platform's canonical Principal to external Principal
and returns the exact array. No new key, account identity, generic hash API,
ledger client, CDK dependency or allocation policy is introduced.
