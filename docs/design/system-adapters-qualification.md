# System API and task adapter qualification — 2026-11-10

Subsequent tooling adoption resolves the earlier IC0 admission and root-file
metadata blockers. This report retains its original qualification scope; see
[the current handoff](../status/current.md) for remaining work.


The [System API contract](system-adapters-contract.md) and
[implementation record](system-adapters-implementation.md), together with the
[task contract](task-adapters-contract.md) and
[implementation record](task-adapters-implementation.md), document separate
specification-only authorship contexts. The coordinator reviewed and integrated
the replacements. This record supplies compatibility evidence, not legal clearance.

## Scope and behavior

`cdk::api`, `cdk::stable` and `cdk::futures` now own their implementations rather
than forwarding into the imported CDK directory. Simple exact-signature System
API calls explicitly delegate to registry IC0. Principal reads remain bounded
to 29 bytes before copying; byte readers initialize spare capacity before setting
vector length. Empty reads still call the supplier once; absent certificates
skip size/copy. UTF-8, strict booleans, counter variants, full u128 cycles/costs,
u64 stable offsets, growth sentinel and error behavior remain unchanged.

Protected tasks use a directly pinned inner future and completion flag. The flag
is set immediately on Ready, before inner destruction. Unfinished cancellation
still panics except during recovery, including cancellation before the first
poll. Weak/migratory routing and the sole executor are unchanged. The retained
call builder and callback/trap-cleanup implementation are unchanged, with their
Apache license and original fingerprint. No cryptographic primitive, storage
allocation, retry, budget or timer policy is added.

## Focused qualification

With Rust 1.99.1, Runtime `cdk` native all-target Clippy and Wasm library Clippy
pass with warnings denied. Runtime tests pass: 39 unit tests, three CDK consumer
tests, 13 public executor tests and two doctests. The new adapters contribute
eight API tests, three stable tests and ten task tests. These cover copy failure,
empty/absent distinctions, invalid text and booleans, integer boundaries,
never-polled cancellation, recovery, non-Unpin address stability, reentrant
destruction and preservation of a completed inner future's destructor panic.
The [public API observation transcript](system-adapters-api-observations.txt)
matches the earlier implementation.

The retained source/license checks and mutation-refusal tests pass. ShellCheck
passes for the changed guard scripts. Native and Wasm feature closure checks
select `ic0,executor,runtime,candid-codec-endpoint`, including empty/default
profiles, with unchanged singleton ownership. Runtime keeps its existing optional
`pin-project-lite` edge; IC0/executor-only selections do not enable it.

Installed qualification uses the prepared PocketIC 16.1 server:

```sh
POCKET_IC_BIN="$PWD/.tools/ic-testkit-server/16.1.0-x86_64-linux/pocket-ic" \
  CARGO_NET_OFFLINE=true bash scripts/ci/check-platform-runtime.sh --prepared-call
```

`target/cdk-runtime.TDdGXt` contains status 0 and four PASS rows: raw/prepared
calls with Wire/external-Candid endpoints. These cover awaits, callback/encoding
traps, cancellation, upgrades, cycles, stable rollback, certificates, management
bytes and endpoint admission. Captured source hashes match after completion.
This is Linux installed evidence; no fresh macOS or Scheduling-owned Timers
composition evidence is claimed. Parent logs are
`/tmp/platform-adapters-projection-{clippy,tests,wasm,installed}.log`.

## Matched Wasm measurements

These are uncompressed Cargo release artifacts measured with
`scripts/benchmarks/wasm-sections.pl`, including custom sections in totals and
using section 10 payload lengths for code. No additional stripping or wasm-opt
pass was applied. Baseline `target/cdk-runtime.w76c08` already has the independent
executor but still uses the old System API/task adapters. Compiler observations,
generated fixture manifests, fixture sources and resolved lock match the final
`target/cdk-runtime.TDdGXt` run. Profile features are recorded in `profiles.log`.

| Profile | Before total | After total | Before code | After code |
| --- | ---: | ---: | ---: | ---: |
| Raw Wire | 226,685 | 224,885 | 150,462 | 150,013 |
| Raw external Candid | 677,515 | 675,407 | 473,762 | 473,327 |
| Prepared Wire | 228,432 | 226,783 | 151,981 | 151,748 |
| Prepared external Candid | 679,056 | 678,147 | 475,392 | 475,461 |

Totals decrease by 909–2,108 bytes. Code decreases in three profiles; prepared
external Candid grows by 69 code bytes while total size decreases. These small
changes are fixture-specific, not an instruction-speed or application-wide claim.

A standard-async completion-guard alternative passed the same tests and installed
profiles (`target/cdk-runtime.XkXPGq`), but was larger than the selected direct
wrapper by 1,798–4,370 total bytes and 1,248–3,027 code bytes. Removing the existing
pin-projection dependency was therefore rejected. The final implementation adds
neither a dependency nor an unsafe pinning implementation.

Final artifact SHA-256 values:

```text
raw-wire adbf4704d197047d90bdb7ee8809163bb62b306b2a29e6d682361dbd4e6e7ce8
raw-external-candid 2b60d9a97370363098fb48c42e98dc5860d7e95557f386e5886c3c6e49f38956
owned-wire dd6cb18aa38ebf5ce419a4c6dfd7b8d11cca91e0dc5720086ba1470095dcb8be
owned-external-candid 6603700fd1bcd9eb9a6df26efe8384f0904777022e4b188ec6328c0cc48881ad
```

First-party license replacement, installed Toko's registry-IC0 admission and the
separate ICP derivation draft still block release. Retained third-party licenses
are not removed by this adapter qualification.
