# Imported runtime adapters

These maintained non-cryptographic adapters derive from `dfinity/cdk-rs`, package
`ic-cdk` 0.20.3 at release revision
`624606f7fd1ecd6668e9608594946486aec29167`. The complete original Apache-2.0
license is the regular `LICENSE-APACHE` file in this directory. Original Platform
code remains MIT; the combined consumer package declares MIT AND Apache-2.0.

Platform is the maintenance owner of these adapters. No upstream acceptance,
registry replacement, consumer Cargo patch or second runtime package is needed.
The executor implementation is not copied: every task/context delegates to
registry `ic-cdk-executor =2.0.0`; raw calls use registry `ic0 =1.2.0`.

The reviewed import retains selected System API functions; raw call submission,
response/error handling and callback/trap cleanup; protected/weak/migratory
spawning; and the existing stable-byte and Candid-checkpoint adapters. Private
stable IO exists only to preserve the pre-existing checkpoint behavior. It is
not a new allocator, storage layout, manager or general public memory API.
No cryptographic implementation is imported or altered. The root-key system
adapter retains its upstream bytes and construction unchanged.

Changes from the import:

- Retarget module/Principal paths; expose only the reviewed facade surface.
- Gate Candid argument/response methods and legacy checkpoints on `candid`.
- Begin raw call builders with empty transport bytes instead of implicit Candid
  unit arguments. Both Wire and external Candid callers supply explicit payloads.
  Bounded calls still default to 300 seconds; cycle and timeout policies remain
  caller-owned.
- Omit unexposed retry-classification extensions, obsolete spawning compatibility,
  unused system helpers and unused stable readers/buffers/seek APIs.
- Retain the imported call state machine and protected-spawn wrapper. The recorded
  upstream fingerprints in `provenance.json` are checked by
  `scripts/ci/check-owned-runtime-source.sh`, preserving strings while ignoring
  formatting/comments and the permitted module-path adjustment. They are change
  admission evidence, not a substitute for installed recovery tests.

Call, response, cost/error and stable-error types now have Platform identities.
`Principal`, `RejectCode`, external DTOs, certification and crypto types retain
registry identities. This is a breaking source migration requiring the next
minor release, 0.2.0. The declared version does not establish registry publication.
