# Message-buffer copy qualification

The pending 0.5.1 change reads message bytes into vector spare capacity through
ic0 1.2.0's all-or-trap initialization API, then marks that prefix initialized.
The production change is confined to `cdk::api::msg_arg_data`; the old
zero-initializing implementation exists only as the installed fixture baseline.
The executor, callback state machine, wire formats and cryptography are unchanged.

## Installed measurements

[Results](input-copy.tsv) record IC instruction-counter deltas on Linux with the
prepared Testkit-owned PocketIC 16.0.0 server, Rust 1.99.1 and 1.91.0 release Wasm.
Both Wire-only and external-Candid feature combinations pass. The measured RPC
itself uses Wire in all combinations.

The fixture queries alternate baseline/candidate order over six rounds, discard
the first round and report each variant's median of five samples. The counter
interval covers the explicit second message-buffer read, including its branch
and allocation, but excludes Wire decoding/encoding and reply submission.
Generated ingress has already copied/decoded the arguments before that interval.
Payload sizes exclude the Wire frame and the boolean selecting the variant.
A zero-byte payload therefore still has a nonempty frame.

On Rust 1.99.1, the read falls from 3,780 to 2,632 instructions at a 1,024-byte
payload (30.4%), and 5,828 to 3,656 at 2,048 bytes (37.3%). Rust 1.91 shows similar
reductions. These are fixture-specific read costs, not whole-RPC speedups,
wall-clock timings, cycle charges or Wasm-size reductions. Larger payloads were
not qualified; the retained samples fit the existing caller-owned Wire budgets.

## Correctness and reproduction

Every measured call returns the complete copied frame for exact-byte comparison.
Additional installed cases cover truly empty malformed ingress, binary
request/reply round trips through an awaited self-call, guard refusal, decode
traps, callback-trap cleanup, timer cancellation, upgrades, cycles and stable
rollback. Native tests check that both new fixture payload sets fit their budgets.
Native macOS and live IC execution were not performed.

Use an explicitly prepared Testkit-admitted server binary:

```sh
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic \
  CARGO_BUILD_JOBS=4 bash scripts/ci/check-platform-runtime.sh
```

The runner retains its source, lock, metadata and four Wasm artifacts under the
printed `target/cdk-runtime.*` directory. This run used `target/cdk-runtime.UsGpFZ`;
[artifact digests](artifact-sha256.txt) identify the measured binaries.
Local logs are `/tmp/platform051-input-*.log`. Package versions remain unchanged;
the evidence describes the pending working tree, not a published release.
