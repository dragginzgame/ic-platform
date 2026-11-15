# Executed marketplace and availability Wasm

`make qualify-marketplace-wasm` builds and executes the private
`marketplace-admission` binary on `wasm32-unknown-unknown`. It reuses the existing
qualification package, Node runner and fixed build settings. The optional
`marketplace` selection enables the candidate Protocol feature; no published
Platform oracle, Runtime, Agent or registry Candid enters this artifact.

The runner validates the Wasm module, requires no imports and executes its main
entrypoint, requiring return code zero. It reports an artifact hash and admission
result only. It neither updates the EngineView size comparison nor selects a
size/performance threshold. Standalone execution is not installed IC acceptance.

## Executed contracts

- Marketplace: all six node classes, 2^140 and zero naturals, i64::MIN, empty
  versus absent scopes, all truncated prefixes and allocation refusal. A literal
  2^32-row count refuses on the Wasm usize boundary instead of wrapping to zero.
- Availability: unready versus empty versus populated pages, full-width version,
  cursor and node identities, all nine application errors, malformed tags,
  unknown selected classes and strict present-page admission. Truncations,
  invalid skipped UTF-8 and all seven configured resource bounds refuse. The
  selected IP fixture passes at four skip-work units and refuses at three.
- Page requests: exact literal bytes for zero and 2^140 natural limits with an
  explicitly empty rejoining selector; exact output bounds pass and one-below
  bounds refuse. Native consumer tests separately cover the broader option,
  Principal, text and 256-digit matrix.
- Fee/status: exact float bits including signed zero, infinity and a NaN payload;
  independent raw copy-version/status values retain full u64 width.

Literal schemas follow the selected #166 contract. They do not use a production
encoder to manufacture response expectations. The natural magnitude conversion
is therefore executed on Wasm's 32-bit target, beyond compile-only evidence.
This corpus complements, rather than replaces, the native literal tests and
published-source comparisons.

## Local result and CI wiring

Local Linux execution passed with Rust 1.100.0, Node 24.21.0 and
V8 13.6.233.17-node.53. Artifact SHA-256:
`444b4155fa327777652d24ced3f6e16536056b26eaba6aba703ee0db943b441c`.
The Make target first runs strict Clippy. The dependency graph selection admits
only the reviewed marketplace closure, requiring one Protocol, Types and bigint
identity. The new target runs in each existing native CI job, but hosted results
for these working-tree changes are not yet claimed.

No network read, sibling migration, publication, size threshold or timing
benchmark is part of this evidence. Actual Query integration, measured response
heap behavior and the remaining release gates are still separate.
