# Ledger protocol composition

## Scope and author inputs

The maintainer requested completion of the two remaining embedded ledger source
replacements on 2026-11-10. This replaces protocol framing/composition only;
SHA-224, SHA-256 and CRC32 remain the existing registry implementations. There
is no new dependency, public type, feature, numeric range or external format.

Two separate implementation agents received public API signatures, value
variants, factual protocol requirements and compatibility constraints. A third
agent independently wrote public consumer tests from those inputs. None read
the previous implementations, imported tests, registry ledger source or Git
history. The parent had read the previous source and handled specification,
integration, documentation and validation. The parent corrected the test output
path to the workspace target directory, formatted the drafts and added API docs.
These records describe technical author/context separation only.

Protocol references are the [ICP ledger account specification](https://legacy.internetcomputer.org/docs/references/ledger)
and [ICRC-3 value hashing specification](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-3/README.md#value-hash).
The supplied requirements were facts, not copied implementations or expressive
test suites. Existing public signatures and compatibility observations supply
the narrower legacy contract below.

## Preserved behavior

ICP account framing retains the length-prefixed `account-id` domain, owner bytes
and 32-byte subaccount as SHA-224 input. Its output remains a big-endian CRC32
prefix followed by that 28-byte digest. There is no intermediate heap buffer.

Value hashing retains raw blob/UTF-8 inputs, natural-number unsigned LEB128,
ordered array child digests, and lexicographically sorted key/value digest pairs
for maps. Arbitrary-size Nat framing reuses the independently owned payload
encoder. ICRC3 values are hashed directly without rebuilding a Value tree.

The existing `Int` compatibility rule is deliberately preserved: only signed
128-bit values hash; larger positive/negative values panic with the same text.
Negative values use signed LEB128; nonnegative values use unsigned LEB128.
That positive-integer behavior differs from the generic ICRC-3 signed framing
at some boundaries. This replacement does not silently migrate persisted hashes
or assert complete current-standard behavior.

## Before/after qualification

Before replacement, both original source guards passed. The previous unqualified
ICP draft was preserved under `target/ledger-composition/before/`; the original
recorded constructor was restored solely to produce the comparison baseline.
The independent cases passed against those original implementations before the
replacement drafts were installed with fresh destination mtimes.

The frozen [observation fixture](../../crates/ic-platform/tests/fixtures/ledger-hash-observations.json)
contains 314 Value hashes, 314 ICRC3Value hashes, 150 ICP account identifiers and
four exact panic results: 782 observations, 79,160 bytes. Its SHA-256 is
`9924455c39e72a6ab3fce5981bed92cbacf97af0869b9463a86882018347066b`.
The fixture was captured from the originals and is unchanged after replacement.
The ignored capture test writes only under workspace target; normal tests never
regenerate the committed expectation.

Cases include integer sign/group boundaries through i128, patterned and
power-boundary naturals through 8192 bits, empty/nested values, Unicode/NUL text,
map digest ordering, array ordering and all principal byte lengths with five
subaccount patterns. Independent protocol assertions supplement frozen outputs.

Native and Wasm Clippy pass. All 20 focused tests pass: six ledger composition,
one existing Canic account destination, seven Value public-contract tests and
six Nat payload tests. The sole ignored test is the explicit observation writer.
No installed-canister or native macOS execution was performed. No performance
or Wasm-size result is inferred from these checks.

Native/Wasm normal-and-build dependency trees are byte-identical before/after,
and Cargo.lock is unchanged. Tooling admission, pins, inventory, documentation
links and Candid/CBOR/Cargo boundaries pass. Workspace Rust formatting and
diff-whitespace checks pass. Two pre-existing codec assertions received only
rustfmt line wrapping during the whole-tree formatting check.

Original imported hash/integer bodies and their four upstream tests are retired.
The obsolete ICP unit-test file and the two constructor/hash fingerprint checkers,
their mutation script and unused SourceFingerprint helper are removed. Ordinary
Cargo tests, including the new integration target, remain in the release gate.
Historical source records remain in Git; current ownership is in the
[dependency inventory](../architecture/dependency-inventory.md).

Raw logs are `/tmp/platform-ledger-*.log`; snapshots and graph evidence are under
`target/ledger-composition/`. Full release qualification and delivery are separate.
