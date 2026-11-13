# Platform qualification workspace

This private package is excluded from the five-package product workspace and
all three release groups. It has its own catalog and lockfile; nothing in
the product workspace depends on it.
The candidate/baseline backends and Runtime IC0 are Wasm-only dependencies;
the native measurement driver selects Testkit, its oracle and Tokio separately.

- `bls-wrapper-probe`: identical baseline/candidate Wasm entrypoints and a native
  Testkit driver. Its key cache exists only to measure caller-owned preparation.

See the [implementation contract](../docs/design/bls-wrapper/contract.md) and
[qualification report](../docs/design/bls-wrapper/qualification.md). The
implementation and comparison tests now belong to `ic-platform`; the
experimental implementation package was removed after adoption. The baseline
selects the latest adopted published Platform (currently 0.32.0), and the candidate
selects this checkout. Both arms use the same prepared-key API and cache policy.
Historical reports retain their original versions and results; new runs compare
against this selected release rather than the pre-adoption implementation.
Testkit 0.102.0 selects registry Platform 0.32.0 and Types 0.20.0 for its native
measurement infrastructure. This registry generation is distinct from local
product paths and stays outside both baseline/candidate Wasm graphs.

Run from the repository root using the pinned compiler and prepared Cargo cache:

```sh
export CARGO_TARGET_DIR="$PWD/target/bls-wrapper-qualification"
cargo clippy --locked --offline -p ic-platform --no-default-features \
  --features signature-verification --lib --test bls_comparison --test bls_consumer \
  --test canister_sig_consumer --test signature_consumer --test signature_comparison -- -D warnings
cargo test --locked --offline -p ic-platform --no-default-features \
  --features signature-verification --lib --test bls_comparison --test bls_consumer \
  --test canister_sig_consumer --test signature_consumer --test signature_comparison
cargo fmt --manifest-path qualification/Cargo.toml --all --check
```

For matched Wasm sizes and installed instruction counts, select an already
prepared simulator through `POCKET_IC_BIN` or `IC_TESTKIT_POCKET_IC_URL`, then run:

```sh
bash scripts/benchmarks/bls-wrapper.sh
```

The runner checks native driver and Wasm Clippy, builds both arms with identical
compiler/optimization settings, captures normal/build feature graphs, rejects
retired candidate dependencies and verifies source stability. It never downloads
a simulator. Raw artifacts/logs remain under `target/evidence/bls-wrapper.*`;
build scratch stays under the root `target/` and is reclaimable by Cargo cleanup.
Do not edit sources or run a second copy against its target cache during a run.

Native Testkit/oracle dependencies are measurement infrastructure, not candidate
Wasm dependencies. Inspect the wrapper's deployed closure independently:

```sh
cargo tree --locked --offline -p ic-platform --no-default-features --features bls \
  --target wasm32-unknown-unknown --edges normal,build
```
