# Independent entrypoint generator qualification

Subsequent tooling adoption resolves the earlier IC0 admission and root-file
metadata blockers. This report retains its original qualification scope; see
[the current handoff](../status/current.md) for remaining work.


Local Linux qualification on 2026-11-10 covers the independently implemented
ordinary entrypoint generator, dispatch and syntax helpers. The existing owned
external-Candid adapter changes only two private module references. No dependency,
public feature, codec policy, executor or cryptographic implementation changes.

## Source separation and scope

The [contract](entrypoint-generator-contract.md) was the implementer's source of
API and behavior requirements. The [implementation record](entrypoint-generator-implementation.md)
records permitted inputs and review corrections. A separate context authored
[public consumers](entrypoint-public-tests-implementation.md) without reading
either generator. The coordinator inspected and integrated the result. These
records describe source separation; they do not establish legal clearance or
exclude model training exposure.

The previous export directory, its tests and macro license file are retired
together. The external-Candid adapter and its three tests remain. Retained
management/ledger imports keep their notices and guards. Replacement first-party
license text and corresponding manifest metadata are still pending; this is not
publication readiness.

## Focused checks

Rust 1.99.1 (`58cf8f9c35b355199727627d07ae34a4e2846d67`), LLVM 23.1.1,
`x86_64-unknown-linux-gnu`:

- Macro all-target Clippy with warnings denied passes. Sixteen unit tests,
  ten public consumer tests and one native-link test pass: 27 total.
- Runtime native all-target Clippy and Wasm library Clippy with `cdk` pass.
  Fifty-two unit tests, seven public call contracts, three CDK consumers,
  thirteen public executor contracts and two doctests pass: 77 total.
- The original eight public consumers and native-link test also passed against
  the previous generator before replacement. Two later inference/hygiene cases
  and debug-environment coverage are replacement regressions, not old-owner
  baseline evidence.
- `scripts/ci/check-platform-runtime.sh --prepared-call` passes all four
  raw/prepared × Wire/external-Candid installed profiles. Evidence is retained at
  `target/cdk-runtime.EpTK6Z`, status 0, with identical before/after source digests.
- Feature-closure checks pass for `ic0,executor,runtime,candid-codec-endpoint`:
  17 selections represented by six unique Cargo graph queries on each of native
  and Wasm, retained at `target/feature-graphs/attempt.m3vG43`. Inventory,
  documentation links, macro Rustfmt and scoped diff-whitespace checks pass.

The installed scenarios cover explicit codec admission, guard-before-decoding,
scalar/tuple/unit replies, management awaits, callback traps and multiple-call
recovery, cancellation, upgrades, cycles/refunds, stable rollback and certificates.
The native-link fixture checks exact crate-prefixed symbols, including lifecycle,
Unicode, punctuation and repeated endpoint names across crates. It also checks
debug output when `IC_CDK_DEBUG` is present with an empty value.

Logs for these checks are under `/tmp/platform-generator-*.log`; durable
contracts, tests and measured results are recorded here. No native macOS run or
Scheduling-owned composition qualification was performed. Darwin symbol
normalization in a test is not macOS evidence.

## Matched Wasm comparison

Earlier baseline artifacts had been cleaned. Therefore the old generator was
reconstructed from commit `a37b15165a4f87a99f2845cb8263ad565f4a4864` into a
disposable generated fixture, using the frozen inputs from the successful
installed run above. Only the macro package was exchanged between phases, at
the same source path. Fixture sources, dependency lock, compiler, feature
selections and Cargo release settings were identical. No wasm-opt, stripping or
compression was applied. The original runtime driver installed and exercised
all four artifacts in each phase; all eight runs passed.

Evidence: `target/generator-comparison.FQ01yz`, status 0. It retains both macro
sources, fixture inputs, compiler/revision records, `profiles.log`, `sizes.tsv`,
`artifacts.sha256` and the eight binaries. Section sizes were measured by the
repository's `scripts/benchmarks/wasm-sections.pl`.

| Profile | Total before | Total after | Change | Code before | Code after | Change |
|---|---:|---:|---:|---:|---:|---:|
| Raw Wire | 229,603 | 228,731 | −872 | 152,874 | 152,423 | −451 |
| Raw external Candid | 679,111 | 678,921 | −190 | 475,778 | 475,816 | +38 |
| Prepared Wire | 231,623 | 231,674 | +51 | 154,412 | 154,287 | −125 |
| Prepared external Candid | 680,815 | 680,855 | +40 | 477,214 | 477,206 | −8 |

All numbers are bytes. Node's `WebAssembly.Module.imports/exports` reports exact
before/after equality for every profile: 49 imports each, 48 exports for Wire
and 66 for external Candid. Full compared descriptors are in `interfaces.json`.
The comparison paths differ from the ordinary installed build, so its total
bytes need not equal these totals; only the same-path pair is the size comparison.

This replacement removes imported implementation ownership with a small mixed
size effect. It is not a general application-size or execution-speed claim.
No instruction count, latency or heap benchmark was performed.
