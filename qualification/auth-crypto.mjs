// Exercise pinned Auth verification with explicit replacement API mappings.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {resolve, relative} from 'node:path';
import {adoptTooling} from './tooling-selection.mjs';
const root = process.cwd();
const source = resolve(process.env.AUTH_SOURCE ?? '../ic-auth');
const revision = 'b78991e9833320692921bf766e51ad361c9c8c4d';
const output = resolve(`target/auth-crypto/${Date.now()}`);
mkdirSync(output, {recursive:true});
function run(command, args, cwd = output) {
  const result = spawnSync(command, args, {cwd, encoding:'utf8', maxBuffer:32*1024*1024, env:{...process.env, CARGO_TARGET_DIR:resolve(root, 'target/auth-crypto-build')}});
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  return result.stdout.trim();
}
run('git', ['archive', `--output=${output}/source.tar`, revision], source);
run('tar', ['-xf', 'source.tar']);
run('git', ['apply', `--directory=${relative(root, output)}`, `${root}/qualification/auth-crypto/consumer.patch`], root);
function edit(name, transform) {
  const file = `${output}/${name}`;
  writeFileSync(file, transform(readFileSync(file, 'utf8')));
}
for (const name of ['Cargo.toml', 'qualification/Cargo.toml']) edit(name, text => text
  .replace('ic-platform = { version = "0.33.7",', `ic-platform = { path = ${JSON.stringify(`${root}/crates/ic-platform`)}, version = "=0.34.12",`)
  .replace('ic-platform-types = { version = "0.20.4",', `ic-platform-types = { path = ${JSON.stringify(`${root}/crates/ic-platform-types`)}, version = "=0.21.2",`)
  .replace('ic-wire = "0.24.1"', 'ic-wire = "=0.24.2"'));
// Select the actual consumer's primitive and certified conformance packages.
// Installed qualification uses the same patch in auth-runtime.
edit('crates/ic-auth/Cargo.toml', text => text.replace('ic-platform/signature-verification',
  'ic-platform/canister-signature-authentication'));
edit('qualification/Cargo.toml', text => text.replace(/members = \[[\s\S]*?\]/,
  'members = ["ic-auth-ed25519-conformance", "ic-auth-certified-conformance"]')
  .replace(/^(?:ic-auth-host-fixture|ic-build|ic-host-artifacts|ic-host-fs|ic-host-process|ic-platform-runtime|ic-testkit|ic-wire|serde|tokio) = .*\n/gm, ''));
edit('qualification/ic-auth-certified-conformance/Cargo.toml', text => text
  .replace('features = ["certification-cbor", "ic-protocol-fixtures"]', 'features = ["ecdsa", "certificate-construction", "certificate-fixture-signing", "canister-signature-authentication"]'));
for (const name of ['Cargo.toml','qualification/Cargo.toml']) adoptTooling(`${output}/${name}`);
run('cargo', ['update']);
run('cargo', ['update', '--manifest-path', 'qualification/Cargo.toml']);
const selected = ['--offline', '--locked', '-p', 'ic-auth', '--features', 'ed25519-verification,certified-verification'];
run('cargo', ['clippy', ...selected, '--lib', '--', '-D', 'warnings']);
const verification = run('cargo', ['test', ...selected, '--test', 'verification']);
run('cargo', ['check', ...selected, '--lib', '--target', 'wasm32-unknown-unknown']);
const qualification = ['--offline', '--locked', '--manifest-path', 'qualification/Cargo.toml'];
const scalar = run('cargo', ['test', ...qualification, '-p', 'ic-auth-ed25519-conformance']);
const delegation = run('cargo', ['test', ...qualification, '-p', 'ic-auth-certified-conformance', '--tests']);
run('cargo', ['clippy', ...qualification, '-p', 'ic-auth-certified-conformance', '--test', 'root_delegation', '--', '-D', 'warnings']);
const metadata = JSON.parse(run('cargo', ['metadata', ...qualification, '--format-version=1']));
const platforms = metadata.packages.filter(p => p.name === 'ic-platform');
assert.equal(platforms.length, 1);
assert.equal(platforms[0].version, '0.34.12');
const bigint = metadata.packages.filter(p => p.name === 'crypto-bigint');
assert.deepEqual(bigint.map(p => p.version), ['0.7.11']);
const evidence = {revision, patchSha256:createHash('sha256').update(readFileSync(`${root}/qualification/auth-crypto/consumer.patch`)).digest('hex'), candidate:'0.34.12', verification, scalar, delegation,
  sourceSha256:createHash('sha256').update(readFileSync(`${output}/source.tar`)).digest('hex'),
  lockSha256:createHash('sha256').update(readFileSync(`${output}/qualification/Cargo.lock`)).digest('hex'),
  scope:'actual Auth Ed25519, root delegation and certified issuer; no legacy Platform; installed migration is separate', status:'passed'};
writeFileSync(`${output}/verified.json`, JSON.stringify(evidence,null,2)+'\n');
console.log(JSON.stringify({output,...evidence},null,2));
