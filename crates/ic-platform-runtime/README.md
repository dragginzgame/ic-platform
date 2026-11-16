# ic-platform-runtime

Explicit call ownership, IC one-way and experimental awaited transport, with one method-attached executor and optional manual-reply update exports. Native call preparation is portable; IC transport requires Wasm. Suspended futures are heap-local. Upgrades require stop, wait for Stopped, upgrade, then start. Failed-cleanup resource retention, broader context/deadline cases and production timer integration remain documented qualification limits. No automatic retries or persisted callbacks are provided.

Defaults are empty. Use one exact package/source cohort at public Types and
Runtime boundaries. This release publishes the implemented scope; it does not
claim completion of the full Platform rebuild or migrate applications.

See the [design and qualification records](https://github.com/dragginzgame/ic-platform/tree/main/docs).
