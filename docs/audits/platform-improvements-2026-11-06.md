# Platform improvement qualification — 2026-11-06

Source baseline: `c1dd388f0f9d40fe8819f3325c61a697d51b5a24`, Shared 0.30.5,
plus this working-tree batch and the pre-existing Candid/derive 0.10.40 lock draft.
The draft remains untouched. #50, dependency updates, sibling changes and package
publication are excluded. The maintainer subsequently authorized scoped commit/push
and hosted qualification. This records investigation and
focused evidence, not a release receipt or whole-stack security assessment.

## Completed changes

- Extend the malformed parser exercise to strict certificate/hash-tree and typed
  IC packet CBOR; check strict/upstream agreement and independent byte limits.
  Deterministic malformed/truncated/oversized/deep inputs run as an integration
  test. The opt-in sanitizer harness now uses prepared nightly-2026-11-05,
  retains its log and seeds typed ingress from the shared corpus.
- Add an eight-packet Rust/browser ingress corpus. Request IDs and complete u64
  precision are checked without mutating original bytes. Intentional differences
  remain: browser accepts extension fields; Rust's existing Serde byte adapter
  accepts text arguments. Rust strict inspection is not byte-kind-strict CBOR.
- Add a same-input Wasm comparison and asynchronous PR/dispatch workflow. Total
  uncompressed modules and code sections are compared before and after shrinking.
  Mismatched tool identities, probe sources, external locks and profile selections
  refuse. Default growth allowance is 128 bytes per measurement, not an application
  budget. Hosted execution awaits source delivery; only local evidence is claimed.
- Use `Vec::pop` for final protected-task membership removal. Non-tail removal
  remains stable; no task index, extra field, dependency or executor is introduced.
  Generation checks, cancellation order and destruction outside borrows remain.

## Performance and size decisions

Native Rust 1.99.1 release-mode benchmarks on Linux x86_64 averaged twenty task
removal batches. These are cancellation bookkeeping timings, not installed IC
instruction counts or application throughput:

| 8,192 protected tasks | Before | After |
| --- | ---: | ---: |
| Forward cancellation | 10.953 ms | 10.923 ms |
| Reverse cancellation | 7.555 ms | 0.141 ms |

Identical probe sources, dependency resolution, Rust 1.99.1, `opt-level=3`, fat
LTO, one codegen unit and `ic-wasm shrink` produced:

| Probe | Uncompressed module before → after | Code section before → after |
| --- | ---: | ---: |
| Runtime, before shrinking | 42,920 → 42,980 B | 36,955 → 37,015 B |
| Runtime, after shrinking | 39,030 → 39,088 B | 33,065 → 33,123 B |
| Verification, after shrinking | 598,477 → 598,477 B | 562,368 → 562,368 B |
| Strict tree decoder, after shrinking | 96,984 → 96,984 B | 87,150 → 87,150 B |

Evidence: `target/evidence/wasm-regression.npS2bm/`. Its initial zero-allowance
comparison correctly refused the 58-byte increase; the explicit 128-byte allowance
passes. The unchanged verification/decoder results validate removal of these
prototypes, not a size saving:

- Borrowing the signing certificate with a Serde byte `Cow` preserved verification
  tests but added 8,974 module / 8,934 code bytes after shrinking. Reverted.
- A fixed-size pruned-digest visitor preserved strict rejection tests but added
  8,674 module / 8,619 code bytes after shrinking. Reverted; retain useful boundary
  tests. Initial experiment source patches and hashes remain in
  `target/evidence/wasm-regression.PjGl2u/`.

Candid's shared-type-path benchmark measured about 0.61 ms for a sixteen-level
branching graph with a one-million work allowance, and about 3 microseconds to
refuse it with a 1,000-unit allowance. Quota decisions remain unchanged. Keep the
benchmark; no memoization implementation is admitted. A future optimisation must
preserve work/depth error decisions and demonstrate a worthwhile Wasm tradeoff.

## Validation and remaining qualification

Affected Platform/Runtime Clippy passes. Platform's selected unit suite passes
50 tests (one manual benchmark ignored), plus the malformed-CBOR integration test.
Runtime executor tests, source/provenance checks, inventory, boundary checks,
formatting, browser ingress and comparison-refusal tests pass. The final-source
sanitizer campaign completed 420,463 executions in 61 seconds under host permissions
(`target/parser-fuzz.lHTGSY/`); an earlier sandbox LeakSanitizer inspection refusal
is not counted as a pass. Later seed-list additions do not claim another campaign.

Installed owner and Timers profiles pass raw awaits, callback/trap cleanup,
cancellation, upgrade, rollback and external-protocol checks against the prepared
Testkit server. Evidence: `target/cdk-runtime.jku1ew/` and
`target/cdk-runtime.Zr6pbu/`. Host socket permissions were required; no refusal was
waived. Compact command logs are under `target/evidence/audit-improvements/`.

Release-group activation remains incomplete: the existing qualification runner
requires clean committed source, followed by full/selected and recovery evidence
on Linux and both supported macOS hosts. Authorized integration excludes the user's
lock draft and does not enable unqualified omissions.
`ci/release-validation-hosts.json` remains empty. Use the existing Release validation
qualification workflow after integration; no new release system is needed.

Native Profiler source is unchanged. Existing #140 Linux proof remains historical
evidence; macOS and Bazel execution are still unavailable here (Linux host, no
Bazel executable). Root-workspace checks do not substitute for either. No native
provider pin, sibling checkout, release group or published version was changed.
