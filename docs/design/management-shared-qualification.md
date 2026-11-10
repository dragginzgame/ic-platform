# Independent shared management DTO qualification

Subsequent fresh-build check: value-helper qualification exposed Cargo reuse of
a moved draft whose mtime predated the baseline build. The coordinator rewrote
the unchanged management source bytes to advance its source mtime and recompiled
this selection. All 23 management tests and native/Wasm Clippy pass again.
Logs `/tmp/platform-management-shared-fresh-*.log` supersede the earlier execution
evidence; fixture and source contents are unchanged.

## Scope and source separation

The authorized implementation author replaced the shared management declaration
module from the reviewed [API contract](management-shared-contract.md), without
reading its old source, tests, codec or supplier implementations. The
[implementation record](management-shared-implementation.md) identifies the
author's inputs and limitations. A separate author supplied four
[public consumer tests](management-shared-public-tests-implementation.md) from
the same contract. The parent integrator has read the original source; its role
was specification review, baseline capture and compatibility qualification.
These records describe context separation, not legal clearance.

The replacement has 49 passive structs/enums and two aliases. Together with the
unchanged TransformFunc adapter they retain all 52 shared schemas. Root exports
remain 44 names on Wasm and 62 on native. Field/variant order, trait sets,
defaults, numeric ranges and Serde attributes match the extracted API facts.
No validation, cryptographic algorithm, transport, storage or timer policy was
introduced. EcdsaCurve's only conversion preserves cost codes 0 and 1.

## Focused checks

The original baseline was the existing dirty checkout over
`a37b15165a4f87a99f2845cb8263ad565f4a4864`, not a published archive. Original
source, compiler, lockfile, graphs and unchanged-boundary hashes are retained in
`target/management-shared.before`; after graphs are in
`target/management-shared.after`. Compiler: Rust 1.99.1, local Linux.

Cargo selections used `--offline --locked -p ic-platform --no-default-features
--features management-canister-types`.

| Check | Evidence |
| --- | --- |
| Original management unit/oracle tests | 8 passed |
| Original existing public management/native tests | 6 + 5 passed |
| New independent public tests against original source | 4 passed before replacement |
| Replacement management unit/oracle tests | 8 passed, including all 52 shared and 19 native schemas |
| Replacement public management/native/shared targets | 6 + 5 + 4 passed |
| Native library and three public targets, Clippy `-D warnings` | Passed before replacement tests |
| Wasm library, Clippy `-D warnings` | Passed |
| Native/Wasm normal/build graphs and Cargo.lock | Byte-identical before/after |
| Root exports, owned codec mappings and TransformFunc | Original SHA-256 hashes match |
| Extracted declaration API facts | Identical before/after |
| Manifest inventory, documentation links and Candid/CBOR/dependency boundaries | Passed |
| Retained ICRC source guard and mutation refusals | Passed |
| Scoped Rustfmt and diff whitespace checks | Passed |

The public baseline initially exposed a test-helper error: its byte-buffer
Deserializer omitted newtype visitation for TransformFunc. The independent
test author corrected that helper before freezing observations. No product code
was changed to accommodate the test.

The permanent fixture contains 24 grouped JSON/CBOR/Candid observations plus
three deserialize-only Candid observations and HTTP Serde input. It covers every
declared enum variant, absent/present options, Nat above u128, integer extrema,
arbitrary bytes/text and byte-buffer versus sequence distinctions. Deserialize-only
HTTP and transform types gained no Serialize implementation. The parent added
the exact fixture assertion only after the original implementation passed.

Fixture: `crates/ic-platform/tests/fixtures/management-shared-observations.json`,
47,755 bytes, SHA-256
`c680877a0bca84eb1ab62dff153a84efa74129f085e975166c38c34f8a056a75`.
Logs: `/tmp/platform-management-shared-{before,after}-*.log`.

## Retired machinery and limits

The old shared declaration implementation, its Apache license/provenance and
the management source checker/mutation fixture are retired. Their two Make
calls are removed. The new ordinary integration test is registered under
`management-canister-types`; existing release test selection already includes it.
A file-scoped CBOR allowance admits only its compatibility observations.
Retained ICRC/ICP and external-schema imports keep their separate source records.

No dependency, feature, canonical type identity or wire format changed. There is
no downstream migration for this replacement. It makes no performance or Wasm
size claim and adds no installed IC, macOS, sibling or full-release evidence.
No commit, push, release or publication was performed.
