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
spawning; and raw stable-byte passthroughs. The unused Candid checkpoint adapters
and private stable IO machinery are retired in the 0.5 hard cut. No allocator,
storage layout, manager or replacement checkpoint owner is introduced.
No cryptographic implementation is imported or altered. The root-key system
adapter retains its upstream bytes and construction unchanged.

Changes from the import:

- In 0.5 replace panic-prone timeout mutation with explicit WaitPolicy selection.
  The stored timeout and submission path retain their original representation
  and semantics; bounded constructors still select 300 seconds.
- Retarget module/Principal paths; expose only the reviewed facade surface.
- Retire Candid argument/response methods in 0.4; encode/decode at the separate
  explicit Candid boundary.
  Remove the combined transport/codec Error and string-only CandidDecodeFailed;
  preserve CallFailed, OnewayError and all call submission/cleanup behavior.
- Begin raw call builders with empty transport bytes instead of implicit Candid
  unit arguments. Both Wire and external Candid callers supply explicit payloads.
  Bounded calls still default to 300 seconds; cycle and timeout policies remain
  caller-owned.
- Omit unexposed retry-classification extensions, obsolete spawning compatibility,
  unused system helpers and unused stable readers/buffers/seek APIs.
- Retire `stable_save`/`stable_restore`, whole-memory copying, StableIO/StableWriter
  and their private StableMemory abstraction in 0.5. Raw grow/read/write calls
  retain the original ic0 behavior and error identity.
- Copy caller, canister and subnet principal bytes into fixed 29-byte stack
  buffers instead of allocating Vecs. Valid IC identities are unchanged;
  malformed system lengths still trap before a principal can escape.
- Retain the imported call state machine and protected-spawn wrapper. The recorded
  upstream fingerprints in `provenance.json` are checked by
  `scripts/ci/check-owned-runtime-source.sh`, preserving strings while ignoring
  formatting/comments and the permitted module-path adjustment. They are change
  admission evidence, not a substitute for installed recovery tests.

Call, response, cost/error and stable-error types now have Platform identities.
`Principal` became Platform-owned in 0.3; the 0.4 runtime RejectCode is owned too.
ExternalPrincipal, external DTOs and certification retain registry identities.
The initial runtime migration shipped in 0.2; these further breaking changes
require 0.4. The declared version does not establish registry publication.
