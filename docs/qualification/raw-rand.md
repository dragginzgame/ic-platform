# Bounded raw_rand supplier

Protocol 0.2.5's opt-in `raw-rand` capability supplies the narrow contract in
[issue #13](https://github.com/dragginzgame/ic-platform/issues/13).
The request is the exact six-byte empty Candid tuple. The selected response
profile is exactly one canonical `vec nat8` type-table entry, one argument
referring to it, and a 32-byte vector: 42 bytes total. Alternate equivalent
Candid encodings are deliberately outside this producer profile.

`encode(maximum_bytes)` borrows static bytes. `decode(bytes, maximum_bytes)`
checks the whole-message allowance before framing and returns fixed `[u8; 32]`
storage. Neither operation allocates. Zero bytes are valid seed data; no entropy,
readiness, stale-generation, transport, mixing or retry policy is inferred.

`make qualify-raw-rand` checks native tests, strict Clippy/docs, independently
encoded upstream Candid frames, and standalone Wasm execution with no imports.
The cases preserve zero, maximum and mixed seed bytes and refuse every truncation,
header mutation, wrong vector length/type, surplus argument, trailing byte and
one-byte-too-small allowance. Upstream Candid is only a native qualification
supplier. The raw-rand-only product graph has no dependencies.

## Entropy migration mapping

Read-only Entropy baseline `cbb6a152116f3c716758a677e111cb4d2c49f50c` uses the old
Platform `raw_rand` facade. Its source should import `ic_platform_protocol::raw_rand`,
pass six as the encode bound and 42 as the complete reply bound. Its existing
`MalformedReply` mapping and claim/stale-generation checks remain consumer-owned.
The awaited call must use the same new Runtime/Types cohort as Timers; retaining
an old Runtime through Timers is not an admissible singleton graph.

The held HTTP response in `qualification/entropy-probe/src/canister/source_checks.rs`
requires explicit pricing version one. Protocol now supplies
`http_request::encode_http_get_args_v1`; Runtime supplies
`raw::cost_http_request_v1`. For this headerless/bodyless request the quote uses
the URL byte length and response allowance 256, not the Candid frame length.
The existing default-pricing encoder remains unchanged. The caller must select
its replication policy explicitly and preserve the held-reply/custody sequence.
`make qualify-http-request` independently decodes the version-one field and
checks exact output limits.

Actual installed Entropy contention, stale-result and cancellation qualification
still requires the Scheduling/Timers migration. No registry-backed consumer or
production adoption is claimed by these supplier checks.

The [publication record](2026-11-16-supplier-publication.md) verifies delivery of
these capabilities. Entropy's [registry reassessment](https://github.com/dragginzgame/ic-platform/issues/13#issuecomment-6926751538)
confirms the supplied API and identifies the remaining Timers singleton conflict
under ic-scheduling#2. Supplier issue #13 is complete; the conditional whole-Entropy
registry integration and installed lifecycle proof remain with their consumer
owners once that prerequisite is available.
