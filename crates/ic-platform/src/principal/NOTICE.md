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

Cryptographic self-authentication delegates to registry `ic_principal` unchanged.
It is selected only by `principal-self-authenticating`; the base owned type has
no external Principal or hashing dependency. `principal-external` separately adds
the byte-copy conversions and registry identities without self-authentication.
No SHA implementation, key encoding or hashing algorithm is copied or rewritten.
`ExternalPrincipal` explicitly exposes that registry identity for external client
and DTO parameters; conversions copy only the significant bytes. Registry identity
is not retained by the owned root `Principal`. This is a breaking source change.
