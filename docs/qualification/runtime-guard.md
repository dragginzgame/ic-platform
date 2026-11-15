# Runtime Cargo singleton guard

The real `ic-platform-runtime-guard` package joins Runtime's release group at
0.23.3, with publication disabled. Runtime's `call-ic0` feature enables its exact
path/version dependency; defaults and call-only preparation do not select it.
The guard implements a Cargo-resolution exclusion and a compile-time cohort
version check. It is not a placeholder package for tooling.

## Baseline and ownership

The official crates.io sparse index was checked on 2026-11-15. The latest
non-yanked guard was 0.22.5, archive checksum
`93880ece1df84dd6462c4b42e7b454c603d216e3f8f2265f258f1153a82cd842`.
The replacement moves into the next 0.23 minor family and uses 0.23.3 to match
the current coordinated Runtime group. The published baseline is used only as
a pinned graph-conflict oracle, not as a product dependency.

The published `links` identity is deliberately retained verbatim:
`ic-cdk async executor, see https://github.com/dfinity/cdk-rs/blob/links-pin/TROUBLESHOOTING.md`.
Changing that identity would permit old/new guarded suppliers to coexist.
This is necessary compatibility metadata, not a bulk source migration. The new
implementation is local, under the repository's canonical licence; sibling
checkouts remain read-only. Cargo requires a build script for a `links` package;
ours only declares its manifest input and builds no native library.

The [Cargo links rule](https://doc.rust-lang.org/cargo/reference/build-scripts.html#the-links-manifest-key)
allows one package with a given links value in a resolved graph. Runtime also
evaluates the guard's `assert_runtime_version` in a const initializer, using its
own package version, so an accidentally mismatched guard cannot compile merely
because a manifest was edited to accept it. This check is not a substitute for
source identity: equal version strings alone remain insufficient.

## Evidence

`make qualify-runtime-guard` checks the guard on native and Wasm targets, runs
three version-check tests and strict Clippy, then resolves isolated manifest
probes. The probes reference real product source paths without copying or
generating Rust code. Temporary manifests and locks are removed afterward.

Five graphs succeed: default Runtime, call-only Runtime, IC0 Runtime, the pinned
published guard alone, and an alternate-source guard alone. The selected
Runtime edge contains zero guards without IC0 and exactly one guard with IC0.
Two graphs fail specifically on the expected `links` identity: IC0 Runtime plus
published 0.22.5, and IC0 Runtime plus a second source at the same version.
Independent positive controls prevent missing sources or invalid manifests
from masquerading as singleton rejection.

Normal probes are offline. Explicit `--prepare` permits Cargo to fetch the exact
published guard control; it does not refresh workspace locks. CI prepares this
oracle after ordinary locked source fetches, then runs the offline checks.

Local Rust 1.100.0 / Linux / Node 24.21.0 passed all guard probes and tests,
25 Runtime tests, existing Wasm ABI corpora, dependency graphs, formatting,
workflow lint and group metadata checks. Hosted/macOS evidence for this increment
is pending. The product and qualification locks admit only the new local guard;
the published oracle stays in temporary graph probes.

## Limits

This gate rejects conflicting guarded suppliers during Cargo resolution. It
does not detect an executor that omits or changes the shared guard, or arbitrary
source modifications that make two Runtime copies share one guard package.
Final-artifact singleton qualification, generated entrypoint ownership and
installed callback/rollback proof remain required. No second executor is linked
into a successful qualification artifact.
