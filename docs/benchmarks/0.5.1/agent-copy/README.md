# Agent request ownership experiment

This is a candidate evaluation, **not an adopted Platform optimization**.
[Platform #39](https://github.com/dragginzgame/ic-platform/issues/39) tracks the
required distribution decision. No production Agent, dependency or crypto changed.

## Source and ownership

Registry `ic-agent` 0.49.2 comes from `dfinity/agent-rs` revision
`5053d3585eb829eb8e75ad1099ad8a4a19721099`, under Apache-2.0.
Private `Agent::execute` clones an owned body before passing it to `request`,
although the original is never used again. The candidate is exactly:

```diff
-        let request_result = self.request(method.clone(), endpoint, body.clone()).await?;
+        let request_result = self.request(method, endpoint, body).await?;
```

`request` already converts its Vec to shared Bytes. Each retry obtains a new URL
and request with a shared Bytes clone. The response `Bytes::to_vec()` is separate
and unchanged. Default transport retries connection errors up to the configured
limit; the enclosing default middleware retries 429/503 up to six additional
attempts with 250ms waits. None of these policies are changed.

Host's `ic-host-tools/src/agent/mod.rs` constructs bounded upstream Agents; Query's
`candid_call.rs`, registry query transport and SNS query paths invoke upstream
query/update operations. These references establish reachability, not traffic
frequency or a production payload-size distribution. Siblings were read-only.

The clone is before the HttpService extension point. A facade or custom client
cannot remove it. Platform retains registry type identity; a dependency-local
Cargo patch is not a consumer distribution mechanism. This single improvement
alone does not justify importing the full Agent. An upstream release carrying
it or a separately reviewed source-ownership plan is needed before adoption.

## Evidence

Linux, Rust 1.88.0, release profile, copied upstream source, locked dependencies.
[Raw results](results.tsv) contain two baseline/candidate passes. Each sample
reports the median (upper middle) of 30 measurements after one warmup; input
construction is outside the timer and allocation counter. The timed interval
executes the actual private method through a test-only visibility change,
constructs requests, receives an in-memory five-byte response and drops results.
It excludes network, signing, serialization, verification and caller processing.

[The probe](probe.rs) checks 72 combinations: six body lengths, one/four request
factory calls and six HTTP statuses. Exact payload bytes, method, path, content
type, retry/limit parameters, response/error bytes and shared retry backing
storage are checked. Allocation counts include reallocations; allocated bytes
are cumulative requested bytes, not peak RSS or retained heap. Timings include
counter instrumentation and allocator effects.

| Request body | Baseline allocated bytes, one call | Candidate | Saved |
| --- | ---: | ---: | ---: |
| empty | 2,607 | 2,607 | 0 |
| 1 KiB | 3,655 | 2,631 | 1,024 |
| 64 KiB | 68,167 | 2,631 | 65,536 |
| 1 MiB | 1,051,207 | 2,631 | 1,048,576 |
| 2 MiB | 2,099,783 | 2,631 | 2,097,152 |

Every nonempty size removes exactly one allocation with either factory count.
One-call median times across the two passes: 64 KiB, 1.78–1.81µs baseline versus
0.83–1.18µs candidate; 1 MiB, 463–577µs versus 1.25–1.28µs; 2 MiB, 921–1064µs
versus 1.65–2.19µs. Large allocation/copy/free costs dominate this synthetic
fixture. Small-body timings are noisy, including regressions between runs.
These are not whole-Agent or network speedups, Wasm savings or charged cycles.

[The loopback fixture](retries.rs) runs the actual default HTTP implementation.
Both variants pass 503→429→200 recovery, seven-attempt 503 exhaustion, immediate
400 rejection and response-size rejection. Every actual attempt has the exact
64 KiB input. TCP connection-error injection, cancellation, native macOS and live
IC were not qualified. Agent is native-only in Platform; no Wasm claim applies.

## Reproduction

Use a prepared registry source tree for exactly 0.49.2 and the repository lock
selection from this experiment. No registry source is edited. Example setup,
from the repository root, with `agent_source` set to that source directory:

```sh
probe_dir=$(mktemp -d /tmp/platform-agent-copy.XXXXXX)
mkdir -p "$probe_dir/src/bin"
cp -a "$agent_source" "$probe_dir/upstream"
cp docs/benchmarks/0.5.1/agent-copy/probe.rs "$probe_dir/src/main.rs"
cp docs/benchmarks/0.5.1/agent-copy/retries.rs "$probe_dir/src/bin/retries.rs"
cp Cargo.lock "$probe_dir/Cargo.lock"
cat > "$probe_dir/Cargo.toml" <<'TOML'
[package]
name = "platform-agent-copy-probe"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
ic-agent = { path = "upstream", default-features = false, features = ["pem"] }
async-trait = "0.1"
bytes = "1"
http = "1"
tokio = { version = "1", features = ["rt", "time"] }
TOML
# Avoid resolving upstream's unused development suite in this standalone probe.
perl -0777 -i -pe 's/^\[(?:[^\n]*\.)?dev-dependencies[^\n]*\]\n.*?(?=^\[|\z)//msg' "$probe_dir/upstream/Cargo.toml"
# Probe-only access to the real private implementation, in both variants.
sed -i 's/    async fn execute(/    pub async fn execute(/' "$probe_dir/upstream/src/agent/mod.rs"
CARGO_BUILD_JOBS=4 cargo +1.88.0 build --release --offline --manifest-path "$probe_dir/Cargo.toml" --bins
"$probe_dir/target/release/platform-agent-copy-probe"
"$probe_dir/target/release/retries"
# Apply the candidate only after recording the baseline.
sed -i 's/self.request(method.clone(), endpoint, body.clone()).await?/self.request(method, endpoint, body).await?/' "$probe_dir/upstream/src/agent/mod.rs"
CARGO_BUILD_JOBS=4 cargo +1.88.0 build --release --offline --locked --manifest-path "$probe_dir/Cargo.toml" --bins
"$probe_dir/target/release/platform-agent-copy-probe"
"$probe_dir/target/release/retries"
```

Commands above are the Linux experiment, not a qualified portable setup tool.
Loopback requires local socket permission. The unused upstream dev dependencies
and visibility adjustment are probe accommodations, not part of the candidate.
Original working evidence is `/tmp/platform-agent-copy/`, including build logs,
resolved lock and unformatted source used for the run. Retained Rust files differ
only by rustfmt. No production tests or installed-canister tests were needed for
this documentation-only repository change.

Original upstream `src/agent/mod.rs` SHA-256:
`ef5dcac4ad583db0148efbca1984708d1ac9a87e6d1ca0f44206a3c8b6f6bf3f`.
Resolved probe lock SHA-256:
`0dc6fb43d69a7862ae7bfa7f281c1ad85933655c36a1b88eb3e74398045d1469`.
Future repository lock changes can change reproduction dependencies and timings.
