// Materialize an unchanged pinned real consumer against a Cargo-produced archive.
// Only disposable validation inputs live below target/; no sibling is modified.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const source = resolve(process.env.AUTH_SOURCE ?? resolve(root, '../ic-auth'));
const revision = 'b78991e9833320692921bf766e51ad361c9c8c4d';
const run = (command, args, cwd = root) => {
  const result = spawnSync(command, args, { cwd, encoding: 'utf8', timeout: 300000,
    maxBuffer: 16 * 1024 * 1024, env: { ...process.env,
      CARGO_TARGET_DIR: resolve(cwd, 'target'), RUSTFLAGS: '', CARGO_ENCODED_RUSTFLAGS: '',
      RUSTC_WRAPPER: '', RUSTC_WORKSPACE_WRAPPER: '', CARGO_PROFILE_RELEASE_OPT_LEVEL: '3',
      CARGO_PROFILE_RELEASE_LTO: 'false', CARGO_PROFILE_RELEASE_CODEGEN_UNITS: '16',
      CARGO_PROFILE_RELEASE_DEBUG: 'false', CARGO_PROFILE_RELEASE_STRIP: 'none',
      CARGO_PROFILE_RELEASE_PANIC: 'unwind', CARGO_PROFILE_RELEASE_INCREMENTAL: 'false',
      CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS: 'false', CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS: 'false' } });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} ${args.join(' ')} failed (${result.status})\n${result.stdout}\n${result.stderr}`);
  if (result.stderr) process.stderr.write(result.stderr);
  return result.stdout;
};
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const metadata = JSON.parse(run('cargo', ['metadata', '--offline', '--locked', '--no-deps', '--format-version', '1']));
const product = metadata.packages.find(p => p.name === 'ic-platform-types');
assert(product);
// A fresh directory preserves all failed and completed attempts for inspection.
const output = resolve(root, `target/types-auth/${Date.now()}-${process.pid}`);
assert(!existsSync(output));
mkdirSync(output, { recursive: true });
console.log(`Evidence: ${output}`);
const tree = run('git', ['rev-parse', `${revision}:crates/ic-auth-protocol-types`], source).trim();
run('git', ['archive', '--format=tar', `--output=${output}/auth.tar`, revision,
  'Cargo.toml', 'LICENSE.md', 'crates/ic-auth-protocol-types/Cargo.toml',
  'crates/ic-auth-protocol-types/src'], source);
run('tar', ['-xf', `${output}/auth.tar`, '-C', output]);
run('cargo', ['package', '--offline', '--locked', '-p', 'ic-platform-types']);
const archive = resolve(root, `target/package/ic-platform-types-${product.version}.crate`);
run('tar', ['-xf', archive, '-C', output]);
const extracted = resolve(output, `ic-platform-types-${product.version}`);
// Cargo searches enclosing workspaces even through another workspace's exclude.
// Add only an isolation marker to the extracted normalized manifest. The archive
// bytes, source, dependency selections and features remain unchanged.
writeFileSync(`${extracted}/Cargo.toml`, `${readFileSync(`${extracted}/Cargo.toml`, 'utf8')}\n[workspace]\n`);
const originalRoot = readFileSync(`${output}/Cargo.toml`, 'utf8');
const lintStart = originalRoot.indexOf('[workspace.lints.cargo]');
assert(lintStart > 0);
// The source manifest alone changes dependency selection; Rust remains exact.
writeFileSync(`${output}/Cargo.toml`, `[workspace]\nmembers = ["crates/ic-auth-protocol-types"]\nexclude = ["ic-platform-types-${product.version}"]\nresolver = "3"\n\n[workspace.package]\nedition = "2024"\nrust-version = "1.100.0"\nlicense-file = "LICENSE.md"\n\n[workspace.dependencies]\nic-platform-types = { path = ${JSON.stringify(extracted)}, version = "=${product.version}", default-features = false }\ntoko-tooling = "=0.44.5"\n\n${originalRoot.slice(lintStart)}`);
const manifest = `${output}/crates/ic-auth-protocol-types/Cargo.toml`;
writeFileSync(manifest, `${readFileSync(manifest, 'utf8')}\n[[bin]]\nname = "types-boundary"\npath = ${JSON.stringify(resolve(root, 'qualification/types-auth/tests.rs'))}\n`);
for (const target of [[], ['--target', 'wasm32-unknown-unknown']]) {
  run('cargo', ['clippy', '--offline', '--locked', '--manifest-path', `${extracted}/Cargo.toml`,
    '--all-features', '--all-targets', ...target, '--', '-D', 'warnings'], output);
}
run('cargo', ['test', '--offline', '--locked', '--manifest-path', `${extracted}/Cargo.toml`, '--all-features'], output);
// Resolve only the isolated candidate graph, then keep every check locked/offline.
run('cargo', ['generate-lockfile', '--offline'], output);
const graph = JSON.parse(run('cargo', ['metadata', '--offline', '--locked', '--format-version', '1'], output));
const types = graph.packages.filter(p => p.name === 'ic-platform-types');
assert.equal(types.length, 1);
assert.equal(types[0].manifest_path, `${extracted}/Cargo.toml`);
assert.equal(types[0].version, product.version);
const results = [];
for (const feature of [false, true]) {
  const flags = feature ? ['--features', 'principal-self-authenticating'] : [];
  for (const target of [[], ['--target', 'wasm32-unknown-unknown']]) {
    run('cargo', ['clippy', '--offline', '--locked', '-p', 'ic-auth-protocol-types',
      '--lib', '--bin', 'types-boundary', ...flags, ...target, '--', '-D', 'warnings'], output);
  }
  run('cargo', ['run', '--offline', '--locked', '-p', 'ic-auth-protocol-types', '--bin', 'types-boundary', ...flags], output);
  run('cargo', ['build', '--offline', '--locked', '-p', 'ic-auth-protocol-types', '--bin', 'types-boundary',
    '--release', '--target', 'wasm32-unknown-unknown', ...flags], output);
  const bytes = readFileSync(`${output}/target/wasm32-unknown-unknown/release/types-boundary.wasm`);
  const module = new WebAssembly.Module(bytes);
  assert.deepEqual(WebAssembly.Module.imports(module), []);
  assert.equal(new WebAssembly.Instance(module).exports.main(0, 0), 0);
  results.push({ feature, wasm_sha256: hash(bytes) });
}
const evidence = { source: revision, consumer_tree: tree, types: product.version,
  archive_sha256: hash(readFileSync(archive)), lock_sha256: hash(readFileSync(`${output}/Cargo.lock`)),
  rust: run('rustc', ['-V']).trim(), node: process.version, results };
writeFileSync(`${output}/evidence.json`, `${JSON.stringify(evidence, null, 2)}\n`);
console.log(JSON.stringify(evidence, null, 2));
