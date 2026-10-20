# Account formatting follow-up

The pending 0.4.1 formatter uses a fixed 64-byte hexadecimal buffer. Checksum,
principal spelling, omission of default subaccounts and trimmed hex output remain
unchanged. No cryptographic implementation or persisted format changes.

[Raw trials](types.tsv) use the maintained `scripts/benchmarks/types.sh` harness:
Linux x86-64, Rust 1.99.1 release, opt-level 3, no LTO, 16 codegen units. Each of
five trials performs 500,000 operations after warmup; owned/external order
alternates. Values below are median nanoseconds per `to_string()` operation.

| Owner bytes, nonzero subaccount | Previous owned run | New owned run | External in new run |
| --- | --- | --- | --- |
| 10 | 540.7 | 234.7 | 377.7 |
| 29 | 613.0 | 289.6 | 491.5 |

The earlier figures are from [the preceding audit](../0.4/README.md), not an
interleaved old/new binary comparison. In the new paired implementation comparison,
owned formatting is approximately 38–41% faster than the external implementation.
The earlier regression is no longer observed in these fixtures. Principal,
Account and Subaccount layouts remain 30, 63 and 32 bytes respectively.

These are native fixture timings, not Wasm size, IC instruction or cycle
measurements. Rust 1.88 differential tests preserve text/Candid bytes across
23,040 account cases and all 64 trimmed hex lengths, including parser mutations.
The source change is confined to Account Display on top of released `26a07c5`;
the existing incoming lock update remains present.
