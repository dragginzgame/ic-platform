# ic-platform-macros

Explicit IC update, synchronous query, init, pre-upgrade and post-upgrade exports
for ic-platform-runtime. This support package is versioned with Runtime.

Every endpoint selects `runtime`, `guard`, `decode`, `max_bytes` and `on_error`.
Updates and queries additionally select `name` and an `encode` function; async
updates may instead select `manual_reply`. Lifecycle handlers return unit and
have fixed IC export names. Every handler takes one decoded argument (including
unit). The error callback must never return. Async queries and lifecycle handlers
are rejected. Pre-upgrade decodes empty bytes without reading IC arguments.

Defaults are empty. Use one exact package/source cohort at public Types and
Runtime boundaries. This release publishes the implemented scope; it does not
claim completion of the full Platform rebuild or migrate applications.

See the [design and qualification records](https://github.com/dragginzgame/ic-platform/tree/main/docs).
