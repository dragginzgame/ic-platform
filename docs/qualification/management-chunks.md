# Bounded management chunk-cache messages

Protocol 0.2.5's opt-in `management-chunks` supplies the codec boundary requested
by [Wasm Store #16](https://github.com/dragginzgame/ic-platform/issues/16).
It selects canonical Types and the existing bounded binary projection reader;
it does not select Runtime, Agent or a general Candid library.

The independently authored request schemas encode `stored_chunks`' required
canister_id record and `upload_chunk`'s required canister_id/chunk record. Exact
principal and chunk bytes survive, including an empty chunk. The complete frame
size is computed with checked arithmetic and admitted before reserving output.
Caller method selection and transport remain explicit.

Replies decode as one vector of hash records or one hash record respectively.
Hash bytes remain opaque: the codec does not enforce a hash algorithm, compare
expected hashes or confer cache authority. All selected and skipped values use
the existing shared byte/header/type/work/allocation/vector/skipping budgets.
Decode does not require encoding capability. There is no legacy facade or copied
upstream codec implementation.

## Consumer mapping and proof

Read-only Wasm Store source `10330163c71e3ba0e61f7585a8d2d6cc46c3d479`,
`crates/ic-wasm-store/src/canister/cache.rs`, uses a 65,536-byte reply ceiling,
4,096-byte header ceiling, 256 constructed types, 131,072 work and 262,144 logical
allocation. Map these directly to `candid::Limits`. Select `vector = 65_536` and
`skipping = 131_072` as additional explicit ceilings; vector traversal and
skipping also consume the same aggregate work/allocation allowances. The latter
bounds logical retained/table storage, not allocator overhead or process memory.
Keep the 1,024-byte observation request ceiling and the consumer's selected
CHUNK_LIMITS for the complete encoded upload request.

`make qualify-management-chunks` runs independent native upstream-management
schema checks, hostile frame and budget checks, strict Clippy/docs, standalone
Wasm literal replay, and product graph checks. Cases include management,
anonymous and 29-byte principal representations; empty/127/128/16,383/16,384-byte
and 1 MiB chunk requests; exact and one-byte-short bounds; reply vectors across
LEB boundaries; every truncation; wrong type, surplus argument, trailing data;
and malformed/over-budget skipped values. Native upstream schemas remain private
qualification dependencies.

The [publication record](2026-11-16-supplier-publication.md) verifies registry
delivery. Store's [actual consumer evidence](https://github.com/dragginzgame/ic-platform/issues/16#issuecomment-6926752673)
then passed all four installed scenarios against published Protocol 0.2.5,
Runtime 0.23.13 and Types 0.21.2: cache preparation/reuse, final callbacks after
controller revocation and close/reopen, closed-maintenance fences and capacity
at 4,096 labels. The predecessor-to-candidate upload/upgrade witness also passed.
This completes the requested supplier and consumer boundary for #16; Store's
uncommitted adoption is not a published Store release. Storage's old raw-memory
edge remains separately owned by ic-storage#3. Controller/pin checks, transport
uncertainty, admission epochs and retry policy stay with Store.
