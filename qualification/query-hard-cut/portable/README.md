# Query fixture corpus v1

This is test data and a replay helper, not a codec or product dependency.
`corpus-v1.json` contains 313 independently generated frames and their exact
typed inputs. Floating-point inputs use decimal strings of their IEEE bit
patterns, preserving negative zero, NaNs and infinities. Integer JSON tokens
retain their full width. Never round-trip this file through a JSON implementation
that narrows integers to floating point.

The independently authored schemas live in
`qualification/engine-view-consumer/src/fixtures.rs`. They describe the selected
Cloud Engine EngineView, marketplace, availability and Registry placement
contracts already recorded in `docs/qualification/{engine-view,marketplace-prices,
available-nodes,query-hard-cut}.md`. Upstream Candid 0.10.40 is the native-only
producer, locked by `qualification/Cargo.lock`; no old Platform decoder or
product projection generator produces these bytes. This first corpus was
selected by executing Query's actual 184 tests at revision
`4a7305c4c804dcd0a0ce3e3433c6a8662d81fd75` plus the input inventory recorded by
the source-bound runner. Same-owner schemas and helper code retain this
repository's licence; upstream Candid remains a dependency, not vendored code.

Verify all bytes from the independent schemas:

```sh
cargo test --locked --manifest-path qualification/Cargo.toml \
  -p engine-view-consumer portable_query_corpus
```

To regenerate after reviewing a schema or adding typed inputs, run the same
command with `REGENERATE_QUERY_CORPUS=1`, review the byte diff, then run verification
without that variable. New input/schema coverage must be added explicitly;
replay never synthesizes missing frames or falls back to a decoder/encoder.

Apply `../consumer.patch` to its recorded Query source, then install `fixtures.rs`
as `crates/ic-topology/src/tests/mod.rs` and `corpus-v1.json` beside it. The
`qualification/query-hard-cut.mjs` runner does this only in disposable validation
storage and checks the untouched sibling afterwards. These files enter the
topology archive through normal module discovery and `include_str!` of JSON data.
The consumer needs only its existing Serde, Types and Protocol dependencies.

Keep this corpus version stable for its recorded contract. Changes to inputs,
schemas or bytes require a new reviewed corpus identity and qualification record;
an API/schema-breaking corpus change must use a new versioned filename.
