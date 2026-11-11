# Platform 0.30 deployment comparison

Measured on 2026-11-11 using the dependency-aware Platform harness. Baseline:
`a8c3fff39094d38510c46451ba1283231ee3cf37` (Platform 0.29.3 / Types 0.19.1).
After: `b45377440384caf073cbd0028b8c9b52f1893bae` (Platform 0.30.0 /
Types 0.19.2). The after tree contains the local harness/report changes only;
product source and Cargo.lock match that commit. Runtime is 0.21.1 in both.

Both arms use the current identical complete probe source, Rust 1.99.1
(`58cf8f9c3`), wasm32-unknown-unknown, opt-level 3, fat LTO, one codegen unit,
panic abort, stripped symbols and `cpubits=auto`. Post-link optimisation uses
`ic-wasm 0.11.1 shrink`. These are library probes, not whole Canic applications
or an instruction/performance benchmark. Each arm starts with its own committed
lock; the generated fixture resolves offline. Source locks are unchanged.

## Final uncompressed Wasm

All sizes and deltas are **bytes**. Code-section size is the section 10 payload,
measured by the existing `wasm-sections.pl` reader, excluding its section framing.

| Probe | Total before | Total after | Delta | Code before | Code after | Delta |
|---|---:|---:|---:|---:|---:|---:|
| Binary Principal | 14,060 | 14,060 | 0 | 12,662 | 12,662 | 0 |
| Self-authenticating Principal | 24,490 | 25,444 | +954 | 22,990 | 23,688 | +698 |
| Runtime | 41,305 | 41,305 | 0 | 35,035 | 35,035 | 0 |
| Signature verification | 477,741 | 477,741 | 0 | 442,214 | 442,214 | 0 |
| Certificate CBOR | 96,984 | 96,984 | 0 | 87,150 | 87,150 | 0 |

## Before post-link optimisation

| Probe | Total before | Total after | Delta | Code before | Code after | Delta |
|---|---:|---:|---:|---:|---:|---:|
| Binary Principal | 15,141 | 15,141 | 0 | 13,743 | 13,743 | 0 |
| Self-authenticating Principal | 25,755 | 26,833 | +1,078 | 24,255 | 25,077 | +822 |
| Runtime | 45,351 | 45,351 | 0 | 39,081 | 39,081 | 0 |
| Signature verification | 496,547 | 496,547 | 0 | 461,046 | 461,046 | 0 |
| Certificate CBOR | 102,388 | 102,388 | 0 | 92,552 | 92,552 | 0 |

## Dependency changes and interpretation

The self-authentication graph drops registry `ic_principal` 0.1.5 and changes
SHA-2 0.10.9 → 0.11.1, digest 0.10.7 → 0.11.3, crypto-common 0.1.6 → 0.2.2,
and block-buffer 0.10.4 → 0.12.1. Hybrid-array 0.4.15 replaces generic-array
0.14.9; typenum gains `const-generics`, and version_check leaves this graph.
The other four probes have unchanged external package/feature selections.
The shared fixture lock already contains both hashing stacks for other features;
its only removed external entry is ic_principal. Per-profile graphs therefore
explain more than a lockfile package-count comparison.

This run measures implementation **and** supplier changes together. It does not
prove that owned framing alone or SHA-2 alone accounts for the growth. Removing
DID tooling/parser did not shrink these production probes; that removal primarily
reduces development dependencies and tooling surface.

The command exited **1** because self-authentication exceeds the existing
128-byte growth allowance. This is a measured regression, not an admission or
build failure. No threshold increase, exemption or cryptographic algorithm change
was made. The harness now includes self-authentication in its default selection.

## Evidence and reproduction

```sh
bash scripts/benchmarks/check-wasm-size-regression.sh \
  a8c3fff39094d38510c46451ba1283231ee3cf37 128 --allow-dependency-changes
```

The measured invocation explicitly selected the same five profiles now used by
default. Local raw evidence is under `target/evidence/wasm-regression.SOTB44/`:
`deltas.tsv`, each arm's size tables, source hashes, tool/settings identities,
Cargo metadata/lock, Wasms and build logs, plus `after/dependency-changes.json`.
These artifacts are reclaimable; this report retains the compact result.

All 12 focused harness tests pass, covering absolute sizes, growth refusal even
with changed dependencies, strict-mode refusal, checksum/feature differences,
normal/build versus dev graph selection, missing graph evidence, compiler/settings
and nested source drift, source capture, and section framing. Shell syntax and
documentation links pass. No new product release or native macOS qualification
is part of this tooling change.

## Attribution and framing improvement

Follow-up controlled variants retained the same multi-call Principal probe,
compiler and recipe. The probe exercises slice, owned Vec and borrowed Vec
arguments. Only the described supplier selection or owned framing changed in
reclaimable source archives under `target/evidence/principal-attribution.Hc3EQj/`.
No registry algorithm source was modified. The SHA-2 0.10 variants are diagnostic
controls, not proposed production downgrades.

Final optimised, uncompressed sizes in bytes:

| Variant | Principal total | Principal code | Combined ledger total | Combined ledger code |
|---|---:|---:|---:|---:|
| Old registry Principal / SHA-2 0.10 | 24,490 | 22,990 | Not measured | Not measured |
| Owned framing / SHA-2 0.10, defaults on | 24,516 | 23,016 | Not measured | Not measured |
| Owned framing / SHA-2 0.10, defaults off | 24,516 | 23,016 | Not measured | Not measured |
| Published owned framing / SHA-2 0.11 | 25,444 | 23,688 | 60,451 | 41,719 |
| Non-generic helper, compiler chooses inlining | 25,431 | 23,591 | 60,941 | 42,209 |
| Non-generic helper, `inline(never)` | 24,026 | 22,186 | 60,743 | 41,914 |
| Streaming update/finalize into output buffer | 25,458 | 23,703 | 60,451 | 41,719 |
| **Direct completed-frame construction (selected)** | **25,247** | **23,491** | **60,451** | **41,719** |

Keeping the same original owned framing, the SHA-2 stack upgrade accounts for
928 total / 672 code bytes. The registry-to-owned framing change with SHA-2 0.10
accounts for 26 total/code bytes. Disabling the older supplier's default features
has no size effect here. These results attribute the *whole supplier stack*
change, not a particular internal hash operation or function.

The helper variants show why a Principal-only optimisation would be misleading:
`inline(never)` saves 1,418 total / 1,502 code bytes against published Types,
but grows the combined ledger workload by 292 total / 195 code bytes. The ordinary
helper also grows that workload. Neither helper is adopted. Streaming uses the
supplier's public Digest API but is also rejected: it grows the Principal probe.

The selected change constructs `Principal { len: 29, bytes }` after filling the
28-byte SHA-224 result and `0x02` suffix, instead of admitting/copying that completed
fixed frame again with `from_slice`. It preserves the generic public signature,
arbitrary input acceptance, upstream digest call, exact identity bytes and type.
No manifest, supplier feature, primitive, constructor for external inputs or
cryptographic policy changed. No instruction-count or runtime-speed claim is made.

A final comparison of the actual edited checkout against published source passed
strict dependency/feature equality and a **zero-byte growth allowance** for both
probes. Evidence is `target/evidence/wasm-regression.J3nMYQ/`:

| Probe / build stage | Total before | Total after | Delta | Code before | Code after | Delta |
|---|---:|---:|---:|---:|---:|---:|
| Principal, before post-link | 26,833 | 26,636 | -197 | 25,077 | 24,880 | -197 |
| Principal, final | 25,444 | 25,247 | -197 | 23,688 | 23,491 | -197 |
| Ledger, before post-link | 62,573 | 62,573 | 0 | 43,841 | 43,841 | 0 |
| Ledger, final | 60,451 | 60,451 | 0 | 41,719 | 41,719 | 0 |

Types native all-target and Wasm library Clippy pass with warnings denied. The
existing independent fixed-byte `abc` test now covers array reference, slice,
borrowed Vec and owned Vec and checks the completed identity against `from_slice`;
it passes. No upstream self-authentication comparison suite is restored.

Relative to the pre-deployment baseline, the selected patch still leaves
**+757 final total / +501 code bytes**, exceeding the old 128-byte allowance.
The saving is qualified; the remaining regression is not waived. SHA-2 stays at
0.11.1. The implementation and prior harness changes remain local and unpublished.
