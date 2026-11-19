# ic-platform-runtime

Explicit call ownership, IC one-way and experimental awaited transport, with one executor for update and timer roots and optional explicit update, query and lifecycle exports. Native call preparation is portable; IC transport requires Wasm. Suspended futures are heap-local. Upgrades require stop, wait for Stopped, upgrade, then start. Failed-cleanup resource retention, broader context/deadline cases and production timer integration remain documented qualification limits. No automatic retries or persisted callbacks are provided.

Defaults are empty. Use one exact package/source cohort at public Types and
Runtime boundaries. This release publishes the implemented scope; it does not
claim completion of the full Platform rebuild or migrate applications.

See the [design and qualification records](https://github.com/dragginzgame/ic-platform/tree/main/docs).

With the `raw-rand` feature, `raw_rand::request(wait_seconds).await` requests
32 bytes from the management canister. The call uses the supplied bounded wait
and attaches zero cycles. Its error preserves submission refusal, rejection,
uncertain outcome or malformed framing.

Caller-owned global timer exports can enter `executor::run_global_timer` through
the same executor, with no ordinary reply authority. `system::global_timer_set`
and `system::canister_version` expose IC operations; scheduling remains with the
consumer. See the [timer contract and migration limits](https://github.com/dragginzgame/ic-platform/blob/main/docs/qualification/runtime-timers.md).

The system surface exposes caller/canister identity, IC time, controller checks,
bounded arguments and query certificates, and 32-byte certified-data publication.
Certificate retrieval does not authenticate evidence. Queries and lifecycle
handlers are synchronous; automatic updates use the existing executor. Codecs,
guards and non-returning error handlers are explicit application choices.

For caller-owned synchronous Wasm exports, `system-raw` exposes `raw` with no
production dependencies or executor. It supplies argument/caller byte copying,
current-controller checks, raw append/reply, byte traps and `instruction_counter()`.
The IC enforces contexts and bounds. Use one reply owner per call; do not mix raw
replies with automatic or executor-managed replies. See the
[Storage migration and qualification](https://github.com/dragginzgame/ic-platform/blob/main/docs/qualification/runtime-storage.md).

Native management and Entropy handlers can use the same Runtime signatures as
Wasm. On native hosts `raw::in_replicated_execution()` returns false and
`executor::run_update` returns `EntryError::NonReplicated` before polling.
Reading the liquid cycle balance, stable page count, or canister version, and
pricing a prepared call or HTTP request, panic outside an IC execution context.
