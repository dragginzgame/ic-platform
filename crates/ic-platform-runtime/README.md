# IC Platform Runtime

The sole executor, CDK adapter and prepared-call owner, with explicit registry
IC0 exports. This package
and its macros/singleton guard have an independent Runtime release group. Crypto,
certificates, Candid and native clients remain in `ic-platform`; identities remain
in `ic-platform-types`. There are no default features.

Select `ic0` for raw system calls, `executor` for method/task contexts, `runtime`
for raw calls and prepared `call::Call`, or `cdk` for explicit-codec entrypoints.
Use `ic_platform_runtime::{ic0, executor, cdk, call}`. The conventional
`cdk::{query, update, init, pre_upgrade, post_upgrade, heartbeat, inspect_message,
on_low_wasm_memory}` macros form the sole ordinary entrypoint family. Endpoint codecs and policy
remain caller-owned. For a renamed dependency, pass the existing macro `crate`
option with the selected runtime's CDK path.

The `ic0` boundary includes `stable64_size`, `stable64_grow`,
`stable64_read(&mut [u8], offset)` and `stable64_write(&[u8], offset)`.
These safe infrastructure primitives retain IC bounds traps and return
`u64::MAX` when growth is refused. They supply no allocation authority or
application storage policy; production applications retain Storage governance.

Existing Platform facade paths delegate to this same owner. They install no
second executor. Different runtime generations and the older embedded Platform
executor remain mutually exclusive through the exact Cargo singleton guard.
IC0/executor-only selections exclude Types, Candid, hashing and host adapters.
IC0-only selects just this package and registry `ic0` 1.2.0, which has no
production dependencies. The copied IC0 source and its local license/provenance
files are removed; the dependency retains its upstream licensing.

The executor, System API, stable-memory and task-policy adapters have independent
implementations and tests. Protected spawning uses a direct pinned wrapper with
pin-project-lite; this measured smaller than an additional async wrapper. The
call transport also has an independent implementation: borrowed request data
stays outside its owned callback completion state, and arbitrary Waker operations
run outside its state lock. The generator also has an independent implementation
and public consumer/export tests. First-party license text and matching manifest
metadata remain pending. See the migration contract for qualification records.
See the repository migration contract for source/archive qualification and the
separate installed owner and Timers composition proofs. This package owns no
scheduler, codecs, retries or storage allocation.
