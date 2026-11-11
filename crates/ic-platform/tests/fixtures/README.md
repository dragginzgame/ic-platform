# Binary protocol evidence

The `*-schema.json` fixtures contain independent upstream Candid binary headers
and absent optional values, with a parallel list of original type names. The
`$service` name identifies a frozen external method catalogue; it is test data,
not a Platform service-export API. `src/tests/reference/schema.rs` reads these
headers through Candid's binary parser, using its existing `binrw` dependency.

On 2026-11-11 these 110 named schemas were captured from the five existing
protocol fixtures and the independently written Registry/ICRC-10 test schemas,
using `candid_parser 0.4.1` and `candid 0.10.40`. Before retiring the text fixtures,
the capture test used upstream structural equality to compare every decoded
binary schema with its source, including service modes and recursive callbacks.
The retained schemas are independent of Platform's owned codec mappings.

`protocol-values.json` preserves the existing independent Registry optional
Principal, ICRC-10 reply and Cycles `Nat64` block examples as binary messages.
The remaining observation and `.hex` fixtures keep their existing custody.

Textual DID and its parser are retired. These are frozen comparison inputs;
do not regenerate expected schemas from the implementation under test. Future
external contract changes require independently reviewed protocol evidence and
updated binary fixtures. Historical source and capture inputs remain available
at the pre-cut source `a8c3fff39094d38510c46451ba1283231ee3cf37`.
