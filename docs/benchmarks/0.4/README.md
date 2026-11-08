# 0.3–0.4 measurements

These selected observations record Linux Rust 1.99.1 measurements.
Three serial fresh-target Wasm builds used cached dependencies, four Cargo jobs,
opt-level 3, no LTO and 16 codegen units. Principal timings use five runs of
500,000 operations with warmup outside measurement.

Signature creation builds fell from 7.55 to 3.32 seconds; verifier builds from
7.87 to 4.50 seconds. Raw Wasm fell by 67 and 2,739 bytes respectively; verifier
executable code fell by only 128 bytes. Runtime executable code size was unchanged.
Principal text parsing/formatting was approximately 1.9–2.0 times faster.
These are fixture results, not application latency or IC instruction measurements.
Historical peak RSS observations did not measure aggregate process-tree memory.

The active historical runner is retired. [Reproduction sources and pinned
inputs](../../../scripts/benchmarks/0.4/README.md) remain at their immutable Git
revision. Use that checkout and its recorded compiler/caches; this is historical
Linux evidence, not current-source or macOS qualification.

The comparison is `390644f` to `9a661f7`; Principal uses `f105386` as its baseline.
Runtime fixtures contain historical Timers/Wire and registry Platform 0.1.18.
Signature fixtures are exported qualification probes, not whole Auth canisters.
No Candid-versus-Wire execution, Account speed, or cycle-cost claim follows.


| Capability | Uncompressed Wasm before → after | Code section before → after |
| --- | ---: | ---: |
| Signature creation | 173,530 → 173,463 | 127,715 → 127,578 |
| Signature verification | 710,373 → 707,634 | 576,639 → 576,511 |
| Raw runtime | 338,370 → 338,393 | 229,769 → 229,769 |
| External Candid | 891,748 → 891,761 | 621,511 → 621,511 |

## Follow-up Account measurements

Run `bash scripts/benchmarks/types.sh` with prepared registry sources. The historical
trials compare owned and external types in one release binary
on Linux x86-64, Rust 1.99.1. Five alternating trials use 500,000 iterations each,
with 10,000 warmup operations. Units below are median nanoseconds per operation.
This run includes the incoming rustls-webpki lock update; the benchmark uses no TLS.

| Owner bytes / subaccount | Parse external → owned | Format external → owned |
| --- | --- | --- |
| 10 / absent | 247.4 → 131.1 | 133.1 → 82.5 |
| 10 / explicit zero | 238.4 → 128.7 | 132.6 → 74.8 |
| 10 / nonzero | 499.6 → 243.7 | 394.3 → 540.7 |
| 29 / absent | 361.3 → 191.6 | 215.1 → 112.1 |
| 29 / explicit zero | 363.8 → 193.6 | 218.9 → 113.0 |
| 29 / nonzero | 631.0 → 318.0 | 472.0 → 613.0 |

Nonzero-subaccount formatting regresses in this fixture; parsing improves.
No implementation change was made in response to the measurements.
`size_of` is 30 bytes for both Principals, 63 bytes for both Accounts and 32 bytes
for Subaccount. These are this compiler/target's layouts, not persisted ABI promises
or allocation measurements. No IC instruction or allocation comparison was run.

Raw trial tables, source snapshots and artifact-hash listings were retired from the
current tree during Shared 0.30.4 cleanup. The selected observations below remain
historical summaries; old local artifacts were not reverified and are not claimed
as independently available evidence. Git history retains the previous records.
