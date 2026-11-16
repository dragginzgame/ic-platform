# Marketplace price projection

`cloud-engine-marketplace` enables `decode_list_marketplace_prices_response`
and decode-only `MarketplaceEntry`, `PriceEntry`, `Price` and `NodeType` records
under `cloud_engine`. It adds `candid::Nat` with upstream BigUint storage.
These implement the price-row portion of [#166](https://github.com/dragginzgame/ic-platform-legacy/issues/166),
using the selected declaration recorded there and its published resolution.
The source declaration hash and live-observation provenance remain in that
issue; this work performs no live service reads.

## Wire and interpretation contract

Decode exactly one vector. Preserve opaque keys, source order, all six type4
class tags, arbitrary nonnegative cycle counts, full signed nanosecond timestamps
and optional provider/DC presence. Present empty DC differs from absent DC;
provider uses the existing canonical Types identity. Amounts are cycles per node
per day. No gross recomputation, key normalization, scope precedence, uniqueness
admission or capacity inference is performed. Unknown selected variant tags and
non-null selected class payloads refuse. Existing strict optional semantics and
bounded unknown-record-field skipping remain in force.

Record and variant schemas, dispatch and public shapes use the existing private
projection declaration. The variant arm now also supports typed payloads for
[availability results and errors](available-nodes.md). The reader is extended rather
than introducing another Candid codec. Default, request-only, EngineView and
Registry-settings selections retain their previous dependency closures.

## Naturals and dependency selection

The optional supplier is num-bigint 0.5.1, already selected by the published
Platform 0.33.7 qualification oracle. Its locked closure adds num-integer 0.1.47,
num-traits 0.2.19 and build-only autocfg 1.5.1 to the product workspace. Existing
lock selections were preserved; the qualification lock adds only the Protocol
edge for this feature. The separate marketplace graph allowlist requires one
BigUint supplier and one canonical Types identity. It does not widen EngineView's
allowlist or admit Runtime, Platform, Agent or registry Candid into production.

Nat decoding scans the complete unsigned LEB payload under input/work limits,
then charges a second linear conversion pass, scratch digit bytes and magnitude
storage before allocation. Magnitude charge is `8 * ceil(7 * wire_bytes / 64)`;
scratch charge is `wire_bytes`. These cumulative logical charges deliberately
count non-minimal encodings and temporary storage. They are not a process heap
quota or an exact comparison with the old decoder. The scratch reservation is
fallible; upstream BigUint construction uses its normal allocator behavior.
Power-of-two radix conversion and arithmetic remain with upstream num-bigint;
no bigint arithmetic implementation was copied or introduced here.

The small Nat adapter preserves the published Serde contract: exact u32 limb
arrays on serialization and nonnegative integers, decimal strings, limb arrays
or Candid tagged byte buffers on deserialization. Its public tuple field exposes
the actual BigUint for consumer arithmetic. Binary decoder limits do not govern
separate Serde deserialization; consumers retain that input-admission boundary.

## Evidence and remaining gates

`make qualify-marketplace` passes 24 product tests with this feature, two focused
public-consumer tests, strict Clippy, Rustdoc and consumer Wasm compilation on
Linux x86-64 / Rust 1.100.0. Literal frames independently specify the selected
record/variant grammar and test every class, 2^140, zero, signed extremes,
optional scopes, all truncated prefixes, unknown variants, incompatible payloads,
arity, trailing data and byte/header/type/work/allocation/vector refusals.
Primitive natural checks establish exact/one-below logical allocation and work
charges, non-minimal zero admission and truncated/signed-type refusal.

Native-only published-oracle tests compare all six classes with 256-digit prices,
canonical provider identity, preserved text and exact Serde values. JSON tests
compare integers, strings and limbs with published Nat, including invalid values.
The separate [Wasm corpus](marketplace-wasm.md) now executes marketplace values
and refusals, including 32-bit count overflow. Tagged-byte Serde fixtures, physical heap comparisons,
actual Query adoption and hosted results for the new source remain pending.
Availability responses and its nine application errors now have a separate
implementation slice, including bounded page requests. Publication remains disabled at
initial version 0.1.0.
