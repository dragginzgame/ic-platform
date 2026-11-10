# Entrypoint generator replacement contract

Input for the authorized specification-only implementation/test contexts. Do not
read existing generator sources, their tests/fixtures, supplier implementations,
Git history or diffs. This contract describes externally observable syntax and
execution plus the private interface used by the existing owned codec adapter.
The coordinator owns integration, old-source retirement and installed proofs.
No codec, scheduling policy, crypto, dependency or public macro is added.

## Deliverables and separation

Author a new `crates/ic-platform-macros/src/entrypoint/` module, a replacement
`src/lib.rs`, and replacement `src/options/mod.rs` and `src/signature/mod.rs`.
Overwrite those latter files only after the coordinator releases its baseline
hold; do not read the existing contents. Keep ordinary private modules and
fresh tests.rs/tests layouts. Parent removes the earlier export module/tests.
Do not edit/read the existing owned `external_candid` adapter; its integration
interfaces are specified below. Existing Syn, Quote and proc-macro2 are available.
Use Rust 2024 and compiler 1.99.1. No new dependency or product capability.

The implementation author records inputs/limits in
`docs/design/entrypoint-generator-implementation.md`. A separately assigned
test context authors public macro consumers from this contract without seeing
the implementation. Parent runs Clippy before tests and installed qualification.
Formatting your authored files with skip_children is allowed; no Cargo until
coordinator handoff. Input separation is not legal clearance or a training-data
exposure guarantee.

## Public attributes and syntax

The proc-macro crate exports exactly these attributes: query, update, init,
pre_upgrade, post_upgrade, heartbeat, inspect_message, on_low_wasm_memory,
codec_query, codec_update. The last two delegate to the unchanged owned
`external_candid::expand_owned(bool, TokenStream2, TokenStream2)`; true=query.
All attributes report syn::Error as compile_error tokens, without panic on
ordinary invalid user syntax. Preserve optional `IC_CDK_DEBUG` stderr expansion
diagnostics (enabled by variable presence, including empty). Exact decoration
of debug output is not a public protocol.

Ordinary attributes parse comma-separated named options, permitting a trailing
comma. Every option name is a single identifier. String-valued options are
`name`, `guard`, `decode_with`, `encode_with`, `crate`; values must be string
literals. All except name are parsed as Rust syn::Path when applicable, supporting
leading `::`, modules and turbofish arguments. The default crate path is
`::ic_platform_runtime::cdk`. Paths refer to user code with its original hygiene.

Boolean options `manual_reply` and `composite` accept a bare flag (=true) or an
actual boolean literal after `=`. Reject string booleans, lists, unknown options,
qualified option keys and duplicate single-value options. Repeated guard options
are deliberately allowed and execute in their supplied order. `hidden` is always
rejected as retired, even false. No Candid path/service registration, runtime_*
macro family, implicit codec or service-export macro is admitted.

`composite=true` is query-only; false is inert on other kinds. Lifecycle hooks
reject manual_reply=true, custom name and any guard. Manual reply on RPCs performs
no automatic encoding/reply; an accompanying encode_with option is ignored,
including not parsing its unused path, matching existing admission.

## Function admission and codec shape

Only a free ItemFn is admitted. Reject generic parameters and receiver arguments;
otherwise preserve the original function, visibility, attributes, signature,
patterns and body. Positional generated bindings must handle arbitrary typed
patterns (destructuring, mut, wildcard), rather than using pattern identifiers.
Additional syntax may be rejected later by Rust (for example an unsafe handler
called without an unsafe block); this is not permission to generate unsafe calls.
Keep ordinary cfg-disabled functions buildable, including a cfg attribute placed
after the proc-macro invocation. Do not accidentally emit active wrappers for
functions removed by cfg processing.

Query/update always require decode_with, including zero-argument handlers. Init
and post_upgrade require it when arguments exist; zero-argument init/post_upgrade
may omit it, causing no message-buffer read. If supplied for zero arguments it
must still run and produce (). All other lifecycle kinds reject arguments and
any decode_with option.

Decoding reads the runtime's message argument bytes once, passes Vec<u8> to the
specified decoder and adapts only argument count: zero expects (), one expects
the single parameter's type directly, multiple expect a tuple in parameter
order. A single tuple-typed parameter is still one argument. Caller codecs own
protocol admission, resource budgets and bytes.

Query/update automatic replies require encode_with unless manual_reply=true.
The return shape is syntactic: omitted return or exactly the empty tuple means
unit; a nonempty tuple is a tuple result; all other types are scalar. Do not
resolve aliases or classify parenthesized scalar syntax as unit. Invoke a unit
encoder with no argument, otherwise pass the whole result once (including
tuples). Require the encoder output to be Vec<u8> by an explicit expected type,
preserving generic encoder inference. Pass those bytes once to msg_reply.
Lifecycle kinds require syntactic unit return and reject encode_with entirely.

Keep major admission ordering stable: parse options and duplicate checks;
composite/manual lifecycle restrictions; function parsing/generics; crate path;
name restrictions; guard restrictions/path parsing; argument/decoder admission;
return/encoder admission. Diagnostic wording may be independently authored,
but identify the refused option or function restriction clearly.

## Export names and execution

Wasm names are exact strings:

| Kind | Export | Context |
| --- | --- | --- |
| query | `canister_query NAME` | query |
| query with composite | `canister_composite_query NAME` | query |
| update | `canister_update NAME` | update |
| init | `canister_init` | update |
| post_upgrade | `canister_post_upgrade` | update |
| pre_upgrade | `canister_pre_upgrade` | update |
| heartbeat | `canister_heartbeat` | update |
| inspect_message | `canister_inspect_message` | query |
| on_low_wasm_memory | `canister_on_low_wasm_memory` | update |

NAME defaults to the Rust handler identifier without an r# prefix. A supplied
name is otherwise verbatim UTF-8, including empty text/punctuation; refuse names
beginning `<ic-cdk internal>`. Lifecycle hooks have no custom name. Native
symbols are `CARGO_CRATE_NAME + "__" + "__ic_export_" + lowercase hexadecimal
encoding of every UTF-8 byte of the complete Wasm export string`. Do not replace
punctuation or omit the crate prefix; unrelated crates with the same lifecycle
exports must link together. Use cfg_attr target_family="wasm" and Rust 2024
unsafe(export_name=...) attributes. Never change the Wasm protocol names.

For compatibility with the existing generated wrapper spelling used by native
observation fixtures, its Rust identifier is `__canister_method_` plus the raw-
prefix-free Rust handler identifier, independent of a custom wire name. It has
no arguments and no return. It is not a public native invocation API.

The selected crate path supplies:

```text
api::msg_arg_data() -> Vec<u8>
api::msg_reply(bytes: Vec<u8>)
api::msg_reject(message: &str)
futures::internals::in_executor_context(f: impl FnOnce())
futures::internals::in_query_executor_context(f: impl FnOnce())
futures::spawn(f: impl Future<Output=()> + 'static)
```

Synchronous entry: run guards first, outside the executor context. Each returns
Result<(), String>; on first Err call msg_reject once and return immediately,
without later guards, argument copying/decoding, handler or encoding. After
successful guards, enter the selected context, decode, call the handler, encode
and reply if automatic. Preserve this ordering.

Asynchronous entry: enter the selected context first, run ordered guards there,
then spawn one protected async task. Decode inside that task before awaiting the
handler; encode/reply only after it completes. Guard refusal spawns nothing.
Use the one existing context/spawn implementation; no executor or polling loop
is emitted. Preserve the internal-spawn clippy::disallowed_methods allow with
an explicit reason: some consumers forbid the method, others do not enable the
lint, so expect would be wrong. Other lint allowances must not be added casually.

Generated local bindings must have mixed-site hygiene and not capture handler,
guard or codec paths with overlapping spelling. Standard types/variants use
absolute paths (Vec, String, Result, Ok/Err) so prelude shadowing is harmless.
Preserve original user spans. Rust pattern names are never generated locals.
No Candid registration or codec may appear in ordinary expansion.

## Private interfaces needed by the existing owned adapter

Parent will redirect its `crate::export` references to `crate::entrypoint`.
The new module must provide `pub(crate)` or appropriately parent-visible:

- `ExportAttributes<P = String>` with public fields:
  `name: Option<String>`, `guard: Vec<P>`, `decode_with: Option<P>`,
  `encode_with: Option<P>`, `manual_reply: bool`, `composite: bool`,
  `cratename: Option<P>`. Default is None/empty/false, with no P:Default bound.
- `external_entry(query: bool, attrs: ExportAttributes<syn::Path>,
  function: syn::ItemFn, item: TokenStream2) -> syn::Result<TokenStream2>`.
  The parsed function is the supplied original item; no reparsing/stringifying
  user paths. It applies the same admission/emission policy to the typed options.
  Both ordinary and external adapters share one generator.

Replace helper modules while retaining these private interfaces:

- `options::unique(meta: &syn::meta::ParseNestedMeta, name: &str,
  seen: &mut BTreeSet<String>, context: &str) -> syn::Result<()>`: guard repeats
  allowed; all other repeats refuse with context/name in diagnostics.
- `options::flag(meta: &syn::meta::ParseNestedMeta) -> syn::Result<bool>` as above.
- `signature::argument_types(signature: &syn::Signature,
  receiver_error: impl Fn(&syn::Receiver) -> syn::Error) -> syn::Result<Vec<&syn::Type>>`:
  preserve argument order, reject any receiver using the provided error factory.
- `signature::ReturnShape<'a>` variants Unit, Tuple(&'a syn::Type, usize),
  Scalar(&'a syn::Type); `of(&'a syn::Signature) -> Self` and `arity(&self) -> usize`.
  Unit arity=0, tuple=element count, scalar=1. Match the syntactic rule above.

## Validation

Fresh tests must cover all kinds, scalar/unit/tuple/one-tuple codecs, typed
patterns, both context orders, guard refusal, manual reply, exact export symbols,
reserved/raw/custom names, parsed paths, duplicate/retired/invalid flags,
lifecycle restrictions, generics/receivers/nonfunctions and hygiene collisions.
Native consumer tests may use a private fake runtime to record events and poll
immediately-ready test futures; that is a generator observation seam, not an
executor implementation or IC evidence. Use safe Wake/Pin helpers, no unsafe
test executor. Compile/link consumer tests must include cross-crate lifecycle
name collisions and repeated endpoint punctuation distinctions.

The existing external codec adapter and its tests remain unchanged except the
parent's mechanical integration-module rename. Parent also runs Runtime consumer
proofs and installed raw/prepared × Wire/external-Candid profiles, preserving
authentication-before-decode, nesting/byte admission, callback traps and upgrade
behavior. Do not replace protocol checks with token-string snapshots alone.
