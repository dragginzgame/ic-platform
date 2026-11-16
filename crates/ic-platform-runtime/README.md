# ic-platform-runtime

Explicit call ownership, IC one-way and experimental awaited transport, with one method-attached executor and optional explicit update, query and lifecycle exports. Native call preparation is portable; IC transport requires Wasm. Suspended futures are heap-local. Upgrades require stop, wait for Stopped, upgrade, then start. Failed-cleanup resource retention, broader context/deadline cases and production timer integration remain documented qualification limits. No automatic retries or persisted callbacks are provided.

Defaults are empty. Use one exact package/source cohort at public Types and
Runtime boundaries. This release publishes the implemented scope; it does not
claim completion of the full Platform rebuild or migrate applications.

See the [design and qualification records](https://github.com/dragginzgame/ic-platform/tree/main/docs).

The system surface exposes caller/canister identity, IC time, controller checks,
bounded arguments and query certificates, and 32-byte certified-data publication.
Certificate retrieval does not authenticate evidence. Queries and lifecycle
handlers are synchronous; automatic updates use the existing executor. Codecs,
guards and non-returning error handlers are explicit application choices.
