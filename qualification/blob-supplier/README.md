# Pinned Blob supplier qualification

Run `make qualify-blob-suppliers` from the Platform root. `prepare.mjs` requires
clean Blob revision `31dd236814879afd2fd0faf0851e6b46184476ac`, reads its source
archive and adapts only a disposable copy below `target/blob-supplier/`.
Set `BLOB_SOURCE` to a clean disposable checkout of that exact commit when the
sibling checkout has ongoing work; the revision and cleanliness checks still apply.

`codecs.mjs` migrates the actual callback/enrollment probe codecs and derives
selected retained tests. `signed-client.rs` is an explicit test-only client that
owns bounded polling; the product transport never polls automatically. Simulator
startup uses this repository's authenticated server supervisor. The original
whole-consumer `test-callbacks` and CLI process tests are not replaced or silently
counted as passed by the selected targets.

`run.mjs` builds the real host and probes, checks strict native/Wasm Clippy, runs
one signed, two callback-core and four enrollment journeys, refuses old Agent or
Platform fallback in the selected native test graph, and writes a receipt binding
source archive, adapters, candidate Rust sources, lockfile and Wasm artifacts.
A source change during the run refuses the receipt. No sibling writes, source
publication or provider network effects occur.

See [the contract and remaining gates](../../docs/qualification/blob-suppliers.md).
The pinned Blob snapshot retains its Toko Studios Network Licence v1.0; it is a
consumer qualification input, not a copied Platform implementation.
