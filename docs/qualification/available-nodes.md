# Available-node page projection

The opt-in `cloud-engine-marketplace` feature now exports `AvailableNode`,
`AvailableNodesPage`, `AvailableNodesPageResult`, `EngineError` and
`decode_list_available_nodes_page_response`. This is the response portion of
[the retained #166 contract](https://github.com/dragginzgame/ic-platform-legacy/issues/166).
The existing declaration macro now handles both null and typed variant payloads;
the shared reader, source identities and dependency closure are unchanged.

The decoder requires one lowercase `ok`/`err` variant. `Ok(None)` means the
Registry copy is unready; `Ok(Some(page))` with zero nodes is a distinct successful
page. Present malformed values and unknown node classes cannot fall back to
unready. All nine declared application errors retain their original case and
raw text; unknown error tags refuse. Caller policy owns terminal escaping.

Node and cursor Principals use canonical Types. Order, DC presence, full-width
Registry versions and the registered-chip SEV flag remain raw evidence. No
attestation, reservation, liveness, capacity or atomic-inventory claim is made.
The caller owns cursor traversal, duplicates, per-page service limits, deadlines,
retries and interpretation. The unselected IP field is validated and skipped
without retaining text, charged to shared total and skipped-work allowances.

Independent literal frames cover all classes/errors, full-width raw versions,
principal identities, cursors, absent/empty pages, all truncated prefixes,
invalid variants and payloads, malformed skipped UTF-8, arity/trailing refusals
and all seven configurable resource bounds. The IP fixture establishes an exact
four-work-unit skip allowance and refusal at three. These fixtures come from the
selected issue declaration, not copied old source/tests or production encoding.

Public tests exercise lowercase Serde tags, all errors and classes, maximum
version, unready/empty distinction and malformed JSON refusal. Native tests
compare against Platform 0.33.7's encoded and decoded values. They run under
`make qualify-marketplace`, together with the existing price tests, strict Clippy,
Rustdoc and consumer Wasm compilation. The separate [Wasm corpus](marketplace-wasm.md)
now also executes page admission and request encoding. Local Linux results do
not establish hosted/macOS acceptance for this new projection.

Measured response allocation behavior and actual Query caller adoption remain
pending. Existing published Query retains
its supplying codec; no sibling changes or mainnet requests were made.

## Page request encoding

`ListAvailableNodesPageArgs` and `encode_list_available_nodes_page_args` now
encode one record with optional arbitrary-natural `limit`, canonical Principal
`start` and opaque UTF-8 `rejoining`. None remains distinct from explicit zero,
the empty Principal and empty text. The encoder does not enforce the service's
100-node page policy or require anonymous/new-engine rejoining behavior.

The existing projection declaration has a selected arguments arm: one declaration
owns Rust fields/types, numeric labels, option schema and field traversal. It
does not add Encode to response projections or expose a general caller-defined
schema. The complete frame is measured against `max_bytes` before reserving output.
Measurement uses the upstream natural's bit length without walking its magnitude;
writing packs borrowed upstream bits into unsigned LEB bytes without a scratch
buffer. No new arithmetic implementation or dependency is introduced.

Three independent literal test groups cover all eight optional-presence patterns,
exact/one-below byte limits, 127/128, u64::MAX, 2^64, 2^140, full-length Principals,
NUL/Unicode/whitespace and a multi-byte text length. A public consumer comparison
covers 45 combinations, including 256-digit natural limits, and matches the
published Platform 0.33.7 encoder byte for byte and Serde value for value.
Clippy, Rustdoc and consumer Wasm compilation pass through `qualify-marketplace`.

A separate native DHAT 0.3.3 release test constructs input before measurement.
For a 256-digit limit and 256-byte UTF-8 selector, one-below admission performs
zero allocations; exact admission performs one allocation of the complete output
length, released on drop. This protects preallocation admission for that workload,
not a universal process heap quota or response-decoder allocation claim.
