# Historical 0.22 benchmark runners

The dated orchestration is retired. Its complete sources, pinned selections and
reproduction instructions remain in [the immutable source tree](https://github.com/dragginzgame/ic-platform/tree/9880ba816461f777266a0aedd54763f412af1522/scripts/benchmarks/0.22).
Use that checkout and its recorded compiler/caches when reproducing old results;
these results are not qualification of current Platform.

Current measurements use [the Wasm-size runner](../wasm-size.sh) and its single
section reader. Read-only Canic capture and comparison reporting live in
[canic-wasm](../canic-wasm/README.md).
