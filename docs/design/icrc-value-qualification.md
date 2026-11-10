# Independent Value helpers and retained hash qualification

The subsequent [ledger-composition replacement](ledger-composition-qualification.md)
retires the temporary imported hash boundary and source guards described below.
The original helper qualification remains historical evidence.

The original source-replacement evidence below is followed by the authorized
[bug fixes](#follow-up-bug-fixes). Empty-account panic and Int Serde rejection
are no longer current behavior.

## Scope and authorship

The [contract](icrc-value-contract.md) supplies public declarations and behavior
facts to a separate [implementation author](icrc-value-implementation.md).
A second [test author](icrc-value-public-tests-implementation.md) wrote independent
public consumers without reading old/new implementations, guards or vectors.
The parent integrator read the original source, reviewed the contract, captured
original observations and integrated the result. These records describe source
separation, not legal clearance or erased model training exposure.

Value/ICRC3Value declarations, constructors, consuming accessors/TryFrom,
Principal/Account conversions and Display now have an independent implementation.
All traits, variant order, public signatures and external bytes remain. No new
value abstraction, dependency, limit, normalization or public method was added.
Codec mappings and re-exports are unchanged. Hash ownership remains separate.

## Retained source extraction

Before replacement, the original complete-source reconstruction guard passed.
The coordinator extracted these original regions byte-for-byte:

- ICRC3Value::hash, including its existing conversion/delegation.
- Value::hash, including all hash branches and integer refusal behavior.
- Both private integer helpers and four existing upstream tests/vectors.

They now live in private `value/hash/mod.rs`. Only module/import/constant
scaffolding was supplied around the unchanged regions. The full ICRC license
and scoped notice remain. Disabled upstream property-generation/conversion tests
were retired with the old non-cryptographic helper implementation; all four
active hash/integer tests remain unchanged and passing.

The old normalized complete-value fingerprint remains historical in provenance.
The current guard protects the entire extracted module, including provider alias,
imports, constants, delegation, helpers and vectors. Normalized fingerprint:
`4036880b7127aa33b69af3d34ef5800bc2c06130ba0492882d91271f9a1d7714`.
Original license and 28-schema reference DID digests remain checked. Mutation
checks reject changes to provider/imports, hash branch, signed/unsigned encoders,
delegation, vector and license. This is a reviewed narrowing of source custody
with independent replacement tests, not deletion of the crypto source guard.

## Focused evidence

Baseline was the dirty checkout over
`a37b15165a4f87a99f2845cb8263ad565f4a4864`; this does not compare published archives.
Original source, regions, guard scripts, provenance, compiler, lock and graphs
are retained in `target/icrc-value.before`; after graphs are in
`target/icrc-value.after`. Rust 1.99.1, local Linux, with all Cargo commands using
`--offline --locked -p ic-platform --no-default-features --features icrc-ledger-types`.

Clippy selected `--lib --test icrc_value_consumer` with `-D warnings` before the
replacement tests. Tests selected `--lib icrc_ledger_types::value::`,
`--lib tests::icrc_consumer` and `--test icrc_value_consumer -- --nocapture`.
Wasm Clippy selected only `--lib --target wasm32-unknown-unknown`.

| Check | Result |
| --- | --- |
| Original retained hash/integer tests | 4 passed |
| Original ICRC consumer/schema tests | 6 passed, including 28 schema comparisons |
| Independent public baseline + allocation observation | 6 + 1 passed |
| Fresh replacement hash/integer tests | 4 passed, now under value::hash |
| Fresh replacement ICRC consumer/schema tests | 6 passed |
| Fresh replacement public tests | 6 + 1 passed |
| Native and Wasm selected Clippy | Passed |
| Original/new frozen public observations | Exact match |
| Native/Wasm normal/build graphs and Cargo.lock | Byte-identical |
| Codec mappings, public re-exports, ICP helper and reference DID | Original SHA-256 hashes match |
| Retained hash source regions | Original bytes match |
| Source guard and all eight mutation refusals | Passed |
| Scoped Rustfmt, shell syntax and ShellCheck | Passed |
| Manifest inventory, documentation links and Candid/CBOR/dependency boundaries | Passed |
| Scoped diff and new-file whitespace checks | Passed |

The permanent `icrc-value-observations.json` fixture contains 15 value rows and
eight exact error cases, covering JSON/CBOR serialization and decoding outcomes,
Candid bytes, Display, ordering, recursive conversion and numeric ranges.
It is 10,428 bytes, SHA-256
`1892ac57bee36ed633ba5ed134d49d3adc846d8abab7e5b0fd29041c0fe884bb`.
The parent added the exact fixture assertion after original capture.

## Confirmed improvements and limits

The parent-owned allocation observation constructs a four-byte payload in a
256-byte-capacity Vec, wraps it in Value and converts back. Original result:
new pointer, capacity 4. Replacement: identical pointer and capacity 256,
with identical bytes. The permanent assertion protects owned-buffer transfer.
This proves removal of that copy/allocation; it is not a timing benchmark.

ICRC3Value Display now walks borrowed children directly. Source review confirms
removal of the complete tree clone/conversion and temporary blob hex String.
Output and formatting-error tests pass. No overall speed, allocation total,
production Wasm size or instruction-count claim is made.

Baseline tests exposed two assumptions that were corrected before freezing:
Nat64-to-ICRC3Value becomes Nat and therefore uses grouped numeric Display;
Int serializes as a limb sequence which its existing Deserialize rejects. The
tests now freeze those behaviors instead of normalizing input. Empty-array
Account conversion also retains its pre-existing panic (without promising a
particular panic message). These are known behavior limitations outside this
source-only replacement; no hash/carrier behavior fix was smuggled into it.

## Fresh-build correction

The initial installation moved a prewritten draft with an older mtime. Cargo
reused the original library; the new allocation assertion correctly failed and
its panic location identified the old source. That initial execution is not
accepted as replacement qualification. The parent rewrote the unchanged source
bytes and facade root to refresh Cargo's input timestamps, reran selected
Clippy, and observed explicit recompilation plus all 17 passing tests and the
new value::hash module paths. Final after-logs contain those fresh results.

The earlier shared/native management selection used the same installation
pattern, so it was also explicitly recompiled: all 23 tests and native/Wasm
Clippy pass. Its source and fixtures are unchanged. This additional check is
recorded in [management qualification](management-shared-qualification.md).
Future staged-source integration must write destination bytes rather than rely
on a rename preserving old timestamps before same-checkout Cargo validation.

Logs: `/tmp/platform-icrc-value-{before,after}-*.log` and
`/tmp/platform-management-shared-fresh-*.log`. No installed IC, macOS, sibling,
full release or new cryptographic qualification is claimed. No commit, push,
release or publication was performed. ICP derivation remains outside this work.

## Follow-up bug fixes

The maintainer explicitly requested both fixes after source replacement. This
is ordinary maintenance of the owned code by the parent integrator, not a claim
that the original isolated author supplied the follow-up changes.

- Account conversion now rejects zero elements using the same length-error
  wording as arrays longer than two. It has no panic/expect path for a missing
  owner. Existing valid cases and error precedence are preserved.
- Int's Serde visitor now delegates sequence input to num-bigint's Deserialize,
  with an explicit end-of-pair check. No serialization or Candid implementation
  changes. Integer, string and tagged-byte inputs retain prior behavior.

Both targeted regressions failed before the fixes. Afterward, four Int tests,
seven public Value tests and four retained hash tests pass (15 total), as do
selected native/Wasm Clippy. Int tests exercise both signs and zero, boundaries
through 8192 bits, exact upstream JSON/CBOR/Candid output, roundtrips, and
BigInt-equivalent admission/refusal for malformed signs, limbs and tuple lengths.
Public tests cover nested values and empty/valid/oversized Account arrays.

The original frozen observation file remains byte-identical. The comparison
changes only its expected JSON/CBOR decode outcomes to success; every serialized
byte, Candid byte, display and recorded conversion error still matches. The
empty-array error has its own explicit regression assertion. Hash source and
its license/provenance remain unchanged.

Logs: `/tmp/platform-value-bugs-{before,after}-*.log` and
`/tmp/platform-value-bugs-wasm-clippy.log`. No dependency, package version,
release group or sibling change; nothing committed or published.
