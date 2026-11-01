# IC Platform Runtime

The sole IC0, executor, CDK adapter and prepared-call implementation. This package
and its macros/singleton guard have an independent Runtime release group. Crypto,
certificates, Candid and native clients remain in `ic-platform`; identities remain
in `ic-platform-types`. There are no default features.

Select `ic0` for raw system calls, `executor` for method/task contexts, `runtime`
for raw calls and prepared `call::Call`, or `cdk` for explicit-codec entrypoints.
Use `ic_platform_runtime::{ic0, executor, cdk, call}`. Endpoint codecs and policy
remain caller-owned. For a renamed dependency, pass the existing macro `crate`
option with the selected runtime's CDK path.

Existing Platform facade paths delegate to this same owner. They install no
second executor. Different runtime generations and the older embedded Platform
executor remain mutually exclusive through the exact Cargo singleton guard.
IC0/executor-only selections exclude Types, Candid, hashing and host adapters.

Imported Apache source/licenses and protected region fingerprints are retained.
See the repository migration contract for source/archive qualification and the
separate installed owner and Timers composition proofs. This package owns no
scheduler, codecs, retries or storage allocation.
