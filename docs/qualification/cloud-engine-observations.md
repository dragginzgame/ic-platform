# Cloud Engine fee and Registry-copy observations

The existing `cloud-engine-response` feature now exposes
`decode_get_network_fee_response`, `decode_registry_copy_status_response` and
`RegistryCopyStatus`. This is the first two-method slice of the retained
[marketplace/availability contract #166](https://github.com/dragginzgame/ic-platform-legacy/issues/166).
The issue's declaration and [published resolution](https://github.com/dragginzgame/ic-platform-legacy/issues/166#issuecomment-6882560916)
were reviewed; earlier qualification does not qualify the replacement.
Query's marketplace design was inspected read-only.

The fee is one Candid float64. Decoding retains every IEEE-754 bit, including
signed zero and NaN payloads; Query owns finite/nonnegative admission. It is a
markup fraction on net, not a historical gross-price recomputation rule.
Registry-copy status is one record with required `latest: nat64`, `version:
nat64` and `caught_up: bool`. Numeric labels are independently calculated from
those names. No ordering, freshness, atomic-snapshot or availability policy is
inferred from the source values.

Both methods use the existing bounded reader and exact single-result admission.
The record uses the same single-declaration mapping as EngineView. Float64 uses
the existing fixed-width primitive reader and its byte/work accounting. No new
dependencies, default features, transport or authentication paths were added.

Local Linux checks on Rust 1.100.0 pass two literal-wire test groups, two public
consumer tests, Clippy, Rustdoc and consumer Wasm compilation through existing
response Make/CI targets. Fixtures cover signed zero, finite/non-finite fee bits,
full-width versions, independent source flags, all truncated prefixes, wrong
types, missing fields, malformed booleans, extra results/trailing bytes and
resource refusals. Public tests check exact status JSON and compare both
decoders with native-only published Platform 0.33.7.

[Marketplace prices](marketplace-prices.md), arbitrary naturals and all six
node-class variants now have a separate opt-in slice. [Available-page responses
and application errors](available-nodes.md) and bounded page argument encoding are
also implemented.
These two methods still require the caller's existing zero-argument encoder.
The [Wasm corpus](marketplace-wasm.md) now executes these observations alongside
marketplace and availability. Actual Query migration and hosted qualification
remain separate; no sibling code changed.
