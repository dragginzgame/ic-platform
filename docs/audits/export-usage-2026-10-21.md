# Platform export usage audit — 2026-10-21

Not every explicit export has a demonstrated sibling consumer. This audit covers
**393 explicit facade exports and owned inherent methods/constants** at Platform
`b5d5ced` (0.6.6), with the pending inventory documentation unchanged in purpose.
The [per-symbol evidence](export-usage-2026-10-21.tsv) records every entry, source
location, original import, classification and representative consumer references.
No API, feature, dependency, cryptographic source or sibling checkout was changed.

## Results

| Class | Entries | Meaning |
| --- | ---: | --- |
| P | 267 | Product/library/host-source reference outside detected test contexts, or manually verified receiver/alias use |
| T | 50 | Detected sibling tests, examples or qualification fixtures; no stronger direct-use evidence established |
| G | 10 | Required by reviewed generated code or exported macro implementation |
| C | 33 | Public signature, error, field or forwarded-namespace contract; direct alias use is not established by this classification |
| L | 21 | Platform qualification use found, but no resolved sibling use found |
| N | 12 | No resolved sibling use found; no positive fixture evidence recorded in this bounded pass |

Counts describe **names/paths**, not independent implementations. Root aliases and
module-qualified aliases count separately. P includes host libraries and tools;
it is not a measurement of deployed execution or runtime frequency. T is useful
consumer demand, not dead code. C means a type is coupled to the supported
contract; it does not prove that its particular convenience alias is necessary.
L/N together identify 33 names without demonstrated sibling use, not 33 safe
removals. Unknown external consumers and compatibility requirements remain.

## What the source review established

- **Owned Account has no observed sibling adoption.** Root and `account::*`
  Account/Subaccount aliases, account errors/defaults and `effective_subaccount`
  have only Platform fixture evidence or no resolved reference. Query's ledger
  accounts are the external `icrc_ledger_types` identity; IcyDB's account is its
  own type. A future Account decision therefore needs a product/compatibility
  decision, not a claim that ledger Account can be removed with it.
- **Several convenience names have only Platform fixture evidence:**
  `candid::contract::encode_args_ref`, parser `TypeContainer`/`type_equal`,
  `cdk::api::global_timer_set`, `ic0::is_controller` and `pocket_ic::Snapshot`.
  Timers uses the distinct `ic0::global_timer_set`; keeping that consumer does
  not establish demand for the CDK alias. Public upstream methods/DTOs may still
  expose the same types without explicitly naming these aliases.
- **Some runtime surface is qualified rather than consumer-driven.**
  `cdk::futures::{spawn_migratory,spawn_weak,is_recovering_from_trap}` appears in
  Platform's installed runtime fixture. No sibling calls to `Call::oneway`,
  `cdk::{heartbeat,on_low_wasm_memory}` or the audited executor convenience
  exports `cancel_task`, `in_tracking_query_executor_context`,
  `is_recovering_from_trap`, `spawn_migratory` were resolved. This is not permission
  to remove trap/cancellation coverage or change the single executor identity.
- **New decoder helpers have not been observed adopted:**
  `candid::reserved::StrictReserved` and `candid::sequence::deserialize` retain
  Platform tests; Backup still has consumer-owned bounded-sequence/wire helpers.
  This is an adoption observation, not grounds to retire recently supplied APIs
  or move consumer policies into Platform.
- **Cryptographic aliases require care.** Direct sibling use of
  `bls::verify_bls_signature` was not found, but upstream verification remains
  necessary inside signature verification. Error/key/signature types can occur
  through inferred return values. Removing an alias would neither eliminate the
  underlying cryptography nor justify rewriting it. All crypto implementations
  and vectors remain untouched.

## Important false positives avoided

Candid derive 0.10.39 emits `types::{CandidType,Compound,FieldDoc,TypeDoc,TypeId}`;
service export emits `pretty::candid::{DocComments,compile_with_docs}`. These
paths are required even when no handwritten sibling source spells them out.
Evidence is the locked registry generator's `src/derive.rs` and `src/func.rs`.
The Platform entrypoint generator likewise emits both executor-context helpers
from `crates/ic-platform-macros/src/export/mod.rs:372-374`. The hidden
`__platform_println` macro implements public `cdk::println`.

Persistence imports the entire stable substrate as `substrate` in Storage's
`src/lib.rs:7`. Its `src/adapter.rs` really uses BTreeMap, BTreeSet, MinHeap, Vec,
Log and iterator/error aliases. Those are not unused merely because the full
Platform-qualified name is absent at each call. Storage also publicly forwards
`storable`; its members are part of that consumer namespace, even without an
individual call site in the scanned trees.

Raw Call builder methods are used through receivers. For example, Blob's funding
transport creates a Platform Call before `take_raw_args`, `get_cost` and
`with_cycles`; its client transport selects `with_wait_policy`/`with_raw_args`.
Query's governance source consumes `Response::into_bytes` and rejection methods.
Name-only searches for common methods such as `len`, `as_slice` or `to_text`
were not accepted as proof without checking the receiver type.

## Scope and limitations

The source-union roster includes 350 named re-exports, 24 owned inherent methods,
three associated constants, four free functions, five structs, four enums, one
type alias, one constant and one exported implementation macro. It includes
feature-gated native and Wasm paths without pretending all configurations run.
Module containers, enum variants and struct fields are covered by their parent
items rather than counted individually. Trait implementations, upstream inherent
methods/fields/variants and everything beneath the compatibility
`candid::contract::serde` module are **not individually audited**. Re-exporting an
upstream type exposes that larger surface; this report cannot establish that
all of it is used. Restricting it could require breaking wrappers or lost type
identity, which is outside this audit.

A temporary Syn 2.0.119 visitor parsed all **7,156** selected Rust files with zero
parse failures. It walked explicit use trees and paths, followed local import
aliases and simple crate-root forwarding, and separated detected test modules,
test functions and fixture/example paths. Manual review handled receiver methods,
Storage aliases and generated paths. It is not Rust name resolution: nested
shadowing, arbitrary cross-module re-exports, glob imports, string templates,
proc-macro expansion and cfg/build-feature combinations are not exhaustively
resolved. Imports are evidence of a declared dependency, not proof of execution.
P/T classification is consequently source evidence, not a successful consumer
build or a complete compiler-produced reachability graph.

The scan used `rg --files ../REPO -g '*.rs'` for the checkouts below. Ignored build
outputs, external registry consumers, non-Rust clients, Toko and the retired
Persistence checkout were excluded. The old Host checkout was included to avoid
mistaking a legacy consumer for absent usage. Source was read only; dirty trees
and changing sibling work mean the review is a dated observation, not an atomic
fleet snapshot or evidence of registry adoption.

- `ic-auth`: `5d7da698d3e4d297f32445fb88adbc66a79c5520`
- `ic-backup`: `21b0a94a1fa8241d26ed1b933e1e353fbc71d2ca`
- `ic-blob-storage`: `ba779305b66acc87bf7af1e4f5a4a4f3ee5f24c7`
- `ic-host-tooling`: `80cc30a4904d3b27aecdb9ed67394b3bd57235b7`
- `ic-jobs`: `7d1f808de0bc14d823bd1d3f04a5a62f06ad7a52`
- `ic-metrics`: `b6bd4e37b6e201ac8c4c5dc99ed40ac409f0c319`
- `ic-persistence`: `fbe1208518950c8f81a8dc8475852d7c330fcaa7`
- `ic-query`: `61292409585d52e510093e6e13a9b03d33c828b8`
- `ic-relay`: `793aa7a125ef26af654528e157feaa8d82e5df04`
- `ic-timers`: `6c3aa211ecf312ec872c4fdc6c0f5624fc05c26d`
- `ic-tooling`: `e5b4095150f49069fc51c3b497e192a8f8eda016`
- `ic-wire`: `83f90a4779aa3cdaf016484dc096322708cd0cdf`
- `canic`: `c63bdca652bc7b4dd82afafc2e16e0de7b6637b7`
- `icydb`: `2a3e1efa551835be8b068a9579c276796f2a6143`

Several trees were dirty (including Auth, Blob, old Host, Persistence, Timers,
Tooling, Canic and IcyDB). Evidence points to working-tree paths/lines, not a claim
that those exact bytes are committed. Full observed status and the file list are
retained locally in `/tmp/platform-export-revisions.txt` and
`/tmp/platform-consumer-rust-files`.

Raw extraction is `/tmp/platform-exports-v4.tsv`; the temporary parser and manual
classification pass are in `/tmp/platform-export-audit`. The repository evidence
file is the durable result, not a new maintained scanner or CI dependency.
Repeat the source/receiver/generator review when considering an actual removal;
the manifest inventory guard does not certify these usage conclusions forever.
Existing compilation, graph, source-integrity and installed-runtime checks retain
their own scope. No new compile/test/runtime qualification is claimed by this
read-only audit; documentation links and inventory consistency were checked.
