# Prepared calls

`ic_platform::call` owns prepared, one-shot calls inside the originating canister. It delegates to
IC Platform's raw runtime and preserves that implementation's executor,
callback cleanup, response and error identities. It adds method/argument custody;
it does not install another runtime or proxy canister.

```rust
use ic_platform::call::{Call, Principal, WaitPolicy};

fn prepare(target: Principal) -> Call<'static, 'static> {
    let method = String::from("runtime-selected-method");
    Call::new(target, method, WaitPolicy::Bounded { timeout_seconds: 300 })
        .take_raw_args(vec![1, 2, 3])
}
```

Construction and unpolled future drop make no IC calls. `execute().await` consumes
the prepared request and submits it once through Platform. Arguments and method
names can borrow independently; passing `String` and `Vec<u8>` transfers ownership.
`get_cost()` delegates Platform's exact current cost query including attachments;
it is neither a stable price quotation nor spending permission. The API requires
an explicit bounded/unbounded wait; zero stays bounded and is forwarded unchanged.

Enable Platform’s existing `runtime` feature to select this module, without Candid, a host parser, Agent, PocketIC, Relay or Timers.
Supply Wire frames for owned endpoints. For mandatory external IC interfaces,
select Platform's separate Candid codec and encode **before** submission, retaining
original bytes for journals/recovery. Decode `Response::into_bytes()` separately
with caller-owned byte/work/arity/depth admission. Candid's Wasm depth limitation
remains tracked by Platform #73; this module does not repair or hide it.

Transport failures retain `CallFailed`/`CallRejected` identity and raw unknown
rejection codes. A reply with invalid bytes is not evidence of no remote effect.
A bounded timeout, cancellation or callback trap does not establish physical
settlement. Callers retain authentication, receipts, metrics, budgets, refunds,
retry authority and durable delivery. Capture matching-callback refund facts
before another await; submission failures are not callback refund observations.

Canic can replace its generic request/execute wrapper with this `Call`, including
owned dynamic method names, and use `get_cost()` instead of repeating its formula.
Its Candid helpers and stage-specific metrics remain separate owner work. Blob,
Query, Jobs and Relay may retain the already qualified raw Platform route when
its borrowed-method contract suffices. Timers' private self-dispatch remains below
this layer. Existing raw callers are not forced to migrate.

Rust 2024 / MSRV 1.91. Native construction/inspection is supported; execution and
cost observations require the existing IC canister/executor context. The module ships
with Platform; actual consumer adoption remains independent.
