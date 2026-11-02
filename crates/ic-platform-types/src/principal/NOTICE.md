# Owned Principal provenance

The representation, byte constructors, traits and Serde scaffolding derive from
`ic_principal` 0.1.5, dfinity/candid revision
`294ec65548456a32070a83e15b9753aa9a59310f`, under Apache-2.0. The complete original
license is retained as `LICENSE-APACHE`. Platform owns this adapted type; the
combined package remains MIT AND Apache-2.0.

Changes: text is admitted at 63 bytes before normalization; parsing and formatting
use fixed stack buffers; binary Serde treats owned and borrowed bytes identically.
No Candid marker handling, generic range stepping, arbitrary generator or schema
implementation is imported. Length-first ordering, zero padding, raw bytes,
checksum algorithm, Base32 spelling and case-insensitive admission remain intact.
Overlong input now returns `TextTooLong` before examining its content.

The optional `self_authenticating` module retains the exact `self_authenticating`
method body and tag from that same ic_principal 0.1.5 revision. Only the surrounding
module/imports and documentation are added. SHA-224 uses RustCrypto `sha2` 0.11.1
with default features disabled. An import alias selects that provider; the obsolete
0.10 deprecation expectation is removed. The source guard reverses only these
scaffolding changes before checking the original admitted module fingerprint.
It also freezes the original Apache license, 29-byte capacity and
existing fixed/differential tests; registry ic_principal is a test oracle only
unless external conversions are selected. No upstream self-authentication test
module existed in that package; Platform's existing vectors remain unchanged.
Self-authentication is selected only by `principal-self-authenticating`; the base owned type has
no external Principal or hashing dependency. `principal-external` separately adds
the byte-copy conversions and registry identities without self-authentication.
No SHA implementation or key encoding is copied or rewritten; the derivation body
is imported without alteration, rather than redesigned.
`ExternalPrincipal` explicitly exposes that registry identity for external client
and DTO parameters; conversions copy only the significant bytes. Registry identity
is not retained by the owned root `Principal`. This is a breaking source change.
