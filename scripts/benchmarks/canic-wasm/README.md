# Canic Wasm measurement support

Run `node scripts/benchmarks/canic-wasm/capture.cjs INPUT OUTPUT` to capture
tracked/untracked regular source without modifying the input checkout. Existing
captures, source changes, symlinks and invalid paths refuse. This command captures
source only; caller-owned dependency selections and builds are separate.
The output directory must be new or empty; retained success metadata cannot be
reused for another attempt.
Copied hashes must match the original inventory; source rosters, bytes and Git
identity are checked again before the capture receives a success manifest.

Run `node scripts/benchmarks/canic-wasm/report.cjs BASE ARM...` on prepared
issuer/receiver Wasms, locks and contract files. The shared Wasm section reader
reports total and code payload/framed sizes; the report retains import/export
and contract equality checks. It does not establish compiler/feature equivalence
by itself. Preserve those actual build identities with each comparison.
Section measurements and WebAssembly import/export inspection use the same
captured module bytes, even if the original path is subsequently replaced.

The original 0.22 orchestration is [historical](../0.22/README.md).
