// Types-specific admission; Toko owns upload custody, source rechecks and recovery.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';

const root = process.cwd();
const run = (command, args) => {
  const result = spawnSync(command, args, { encoding: 'utf8', timeout: 900000,
    maxBuffer: 32 * 1024 * 1024 });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${command} ${args.join(' ')}\n${result.stdout}\n${result.stderr}`);
  return result.stdout;
};
run('bash', ['vendor/toko-tooling/scripts/ci/check-release-source.sh']);
assert.equal(run('git', ['remote', 'get-url', 'origin']).trim(), 'https://github.com/dragginzgame/ic-platform.git');
const commit = run('git', ['rev-parse', 'HEAD']).trim();
const metadata = JSON.parse(run('cargo', ['metadata', '--locked', '--offline', '--format-version', '1']));
const packages = metadata.packages.filter(p => metadata.workspace_members.includes(p.id));
const product = packages.find(p => p.name === 'ic-platform-types');
assert(product);
assert.deepEqual(product.publish, ['crates-io']);
assert.equal(product.repository, 'https://github.com/dragginzgame/ic-platform');
assert.equal(product.features.default.length, 0);
assert(packages.filter(p => p !== product).every(p => Array.isArray(p.publish) && p.publish.length === 0), 'only Types is eligible');
const tag = `types-v${product.version}`;
assert.equal(run('git', ['cat-file', '-t', tag]).trim(), 'tag', 'annotated Types release tag required');
assert.equal(run('git', ['rev-parse', `${tag}^{commit}`]).trim(), commit);
const remote = run('git', ['ls-remote', '--exit-code', 'origin', `refs/tags/${tag}^{}`]).trim().split(/\s+/);
assert.equal(remote[0], commit, 'exact release tag must already be delivered');
const hosted = JSON.parse(run('gh', ['run', 'list', '--repo', 'dragginzgame/ic-platform',
  '--workflow', 'types-release.yml', '--branch', 'main', '--commit', commit, '--limit', '1',
  '--json', 'databaseId,headSha,status,conclusion']));
assert.equal(hosted.length, 1, 'exact source needs hosted CI evidence');
assert.equal(hosted[0].headSha, commit);
assert.equal(hosted[0].status, 'completed');
assert.equal(hosted[0].conclusion, 'success', 'all supported hosts must pass');
const index = run('curl', ['--fail', '--silent', '--show-error',
  'https://index.crates.io/ic/-p/ic-platform-types']).trim().split('\n').map(JSON.parse);
assert(!index.some(p => p.vers === product.version), 'version already exists; reconcile instead of uploading');
const catalogBytes = readFileSync('ci/types-release-profiles.json');
const profiles = JSON.parse(catalogBytes);
assert.equal(new Set(profiles.map(p => p.id)).size, profiles.length);
assert(profiles.length > 0);
const evidence = resolve(root, `target/types-publication/${Date.now()}-${process.pid}`);
mkdirSync(evidence, { recursive: true });
writeFileSync(`${evidence}/source.json`, JSON.stringify({ commit, tag, version: product.version,
  hosted, registry_observed_at: new Date().toISOString(),
  catalog_sha256: createHash('sha256').update(catalogBytes).digest('hex') }, null, 2));
writeFileSync(`${evidence}/audit.json`, run('cargo', ['audit', '--no-fetch', '--json', '--file', 'Cargo.lock']));
console.log(`Types publication evidence: ${evidence}`);
for (const profile of profiles) {
  assert.equal(profile.owner, 'types');
  assert(/^[a-z-]+$/.test(profile.id));
  assert.equal(profile.command[0], 'make');
  console.log(`Checking ${profile.id}: ${profile.scope}`);
  const result = spawnSync(profile.command[0], profile.command.slice(1), {
    encoding: 'utf8', timeout: 900000, maxBuffer: 32 * 1024 * 1024 });
  writeFileSync(`${evidence}/${profile.id}.log`, `${result.stdout ?? ''}\n${result.stderr ?? ''}`);
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${profile.id} refused; inspect ${evidence}`);
}
run('bash', ['vendor/toko-tooling/scripts/ci/check-release-source.sh']);
assert.equal(run('git', ['rev-parse', 'HEAD']).trim(), commit);
run('bash', ['vendor/toko-tooling/scripts/ci/check-cargo-package-provenance.sh',
  `${evidence}/provenance`, 'target/package', product.repository, commit,
  '--package', `${product.name}@${product.version}`, 'crates/ic-platform-types']);
writeFileSync(`${evidence}/admitted`, `${commit}\n`);
console.log(`Types ${product.version} source, contracts, archive and provenance admitted.`);
