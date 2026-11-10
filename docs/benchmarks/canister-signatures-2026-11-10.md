# Canister-signature comparison — 2026-11-10

The independent Platform modules remove upstream Candid and registry Principal
from both signature production graphs while preserving canonical Types Principal,
public APIs, key bytes, DER/identity-key admission and authenticated signing time.
BLS/curve operations, SHA-256 and certified-tree hashing remain external packages.
DFINITY creation 1.3.1 and verification 0.3.0 are development-only references.
This includes native measurements and installed PocketIC instruction/cycle
measurements. It is not a release or live-network acceptance result.

## Native latency

Rust 1.99.1 (`58cf8f9c3`), default Cargo release optimization, x86_64 Linux,
AMD Ryzen Threadripper 7970X. Each workload warms up 16 calls per implementation,
then takes nine samples in alternating implementation order using black-box
inputs/results. There are 50,000 iterations/sample for key operations, 10,000
for hashing and 100 for verification. No competing build was run during timing;
CPU frequency and scheduling are not pinned. Full min/median/max values and input
sizes are in [native.tsv](canister-signatures-2026-11-10/native.tsv).

| Workload | Owned median | DFINITY median | Owned / DFINITY |
| --- | ---: | ---: | ---: |
| Raw key encoding | 15.5 ns | 49.4 ns | 0.315 |
| DER key encoding | 25.1 ns | 131.7 ns | 0.190 |
| Raw key decoding | 21.3 ns | 37.3 ns | 0.570 |
| DER key decoding | 23.4 ns | 41.6 ns | 0.563 |
| SHA-256, 32 bytes | 41.9 ns | 46.2 ns | 0.906 |
| SHA-256, 4,096 bytes | 1.796 µs | 1.782 µs | 1.008 |
| Direct verification, 264-byte signature | 1.924 ms | 1.679 ms | 1.146 |
| Legacy delegated verification, 601 bytes | 3.787 ms | 3.297 ms | 1.149 |
| Sharded delegated verification, 627 bytes | 3.859 ms | 3.360 ms | 1.149 |

Key operations are faster in this run. Verification is about 15% slower than
DFINITY's raw verifier. The owned path additionally admits root/delegated keys
with checked point parsing and identity refusal; the reference timing calls its
unmodified public verifier, which lacks those refusals. This does not establish
that protocol composition itself regressed by 15%, nor isolate the cost of any
single check. No security check was removed for a benchmark result. The 4 KiB hash
results are effectively similar within observed run dispersion. Owned SHA-256
uses the selected workspace 0.11 supplier; DFINITY selects 0.10.

## Installed Wasm instructions and cycles

PocketIC server 16.1.0 and the selected Testkit 0.92.0 run three independently
built canisters on a local application subnet: owned, raw DFINITY, and DFINITY
with Platform's additional root/delegated identity-key admission. The last arm is
a benchmark adapter using external checked point parsing and certificate
decoding. It models that additional work, not an exact historical binary or
universal equivalence of malformed-input admission. All arms use Rust 1.99.1,
release optimization 3, fat LTO, one codegen unit and the same `ic-wasm shrink`.

Each workload starts on a fresh installation. Sample zero is cold; the table
uses the median of three subsequent updates. An update runs 1,000 operations
for key/hash cases or one for verification. Instruction windows use
`performance_counter(0)` around the operation loop, subtract the matching
no-op window, and divide by repetitions. Whole-update cycles are measured
separately by the canister balance debit; they include input preparation,
reply and update overhead and are **not** isolated verifier cycle charges.

| Verification | Owned instructions | Raw DFINITY instructions | DFINITY + identity admission instructions |
| --- | ---: | ---: | ---: |
| Direct | 245,671,180 | 211,620,449 | 245,688,006 |
| Delegated, legacy ranges | 491,305,225 | 423,202,090 | 491,340,158 |
| Delegated, sharded ranges | 491,330,801 | 423,227,442 | 491,368,294 |

With the additional identity checks present on both sides, owned verification
uses within **0.01%** of the reference instruction count. Raw DFINITY is about
16% cheaper in instructions, but does not perform these required checks. This
comparison explains the earlier native gap much better than treating it as a
protocol rewrite penalty; it does not isolate every individual operation.

| Verification | Owned warm update cycles | DFINITY + admission warm update cycles | Owned cold update cycles |
| --- | ---: | ---: | ---: |
| Direct | 252,627,575 | 252,646,306 | 266,227,457 |
| Delegated, legacy ranges | 498,261,620 | 498,298,458 | 511,861,609 |
| Delegated, sharded ranges | 498,287,878 | 498,326,594 | 511,887,233 |

The first verification costs about 13.6 million additional cycles in these
installations. These are local simulator balance deltas, not an exact mainnet
bill for arbitrary subnet configuration, certificate size or application code.

| Other operation | Owned instructions/op | DFINITY instructions/op | Owned difference |
| --- | ---: | ---: | ---: |
| Raw key encoding | 616.5 | 1,282.5 | −51.9% |
| DER key encoding | 1,075.5 | 2,656.5 | −59.5% |
| Raw key decoding | 625.5 | 624.5 | +0.2% |
| DER key decoding | 1,071.5 | 1,429.5 | −25.0% |
| SHA-256, 32 bytes | 5,783.5 | 5,867.5 | −1.4% |
| SHA-256, 4 KiB | 333,528.5 | 317,255.5 | +5.1% |

The native raw-decoding improvement does not carry through to Wasm. The larger
hash is modestly more expensive with the selected external SHA-256 supplier.
Wrong-message refusal takes 21,639 owned instructions versus 70,330 raw reference;
the invalid identity-root fixture takes 34.12 million versus 211.62 million,
because owned admission refuses before signature verification. These fixtures
do not establish costs for every invalid input or include a signature store.

All 156 installed updates completed with their expected success/refusal result.
The three Wasm selections and native driver passed Clippy before execution;
owned probe dependency admission and source identity checks passed. Raw rows
are in [instructions.tsv](canister-signatures-2026-11-10/instructions.tsv), with
[derived summaries](canister-signatures-2026-11-10/cycles-summary.json) and
[input hashes](canister-signatures-2026-11-10/instructions-inputs.sha256).
Retained server/artifact hashes and validation logs accompany them. Reclaimable
binaries and complete metadata are under
`target/evidence/signature-instructions.Fknhmz/`. The sandbox initially refused
the server's local listener; the prepared driver then ran with approved local
execution against the same artifacts.

## Wasm footprint

The same compiler builds standalone probes with optimization level 3, fat LTO,
one codegen unit, aborting panics and stripped debug information. `ic-wasm 0.11.1
shrink` processes both artifacts identically. The creation probe encodes/decodes
a 29-byte issuer and 32-byte seed, then hashes the raw key with a domain. The
verification probe uses the same 627-byte sharded delegation fixture for both
implementations and accepts a runtime choice of correct/changed message. These
are linked probes, not complete application canisters or instruction counts.

| Probe | Owned raw total | Reference raw total | Owned shrunk total | Reference shrunk total | Shrunk difference |
| --- | ---: | ---: | ---: | ---: | ---: |
| Creation | 71,890 B | 67,796 B | 61,896 B | 58,376 B | +6.0% |
| Verification | 560,055 B | 561,974 B | 478,307 B | 473,465 B | +1.0% |

The complete [wasm.tsv](canister-signatures-2026-11-10/wasm.tsv) also records code
and custom-section payloads. Shrunk code payloads are 37,875 vs 34,905 bytes for
creation and 442,144 vs 437,149 bytes for verification. All shrunk custom-section
payloads are zero. Removing Candid from the dependency graph did **not** make
these linked workloads smaller. The probes also differ in their retained
admission and Principal/hash implementations; no single-dependency size saving
is inferred from the comparison.

The installed three-arm benchmark also supplies a matched complete canister
comparison, retaining all key/hash/verification benchmark entry paths:

| Optimized installed benchmark | Total uncompressed bytes | Code-section payload bytes |
| --- | ---: | ---: |
| Raw DFINITY | 480,792 | 443,345 |
| DFINITY + identity admission adapter | 606,324 | 568,430 |
| Owned | 483,325 | 446,034 |

Against the identity-admitted adapter, owned saves **122,999 total bytes (20.3%)**
and **122,396 code bytes (21.5%)**. Against raw DFINITY, owned adds 2,533 total
and 2,689 code bytes. Artifact checksums were revalidated before extracting these
sections with `scripts/benchmarks/wasm-sections.pl`; the rows are retained in
[installed-wasm.tsv](canister-signatures-2026-11-10/installed-wasm.tsv).
The adapter is a model of the extra checks, not the exact former Platform
artifact. Its extra decoding and retained implementations affect size, so the
122,999-byte difference cannot be attributed solely to Candid removal. The
[refreshed codec comparison](candid-codec-2026-11-10/matrix.md#refreshed-complete-removal-size-check)
measures the larger saving from replacing actual upstream codec calls. These
isolated deltas are not additive predictions for an application canister.

Including the probe root and build dependencies, the selected graphs contain
30 vs 52 unique package/version pairs for creation and 40 vs 60 for verification.
These counts describe dependency maintenance, not binary size. Both owned native
and Wasm production graphs exclude Candid, registry Principal, registry signature
composition, IC0 and an executor. The production all-feature Wasm graph also has
no Candid path. Native Agent/transport and development/qualification oracles still
use Candid; they are outside this signature replacement.

## Correctness and ownership evidence

The [reviewed contract](../design/canister-signatures/contract.md) defines the
scope, sources, public constants, compatibility behavior and context separation.
The implementer started without the specification author's source-exposed
conversation, read no old signature-module or DFINITY signature-composition
bodies/tests, and independently wrote the modules and protocol tests. Public
constants and signatures were supplied as factual inputs. One declaration search
exposed the retained BLS supplier's three-line delegation wrapper; this was
reported, was outside the replaced protocol modules, and was not incorporated.
This records engineering provenance, not a legal clearance claim.

Validation includes:

- Six independent creation tests and twenty verification tests, including the
  retained Platform regressions and ten independently authored protocol tests.
- Three public creation tests, two public verification tests and three new
  differential matrix tests. The explicit timing test is normally ignored.
- The key matrix spans every issuer length 0..29 and sixteen seed lengths
  (480 combinations), including empty, short-form boundaries and large seeds.
  Raw/DER encoding bytes and raw decoding match the public reference.
- Hash comparisons span six domain lengths and six payload sizes; decoder
  comparisons cover 455 raw-key shapes, 151 root lengths and every root-prefix
  byte mutation. DER admission intentionally remains stricter than DFINITY.
- Direct and both delegated-range signatures, mutations, truncations, wrong
  messages/keys, root/delegated identity keys, nested delegation, packed/sequence
  CBOR, duplicate fields, time visibility/overflow/nonminimal encodings and
  owned Principal range ordering. The retained range/Serde matrix covers 8,100
  Principal pairs without selecting the native Agent capability.
- Native/Wasm Clippy for both capability graphs, all four Wasm probe selections,
  143 graph admission cases, feature selection/refusal tests, runtime oracle
  rejection and Candid/CBOR source policy checks.

The independent decoder rejects duplicate required certificate fields and refuses
an unusable selected shard without falling back to a less-specific shard or the
legacy layout. These are fail-closed framing choices; the tests do not establish
universal equivalence on every malformed representation. The migration preserves
the selected supplier's length-first Principal interval ordering and current
subnet policy; it does not claim implementation of every newer live-spec rule.
Trust roots, freshness, authority and input budgets remain consumer-owned.

## Reproduce

```sh
cargo test --locked --offline -p ic-platform --no-default-features \
  --features signature-verification --lib
cargo test --locked --offline -p ic-platform --no-default-features \
  --features signature-verification \
  --test canister_sig_consumer --test signature_consumer --test signature_comparison
cargo test --locked --offline --release -p ic-platform --no-default-features \
  --features signature-verification --test signature_comparison \
  benchmark_signature_protocol -- --ignored --exact --nocapture
bash scripts/benchmarks/canister-signatures-size.sh
POCKET_IC_BIN=/explicit/prepared/pocket-ic \
  bash scripts/benchmarks/canister-signatures-instructions.sh
```

[Input checksums](canister-signatures-2026-11-10/inputs.sha256) bind the measured
source/manifest/lock inputs; [validation log](canister-signatures-2026-11-10/validation.txt)
retains the focused outputs. Wasm binaries, graph and compiler/optimizer logs
remain under `target/evidence/signature-size.dYBDJz/` and are reclaimable build
artifacts. Native raw timing output was captured in
`/tmp/platform-signature-native-benchmark.log`; its full timing rows are retained
above. No commit, publication, sibling adoption or live-network benchmark is
implied. Pre-existing unrelated dirty work remains outside this qualification.
