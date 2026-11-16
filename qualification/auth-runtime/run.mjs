// Qualify the pinned real Auth fixture without modifying its checkout.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdirSync, writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
const root = process.cwd();
function run(command, args, cwd, env = process.env) {
  const result = spawnSync(command, args, {cwd, env, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024});
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  return result.stdout.trim();
}
const directory = run('node', ['qualification/auth-runtime/prepare.mjs'], root);
console.log(`Pinned consumer: ${directory}`);
run('cargo', ['metadata', '--offline', '--manifest-path', 'qualification/Cargo.toml', '--format-version=1'], directory);
const vectors = resolve(directory, 'qualification/target/host-vectors/memory.tsv');
mkdirSync(resolve(vectors, '..'), {recursive:true});
writeFileSync(vectors, run('node', ['crates/ic-auth/tests/verify-memory-fixtures.mjs', '--host'], directory) + '\n');
const env = {...process.env,
  AUTH_HOST_CARGO: run('rustup', ['which', 'cargo'], directory),
  AUTH_HOST_RUSTC: run('rustup', ['which', 'rustc'], directory),
  AUTH_HOST_PACKAGE_ID: run('cargo', ['pkgid', '--offline', '--locked', '--manifest-path', 'qualification/Cargo.toml', '-p', 'ic-auth-host-fixture'], directory),
  AUTH_HOST_VECTORS: vectors,
};
console.log(run('cargo', ['test', '--offline', '--locked', '--manifest-path', 'qualification/Cargo.toml', '-p', 'ic-auth-host-conformance', '--test', 'installed', '--', '--ignored'], directory, env));
