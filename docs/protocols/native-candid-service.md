# Native external Candid service export

Platform owns export from its owned binary schemas through
`candid_codec::service::{Method, Mode, export}`. Select `candid-codec`; this module
exists only on non-Wasm targets. It adds no dependency or generic upstream
forwarder. The pinned native **didc** executable, supplied through Toko's
[IC tool setup](https://github.com/dragginzgame/toko-tooling/blob/5806e6e4943f6be600cd53385498ce08a5f804c4/docs/tools/ic-tools.md), owns DID parsing and structural service equality.
Neither this module nor didc restores native method registration or reflection.
Host Tools' old reflection APIs are not the owner for this migration.

## Consumer migration

This implements [#153 — Document the native external-service schema/export owner](https://github.com/dragginzgame/ic-platform/issues/153).

Blob's required Caffeine certificate and GC callbacks remain Candid. Keep
`provider_candid_interface()` and `examples/export_candid.rs` when callers need
a generated external DID. Replace their `candid_method` attributes,
`export_service!` and `__export_service()` with descriptors emitted by the same
`mount_service_endpoints!` catalogue that emits the mounted functions. Do not
hand-maintain another method roster. The descriptor's name, mode, argument types
and result types must come from that catalogue expansion; owned Wire methods
must not be labelled as Candid exports.

```rust
use ic_platform::candid_codec::service::{self, Method, Mode};

// Illustrative expansion of one external entry in the consumer-owned catalogue.
const METHODS: &[Method] = &[
    Method::new::<(Vec<Vec<u8>>,), (Vec<bool>,)>(
        "_immutableObjectStorageBlobsAreLive", Mode::Query,
    ),
];
let did = service::export(METHODS, 16_384)?;
# Ok::<(), ic_platform::candid_codec::Error>(())
```

Use tuple *sequences*: `()` for no arguments/results, `(T,)` for one value,
`(A, B)` for two. A unit value is `((),)`, not an empty reply. A tuple-record
result is `((A, B),)`. `Method::new` uses `EncodeArgs::TYPES`, including canonical
Principal/Nat and the actual owned record/variant mappings. Numeric field labels
and generated alias names can render differently from the frozen DID while
preserving its schema. Export refuses duplicate method names, invalid schemas,
oneway results and output overflow; callers handle the returned error.

The exporter cannot inspect linked Wasm exports or prove that a consumer's
catalogue was mounted. Keep the consumer's mounted-provider and installed
callback checks. Platform's public test emits both executable fixture functions
and descriptors from one catalogue, and independently compares the four Caffeine
methods with frozen text. It separately qualifies owned Principal/Nat schemas.

## Native equality and malformed-input checks

Prepare the existing pinned executable explicitly, then check offline:

```sh
make install-ic-tools IC_TOOLS=didc
make ic-tools-check IC_TOOLS=didc
.tools/ic/bin/didc check --strict generated.did frozen-external.did
```

Require `--strict`: one-way upgrade compatibility does not prove exact external
service equivalence. Replace whitespace-stripped/string equality with this
structural equality check; retain the frozen external DID as the independent
expected contract. Mismatched methods, signatures, modes or arity and malformed
DIDs must exit unsuccessfully. This uses a native executable, not a consumer
Cargo Candid/parser exception. Do not copy a parser, printer or reflection facade
into Blob or another caller. The only Rust parser dependencies used by Platform's
own qualification remain its existing development-only oracles.

`make check-candid-service` runs the public native fixture and those positive and
negative cases through the selected didc; it requires the explicit setup above
and never installs tools. Ordinary `cargo test -p ic-platform --features
candid-codec --test candid_service_consumer` runs the same structural assertions
using the development oracle without requiring a CLI installation.

Textual *argument* parsing is a different capability and is not provided by this
API. Retire old Platform parsing calls in fixtures that can construct typed
values: use `arguments::encode_args`, preserve golden DIDL bytes, and retain
malformed binary and independent byte/work/allocation/skip rejection tests.
This is not permission to remove an actual textual-input product contract or
replace its bounded parser with an unbounded subprocess. Such a product still
needs explicit native input admission and its own qualified parser owner; didc's
trusted generated/frozen-DID workflow does not provide those runtime guarantees.

Authentication, callback behavior, external wire bytes, application limits and
installed acceptance remain consumer-owned. This change needs no Types or
Runtime release and provides no automatic caller migration or release authority.
